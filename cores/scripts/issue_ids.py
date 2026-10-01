"""Reserve UUID-prefix issue IDs across processes sharing a Git common directory."""
from __future__ import annotations

import argparse
import json
from functools import lru_cache
import os
from pathlib import Path
import re
import subprocess
import sqlite3
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]
ID_PATTERN = re.compile(r"[0-9a-f]{8}")


def _evidence(root: Path, event: str, **details):
    """Best-effort evidence; reservation safety never depends on logging."""
    try:
        folder = root / "ref/actual/logs"
        folder.mkdir(parents=True, exist_ok=True)
        line = json.dumps({"time_ns": time.time_ns(), "event": event, **details}) + "\n"
        descriptor = os.open(folder / "issue-ids.jsonl", os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
        try:
            os.write(descriptor, line.encode("utf-8"))
        finally:
            os.close(descriptor)
    except OSError:
        pass


@lru_cache(maxsize=32)
def _common_directory(root: Path) -> Path:
    result = subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"],
                            cwd=root, capture_output=True, text=True, encoding="utf-8", check=True)
    return Path(result.stdout.strip())


def _initialize(connection, root, common):
    """One transaction imports existing files and previous reservations once."""
    connection.execute("CREATE TABLE IF NOT EXISTS ids (id TEXT PRIMARY KEY, uuid TEXT) WITHOUT ROWID")
    connection.execute("CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY) WITHOUT ROWID")
    if connection.execute("SELECT 1 FROM metadata WHERE key='initialized'").fetchone():
        return
    used = set()
    for state in ("backlog", "working", "closed"):
        for path in (root / "issues" / state).rglob("*.md"):
            identity = path.stem.lower()
            if not ID_PATTERN.fullmatch(identity):
                continue
            if identity in used:
                raise ValueError(f"duplicate existing issue ID: {identity}")
            used.add(identity)
    legacy = common / "issue_ids"
    if legacy.is_dir():
        used.update(path.name for path in legacy.iterdir() if ID_PATTERN.fullmatch(path.name))
    connection.executemany("INSERT OR IGNORE INTO ids VALUES (?, NULL)", ((value,) for value in used))
    connection.execute("INSERT INTO metadata VALUES ('initialized')")
    _evidence(root, "issue_ids_initialized", imported=len(used))


def _retire_legacy(connection, common):
    """Remove old reservation files only after their IDs have durably migrated."""
    legacy = common / "issue_ids"
    if not legacy.is_dir():
        return
    for path in legacy.iterdir():
        if (path.is_file() and not path.is_symlink() and ID_PATTERN.fullmatch(path.name)
                and connection.execute("SELECT 1 FROM ids WHERE id=?", (path.name,)).fetchone()):
            path.unlink(missing_ok=True)
    try:
        legacy.rmdir()
    except OSError:
        pass


def get_issue_id(root: Path | str = ROOT, *, max_attempts: int = 128) -> str:
    """Allocate only through the single durable SQLite registry.

    Existing issues are imported once; all subsequent writers must use this API.
    The database must not be removed/recreated. Separate clones do not share it.
    SQLite's unique key and transaction serialize concurrent writers; rollback
    journals are temporary recovery files, not an additional source of truth.
    """
    root = Path(root).resolve()
    started = time.perf_counter_ns()
    if not isinstance(max_attempts, int) or max_attempts < 1:
        raise ValueError("max_attempts must be a positive integer")
    try:
        common = _common_directory(root)
        connection = sqlite3.connect(common / "issue_ids.sqlite3", timeout=30, isolation_level=None)
        try:
            # FULL synchronous + commit before return preserves assigned IDs after a crash.
            connection.execute("PRAGMA synchronous=FULL")
            connection.execute("BEGIN IMMEDIATE")
            _initialize(connection, root, common)
            for attempt in range(max_attempts):
                full_uuid = uuid.uuid4()
                identity = full_uuid.hex[:8]
                try:
                    connection.execute("INSERT INTO ids VALUES (?, ?)", (identity, str(full_uuid)))
                except sqlite3.IntegrityError:
                    _evidence(root, "issue_id_collision", issue_id=identity, source="registry")
                    continue
                connection.commit()
                _retire_legacy(connection, common)
                _evidence(root, "issue_id_reserved", issue_id=identity, attempts=attempt + 1,
                          elapsed_us=(time.perf_counter_ns() - started) // 1000)
                return identity
            raise RuntimeError("issue ID reservation attempts exhausted")
        finally:
            connection.close()
    except Exception as exc:
        _evidence(root, "issue_id_failed", error_type=type(exc).__name__)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    arguments = parser.parse_args()
    print(get_issue_id(arguments.root))
