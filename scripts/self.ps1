# Stage bootstrap. llrc builds llrs0, llrs0 builds llrs1, llrs1 builds llrs2.
# Every stage is linked as the same path and then copied, so the output file
# name is not part of the image. The hashes must match, then the run suite uses llrs2.
# Usage: scripts/self.ps1 [--stage]
# --stage stops after self|stage2=equal and does not run the suite.
# Output: one pipe record per step. Exit 1 on failure.
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$stageOnly = $false
if ($args.Count -gt 0 -and $args[0] -eq '--stage') {
    $stageOnly = $true
} elseif ($args.Count -gt 0) {
    Write-Output "fail|setup|arg=$($args[0])|fix=scripts/self.ps1 [--stage]"
    exit 1
}

function Die($msg) {
    Write-Output "fail|$msg"
    exit 1
}

function Pick-Exe($path) {
    if ($path -and (Test-Path -LiteralPath $path)) { return (Resolve-Path -LiteralPath $path).Path }
    if ($path -and (Test-Path -LiteralPath "$path.exe")) { return (Resolve-Path -LiteralPath "$path.exe").Path }
    return $null
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

$llrc = Pick-Exe $env:LLRC
if (-not $llrc) { $llrc = Pick-Exe (Join-Path $root 'target\release\llrc') }
if (-not $llrc) { $llrc = Pick-Exe (Join-Path $root 'target\debug\llrc') }
if (-not $llrc) {
    $cmd = Get-Command llrc -ErrorAction SilentlyContinue
    if ($cmd) { $llrc = $cmd.Source }
}
if (-not $llrc -or -not (Test-Path -LiteralPath $llrc)) {
    Die 'setup|llrc=not found|fix=cargo build --release or set LLRC'
}

$runtime = Join-Path $root 'runtime'
foreach ($name in @('llr_wsa.c', 'llr_win.c', 'llr_win_sock.c', 'llr_win_proc.c')) {
    Compile-Shim $runtime $name
}

$env:LLRST_STD = Join-Path $root 'std'
$env:LLRST_RUNTIME = $runtime
$env:SOURCE_DATE_EPOCH = '0'
$out = Join-Path $root 'target\self'
$work = Join-Path $out 'work'
New-Item -ItemType Directory -Force -Path $work | Out-Null
foreach ($name in @('llrs0', 'llrs1', 'llrs2')) {
    foreach ($ext in @('', '.exe')) {
        $p = Join-Path $out "$name$ext"
        if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Force }
    }
}

Invoke-Stage 'llrs0' $llrc
Invoke-Stage 'llrs1' (Join-Path $out 'llrs0.exe')
Invoke-Stage 'llrs2' (Join-Path $out 'llrs1.exe')
$llrs1 = Join-Path $out 'llrs1.exe'
$llrs2 = Join-Path $out 'llrs2.exe'
$h1 = (Get-FileHash -Algorithm SHA256 -LiteralPath $llrs1).Hash
$h2 = (Get-FileHash -Algorithm SHA256 -LiteralPath $llrs2).Hash
if ($h1 -ne $h2) { Die 'stage2|llrs1!=llrs2' }
Write-Output 'self|stage2=equal'
Remove-Item Env:SOURCE_DATE_EPOCH -ErrorAction SilentlyContinue
if ($stageOnly) { exit 0 }

$env:LLRC = $llrs2
& powershell.exe -NoProfile -File (Join-Path $root 'tests\run.ps1')
exit $LASTEXITCODE
