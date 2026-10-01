"""Thin root entry point for the local one-shot pipeline."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / "cores/scripts/one_shot"))
from pipeline import main

if __name__ == "__main__":
    raise SystemExit(main())
