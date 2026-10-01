"""Cost of the generated path constants in skim_search/__init__.py (build_env/paths-ssot).

    uv run python -m benchmarks.paths [--runs 61] [--limit-us 50]

Fresh process per sample: time `import <pkg>.sub` for a package whose __init__ is the generated
region vs an empty __init__ (same layout). Exit 1 if the median overhead exceeds --limit-us.
Log: ref/actual/logs/paths-bench.log
"""
from __future__ import annotations

import argparse
import shutil
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path

import skim_search
from skim_search import gen_paths

PROBE = "import time; t=time.perf_counter(); import {pkg}.sub; print((time.perf_counter()-t)*1e6)"


def once(cwd: Path, pkg: str) -> float:
    return float(subprocess.run([sys.executable, "-c", PROBE.format(pkg=pkg)], cwd=cwd, check=True,
                                capture_output=True, text=True).stdout)


def sample(cwd: Path, runs: int) -> tuple[list[float], list[float]]:
    """Alternate the two packages so machine-load drift affects both equally."""
    once(cwd, "gen_pkg"), once(cwd, "empty_pkg")  # write .pyc
    gen, base = [], []
    for _ in range(runs):
        gen.append(once(cwd, "gen_pkg"))
        base.append(once(cwd, "empty_pkg"))
    return gen, base


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=61)
    ap.add_argument("--limit-us", type=float, default=50.0)
    a = ap.parse_args()
    region = gen_paths.generated(gen_paths.find_ini())
    with tempfile.TemporaryDirectory(prefix="paths-bench-") as tmp:
        tmp = Path(tmp)
        for name, init in (("gen_pkg", region), ("empty_pkg", "")):
            (tmp / name / "sub").mkdir(parents=True)
            (tmp / name / "__init__.py").write_text(init, encoding="utf-8")
            (tmp / name / "sub" / "__init__.py").write_text("", encoding="utf-8")
        gen, base = sample(tmp, a.runs)
        shutil.rmtree(tmp / "gen_pkg", ignore_errors=True)
    overhead = statistics.median(gen) - statistics.median(base)
    keys = len(skim_search.REL)
    line = (f"paths constants: keys={keys} runs={a.runs} median import gen={statistics.median(gen):.1f}us "
            f"empty={statistics.median(base):.1f}us overhead={overhead:.1f}us limit={a.limit_us}us "
            f"result={'PASS' if overhead <= a.limit_us else 'FAIL'}")
    print(line)
    log = Path(skim_search.LOGS) / "paths-bench.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("a", encoding="utf-8") as f:
        f.write(line + "\n")
    return 0 if overhead <= a.limit_us else 1


if __name__ == "__main__":
    sys.exit(main())
