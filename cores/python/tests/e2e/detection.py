"""e2e for search_engine/rg-sk-executable-detection: A (configured), B (PATH), C (missing).

    uv run python -m tests.e2e.detection [--no-build]
Exit code 0 = all checks PASS. Log: ref/actual/logs/e2e-detection.log
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sys

from ..support import capture
from ..support.common import APP_LOG, EXE_DEBUG, RG, SHOTS, SK, TEMP, AppLog, Checker, Log, cargo_build


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--no-build", action="store_true")
    a = p.parse_args()
    # `cargo test` does not refresh target\debug\skim-search.exe
    if not a.no_build and not cargo_build(release=False):
        print("BUILD_FAILED")
        return 1

    log = Log("e2e-detection.log", "e2e_detection")
    check = Checker(log)
    applog = AppLog()
    tmp = TEMP / "skim-search-e2e"
    tmp.mkdir(parents=True, exist_ok=True)
    sysdirs = f"{os.environ['SystemRoot']}\\system32;{os.environ['SystemRoot']}"
    rg_dir, sk_dir = RG.parent, SK.parent

    def case(name, exe, shot, env, expect):
        applog.mark()
        # explicit log path: an exe outside the repository (case C) logs next to itself by default
        code, size = capture.run(exe, SHOTS / shot, env={**env, "SKIM_SEARCH_LOG": str(APP_LOG)})
        new = applog.new()
        check(f"{name} exit", code == 0, f"exit={code} capture={size}")
        for pattern in expect:
            check(f"{name} log", any(re.search(pattern, l) for l in new), f"expects /{pattern}/")

    try:
        settings = tmp / "settings.json"
        settings.write_text(json.dumps({"rg_path": str(RG), "sk_path": str(SK)}), encoding="utf-8")
        case("case A configured", EXE_DEBUG, "rg-sk-executable-detection_100_caseA.png",
             {"SKIM_SEARCH_SETTINGS": str(settings), "PATH": sysdirs},
             [r"rg detected source=Configured", r"sk detected source=Configured"])
        case("case B PATH", EXE_DEBUG, "rg-sk-executable-detection_100_caseB.png",
             {"SKIM_SEARCH_SETTINGS": str(tmp / "none.json"), "PATH": f"{rg_dir};{sk_dir};{sysdirs}"},
             [r"rg detected source=Path", r"sk detected source=Path"])
        # exe copied outside the repo so the fallback cannot reach 3rd_party
        isolated = tmp / "isolated"
        isolated.mkdir(exist_ok=True)
        shutil.copy2(EXE_DEBUG, isolated / EXE_DEBUG.name)
        case("case C missing", isolated / EXE_DEBUG.name, "rg-sk-executable-detection_100_caseC.png",
             {"SKIM_SEARCH_SETTINGS": str(tmp / "none.json"), "PATH": sysdirs},
             [r"rg missing reason=", r"sk missing reason=",
              r"\[toast\] push .*kind=error.*rg\.exe", r"\[toast\] push .*kind=error.*sk\.exe"])
    except Exception as e:  # noqa: BLE001 - any error is a failed run
        check("e2e_detection", False, f"aborted: {e!r}")
    return check.finish("e2e_detection")


if __name__ == "__main__":
    sys.exit(main())
