& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'push.py') -Arguments $args
exit $LASTEXITCODE
