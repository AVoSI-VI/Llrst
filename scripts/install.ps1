# Builds llrc, compiles every tool in tools/, installs binaries and std, verifies.
# Usage: scripts/install.ps1 [PREFIX]   (default ~/.local)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$prefix = if ($args.Count -gt 0 -and $args[0]) { $args[0] } else { Join-Path $env:USERPROFILE '.local' }
$bin = Join-Path $prefix 'bin'
$share = Join-Path $prefix 'share\llrst'

function Die($msg) {
    Write-Output "fail|$msg"
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Die 'setup|cargo=missing|fix=install rustup'
}
Write-Output 'step|build-llrc'
Push-Location $root
& cargo build --release -q
if ($LASTEXITCODE -ne 0) { Pop-Location; Die 'build-llrc|cargo build --release' }
$meta = & cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
Pop-Location
$target = if ($meta.target_directory) { $meta.target_directory } else { Join-Path $root 'target' }
$llrc = Join-Path $target 'release\llrc.exe'
if (-not (Test-Path -LiteralPath $llrc)) { Die "build-llrc|binary=missing|path=$llrc" }

New-Item -ItemType Directory -Force -Path $bin, $share | Out-Null
$stdDest = Join-Path $share 'std'
if (Test-Path -LiteralPath $stdDest) { Remove-Item -LiteralPath $stdDest -Recurse -Force }
Copy-Item -LiteralPath (Join-Path $root 'std') -Destination $stdDest -Recurse
Copy-Item -LiteralPath (Join-Path $root 'runtime\llr_win.c') -Destination (Join-Path $share 'llr_win.c') -Force
Copy-Item -LiteralPath (Join-Path $root 'runtime\llr_wsa.c') -Destination (Join-Path $share 'llr_wsa.c') -Force
Copy-Item -LiteralPath $llrc -Destination (Join-Path $bin 'llrc.exe') -Force
Write-Output "ok|llrc|$bin\llrc.exe"
Write-Output "ok|std|$stdDest"

$env:LLRST_STD = $stdDest
$tools = @(Get-ChildItem -LiteralPath (Join-Path $root 'tools') -Filter 'llr-*.llr')
foreach ($src in $tools) {
    $name = $src.BaseName
    Write-Output "step|build-$name"
    $err = Join-Path ([System.IO.Path]::GetTempPath()) "llr_install_$name.err"
    & (Join-Path $bin 'llrc.exe') build $src.FullName -o (Join-Path $bin $name) 2>$err
    if ($LASTEXITCODE -ne 0) {
        Get-Content -LiteralPath $err -ErrorAction SilentlyContinue
        Die "build-$name"
    }
    Write-Output "ok|$name|$bin\$name.exe"
}

Write-Output 'step|verify'
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('llr-install-' + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $hello = Join-Path $tmp 'hello.llr'
    [System.IO.File]::WriteAllText($hello, "(use std.io)`n(fn main () unit (io.puts `"hello world`"))`n")
    & (Join-Path $bin 'llrc.exe') build $hello -o (Join-Path $tmp 'hello')
    if ($LASTEXITCODE -ne 0) { Die 'verify|compile hello' }
    $out = (& (Join-Path $tmp 'hello.exe') | Out-String).Trim()
    if ($out -ne 'hello world') { Die "verify|want=hello world|got=$out" }
    & (Join-Path $bin 'llr-grep.exe') 'fn main' $tmp | Out-Null
    if ($LASTEXITCODE -ne 0) { Die 'verify|llr-grep' }
} finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
Write-Output 'ok|verify|hello world'

$pathParts = $env:PATH -split ';'
if ($pathParts -notcontains $bin) {
    Write-Output "W|path|$bin is not on PATH|fix=add it to PATH"
}
Write-Output "done|prefix=$prefix|tools=$($tools.Count)"
