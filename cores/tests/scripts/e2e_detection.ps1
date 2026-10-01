# e2e for search_engine/rg-sk-executable-detection/5ea570b8: cases A (configured), B (PATH), C (missing).
$root = "%USERPROFILE%\Downloads\CavemanDrive\backlog\sk_search"
$tp = "%USERPROFILE%\Downloads\CavemanDrive\3rd_party"
$cap = Join-Path $PSScriptRoot "run_capture.ps1"
$shots = "$root\ref\actual\screenshot\frames"
$exe = "$root\cores\target\debug\skim-search.exe"
$log = "$root\ref\actual\logs\skim-search.log"
# Always rebuild: `cargo test` does not refresh target\debug\skim-search.exe.
Push-Location "$root\cores"; cargo build; $built = $LASTEXITCODE; Pop-Location
if ($built -ne 0) { "BUILD_FAILED"; exit 1 }

$tmp = Join-Path $env:TEMP "skim-search-e2e"
New-Item -ItemType Directory -Force $tmp | Out-Null
$origPath = $env:PATH
$sys = "$env:SystemRoot\system32;$env:SystemRoot"

# Case A: configured paths via settings file
$settings = Join-Path $tmp "settings.json"
@{ rg_path = "$tp\ripgrep\rg.exe"; sk_path = "$tp\skim\sk.exe" } | ConvertTo-Json | Out-File -Encoding ascii $settings
$env:SKIM_SEARCH_SETTINGS = $settings
$env:PATH = $sys
"== case A"; & $cap -Exe $exe -Out "$shots\rg-sk-executable-detection_100_caseA.png"

# Case B: PATH only
$env:SKIM_SEARCH_SETTINGS = Join-Path $tmp "none.json"
$env:PATH = "$tp\ripgrep;$tp\skim;$sys"
"== case B"; & $cap -Exe $exe -Out "$shots\rg-sk-executable-detection_100_caseB.png"

# Case C: none. exe copied outside the repo so the fallback cannot reach 3rd_party.
$isolated = Join-Path $tmp "isolated"
New-Item -ItemType Directory -Force $isolated | Out-Null
Copy-Item $exe $isolated -Force
$env:PATH = $sys
"== case C"; & $cap -Exe "$isolated\skim-search.exe" -Out "$shots\rg-sk-executable-detection_100_caseC.png"

$env:PATH = $origPath
$env:SKIM_SEARCH_SETTINGS = $null
"== log"
Get-Content $log -Tail 24 -Encoding UTF8 | Where-Object { $_ -match '\[(app|settings|search_engine|toast)\]' }
