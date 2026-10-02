& (Join-Path $PSScriptRoot 'launch.ps1') -Module 'skim_search.diagnostics.issue_ids' -Arguments $args
exit $LASTEXITCODE
