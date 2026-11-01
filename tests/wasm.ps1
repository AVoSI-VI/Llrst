# tests/run and tests/panic goldens under wasmtime. Not part of tests/run.ps1.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$fails = 0

function Quote-Arg([string]$s) {
    if ($s -match '[\s"]') { return '"' + ($s -replace '"', '\"') + '"' }
    return $s
}

function Invoke-Native($file, $argList, $stdout, $stderr, $work) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $file
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true
    $psi.WorkingDirectory = $work
    $psi.Arguments = (($argList | ForEach-Object { Quote-Arg ([string]$_) }) -join ' ')
    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()
    $outStream = if ($stdout) { [System.IO.File]::Create($stdout) } else { New-Object System.IO.MemoryStream }
    $errStream = if ($stderr) { [System.IO.File]::Create($stderr) } else { New-Object System.IO.MemoryStream }
    $outTask = $proc.StandardOutput.BaseStream.CopyToAsync($outStream)
    $errTask = $proc.StandardError.BaseStream.CopyToAsync($errStream)
    $proc.WaitForExit()
    $outTask.Wait()
    $errTask.Wait()
    $code = $proc.ExitCode
    $outStream.Dispose()
    $errStream.Dispose()
    $proc.Dispose()
    return $code
}

function Same-Bytes($got, $want) {
    if (-not (Test-Path -LiteralPath $got) -or -not (Test-Path -LiteralPath $want)) { return $false }
    $x = [System.IO.File]::ReadAllBytes($got)
    $raw = [System.IO.File]::ReadAllBytes($want)
    $y = New-Object System.Collections.Generic.List[byte]
    foreach ($b in $raw) {
        if ($b -ne 13) { [void]$y.Add($b) }
    }
    if ($x.Length -ne $y.Count) { return $false }
    for ($i = 0; $i -lt $x.Length; $i++) {
        if ($x[$i] -ne $y[$i]) { return $false }
    }
    return $true
}

function Head-Text($path) {
    if (-not (Test-Path -LiteralPath $path)) { return '' }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $n = [Math]::Min(200, $bytes.Length)
    $text = [System.Text.Encoding]::UTF8.GetString($bytes, 0, $n)
    return ($text -replace "`r", '' -replace "`n", ' ')
}

function Read-Flags($path) {
    if (-not (Test-Path -LiteralPath $path)) { return @() }
    $line = [System.IO.File]::ReadAllText($path).Trim()
    if ($line -eq '') { return @() }
    return @($line.Split(' ', [System.StringSplitOptions]::RemoveEmptyEntries))
}

$llrc = $env:LLRC
if (-not $llrc) { $llrc = Join-Path $root 'target\release\llrc.exe' }
if (-not (Test-Path -LiteralPath $llrc)) {
    $cmd = Get-Command llrc -ErrorAction SilentlyContinue
    if ($cmd) { $llrc = $cmd.Source }
}
if (-not $llrc -or -not (Test-Path -LiteralPath $llrc)) {
    Write-Output 'fail|setup|llrc=not found|fix=cargo build --release or set LLRC'
    exit 1
}
$wasmtime = Get-Command wasmtime -ErrorAction SilentlyContinue
if (-not $wasmtime) {
    Write-Output 'fail|setup|wasmtime=missing'
    exit 1
}
$env:LLRST_STD = Join-Path $root 'std'
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("llr-wasm-" + [System.Guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp | Out-Null

function Run-One($kind, $name, $wasm) {
    $out = Join-Path $tmp "$name.out"
    $err = Join-Path $tmp "$name.err"
    $code = Invoke-Native $wasmtime.Source @('run', $wasm) $out $err $root
    if ($kind -eq 'panic') {
        $want = Join-Path $root "tests\panic\$name.err"
        if ($code -eq 101 -and (Same-Bytes $err $want)) {
            Write-Output "ok|panic/$name"
        } else {
            Write-Output "fail|panic/$name|code=$code|want=$(Head-Text $want)|got=$(Head-Text $err)"
            $script:fails++
        }
        return
    }
    $want = Join-Path $root "tests\run\$name.out"
    if ($name -eq 'os_id') {
        $want = Join-Path $tmp 'os_id.want'
        [System.IO.File]::WriteAllText($want, "3`n")
    }
    $ok = $true
    if (-not (Same-Bytes $out $want)) {
        Write-Output "fail|run/$name|want=$(Head-Text $want)|got=$(Head-Text $out)"
        $script:fails++
        $ok = $false
    }
    $errWant = Join-Path $root "tests\run\$name.err"
    if (Test-Path -LiteralPath $errWant) {
        if (-not (Same-Bytes $err $errWant)) {
            Write-Output "fail|run/$name|stderr|want=$(Head-Text $errWant)|got=$(Head-Text $err)"
            $script:fails++
            $ok = $false
        }
    } elseif ((Test-Path -LiteralPath $err) -and ((Get-Item -LiteralPath $err).Length -gt 0)) {
        Write-Output "fail|run/$name|stderr|got=$(Head-Text $err)"
        $script:fails++
        $ok = $false
    }
    $codeWant = Join-Path $root "tests\run\$name.code"
    if (Test-Path -LiteralPath $codeWant) {
        $wantCode = [int]([System.IO.File]::ReadAllText($codeWant).Trim())
        if ($code -ne $wantCode) {
            Write-Output "fail|run/$name|code|want=$wantCode|got=$code"
            $script:fails++
            $ok = $false
        }
    } elseif ($code -ne 0) {
        Write-Output "fail|run/$name|code=$code"
        $script:fails++
        $ok = $false
    }
    if ($ok) { Write-Output "ok|run/$name" }
}

Get-ChildItem -LiteralPath (Join-Path $root 'tests\run') -Filter '*.llr' | ForEach-Object {
    $name = $_.BaseName
    $want = Join-Path $_.DirectoryName "$name.out"
    if (-not (Test-Path -LiteralPath $want)) { return }
    $wasm = Join-Path $tmp "$name.wasm"
    $cerr = Join-Path $tmp "$name.cerr"
    $args = @('build', $_.FullName, '--std', 'std', '--target', 'wasm32-wasi') + (Read-Flags (Join-Path $_.DirectoryName "$name.flags")) + @('-o', $wasm)
    $build = Invoke-Native $llrc $args $null $cerr $root
    if ($build -ne 0) {
        Write-Output "fail|run/$name|compile|$(Head-Text $cerr)"
        $script:fails++
        return
    }
    Run-One 'run' $name $wasm
}

Get-ChildItem -LiteralPath (Join-Path $root 'tests\panic') -Filter '*.llr' | ForEach-Object {
    $name = $_.BaseName
    $want = Join-Path $_.DirectoryName "$name.err"
    if (-not (Test-Path -LiteralPath $want)) { return }
    $wasm = Join-Path $tmp "panic_$name.wasm"
    $cerr = Join-Path $tmp "panic_$name.cerr"
    $args = @('build', "tests/panic/$name.llr", '--std', 'std', '--target', 'wasm32-wasi') + (Read-Flags (Join-Path $_.DirectoryName "$name.flags")) + @('-o', $wasm)
    $build = Invoke-Native $llrc $args $null $cerr $root
    if ($build -ne 0) {
        Write-Output "fail|panic/$name|compile|$(Head-Text $cerr)"
        $script:fails++
        return
    }
    Run-One 'panic' $name $wasm
}

Write-Output 'skip|net|reason=target.unsupported'
Write-Output 'skip|thread|reason=target.unsupported'
Write-Output 'skip|os.shell|reason=target.unsupported'
Write-Output 'skip|fs.list|reason=target.unsupported'
Write-Output "done|fails=$fails"
Remove-Item -LiteralPath $tmp -Recurse -Force
if ($fails -ne 0) { exit 1 }
