& (Join-Path $PSScriptRoot 'launch.ps1') -Entry (Join-Path $PSScriptRoot 'security_policy.py') -Arguments $args
exit $LASTEXITCODE
