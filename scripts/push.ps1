& (Join-Path $PSScriptRoot 'launch.ps1') -Module 'skim_search.diagnostics.one_shot.push' -Arguments $args
exit $LASTEXITCODE
