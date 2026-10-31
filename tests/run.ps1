# Golden tests for Windows. Same sets as tests/run.sh.
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$utf8 = New-Object System.Text.UTF8Encoding $false

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

function Exe-Path($path) {
    if ($path.EndsWith('.exe')) { return $path }
    return "$path.exe"
}

function Compile-WinShim($name) {
    $dir = Join-Path $root 'runtime'
    $c = Join-Path $dir $name
    $obj = Join-Path $dir ($name -replace '\.c$', '.obj')
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    if ($cl) {
        & cl.exe /nologo /c "/Fo$obj" $c | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Output "fail|setup|cc=$name"
            exit 1
        }
        return
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) {
        Write-Output 'fail|setup|cl=missing'
        exit 1
    }
    $inst = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
    if (-not $inst) {
        Write-Output 'fail|setup|cl=missing'
        exit 1
    }
    $bat = Join-Path $inst.Trim() 'VC\Auxiliary\Build\vcvars64.bat'
    cmd /c "call `"$bat`" >nul && cl /nologo /c /Fo`"$obj`" `"$c`"" | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Output "fail|setup|cc=$name"
        exit 1
    }
}

if ($env:LLRC) {
    $llrc = $env:LLRC
} elseif (Test-Path -LiteralPath (Join-Path $root 'target\release\llrc.exe')) {
    $llrc = Join-Path $root 'target\release\llrc.exe'
} elseif (Test-Path -LiteralPath (Join-Path $root 'target\debug\llrc.exe')) {
    $llrc = Join-Path $root 'target\debug\llrc.exe'
} else {
    $cmd = Get-Command llrc -ErrorAction SilentlyContinue
    if ($cmd) { $llrc = $cmd.Source } else { $llrc = $null }
}
if (-not $llrc -or -not (Test-Path -LiteralPath $llrc)) {
    Write-Output 'fail|setup|llrc=not found|fix=cargo build or set LLRC'
    exit 1
}
$env:LLRST_STD = Join-Path $root 'std'
$env:LLRST_UI = Join-Path $root 'ui'
$env:LLRST_RUNTIME = Join-Path $root 'runtime'
foreach ($shim in @('llr_wsa.c', 'llr_win.c', 'llr_win_sock.c', 'llr_win_proc.c')) {
    Compile-WinShim $shim
}
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('llr-tests-' + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp | Out-Null
$fails = 0

try {
    Get-ChildItem -LiteralPath (Join-Path $root 'tests\run') -Filter '*.llr' | ForEach-Object {
        $name = $_.BaseName
        $want = Join-Path $_.DirectoryName "$name.out"
        if (-not (Test-Path -LiteralPath $want)) { return }
        if ($name -eq 'os_id') {
            $want = Join-Path $tmp 'os_id.want'
            [System.IO.File]::WriteAllText($want, "2`n", $utf8)
        }
        $flags = Read-Flags (Join-Path $_.DirectoryName "$name.flags")
        $bin = Join-Path $tmp $name
        $cerr = Join-Path $tmp "$name.cerr"
        $args = @('build', $_.FullName) + $flags + @('-o', $bin)
        $build = Invoke-Native $llrc $args $null $cerr $root
        if ($build -ne 0) {
            Write-Output "fail|run/$name|compile|$(Head-Text $cerr)"
            $script:fails++
            return
        }
        $out = Join-Path $tmp "$name.out"
        $stderr = Join-Path $tmp "$name.stderr"
        $code = Invoke-Native (Exe-Path $bin) @() $out $stderr $root
        [System.IO.File]::WriteAllText((Join-Path $tmp "$name.code"), "$code`n", $utf8)
        if ($name -eq 'inline') {
            $mir = Join-Path $tmp 'inline.mir'
            Invoke-Native $llrc @('emit-mir', $_.FullName) $mir $null $root | Out-Null
            $text = [System.IO.File]::ReadAllText($mir)
            if ($text -match '(?m)^fn (str\.at|vec\.get)( |$)') {
                Write-Output 'fail|run/inline|mir=str.at or vec.get was not substituted'
                $script:fails++
            }
        }
        $ok = $true
        if (-not (Same-Bytes $out $want)) {
            Write-Output "fail|run/$name|want=$(Head-Text $want)|got=$(Head-Text $out)"
            $script:fails++
            $ok = $false
        }
        $errWant = Join-Path $_.DirectoryName "$name.err"
        if (Test-Path -LiteralPath $errWant) {
            if (-not (Same-Bytes $stderr $errWant)) {
                Write-Output "fail|run/$name|stderr|want=$(Head-Text $errWant)|got=$(Head-Text $stderr)"
                $script:fails++
                $ok = $false
            }
        } elseif ((Test-Path -LiteralPath $stderr) -and ((Get-Item -LiteralPath $stderr).Length -gt 0)) {
            Write-Output "fail|run/$name|stderr|got=$(Head-Text $stderr)"
            $script:fails++
            $ok = $false
        }
        $codeWant = Join-Path $_.DirectoryName "$name.code"
        $codeGot = Join-Path $tmp "$name.code"
        if (Test-Path -LiteralPath $codeWant) {
            if (-not (Same-Bytes $codeGot $codeWant)) {
                Write-Output "fail|run/$name|code|want=$(Head-Text $codeWant)|got=$(Head-Text $codeGot)"
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

    Get-ChildItem -LiteralPath (Join-Path $root 'tests\ui') -Filter '*.llr' | ForEach-Object {
        $name = $_.BaseName
        $want = Join-Path $_.DirectoryName "$name.out"
        if (-not (Test-Path -LiteralPath $want)) { return }
        $bin = Join-Path $tmp "ui_$name"
        $cerr = Join-Path $tmp "ui_$name.cerr"
        $args = @('build', "tests/ui/$name.llr", '--std', 'std', '--ui', 'ui', '-o', $bin)
        $build = Invoke-Native $llrc $args $null $cerr $root
        if ($build -ne 0) {
            Write-Output "fail|ui/$name|compile|$(Head-Text $cerr)"
            $script:fails++
            return
        }
        $out = Join-Path $tmp "ui_$name.out"
        $stderr = Join-Path $tmp "ui_$name.stderr"
        $ppm = Join-Path $_.DirectoryName "$name.ppm"
        $shot = Join-Path $tmp "ui_$name.shot"
        $runArgs = @()
        if (Test-Path -LiteralPath $ppm) { $runArgs = @('--shot', $shot) }
        $code = Invoke-Native (Exe-Path $bin) $runArgs $out $stderr $root
        $ok = $true
        if (-not (Same-Bytes $out $want)) {
            Write-Output "fail|ui/$name|want=$(Head-Text $want)|got=$(Head-Text $out)"
            $script:fails++
            $ok = $false
        }
        if ((Test-Path -LiteralPath $stderr) -and ((Get-Item -LiteralPath $stderr).Length -gt 0)) {
            Write-Output "fail|ui/$name|stderr|got=$(Head-Text $stderr)"
            $script:fails++
            $ok = $false
        }
        if ($code -ne 0) {
            Write-Output "fail|ui/$name|code=$code"
            $script:fails++
            $ok = $false
        }
        if ((Test-Path -LiteralPath $ppm) -and -not (Same-Bytes $shot $ppm)) {
            Write-Output "fail|ui/$name|ppm|want=$(Head-Text $ppm)|got=$(Head-Text $shot)"
            $script:fails++
            $ok = $false
        }
        if ($ok) { Write-Output "ok|ui/$name" }
    }

    $checkBin = Join-Path $tmp 'llr-check'
    $checkErr = Join-Path $tmp 'llr-check.err'
    $checkBuild = Invoke-Native $llrc @('build', (Join-Path $root 'tools\llr-check.llr'), '-o', $checkBin) $null $checkErr $root
    if ($checkBuild -ne 0) {
        Write-Output "fail|setup|llr-check=$(Head-Text $checkErr)"
        exit 1
    }

    Get-ChildItem -LiteralPath (Join-Path $root 'tests\err') -Filter '*.llr' | ForEach-Object {
        $name = $_.BaseName
        $want = Join-Path $_.DirectoryName "$name.err"
        if (-not (Test-Path -LiteralPath $want)) { return }
        $got = Join-Path $tmp "$name.err"
        Invoke-Native $llrc @('check', "tests/err/$name.llr") $null $got $root | Out-Null
        if (Same-Bytes $got $want) {
            Write-Output "ok|err/$name"
        } else {
            Write-Output "fail|err/$name|want=$(Head-Text $want)|got=$(Head-Text $got)"
            $script:fails++
        }
        $wantText = [System.IO.File]::ReadAllText($want)
        if ($wantText -notmatch '\|(borrow|move|init)\.|name\.dup|name\.not_module|name\.not_type|flow\.brk_outside|io\.read') {
            $lerr = Join-Path $tmp "$name.lerr"
            Invoke-Native (Exe-Path $checkBin) @("tests/err/$name.llr") $null $lerr $root | Out-Null
            if (Same-Bytes $lerr $want) {
                Write-Output "ok|err/$name|llr-check"
            } else {
                Write-Output "fail|err/$name|llr-check|want=$(Head-Text $want)|got=$(Head-Text $lerr)"
                $script:fails++
            }
        }
    }

    Get-ChildItem -LiteralPath (Join-Path $root 'tests\panic') -Filter '*.llr' | ForEach-Object {
        $name = $_.BaseName
        $want = Join-Path $_.DirectoryName "$name.err"
        if (-not (Test-Path -LiteralPath $want)) { return }
        $flags = @('--std', 'std') + (Read-Flags (Join-Path $_.DirectoryName "$name.flags"))
        $bin = Join-Path $tmp "panic_$name"
        $cerr = Join-Path $tmp "panic_$name.cerr"
        $args = @('build', "tests/panic/$name.llr") + $flags + @('-o', $bin)
        $build = Invoke-Native $llrc $args $null $cerr $root
        if ($build -ne 0) {
            Write-Output "fail|panic/$name|compile|$(Head-Text $cerr)"
            $script:fails++
            return
        }
        $err = Join-Path $tmp "panic_$name.err"
        $code = Invoke-Native (Exe-Path $bin) @() $null $err $root
        if ($code -eq 101 -and (Same-Bytes $err $want)) {
            Write-Output "ok|panic/$name"
        } else {
            Write-Output "fail|panic/$name|code=$code|want=$(Head-Text $want)|got=$(Head-Text $err)"
            $script:fails++
        }
    }

    Get-ChildItem -LiteralPath (Join-Path $root 'tests\warn') -Filter '*.llr' | ForEach-Object {
        $name = $_.BaseName
        $want = Join-Path $_.DirectoryName "$name.err"
        if (-not (Test-Path -LiteralPath $want)) { return }
        $flags = Read-Flags (Join-Path $_.DirectoryName "$name.flags")
        $got = Join-Path $tmp "warn_$name.err"
        $args = @('check', "tests/warn/$name.llr") + $flags
        $code = Invoke-Native $llrc $args $null $got $root
        $codeGot = Join-Path $tmp "warn_$name.code"
        [System.IO.File]::WriteAllText($codeGot, "$code`n", $utf8)
        $ok = $true
        if (-not (Same-Bytes $got $want)) {
            Write-Output "fail|warn/$name|want=$(Head-Text $want)|got=$(Head-Text $got)"
            $script:fails++
            $ok = $false
        }
        $codeWant = Join-Path $_.DirectoryName "$name.code"
        if (Test-Path -LiteralPath $codeWant) {
            if (-not (Same-Bytes $codeGot $codeWant)) {
                Write-Output "fail|warn/$name|code=$code|want=$(Head-Text $codeWant)"
                $script:fails++
                $ok = $false
            }
        } elseif ($code -ne 0) {
            Write-Output "fail|warn/$name|code=$code"
            $script:fails++
            $ok = $false
        }
        if ($ok) { Write-Output "ok|warn/$name" }
    }

    foreach ($dirName in @('std', 'ui')) {
    Get-ChildItem -LiteralPath (Join-Path $root $dirName) -Filter '*.llr' | ForEach-Object {
        $text = [System.IO.File]::ReadAllText($_.FullName)
        if ($text -notmatch '(?m)^\(test ') { return }
        $name = $_.BaseName
        $tag = if ($dirName -eq 'ui') { "ui-mod/$name" } else { "std/$name" }
        $bin = Join-Path $tmp "${dirName}_$name"
        $err = Join-Path $tmp "${dirName}_$name.err"
        $buildArgs = @('build', $_.FullName, '--test', '-o', $bin)
        if ($dirName -eq 'ui') { $buildArgs = @('build', $_.FullName, '--test', '--std', 'std', '--ui', 'ui', '-o', $bin) }
        $build = Invoke-Native $llrc $buildArgs $null $err $root
        if ($build -ne 0) {
            Write-Output "fail|$tag|compile|$(Head-Text $err)"
            $script:fails++
            return
        }
        $out = Join-Path $tmp "${dirName}_$name.out"
        $stderr = Join-Path $tmp "${dirName}_$name.stderr"
        $code = Invoke-Native (Exe-Path $bin) @() $out $stderr $root
        if ($code -eq 0) {
            Write-Output "ok|$tag"
        } else {
            $both = Join-Path $tmp "${dirName}_$name.both"
            $bytes = New-Object System.Collections.Generic.List[byte]
            foreach ($p in @($out, $stderr)) {
                if (Test-Path -LiteralPath $p) {
                    $bytes.AddRange([System.IO.File]::ReadAllBytes($p))
                }
            }
            [System.IO.File]::WriteAllBytes($both, $bytes.ToArray())
            $lines = @([System.IO.File]::ReadAllLines($both) | Where-Object { $_ -match '^(fail|F)\|' } | Select-Object -First 3)
            Write-Output "fail|$tag|$($lines -join ' ')"
            $script:fails++
        }
    }
    }

    $lineFail = $false
    foreach ($dir in @('compiler\src', 'std', 'ui', 'web', 'tools', 'docs', 'scripts', 'tests', 'self', 'runtime')) {
        $full = Join-Path $root $dir
        if (-not (Test-Path -LiteralPath $full)) { continue }
        Get-ChildItem -LiteralPath $full -Recurse -File | Where-Object { $_.Extension -in '.rs', '.llr', '.md', '.sh', '.c', '.m' } | ForEach-Object {
            $body = [System.IO.File]::ReadAllText($_.FullName)
            $n = ([regex]::Matches($body, "`n")).Count
            $rel = $_.FullName.Substring($root.Length).TrimStart('\').Replace('\', '/')
            if ($n -ge 1000) {
                Write-Output "fail|lines|file=$rel|lines=$n|max=999"
                $script:lineFail = $true
            } elseif ($n -ge 500) {
                Write-Output "W|lines|file=$rel|lines=$n|fix=split under 500"
            }
        }
    }
    if (-not $lineFail) { Write-Output 'ok|lines' } else { $fails++ }
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

function Restore-Llrc($saved) {
    if ($null -eq $saved) {
        Remove-Item Env:LLRC -ErrorAction SilentlyContinue
    } else {
        $env:LLRC = $saved
    }
}

$stage0 = Join-Path $root 'target\release\llrc.exe'
if (-not (Test-Path -LiteralPath $stage0)) { $stage0 = Join-Path $root 'target\debug\llrc.exe' }
if (-not (Test-Path -LiteralPath $stage0)) {
    Write-Output 'fail|setup|stage0=not found|fix=cargo build --release'
    $fails++
} else {
    $same = [System.IO.Path]::GetFullPath($llrc) -eq [System.IO.Path]::GetFullPath($stage0)
    foreach ($diff in @('tests\check.ps1', 'tests\mir.ps1')) {
        $saved = $env:LLRC
        $env:LLRC = $stage0
        $script = Join-Path $root $diff
        if ($same) {
            & powershell.exe -NoProfile -File $script
        } else {
            & powershell.exe -NoProfile -File $script $llrc
        }
        if ($LASTEXITCODE -ne 0) { $fails++ }
        Restore-Llrc $saved
    }
    $saved = $env:LLRC
    $env:LLRC = $stage0
    & powershell.exe -NoProfile -File (Join-Path $root 'tests\self.ps1')
    if ($LASTEXITCODE -ne 0) { $fails++ }
    Restore-Llrc $saved
    $bash = Get-Command bash -ErrorAction SilentlyContinue
    if (-not $bash) {
        Write-Output 'W|cross|skipped|fix=bash file ld.lld clang'
    } else {
        $saved = $env:LLRC
        $slash = $stage0 -replace '\\', '/'
        $crossLf = Join-Path $root 'target\cross.sh'
        $crossText = [System.IO.File]::ReadAllText((Join-Path $root 'tests\cross.sh')) -replace "`r", ''
        [System.IO.File]::WriteAllText($crossLf, $crossText)
        Push-Location $root
        & bash -c "LLRC=`$(wslpath -u '$slash' 2>/dev/null || cygpath -u '$slash') bash ./target/cross.sh"
        $crossCode = $LASTEXITCODE
        Pop-Location
        if ($crossCode -ne 0) { $fails++ }
        Restore-Llrc $saved
    }
}

Write-Output "done|fails=$fails"
if ($fails -ne 0) { exit 1 }
exit 0
