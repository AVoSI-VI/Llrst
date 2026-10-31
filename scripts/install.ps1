# Builds stage-0 llrc, installs llrs2 as llrc, compiles every tool, verifies.
# Usage: scripts/install.ps1 [PREFIX]   (default ~/.local)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$prefix = if ($args.Count -gt 0 -and $args[0]) { $args[0] } else { Join-Path $env:USERPROFILE '.local' }
$bin = Join-Path $prefix 'bin'
$share = Join-Path $prefix 'share\llrst'

function Die($msg) {
    Write-Output "fail|$msg"
    exit 1
}

function Compile-Shim($dir, $name) {
    $c = Join-Path $dir $name
    $obj = Join-Path $dir ($name -replace '\.c$', '.obj')
    $cl = Get-Command cl -ErrorAction SilentlyContinue
    if ($cl) {
        & cl.exe /nologo /c "/Fo$obj" $c
        if ($LASTEXITCODE -ne 0) { Die "setup|cc=$name" }
        return
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { Die 'setup|cl=missing' }
    $inst = (& $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1)
    if (-not $inst) { Die 'setup|cl=missing' }
    $bat = Join-Path $inst.Trim() 'VC\Auxiliary\Build\vcvars64.bat'
    cmd /c "call `"$bat`" >nul && cl /nologo /c /Fo`"$obj`" `"$c`""
    if ($LASTEXITCODE -ne 0) { Die "setup|cc=$name" }
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
$exDest = Join-Path $share 'examples'
if (Test-Path -LiteralPath $exDest) { Remove-Item -LiteralPath $exDest -Recurse -Force }
Copy-Item -LiteralPath (Join-Path $root 'examples') -Destination $exDest -Recurse
foreach ($name in @('llr_wsa.c', 'llr_win.c', 'llr_win_sock.c', 'llr_win_proc.c')) {
    Copy-Item -LiteralPath (Join-Path $root "runtime\$name") -Destination (Join-Path $share $name) -Force
}
Write-Output 'step|runtime'
foreach ($name in @('llr_wsa.c', 'llr_win.c', 'llr_win_sock.c', 'llr_win_proc.c')) {
    Compile-Shim $share $name
}
Write-Output "ok|runtime|$share"
$savedLlrc = $env:LLRC
$env:LLRC = $llrc
& powershell.exe -NoProfile -File (Join-Path $PSScriptRoot 'self.ps1') --stage
$stageCode = $LASTEXITCODE
if ($null -eq $savedLlrc) {
    Remove-Item Env:LLRC -ErrorAction SilentlyContinue
} else {
    $env:LLRC = $savedLlrc
}
if ($stageCode -ne 0) { exit $stageCode }
$llrs2 = Join-Path $root 'target\self\llrs2.exe'
Copy-Item -LiteralPath $llrs2 -Destination (Join-Path $bin 'llrc.exe') -Force
Write-Output "ok|llrs2|$bin\llrc.exe"
Write-Output "ok|std|$stdDest"
Remove-Item Env:LLRST_RUNTIME -ErrorAction SilentlyContinue
$env:LLRST_STD = $stdDest
$tools = @(Get-ChildItem -LiteralPath (Join-Path $root 'tools') -Filter 'llr-*.llr')
foreach ($src in $tools) {
    $name = $src.BaseName
    Write-Output "step|build-$name"
    $err = Join-Path ([System.IO.Path]::GetTempPath()) "llr_install_$name.err"
    $pref = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & (Join-Path $bin 'llrc.exe') build $src.FullName -o (Join-Path $bin $name) 2>$err
    $code = $LASTEXITCODE
    $ErrorActionPreference = $pref
    if ($code -ne 0) {
        Get-Content -LiteralPath $err -ErrorAction SilentlyContinue
        Die "build-$name"
    }
    Write-Output "ok|$name|$bin\$name.exe"
}

Write-Output 'step|verify'
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('llr-install-' + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $hello = Join-Path $root 'examples\hello.llr'
    $pref = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & (Join-Path $bin 'llrc.exe') build $hello -o (Join-Path $tmp 'hello')
    $code = $LASTEXITCODE
    $ErrorActionPreference = $pref
    if ($code -ne 0) { Die 'verify|compile hello' }
    $out = (& (Join-Path $tmp 'hello.exe') | Out-String).Trim()
    if ($out -ne 'hello world') { Die "verify|want=hello world|got=$out" }
    & (Join-Path $bin 'llr-grep.exe') 'fn main' (Join-Path $root 'examples') | Out-Null
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
