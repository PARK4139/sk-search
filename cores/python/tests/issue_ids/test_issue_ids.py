# issue: diagnostics/one-shot-failure-issue/38926773 (ID allocation prerequisite)
import concurrent.futures
from contextlib import closing
import importlib.util
import json
from pathlib import Path
import subprocess
import sqlite3
import statistics
import time
import sys
import tempfile
import unittest
from unittest.mock import patch
import uuid

from skim_search import REL
from skim_search.diagnostics import issue_ids as _module

SCRIPT = Path(_module.__file__)
spec = importlib.util.spec_from_file_location("issue_ids", SCRIPT)
ids = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ids)


class IssueIdTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="issue ids ")
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "--quiet", self.root], check=True)

    def tearDown(self):
        self.temporary.cleanup()

    def existing(self, state, identity):
        path = self.root / "issues" / state / "diagnostics/sample" / (identity + ".md")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("existing issue", encoding="utf-8")

    def test_existing_states_and_persistent_reservation_collisions(self):
        values = ["11111111", "22222222", "33333333", "44444444", "55555555"]
        for state, identity in zip(("backlog", "working", "closed"), values):
            self.existing(state, identity)
        candidates = [uuid.UUID(value + "-0000-4000-8000-000000000000") for value in values]
        with patch.object(ids.uuid, "uuid4", side_effect=candidates):
            self.assertEqual(ids.get_issue_id(self.root), values[3])
        with patch.object(ids.uuid, "uuid4", side_effect=candidates[3:]):
            self.assertEqual(ids.get_issue_id(self.root), values[4])
        events = [json.loads(line) for line in (self.root / REL["LOGS"] / "issue-ids.jsonl").read_text().splitlines()]
        self.assertEqual(sum(e["event"] == "issue_id_collision" for e in events), 4)

    def test_same_candidate_competes_atomically(self):
        candidate = uuid.UUID("12345678-0000-4000-8000-000000000000")
        def allocate():
            try:
                return ids.get_issue_id(self.root, max_attempts=1)
            except RuntimeError:
                return None
        with patch.object(ids.uuid, "uuid4", return_value=candidate):
            with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
                results = list(pool.map(lambda _: allocate(), range(8)))
        self.assertEqual(results.count("12345678"), 1)
        self.assertEqual(results.count(None), 7)

    def test_multiple_processes_unique_and_restart(self):
        def allocate(_):
            return subprocess.check_output([sys.executable, SCRIPT, "--root", self.root], text=True).strip()
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            values = list(pool.map(allocate, range(32)))
        self.assertEqual(len(set(values)), 32)
        self.assertTrue(all(ids.ID_PATTERN.fullmatch(value) for value in values))
        self.assertNotIn(allocate(None), values)

    def test_duplicate_existing_ids_fail(self):
        self.existing("backlog", "abcdef12")
        self.existing("closed", "abcdef12")
        with self.assertRaisesRegex(ValueError, "duplicate existing"):
            ids.get_issue_id(self.root)

    def test_log_failure_does_not_release_reservation(self):
        original = ids.os.open
        def open_file(path, *args, **kwargs):
            if Path(path).name == "issue-ids.jsonl":
                raise OSError("fixture evidence failure")
            return original(path, *args, **kwargs)
        with patch.object(ids.os, "open", side_effect=open_file):
            identity = ids.get_issue_id(self.root)
        with closing(sqlite3.connect(self.root / '.git/issue_ids.sqlite3')) as connection:
            self.assertIsNotNone(connection.execute('SELECT 1 FROM ids WHERE id=?', (identity,)).fetchone())

    def test_legacy_migration_and_no_rescan(self):
        legacy = self.root / '.git/issue_ids'
        legacy.mkdir()
        (legacy / 'aabbccdd').write_text('old reservation')
        ids.get_issue_id(self.root)
        self.assertFalse(legacy.exists())
        candidate = uuid.UUID('aabbccdd-0000-4000-8000-000000000000')
        with patch.object(Path, 'rglob', side_effect=AssertionError('hot path must not scan')):
            with patch.object(ids.uuid, 'uuid4', return_value=candidate):
                with self.assertRaises(RuntimeError):
                    ids.get_issue_id(self.root, max_attempts=1)
            ids.get_issue_id(self.root)

    def test_speed_single_registry(self):
        ids.get_issue_id(self.root)
        measurements = []
        for _ in range(200):
            start = time.perf_counter_ns()
            ids.get_issue_id(self.root)
            measurements.append((time.perf_counter_ns() - start) / 1000)
        report = {'calls': len(measurements), 'p50_us': statistics.median(measurements),
                  'p95_us': sorted(measurements)[189], 'max_us': max(measurements),
                  'scope': 'warm in-process API, durable FULL commit and evidence logging'}
        destination = ids.ROOT / REL["LOGS"] / "issue-id-bench.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps(report, indent=2), encoding='utf-8')
        files = [path.name for path in (self.root / '.git').glob('issue_ids*')]
        self.assertEqual(files, ['issue_ids.sqlite3'])


if __name__ == "__main__":
    destination = ids.ROOT / REL["LOGS"] / "issue-id-tests.log"
    destination.parent.mkdir(parents=True, exist_ok=True)
    with destination.open("w", encoding="utf-8") as stream:
        result = unittest.TextTestRunner(stream=stream, verbosity=2).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(IssueIdTests))
    print(destination.read_text(encoding="utf-8"))
    raise SystemExit(not result.wasSuccessful())
