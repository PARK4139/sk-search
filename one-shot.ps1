& (Join-Path $PSScriptRoot 'cores/scripts/one_shot/launch.ps1') -Entry (Join-Path $PSScriptRoot 'one_shot.py') -Arguments $args
exit $LASTEXITCODE