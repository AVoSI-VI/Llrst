# Time Llrst against the other languages and the two compilers against each other.
#   benchmarks/run.ps1 [--check] [--update-baseline] [--once] [--only NAME]
# Same flags and line grammar as benchmarks/run.sh.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $root

$once = $false
$check = $false
$update = $false
$only = ''
$i = 0
while ($i -lt $args.Count) {
    $a = $args[$i]
    switch ($a) {
        '--once' { $once = $true }
        '--check' { $check = $true }
        '--update-baseline' { $update = $true }
        '--only' {
            $i++
            if ($i -ge $args.Count) { Write-Output 'fail|args|--only'; exit 1 }
            $only = $args[$i]
        }
        default { Write-Output "fail|args|$a"; exit 1 }
    }
    $i++
}

$runs = 5
if ($once) { $runs = 1 }

$results = Join-Path $root 'benchmarks\results'
New-Item -ItemType Directory -Force -Path (Join-Path $results 'compile') | Out-Null
$log = Join-Path $results 'last.txt'
Set-Content -LiteralPath $log -Value '' -Encoding ascii

function Note($msg) {
    Write-Output $msg
    Add-Content -LiteralPath $log -Value $msg -Encoding ascii
}
function Fail($msg) {
    Note "fail|$msg"
    exit 1
}
function Have($name) {
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

# LLRC is the installed compiler and is often llrs2. Stage 0 is the Rust binary.
if ($env:LLRC0) { $llrc = $env:LLRC0 }
elseif (Test-Path -LiteralPath (Join-Path $root 'target\release\llrc.exe')) { $llrc = Join-Path $root 'target\release\llrc.exe' }
else { $llrc = Join-Path $root 'target\release\llrc' }
if ($env:LLRS2) { $llrs2 = $env:LLRS2 }
elseif (Test-Path -LiteralPath (Join-Path $root 'target\self\llrs2.exe')) { $llrs2 = Join-Path $root 'target\self\llrs2.exe' }
else { $llrs2 = Join-Path $root 'target\self\llrs2' }
if (-not (Test-Path -LiteralPath $llrc)) { Write-Output 'fail|setup|llrc=not found|fix=cargo build --release'; exit 1 }
if (-not (Test-Path -LiteralPath $llrs2)) { Write-Output 'fail|setup|llrs2=not found|fix=scripts/self.ps1 --stage'; exit 1 }

$env:LLRST_STD = Join-Path $root 'std'
$env:LLRST_RUNTIME = Join-Path $root 'runtime'

function Compile-WinShim($name) {
    $dir = Join-Path $root 'runtime'
    $c = Join-Path $dir $name
    $obj = Join-Path $dir ($name -replace '\.c$', '.obj')
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    if ($cl) {
        & cl.exe /nologo /c "/Fo$obj" $c | Out-Null
        if ($LASTEXITCODE -ne 0) { Fail "setup|cc=$name" }
        return
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { Fail 'setup|cl=missing' }
    $inst = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
    if (-not $inst) { Fail 'setup|cl=missing' }
    $bat = Join-Path $inst.Trim() 'VC\Auxiliary\Build\vcvars64.bat'
    cmd /c "call `"$bat`" >nul && cl /nologo /c /Fo`"$obj`" `"$c`"" | Out-Null
    if ($LASTEXITCODE -ne 0) { Fail "setup|cc=$name" }
}
foreach ($shim in @('llr_stat.c', 'llr_now.c', 'llr_reload.c', 'llr_dl.c')) {
    Compile-WinShim $shim
}

function Zig-Works($path) {
    if (-not $path -or -not (Test-Path -LiteralPath $path)) { return $false }
    $lib = Join-Path (Split-Path -Parent $path) 'lib'
    if (-not (Test-Path -LiteralPath $lib)) { return $false }
    & $path version 2>$null | Out-Null
    return ($LASTEXITCODE -eq 0)
}
$zigbin = $null
if (Zig-Works $env:ZIG) { $zigbin = $env:ZIG }
elseif (Have 'zig') {
    $onPath = (Get-Command zig).Source
    if (Zig-Works $onPath) { $zigbin = $onPath }
}
if (-not $zigbin) {
    $cands = Get-ChildItem -Path (Join-Path $env:USERPROFILE 'Downloads') -Filter zig.exe -Recurse -Depth 2 -ErrorAction SilentlyContinue
    foreach ($c in $cands) {
        if (Zig-Works $c.FullName) { $zigbin = $c.FullName; break }
    }
}

$cpu = $env:PROCESSOR_IDENTIFIER
if (-not $cpu) { $cpu = 'Windows' }
$machine = "Windows|$($env:PROCESSOR_ARCHITECTURE)|$cpu"

$tasks = @('fib', 'sieve', 'sort', 'string_build', 'map_ops', 'json_parse', 'nbody', 'mandelbrot', 'spectral_norm', 'fannkuch', 'binary_trees')
$micros = @('fib', 'sieve', 'sort', 'string_build', 'map_ops', 'alloc_churn')
$oneTask = ''
$doCross = $false
$doMicro = $false
$doCompile = $false
if ($only -eq '') {
    $doCross = $true; $doMicro = $true; $doCompile = $true
} elseif ($only -eq 'micro') { $doMicro = $true }
elseif ($only -eq 'compile') { $doCompile = $true }
elseif ($only -eq 'cross') { $doCross = $true }
elseif ($tasks -contains $only) { $doCross = $true; $oneTask = $only }
else { Write-Output "fail|args|$only"; exit 1 }

function Median-Min($nums) {
    $xs = @($nums | Sort-Object)
    return @($xs[[int]($xs.Count / 2)], $xs[0])
}

function Invoke-Samples($outPath, $exe, $argList) {
    $samples = @()
    for ($n = 0; $n -lt $runs; $n++) {
        $sw = [Diagnostics.Stopwatch]::StartNew()
        if ($null -eq $argList) {
            & $exe > $outPath 2> "$outPath.err"
        } else {
            & $exe @argList > $outPath 2> "$outPath.err"
        }
        $code = $LASTEXITCODE
        $sw.Stop()
        if ($code -ne 0) { return $null }
        $samples += [int]$sw.ElapsedMilliseconds
    }
    return Median-Min $samples
}

function Build-Llr($comp, $src, $out) {
    & $comp build $src -o $out --std (Join-Path $root 'std') > "$out.build" 2>&1
    return ($LASTEXITCODE -eq 0)
}

function Run-Cross($task) {
    $dir = Join-Path $root "benchmarks\tasks\$task"
    $expect = Join-Path $dir 'expected.txt'
    $slot = Join-Path $results $task
    New-Item -ItemType Directory -Force -Path $slot | Out-Null
    foreach ($lang in @('llr-stage0', 'llr-stage2', 'c', 'rust', 'nim', 'python', 'node', 'go', 'zig', 'java')) {
        $bin = Join-Path $slot $lang
        $src = switch ($lang) {
            'llr-stage0' { Join-Path $dir "$task.llr" }
            'llr-stage2' { Join-Path $dir "$task.llr" }
            'c' { Join-Path $dir "$task.c" }
            'rust' { Join-Path $dir "$task.rs" }
            'nim' { Join-Path $dir "$task.nim" }
            'python' { Join-Path $dir "$task.py" }
            'node' { Join-Path $dir "$task.js" }
            'go' { Join-Path $dir "$task.go" }
            'zig' { Join-Path $dir "$task.zig" }
            'java' { Join-Path $dir 'Main.java' }
        }
        if (-not (Test-Path -LiteralPath $src)) { Note "skip|$task|$lang|missing"; continue }
        switch ($lang) {
            'llr-stage0' { if (-not (Build-Llr $llrc $src $bin)) { Fail "$task|$lang|build" } }
            'llr-stage2' { if (-not (Build-Llr $llrs2 $src $bin)) { Fail "$task|$lang|build" } }
            'c' {
                if (-not (Have 'gcc')) { Note "skip|$task|c|not-installed"; continue }
                & gcc -O2 -std=c11 -o $bin $src -lm > "$bin.build" 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "$task|c|build" }
            }
            'rust' {
                if (-not (Have 'rustc')) { Note "skip|$task|rust|not-installed"; continue }
                & rustc -O --edition 2021 -o $bin $src > "$bin.build" 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "$task|rust|build" }
            }
            'nim' {
                if (-not (Have 'nim')) { Note "skip|$task|nim|not-installed"; continue }
                & nim c -d:release --hints:off --warnings:off "--nimcache:$slot\nimcache" "-o:$bin" $src > "$bin.build" 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "$task|nim|build" }
            }
            'python' { if (-not (Have 'python')) { Note "skip|$task|python|not-installed"; continue } }
            'node' { if (-not (Have 'node')) { Note "skip|$task|node|not-installed"; continue } }
            'go' {
                if (-not (Have 'go')) { Note "skip|$task|go|not-installed"; continue }
                Push-Location $dir
                & go build -o $bin "$task.go" > "$bin.build" 2>&1
                $code = $LASTEXITCODE
                Pop-Location
                if ($code -ne 0) { Fail "$task|go|build" }
            }
            'zig' {
                if (-not $zigbin) { Note "skip|$task|zig|not-installed"; continue }
                New-Item -ItemType Directory -Force -Path (Join-Path $results 'zig-cache'), (Join-Path $results 'zig-global') | Out-Null
                & $zigbin build-exe -O ReleaseFast "-femit-bin=$bin" "--cache-dir=$(Join-Path $results 'zig-cache')" "--global-cache-dir=$(Join-Path $results 'zig-global')" $src > "$bin.build" 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "$task|zig|build" }
            }
            'java' {
                if (-not (Have 'javac')) { Note "skip|$task|java|not-installed"; continue }
                New-Item -ItemType Directory -Force -Path (Join-Path $slot 'java') | Out-Null
                & javac -d (Join-Path $slot 'java') $src > "$bin.build" 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "$task|java|build" }
            }
        }
        $out = Join-Path $slot "$lang.out"
        $pair = switch ($lang) {
            'python' { Invoke-Samples $out 'python' @($src) }
            'node' { Invoke-Samples $out 'node' @($src) }
            'java' { Invoke-Samples $out 'java' @('-cp', (Join-Path $slot 'java'), 'Main') }
            default { Invoke-Samples $out $bin $null }
        }
        if ($null -eq $pair) { Fail "$task|$lang|run" }
        $got = [System.IO.File]::ReadAllText($out).Replace("`r`n", "`n")
        $want = [System.IO.File]::ReadAllText($expect).Replace("`r`n", "`n")
        if ($got -ne $want) {
            Note "fail|$task|$lang|stdout"
            Write-Output "got $got"
            Write-Output "want $want"
            exit 1
        }
        Note "cross|$task|$lang|$($pair[0])"
        Note "spread|$task|$lang|$($pair[1])"
    }
}

function Run-Micro($stage, $comp) {
    foreach ($name in $micros) {
        $err = Join-Path $results "micro-$stage-$name.err"
        $env:LLRC = $comp
        $samples = @()
        for ($n = 0; $n -lt $runs; $n++) {
            $out = & $llrc run (Join-Path $root 'tools\llr-bench.llr') -- (Join-Path $root "benchmarks\micro\$name.llr") 2> $err
            if ($LASTEXITCODE -ne 0) { Fail "micro|$stage|$name" }
            $ns = $null
            foreach ($line in @($out)) {
                if ($line -match '^bench\|') { $ns = [int](($line -split '\|')[2]); break }
            }
            if ($null -eq $ns) { Fail "micro|$stage|$name|parse" }
            $samples += $ns
        }
        $pair = Median-Min $samples
        Note "bench|$stage|$name|$($pair[0])"
    }
}

function Run-Compile {
    $src = Join-Path $root 'self\llrs.llr'
    $env:SOURCE_DATE_EPOCH = '0'
    $std = Join-Path $root 'std'
    $steps = @(
        @($llrc, 'llrc', 'llrs0'),
        @((Join-Path $results 'compile\llrs0'), 'llrs0', 'llrs1'),
        @((Join-Path $results 'compile\llrs1'), 'llrs1', 'llrs2')
    )
    foreach ($step in $steps) {
        $sw = [Diagnostics.Stopwatch]::StartNew()
        & $step[0] build $src -o (Join-Path $results "compile\$($step[2])") --std $std > (Join-Path $results "compile\$($step[2]).log") 2>&1
        $sw.Stop()
        if ($LASTEXITCODE -ne 0) { Fail "compile|$($step[1])|self/llrs.llr" }
        Note "compile|$($step[1])|self/llrs.llr|$([int]$sw.ElapsedMilliseconds)"
    }
    foreach ($pair in @(@($llrc, 'llrc', 'http-llrc.log'), @($llrs2, 'llrs2', 'http-llrs2.log'))) {
        $sw = [Diagnostics.Stopwatch]::StartNew()
        & $pair[0] check (Join-Path $root 'std\http.llr') --std $std > (Join-Path $results "compile\$($pair[2])") 2>&1
        $sw.Stop()
        if ($LASTEXITCODE -ne 0) { Fail "compile|$($pair[1])|std/http.llr" }
        Note "compile|$($pair[1])|std/http.llr|$([int]$sw.ElapsedMilliseconds)"
    }
    Remove-Item Env:SOURCE_DATE_EPOCH -ErrorAction SilentlyContinue
}

if ($doCross) {
    foreach ($task in $tasks) {
        if ($oneTask -eq '' -or $oneTask -eq $task) { Run-Cross $task }
    }
}
if ($doMicro) {
    Run-Micro 'stage0' $llrc
    Run-Micro 'stage2' $llrs2
}
if ($doCompile) { Run-Compile }

if ($update) {
    $lines = @("# machine|$machine", '# clock|time.mono_ns')
    foreach ($line in Get-Content -LiteralPath $log) {
        if ($line -match '^(bench|cross|compile)\|') { $lines += $line }
    }
    Set-Content -LiteralPath (Join-Path $root 'benchmarks\baseline.txt') -Value $lines -Encoding ascii
    Note 'ok|baseline|benchmarks/baseline.txt'
}

if ($check) {
    $basePath = Join-Path $root 'benchmarks\baseline.txt'
    if (-not (Test-Path -LiteralPath $basePath)) { Fail 'check|baseline=missing' }
    $base = @(Get-Content -LiteralPath $basePath)
    $baseM = ($base | Where-Object { $_ -like '# machine|*' } | Select-Object -First 1) -replace '^# machine\|', ''
    $advisory = $false
    if ($baseM -ne $machine) {
        Note "note|machine|baseline=$baseM|now=$machine|advisory"
        $advisory = $true
    }
    $status = 0
    foreach ($line in (Get-Content -LiteralPath $log)) {
        $gate = $false
        if ($line -match '^(bench|compile)\|') { $gate = $true }
        elseif ($line -match '^cross\|') {
            $lang = ($line -split '\|')[2]
            if ($lang -eq 'llr-stage0' -or $lang -eq 'llr-stage2') { $gate = $true }
        }
        if (-not $gate) { continue }
        $parts = $line -split '\|'
        $now = [int]$parts[-1]
        $key = ($parts[0..($parts.Count - 2)] -join '|')
        $prev = $null
        foreach ($b in $base) {
            if ($b.StartsWith("$key|")) { $prev = [int](($b -split '\|')[-1]) }
        }
        if ($null -eq $prev) { Note "fail|check|$key|missing"; $status = 1; continue }
        $limit = $prev + [int]($prev * 3 / 10)
        if ($now -gt $limit) {
            $pct = [int](($now - $prev) * 100 / $prev)
            Note "fail|$key|base=$prev|now=$now|+$pct%"
            $status = 1
        }
    }
    if ($status -ne 0 -and $advisory) { Note 'note|check|advisory'; $status = 0 }
    if ($status -ne 0) { exit 1 }
    Note 'ok|check'
}

Note 'ok|bench'
exit 0
