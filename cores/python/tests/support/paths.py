"""Canonical repository, tool, fixture, build, log and temporary paths."""

from __future__ import annotations

import os
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]  # cores/python/tests/support -> repository
THIRD_PARTY = next(p / "3rd_party" for p in REPO.parents if (p / "3rd_party").is_dir())
RG = THIRD_PARTY / "ripgrep" / "rg.exe"
SK = THIRD_PARTY / "skim" / "sk.exe"
CORES = REPO / "cores" / "rust"  # cargo workspace
SAMPLE_TREE = CORES / "tests" / "fixtures" / "sample" / "tree"
LOGS = REPO / "ref" / "actual" / "logs"
SHOTS = REPO / "ref" / "actual" / "screenshot" / "frames"
APP_LOG = LOGS / "skim-search.log"
TARGET = Path(os.environ.get("CARGO_TARGET_DIR") or REPO / "target")  # .cargo/config.toml target-dir
EXE_DEBUG = TARGET / "debug" / "skim-search.exe"
EXE_RELEASE = TARGET / "release" / "skim-search.exe"
WINDOW_TITLE = "skim-search"
TEMP = Path(os.environ.get("TEMP", REPO))
# Shown in the search-root box of committed screenshots: must not contain the Windows account name
# (rules/security.md SEC-LOCALPATH), so not under %TEMP%/%USERPROFILE%.
E2E_WS = Path(os.environ.get("PUBLIC", r"C:\Users\Public")) / "skim-search-e2e"
