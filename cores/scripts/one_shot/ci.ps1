& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'ci.py') -Arguments $args
exit $LASTEXITCODE
