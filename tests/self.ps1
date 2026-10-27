# Diff a stage compiler against llrc check on every tests/err and tests/warn golden.
# Usage: powershell -File tests/self.ps1 [STAGE]
$ErrorActionPreference = 'Continue'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

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

function Err-Text($path) {
    if (-not (Test-Path -LiteralPath $path)) { return '' }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    return ($text -replace "`r", '')
}

function Same-Err($a, $b) {
    return (Err-Text $a) -ceq (Err-Text $b)
}

function Head-Text($path) {
    $text = Err-Text $path
    if ($text.Length -gt 200) { $text = $text.Substring(0, 200) }
    return ($text -replace "`n", ' ')
}

function Read-Flags($path) {
    if (-not (Test-Path -LiteralPath $path)) { return @() }
    $line = [System.IO.File]::ReadAllText($path).Trim()
    if ($line -eq '') { return @() }
    return @($line.Split(' ', [System.StringSplitOptions]::RemoveEmptyEntries))
}

function Exe-Path($path) {
    if ($path.EndsWith('.exe')) { return $path }
    if (Test-Path -LiteralPath "$path.exe") { return "$path.exe" }
    return $path
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
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('llr-self-' + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp | Out-Null
$fails = 0

try {
    if ($args.Count -ge 1) {
        $stage = $args[0]
    } elseif ($env:STAGE) {
        $stage = $env:STAGE
    } else {
        $checkBin = Join-Path $tmp 'llr-check'
        $checkErr = Join-Path $tmp 'llr-check.err'
        $built = Invoke-Native $llrc @('build', (Join-Path $root 'tools\llr-check.llr'), '-o', $checkBin) $null $checkErr $root
        if ($built -ne 0) {
            Write-Output "fail|setup|llr-check=$(Head-Text $checkErr)"
            exit 1
        }
        $stage = Exe-Path $checkBin
    }
    if (-not (Test-Path -LiteralPath $stage)) {
        Write-Output "fail|setup|stage=not found|path=$stage"
        exit 1
    }
    foreach ($dir in @('err', 'warn')) {
        Get-ChildItem -LiteralPath (Join-Path $root "tests\$dir") -Filter '*.llr' | ForEach-Object {
            $name = $_.BaseName
            $golden = Join-Path $_.DirectoryName "$name.err"
            if (-not (Test-Path -LiteralPath $golden)) { return }
            $flags = Read-Flags (Join-Path $_.DirectoryName "$name.flags")
            $rel = "tests/$dir/$name.llr"
            $want = Join-Path $tmp "$dir-$name.want"
            $got = Join-Path $tmp "$dir-$name.got"
            Invoke-Native $llrc (@('check', $rel) + $flags) $null $want $root | Out-Null
            Invoke-Native $stage (@($rel) + $flags) $null $got $root | Out-Null
            if (Same-Err $want $got) {
                Write-Output "ok|self/$dir/$name"
            } else {
                Write-Output "fail|self/$dir/$name|want=$(Head-Text $want)|got=$(Head-Text $got)"
                $script:fails++
            }
        }
    }
    Write-Output "done|fails=$fails"
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
if ($fails -ne 0) { exit 1 }
