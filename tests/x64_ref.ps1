# Snapshot of every x86-64 object self/ emits. --save DIR writes them.
# The default compares against target/self/x64_ref.
# LLRC defaults to target/self/llrs0.exe and is not rebuilt.
# An explicit LLRC is used as-is.
# Output: ok|x64/elf/NAME or fail|x64/coff/NAME|first=<offset>. Last line done|fails=N.
# Usage: powershell -File tests/x64_ref.ps1 [--save DIR]
$ErrorActionPreference = "Continue"
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
  $PSNativeCommandUseErrorActionPreference = $false
}
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root
$save = ""
if ($args.Count -gt 0 -and $args[0] -eq "--save") {
  if ($args.Count -lt 2 -or $args[1] -eq "") {
    Write-Output "fail|setup|arg=--save|fix=tests/x64_ref.ps1 [--save DIR]"
    exit 1
  }
  $save = $args[1]
} elseif ($args.Count -gt 0) {
  Write-Output "fail|setup|arg=$($args[0])|fix=tests/x64_ref.ps1 [--save DIR]"
  exit 1
}

$ref = Join-Path $root "target\self\x64_ref"
$log = Join-Path $env:TEMP ("llrs-x64log-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Path $log | Out-Null
$made = $false
if ($save -ne "") {
  $out = $save
  New-Item -ItemType Directory -Force -Path $out | Out-Null
  if (-not (Test-Path -LiteralPath $out)) {
    Write-Output "fail|setup|mkdir=$out"
    exit 1
  }
} else {
  $out = Join-Path $env:TEMP ("llrs-x64-" + [guid]::NewGuid().ToString("n"))
  New-Item -ItemType Directory -Path $out | Out-Null
  $made = $true
}

function Pick-Exe($path) {
  if ($path -and (Test-Path -LiteralPath $path)) { return (Resolve-Path -LiteralPath $path).Path }
  if ($path -and (Test-Path -LiteralPath "$path.exe")) { return (Resolve-Path -LiteralPath "$path.exe").Path }
  return $null
}

if ($env:LLRC) {
  $llrc = Pick-Exe $env:LLRC
  if (-not $llrc) {
    Write-Output "fail|setup|llrc=not found|path=$env:LLRC"
    exit 1
  }
} else {
  $llrc = Pick-Exe (Join-Path $root "target\self\llrs0")
  if (-not $llrc) {
    Write-Output "fail|setup|llrc=not found|path=$(Join-Path $root 'target\self\llrs0.exe')"
    exit 1
  }
}

$env:LLRST_STD = Join-Path $root "std"
$env:SOURCE_DATE_EPOCH = "0"
$std = Join-Path $root "std"
$script:fails = 0

if (-not ("LlrstX64Diff" -as [type])) {
  Add-Type -TypeDefinition @'
public static class LlrstX64Diff {
  public static long First(byte[] a, byte[] b) {
    int n = a.Length < b.Length ? a.Length : b.Length;
    for (int i = 0; i < n; i++) if (a[i] != b[i]) return (long)i + 1;
    if (a.Length != b.Length) return (long)n + 1;
    return 0;
  }
}
'@
}

function First-Diff($got, $want) {
  $a = [System.IO.File]::ReadAllBytes($got)
  $b = [System.IO.File]::ReadAllBytes($want)
  $off = [LlrstX64Diff]::First($a, $b)
  if ($off -eq 0) { return $null }
  return [string]$off
}

function Emit($kind, $name, $src, $testmode) {
  if ($kind -eq "elf") {
    $triple = "x86_64-unknown-linux-gnu"
    $ext = ".o"
  } else {
    $triple = "x86_64-pc-windows-gnu"
    $ext = ".obj"
  }
  $rel = ($kind + "/" + $name) -replace "/", "\"
  $base = Join-Path $out $rel
  $dir = Split-Path -Parent $base
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  if (-not (Test-Path -LiteralPath $dir)) {
    Write-Output "fail|x64/$kind/$name|mkdir"
    $script:fails++
    return
  }
  $errFile = Join-Path $log "err"
  if (Test-Path -LiteralPath $errFile) { Remove-Item -LiteralPath $errFile -Force }
  $testArg = ""
  if ($testmode -eq 1) { $testArg = "--test " }
  cmd /c "`"$llrc`" build `"$src`" --emit-obj --target $triple --std `"$std`" ${testArg}-o `"$base`" >nul 2> `"$errFile`""
  if ($LASTEXITCODE -ne 0) {
    $msg = "build failed"
    if (Test-Path -LiteralPath $errFile) {
      $line = Get-Content -LiteralPath $errFile -TotalCount 1 -ErrorAction SilentlyContinue
      if ($line) { $msg = "$line" }
    }
    Write-Output "fail|x64/$kind/$name|$msg"
    $script:fails++
    return
  }
  $got = $base + $ext
  if (-not (Test-Path -LiteralPath $got)) {
    Write-Output "fail|x64/$kind/$name|object=missing|path=$got"
    $script:fails++
    return
  }
  if ($save -ne "") {
    Write-Output "ok|x64/$kind/$name"
    return
  }
  $wantRel = ($kind + "/" + $name + $ext) -replace "/", "\"
  $want = Join-Path $ref $wantRel
  if (-not (Test-Path -LiteralPath $want)) {
    Write-Output "fail|x64/$kind/$name|missing"
    $script:fails++
    return
  }
  $off = First-Diff $got $want
  if ($null -eq $off) {
    Write-Output "ok|x64/$kind/$name"
    return
  }
  Write-Output "fail|x64/$kind/$name|first=$off"
  $script:fails++
}

foreach ($f in (Get-ChildItem (Join-Path $root "tests\run\*.llr") | Sort-Object Name)) {
  $hit = Select-String -LiteralPath $f.FullName -Pattern '^\(fn main' -Quiet
  if (-not $hit) { continue }
  $name = "run/" + $f.BaseName
  Emit "elf" $name $f.FullName 0
  Emit "coff" $name $f.FullName 0
}
foreach ($f in (Get-ChildItem (Join-Path $root "std\*.llr") | Sort-Object Name)) {
  $hit = Select-String -LiteralPath $f.FullName -Pattern '^\(test ' -Quiet
  if (-not $hit) { continue }
  $name = "std/" + $f.BaseName
  Emit "elf" $name $f.FullName 1
  Emit "coff" $name $f.FullName 1
}

# The compiler image records source paths as given. The snapshot uses a relative path.
Emit "elf" "self/llrs" "self/llrs.llr" 0
Emit "coff" "self/llrs" "self/llrs.llr" 0

Write-Output "done|fails=$script:fails"
Remove-Item -LiteralPath $log -Recurse -Force -ErrorAction SilentlyContinue
if ($made) { Remove-Item -LiteralPath $out -Recurse -Force -ErrorAction SilentlyContinue }
if ($script:fails -ne 0) { exit 1 }
