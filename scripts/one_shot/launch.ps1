param([Parameter(Mandatory=$true)][string]$Module, [string[]]$Arguments)
# Thin bridge: run a skim_search module in the uv project cores/python (logic lives in Python).
$ErrorActionPreference = 'Stop'
try {
    $repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
    $uv = (Get-Command uv -ErrorAction SilentlyContinue).Source
    $ancestor = [System.IO.DirectoryInfo]$repo
    while (-not $uv -and $ancestor) {
        $candidate = Join-Path $ancestor.FullName '3rd_party/pk_system/uv.exe'
        if (Test-Path -LiteralPath $candidate) { $uv = $candidate }
        $ancestor = $ancestor.Parent
    }
    if (-not $uv) { throw 'uv not found (PATH or 3rd_party/pk_system)' }
    & $uv run --locked --project (Join-Path $repo 'cores/python') python -m $Module @Arguments
    exit $LASTEXITCODE
} catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    exit 1
}
