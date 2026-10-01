& (Join-Path $PSScriptRoot 'launch.ps1') -Module 'skim_search.diagnostics.one_shot.commit' -Arguments $args
exit $LASTEXITCODE
