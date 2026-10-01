& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'security.py') -Arguments $args
exit $LASTEXITCODE
