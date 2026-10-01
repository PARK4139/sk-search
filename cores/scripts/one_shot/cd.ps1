& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'cd.py') -Arguments $args
exit $LASTEXITCODE
