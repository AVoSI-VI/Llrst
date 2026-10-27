# Diff llrs check stderr against llrc check.
# Usage: powershell -File tests/check.ps1 [LLRS]
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$llrc = if ($env:LLRC) { $env:LLRC } else { Join-Path $root "target\release\llrc.exe" }
if (-not (Test-Path $llrc)) { $llrc = Join-Path $root "target\debug\llrc.exe" }
if (-not (Test-Path $llrc)) {
  Write-Output "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
}
$env:LLRST_STD = Join-Path $root "std"
$tmp = Join-Path $env:TEMP ("llrs-check-" + [guid]::NewGuid().ToString("n"))
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
foreach ($dir in @("err", "warn")) {
  foreach ($f in Get-ChildItem (Join-Path $root "tests\$dir\*.llr")) {
    $name = $f.BaseName
    $golden = Join-Path $f.DirectoryName "$name.err"
    if (-not (Test-Path $golden)) { continue }
    $flags = @()
    $flagFile = Join-Path $f.DirectoryName "$name.flags"
    if (Test-Path $flagFile) {
      $line = ([IO.File]::ReadAllText($flagFile)).Trim()
      if ($line -ne "") { $flags = $line.Split(" ", [StringSplitOptions]::RemoveEmptyEntries) }
    }
    $rel = "tests/$dir/$name.llr"
    Push-Location $root
    $arg = @("check", $rel) + $flags
    $sarg = @("check", $rel) + $flags
    cmd /c "`"$llrc`" $($arg -join ' ') > nul 2> `"$tmp\want`""
    cmd /c "`"$stage`" $($sarg -join ' ') > nul 2> `"$tmp\got`""
    Pop-Location
    $want = ""
    $got = ""
    if (Test-Path "$tmp\want") { $want = ([IO.File]::ReadAllText("$tmp\want") -replace "`r", "") }
    if (Test-Path "$tmp\got") { $got = ([IO.File]::ReadAllText("$tmp\got") -replace "`r", "") }
    if ($null -eq $want) { $want = "" }
    if ($null -eq $got) { $got = "" }
    if ($want -eq $got) {
      Write-Output "ok|check/$rel"
    } else {
      $w = ($want -replace "`n", " ")
      $g = ($got -replace "`n", " ")
      if ($w.Length -gt 180) { $w = $w.Substring(0, 180) }
      if ($g.Length -gt 180) { $g = $g.Substring(0, 180) }
      Write-Output "fail|check/$rel|want=$w|got=$g"
      $fails++
    }
  }
}
Write-Output "done|fails=$fails"
Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
if ($fails -ne 0) { exit 1 }
