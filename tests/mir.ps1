# Diff llrs emit-mir against llrc emit-mir.
# Usage: powershell -File tests/mir.ps1 [LLRS]
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$llrc = if ($env:LLRC) { $env:LLRC } else { Join-Path $root "target\release\llrc.exe" }
if (-not (Test-Path $llrc)) { $llrc = Join-Path $root "target\debug\llrc.exe" }
if (-not (Test-Path $llrc)) {
  Write-Output "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
}
$env:LLRST_STD = Join-Path $root "std"
$tmp = Join-Path $env:TEMP ("llrs-mir-" + [guid]::NewGuid().ToString("n"))
New-Item -ItemType Directory -Path $tmp | Out-Null
$stage = if ($args.Count -gt 0) { $args[0] } else { "" }
if ($stage -eq "") {
  $buildOut = Join-Path $tmp "llrs.exe"
  $buildErr = Join-Path $tmp "llrs.err"
  cmd /c "`"$llrc`" build `"$(Join-Path $root 'self\llrs.llr')`" -o `"$buildOut`" --std `"$env:LLRST_STD`" > `"$buildErr`" 2>&1"
  if ($LASTEXITCODE -ne 0) {
    $err = Get-Content "$tmp\llrs.err" -TotalCount 1
    Write-Output "fail|setup|llrs=$err"
    exit 1
  }
  $stage = Join-Path $tmp "llrs.exe"
}
if (-not (Test-Path $stage)) {
  Write-Output "fail|setup|stage=not found|path=$stage"
  exit 1
}
$fails = 0
$files = @()
$files += Get-ChildItem (Join-Path $root "tests\run\*.llr")
$files += Get-ChildItem (Join-Path $root "std\*.llr")
foreach ($f in $files) {
  $rel = $f.FullName.Substring($root.Length + 1).Replace("\", "/")
  Push-Location $root
  cmd /c "`"$llrc`" emit-mir `"$rel`" --std `"$env:LLRST_STD`" > `"$tmp\want`" 2> `"$tmp\want.err`""
  cmd /c "`"$stage`" emit-mir `"$rel`" > `"$tmp\got`" 2> `"$tmp\got.err`""
  Pop-Location
  $want = ""
  $got = ""
  if (Test-Path "$tmp\want") { $want = ([IO.File]::ReadAllText("$tmp\want") -replace "`r", "") }
  if (Test-Path "$tmp\got") { $got = ([IO.File]::ReadAllText("$tmp\got") -replace "`r", "") }
  $wantErr = ""
  $gotErr = ""
  if (Test-Path "$tmp\want.err") { $wantErr = ([IO.File]::ReadAllText("$tmp\want.err") -replace "`r", "") }
  if (Test-Path "$tmp\got.err") { $gotErr = ([IO.File]::ReadAllText("$tmp\got.err") -replace "`r", "") }
  if ($null -eq $want) { $want = "" }
  if ($null -eq $got) { $got = "" }
  if ($null -eq $wantErr) { $wantErr = "" }
  if ($null -eq $gotErr) { $gotErr = "" }
  if ($want -eq $got -and $wantErr -eq $gotErr) {
    Write-Output "ok|mir/$rel"
  } else {
    $w = ($want -replace "`n", " ")
    $g = ($got -replace "`n", " ")
    if ($w.Length -gt 180) { $w = $w.Substring(0, 180) }
    if ($g.Length -gt 180) { $g = $g.Substring(0, 180) }
    Write-Output "fail|mir/$rel|want=$w|got=$g"
    $fails++
  }
}
Write-Output "done|fails=$fails"
if ($fails -ne 0) { exit 1 }
