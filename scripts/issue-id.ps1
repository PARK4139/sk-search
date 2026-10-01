& (Join-Path $PSScriptRoot 'one_shot/launch.ps1') -Module 'skim_search.diagnostics.issue_ids' -Arguments $args
exit $LASTEXITCODE
