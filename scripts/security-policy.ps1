& (Join-Path $PSScriptRoot 'launch.ps1') -Module 'skim_search.diagnostics.security_policy' -Arguments $args
exit $LASTEXITCODE
