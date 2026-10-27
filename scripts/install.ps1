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

function Invoke-Stage($name, $comp) {
    Write-Output "step|$name"
    foreach ($leaf in @('llrs', 'llrs.exe', 'llrs.o', 'llrs.obj')) {
        $p = Join-Path $work $leaf
        if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Force }
    }
    $err = Join-Path $out "$name.err"
    $pref = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & $comp build (Join-Path $root 'self\llrs.llr') -o (Join-Path $work 'llrs') --std (Join-Path $root 'std') 2>$err
    $code = $LASTEXITCODE
    $ErrorActionPreference = $pref
    if ($code -ne 0) {
        $msg = Get-Content -LiteralPath $err -TotalCount 1 -ErrorAction SilentlyContinue
        if (-not $msg) { $msg = 'build failed' }
        Die "$name|$msg"
    }
    $src = Join-Path $work 'llrs.exe'
    if (-not (Test-Path -LiteralPath $src)) { $src = Join-Path $work 'llrs' }
    if (-not (Test-Path -LiteralPath $src)) { Die "$name|binary=missing|path=$work\llrs" }
    $dest = Join-Path $out "$name.exe"
    Copy-Item -LiteralPath $src -Destination $dest -Force
    Write-Output "ok|$name|$dest"
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
Write-Output 'step|runtime'
$runtime = Join-Path $root 'runtime'
foreach ($name in @('llr_wsa.c', 'llr_win.c')) {
    Compile-Shim $share $name
    Compile-Shim $runtime $name
}
Write-Output "ok|runtime|$share"
$out = Join-Path $root 'target\self'
$work = Join-Path $out 'work'
New-Item -ItemType Directory -Force -Path $work | Out-Null
foreach ($name in @('llrs0', 'llrs1', 'llrs2')) {
    foreach ($ext in @('', '.exe')) {
        $p = Join-Path $out "$name$ext"
        if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Force }
    }
}
$env:LLRST_STD = Join-Path $root 'std'
$env:LLRST_RUNTIME = $runtime
Invoke-Stage 'llrs0' $llrc
Invoke-Stage 'llrs1' (Join-Path $out 'llrs0.exe')
Invoke-Stage 'llrs2' (Join-Path $out 'llrs1.exe')
$llrs1 = Join-Path $out 'llrs1.exe'
$llrs2 = Join-Path $out 'llrs2.exe'
$h1 = (Get-FileHash -Algorithm SHA256 -LiteralPath $llrs1).Hash
$h2 = (Get-FileHash -Algorithm SHA256 -LiteralPath $llrs2).Hash
if ($h1 -ne $h2) { Die 'stage2|llrs1!=llrs2' }
Write-Output 'self|stage2=equal'
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
    $hello = Join-Path $tmp 'hello.llr'
    [System.IO.File]::WriteAllText($hello, "(use std.io)`n(fn main () unit (io.puts `"hello world`"))`n")
    $pref = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    & (Join-Path $bin 'llrc.exe') build $hello -o (Join-Path $tmp 'hello')
    $code = $LASTEXITCODE
    $ErrorActionPreference = $pref
    if ($code -ne 0) { Die 'verify|compile hello' }
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
