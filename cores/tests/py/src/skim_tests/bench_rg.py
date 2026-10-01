"""rg.exe standalone latency per flag variant (diagnostics/first-result-latency analysis).

    uv run python -m skim_tests.bench_rg [--runs 15]
Report only; exit 1 only if rg fails or returns nothing. Log: ref/actual/logs/bench-rg.log
"""

from __future__ import annotations

import argparse
import shutil
import statistics
import subprocess
import sys
import time

from .common import RG, TEMP, Log

# same arguments as the app (cores/app/src/search_engine/ripgrep.rs)
BASE = ["--null", "--line-number", "--column", "--no-heading", "--with-filename", "--color", "never",
        "--ignore-case", "--max-columns", "500", "--max-columns-preview", "--regexp", "l.*o.*g.*i.*n", "--"]
VARIANTS = [
    ("app flags", []),
    ("+ --no-config", ["--no-config"]),
    ("+ --threads 1", ["--threads", "1"]),
    ("+ --no-ignore (semantics differ)", ["--no-ignore"]),
    ("+ --line-buffered", ["--line-buffered"]),
]


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--runs", type=int, default=15)
    a = p.parse_args()
    log = Log("bench-rg.log", f"bench_rg runs={a.runs}")
    ws = TEMP / "skim-search-bench-ws"
    shutil.rmtree(ws, ignore_errors=True)
    for rel in ("src/auth/login.ts", "src/api/auth.ts", "docs/login-guide.md", "README.md", "config.json"):
        f = ws / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text("login line\nother line\n", encoding="utf-8")

    fail = 0
    for label, extra in VARIANTS:
        times, lines = [], 0
        for _ in range(a.runs):
            t = time.perf_counter()
            r = subprocess.run([str(RG), *extra, *BASE, str(ws)], capture_output=True)
            times.append((time.perf_counter() - t) * 1000)
            fail += r.returncode != 0
            lines = len(r.stdout.splitlines())
        fail += lines < 1
        log(f"{label:<34} median={statistics.median(times):6.1f}ms min={min(times):6.1f} max={max(times):6.1f} lines={lines}")
    log(f"bench_rg fail={fail}")
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
