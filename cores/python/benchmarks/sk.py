"""sk.exe latency after stdin is closed, with the process already started
(models the pre-spawned sk in the engine).

    uv run python -m benchmarks.sk [--runs 20]
Report only; exit 1 if sk returns the wrong number of lines. Log: ref/actual/logs/bench-sk.log
"""

from __future__ import annotations

import argparse
import statistics
import subprocess
import sys
import time

from tests.support.common import SK, Log

CREATE_NO_WINDOW = 0x08000000
INPUT = "".join(f"{i}\tlogin line {i} with some text\n" for i in range(1, 15)).encode()
VARIANTS = [
    ("app args (--no-sort --delimiter tab --nth 2..)", ["--filter", "login", "--no-sort", "--delimiter", "\t", "--nth", "2.."], 14),
    ("no field restriction", ["--filter", "login", "--no-sort"], 14),
    ("--tiebreak index", ["--filter", "login", "--no-sort", "--delimiter", "\t", "--nth", "2..", "--tiebreak", "index"], 14),
    ("no match", ["--filter", "zzzz", "--no-sort", "--delimiter", "\t", "--nth", "2.."], 0),
]


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--runs", type=int, default=20)
    a = p.parse_args()
    log = Log("bench-sk.log", f"bench_sk runs={a.runs}")
    fail = 0
    for label, args, expected in VARIANTS:
        times = []
        for _ in range(a.runs):
            proc = subprocess.Popen([str(SK), *args], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=subprocess.DEVNULL, creationflags=CREATE_NO_WINDOW)
            time.sleep(0.06)  # let the process finish starting (pre-spawned in the app)
            t = time.perf_counter()
            out, _ = proc.communicate(INPUT)
            times.append((time.perf_counter() - t) * 1000)
            fail += len([l for l in out.splitlines() if l]) != expected
        log(f"{label:<48} after-input median={statistics.median(times):5.1f}ms min={min(times):5.1f} max={max(times):5.1f}")
    log(f"bench_sk fail={fail}")
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
