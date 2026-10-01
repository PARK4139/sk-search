& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'commit.py') -Arguments $args
exit $LASTEXITCODE
