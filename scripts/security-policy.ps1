& (Join-Path $PSScriptRoot 'one_shot/launch.ps1') -Module 'skim_search.diagnostics.security_policy' -Arguments $args
exit $LASTEXITCODE
