& (Join-Path $PSScriptRoot 'one_shot/launch.ps1') -Module 'skim_search.diagnostics.one_shot' -Arguments $args
exit $LASTEXITCODE
