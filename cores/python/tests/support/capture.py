"""Launch skim-search, capture its window to PNG, close it with WM_CLOSE, report the exit code.

Replaces run_capture.ps1.
    uv run python -m tests.support.capture --exe <exe> --out <png> [--wait-ms 3000]
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

from .common import App, capture


def run(exe: Path, out: Path, wait_ms: int = 3000, env: dict[str, str] | None = None) -> tuple[int | None, tuple[int, int]]:
    app = App(exe, env)
    try:
        time.sleep(wait_ms / 1000)
        size = capture(app.hwnd, out)
        return app.close(), size
    finally:
        app.kill()


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--exe", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    p.add_argument("--wait-ms", type=int, default=3000)
    a = p.parse_args()
    code, (w, h) = run(a.exe, a.out, a.wait_ms)
    print(f"CAPTURED {w}x{h} -> {a.out}")
    print(f"EXIT_CODE {code}")
    return 0 if code == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
