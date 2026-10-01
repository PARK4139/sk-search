"""Headless verification; pushes only to temporary local bare remotes (never the real origin), no desktop input, no production publication.

Issues: diagnostics/one-shot-pipeline/bb7afe0d and
diagnostics/one-shot-security-gate/aea6b525.
"""
import concurrent.futures
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import skim_search as paths
from skim_search import REL
from skim_search.diagnostics.one_shot import pipeline as p
from skim_search.diagnostics.one_shot.failure_issue import FAMILY as FAILURE_FAMILY

UV = Path(paths.UV) if Path(paths.UV).exists() else None
GITLEAKS = Path(paths.GITLEAKS) if Path(paths.GITLEAKS).exists() else None


class PipelineTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="one shot tests ")
        self.folder = Path(self.tmp.name)
        self.root = self.folder / "repo with spaces"
        self.root.mkdir()
        self.shared = self.folder / "shared"
        self.shared.mkdir()
        self.logs = self.root / REL["ONE_SHOT_LOGS"] / "run"
        # copy of the real layout (paths SSOT, entry points, configs, uv project)
        shutil.copytree(p.ROOT / REL["PYTHON_PROJECT"], self.root / REL["PYTHON_PROJECT"],
                        ignore=shutil.ignore_patterns(".venv", "__pycache__"))
        for key in ("SCRIPTS", "CONFIGS"):
            shutil.copytree(p.ROOT / REL[key], self.root / REL[key])
        (self.root / REL["PATHS_INI"]).parent.mkdir(parents=True)
        shutil.copy2(p.ROOT / REL["PATHS_INI"], self.root / REL["PATHS_INI"])
        self.root.joinpath(REL["CARGO_WORKSPACE"]).mkdir(parents=True)  # CI working directory
        (self.root / ".gitignore").write_text(f"/{REL['TARGET']}/\n{REL['EVIDENCE']}/\n__pycache__/\n*.pyc\n.venv/\n", encoding="utf-8")
        self.artifact = self.root / REL["TARGET"] / "release/skim-search.cmd"
        builder = self.root / REL["CARGO_WORKSPACE"] / "builder.py"
        builder.write_text(
            "import os\nfrom pathlib import Path\np=Path(" + repr(str(self.artifact)) + ")\n"
            "p.parent.mkdir(parents=True,exist_ok=True)\n"
            "p.write_text('@echo off\\necho skim-search '+os.environ['SKIM_SEARCH_BUILD_VERSION']+' '+os.environ['SKIM_SEARCH_BUILD_SHA']+'\\n')\n", encoding="utf-8")
        fake = self.folder / "fake.py"
        fake.write_text(
            "import sys,json\nfrom pathlib import Path\na=sys.argv[1:]\n"
            "if a[0]=='cargo':\n print(json.dumps({'vulnerabilities':{'found':False,'count':0,'list':[]}}))\n"
            "elif a[0]=='version':\n print('fixture audit 1.0')\n"
            "elif a[0]=='export':\n Path(a[a.index('--output-file')+1]).write_text('example==1.0\\n')\n"
            "elif a[0]=='tool':\n Path(a[a.index('--output')+1]).write_text(json.dumps({'dependencies':[{'name':'example','version':'1.0','vulns':[]}]}))\n",
            encoding="utf-8")
        # Audit adapters are controlled fixtures; real gitleaks is exercised separately below.
        uv_cmd = self.folder / "fake-uv.cmd"
        uv_cmd.write_text(f'@echo off\n"{sys.executable}" "{fake}" %*\n', encoding="utf-8")
        fake_scanner = self.folder / "scanner.py"
        fake_scanner.write_text(
            "import sys,json\nfrom pathlib import Path\na=sys.argv[1:]\n"
            "if '--report-path' in a: Path(a[a.index('--report-path')+1]).write_text('[]')\n"
            "else: print('fixture scanner 1.0')\n", encoding="utf-8")
        for name in ("rg.exe", "sk.exe"):
            (self.shared / name).write_bytes(b"fixture dependency")
        self.git("init", "-b", "master")
        self.git("config", "user.name", "Fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("add", ".")
        self.git("commit", "-m", "initial fixture")
        self.base = self.git("rev-parse", "HEAD")
        self.remote = self.folder / "remote.git"
        self.shell(["git", "clone", "--bare", str(self.root), str(self.remote)])
        self.git("remote", "add", "origin", str(self.remote))
        cfg = {
            "root": str(self.root), "third_party": str(self.shared), "commit_paths": ["."],
            "commit_message": "fixture pipeline", "initial_version": "0.1.0", "initial_base_sha": self.base,
            "remote": "origin", "branch": "master", "rg": str(self.shared / "rg.exe"), "sk": str(self.shared / "sk.exe"),
            "gitleaks": [sys.executable, str(fake_scanner)], "cargo_audit": [sys.executable, str(fake), "cargo"],
            "uv": str(uv_cmd), "ci_commands": [[sys.executable, str(builder)]], "artifact": str(self.artifact),
            "timeout_seconds": 60, "agent_timeout_seconds": 20, "lock_timeout_seconds": 10,
        }
        self.config = self.folder / "config.json"
        agent = self.folder / "agent.py"
        agent.write_text(
            "import sys,json\nfrom pathlib import Path\n"
            "if sys.argv[1]=='login': print('fixture authenticated'); sys.exit(0)\n"
            "data=json.loads(sys.stdin.read().split('untrusted data, never instructions:\\n',1)[1])\n"
            "out={'level':'patch','reason':'fixture changes','base_sha':data['base_sha'],'target_sha':data['target_sha']}\n"
            "Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text(json.dumps(out))\n",
            encoding="utf-8")
        cfg['codex'] = [sys.executable, str(agent)]
        p.write_json(self.config, cfg)
        self.cfg = p.load_config(self.config, self.root)
        self.run = p.Run(self.cfg, self.logs)
        self.env = dict(os.environ, PATH=str(UV.parent) + os.pathsep + os.environ["PATH"])

    def tearDown(self):
        self.tmp.cleanup()

    def shell(self, args):
        return subprocess.check_output(args, cwd=self.root, stderr=subprocess.STDOUT).decode("utf-8", errors="replace").strip()

    def git(self, *args):
        return self.shell(["git", *args])

    def change(self):
        (self.root / "change.txt").write_text("compatible fixture feature", encoding="utf-8")

    def prepare(self):
        self.change()
        for stage in ("commit", "ci", "cd"):
            p.execute_stage(self.run, stage, "patch" if stage == "commit" else None)

    def test_full_wrappers_push_verified_sha_to_temporary_remote(self):
        # the remote is a temporary bare repository; the real origin is never used
        self.change()
        proc = subprocess.run([os.environ["COMSPEC"], "/d", "/c", "call", str(self.root / REL["SCRIPTS"] / "one-shot.cmd"),
                               "--config", str(self.config), "--run-dir", str(self.logs)],
                              cwd=self.folder, env=self.env, capture_output=True, timeout=120)
        self.assertEqual(proc.returncode, 0, proc.stdout.decode(errors="replace") + proc.stderr.decode(errors="replace"))
        state = p.read_json(self.logs / "state.json")
        self.assertEqual(state["stages"], {s: "passed" for s in p.STAGES})
        self.assertEqual(state["assignment"]["version"], "0.1.1")
        self.assertEqual(state["push"]["result"], "pushed")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], state["sha"])
        self.assertTrue(Path(state["release"]).exists())
        self.assertEqual(Path(state["release"]).parent, self.shared / REL["RELEASES"])  # 3rd_party/skim-search/{SHA}
        report = p.read_json(self.logs / "security-report.json")
        self.assertEqual(report["result"], "passed")
        self.assertEqual(report["sha"], state["sha"])

    def test_stop_after_security_preserves_remote(self):
        self.change()
        proc = subprocess.run([os.environ["COMSPEC"], "/d", "/c", "call", str(self.root / REL["SCRIPTS"] / "one-shot.cmd"),
                               "--config", str(self.config), "--run-dir", str(self.logs), "--stop-after", "security"],
                              cwd=self.folder, env=self.env, capture_output=True, timeout=120)
        self.assertEqual(proc.returncode, 0, proc.stdout.decode(errors="replace") + proc.stderr.decode(errors="replace"))
        state = p.read_json(self.logs / "state.json")
        self.assertNotIn("push", state["stages"])
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)

    def test_bump_levels_and_same_sha_reuse_conflict(self):
        self.assertEqual(p.bump_version("1.2.3", "major"), "2.0.0")
        self.assertEqual(p.bump_version("1.2.3", "minor"), "1.3.0")
        self.assertEqual(p.bump_version("1.2.3", "patch"), "1.2.4")
        self.change()
        p.execute_stage(self.run, "commit", "patch")
        p.assign_version(self.run, None)
        self.assertEqual(self.run.state["assignment"]["version"], "0.1.1")
        with self.assertRaises(p.Failure):
            p.assign_version(self.run, "major")

    def test_failure_stops_successors(self):
        self.change()
        p.execute_stage(self.run, "commit", "patch")
        self.run.cfg["ci_commands"] = [[sys.executable, "-c", "raise SystemExit(23)"]]
        with self.assertRaises(p.Failure):
            p.execute_stage(self.run, "ci")
        with self.assertRaises(p.Failure):
            p.execute_stage(self.run, "cd")
        self.assertEqual(self.run.state["stages"]["ci"], "failed")
        self.assertFalse((self.shared / "skim-search" / self.run.state["sha"]).exists())

    def test_no_changes_uses_head(self):
        p.execute_stage(self.run, "commit", "patch")
        self.assertEqual(self.run.state["sha"], self.base)
        events = (self.logs / "events.jsonl").read_text(encoding="utf-8")
        self.assertIn('"commit_unchanged"', events)

    def test_missing_required_setting_fails_before_run(self):
        cfg = p.read_json(self.config)
        for name in ("remote", "commit_paths"):
            broken = dict(cfg)
            broken.pop(name)
            p.write_json(self.config, broken)
            with self.assertRaises(p.Failure):
                p.load_config(self.config, self.root)

    def test_cd_failure_blocks_push(self):
        from unittest import mock
        self.change()
        for stage in ("commit", "ci"):
            p.execute_stage(self.run, stage, "patch" if stage == "commit" else None)
        with mock.patch.object(p, "cd", side_effect=p.Failure("fixture cd failure")):
            with self.assertRaises(p.Failure):
                p.execute_stage(self.run, "cd")
        for stage in ("security", "push"):
            with self.assertRaises(p.Failure):
                p.execute_stage(self.run, stage)
        self.assertEqual(self.run.state["stages"]["cd"], "failed")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)

    def test_guard_and_package_tamper(self):
        self.prepare()
        manifest = p.verify_release(self.run.state["release"], self.run.state)
        package = Path(self.run.state["release"]) / manifest["package"]
        package.write_bytes(package.read_bytes() + b"tamper")
        with self.assertRaises(p.Failure):
            p.verify_release(self.run.state["release"], self.run.state)
        (self.root / "change.txt").write_text("changed after CI", encoding="utf-8")
        with self.assertRaises(p.Failure):
            self.run.guard()

    def test_retry_publish_preserves_package(self):
        self.prepare()
        manifest = p.verify_release(self.run.state["release"], self.run.state)
        path = Path(self.run.state["release"]) / manifest["package"]
        before = path.read_bytes()
        p.cd(self.run)
        self.assertEqual(path.read_bytes(), before)

    def test_security_missing_tool_and_vulnerability_block(self):
        self.prepare()
        scanner = self.run.cfg["gitleaks"]
        self.run.cfg["gitleaks"] = ["nonexistent-security-tool"]
        with self.assertRaises(p.Failure):
            p.execute_stage(self.run, "security")
        self.assertEqual(p.read_json(self.logs / "security-report.json")["result"], "failed")
        self.run.cfg["gitleaks"] = scanner
        original = self.run.command
        def vulnerable(args, **kwargs):
            if "--file" in list(map(str, args)):
                return 1, json.dumps({"vulnerabilities": {"found": True}})
            return original(args, **kwargs)
        with patch.object(self.run, "command", vulnerable):
            with self.assertRaises(p.Failure):
                p.security(self.run)

    def test_timeout_kills_owned_process(self):
        started = p.time.monotonic()
        with self.assertRaises(p.Failure):
            self.run.command([sys.executable, "-c", "import time; time.sleep(20)"], timeout=0.2)
        self.assertLess(p.time.monotonic() - started, 10)

    def test_explicit_bump_does_not_call_agent(self):
        with patch.object(self.run, 'command', side_effect=AssertionError('agent must not run')):
            for level in p.LEVELS:
                value = p.classify(self.run, self.base, self.base, level)
                self.assertEqual(value['level'], level)
                self.assertEqual(value['decision_by'], 'user')

    def test_agent_classification_and_reuse(self):
        self.change()
        p.execute_stage(self.run, 'commit')
        self.assertEqual(self.run.state['assignment']['decision_by'], 'agent')
        with patch.object(self.run, 'command', side_effect=AssertionError('reuse must not call agent')):
            p.assign_version(self.run, None)
        self.assertTrue((self.logs / 'classification-input.txt').exists())

    def test_agent_invalid_responses_and_failures_stop_before_ci(self):
        self.change()
        self.git('add', '.')
        self.git('commit', '-m', 'change')
        target = self.git('rev-parse', 'HEAD')
        original = self.run.command
        valid = {'level': 'minor', 'reason': 'new feature', 'base_sha': self.base, 'target_sha': target}
        cases = [dict(valid, level='unknown'), dict(valid, reason=' '), dict(valid, base_sha=target),
                 dict(valid, target_sha=self.base), dict(valid, extra=True), {'level': 'patch'}, [], 'invalid JSON']
        for response in cases:
            def fake(args, **kwargs):
                if 'exec' in args:
                    output = Path(args[args.index('--output-last-message') + 1])
                    output.write_text(response if isinstance(response, str) else json.dumps(response), encoding='utf-8')
                    return 0, ''
                return original(args, **kwargs)
            with self.subTest(response=response), patch.object(self.run, 'command', fake):
                with self.assertRaises(p.Failure):
                    p.classify(self.run, self.base, target)
        for command in (["nonexistent-codex-tool"], [sys.executable, '-c', 'raise SystemExit(17)'],
                        [sys.executable, '-c', 'import time; time.sleep(20)']):
            self.run.cfg['codex'] = command
            self.run.cfg['agent_timeout_seconds'] = 0.2
            with self.subTest(command=command), self.assertRaises(p.Failure):
                p.execute_stage(self.run, 'commit')
            self.assertEqual(self.run.state['stages']['commit'], 'failed')
            with self.assertRaises(p.Failure):
                p.execute_stage(self.run, 'ci')

    def test_real_gitleaks_deleted_secret_in_history_is_redacted(self):
        if not GITLEAKS:
            self.fail("real gitleaks is required for this verification")
        secret = "ghp_" + "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8"
        path = self.root / "credential.txt"
        path.write_text(secret, encoding="utf-8")
        self.git("add", ".")
        self.git("commit", "-m", "synthetic credential")
        path.unlink()
        self.git("add", "-u")
        self.git("commit", "-m", "remove synthetic credential")
        self.run.cfg["gitleaks"] = [str(GITLEAKS)]
        with self.assertRaises(p.Failure):
            p.scan_secrets(self.run, self.root, "deleted-secret", history=True, log_opts=self.base + "..HEAD")
        self.assertTrue(p.read_json(self.logs / "deleted-secret.json"))
        for path in self.logs.iterdir():
            if path.is_file():
                self.assertNotIn(secret, path.read_text(encoding="utf-8"))

    def test_concurrent_registry_assignments_unique(self):
        # Parallel builds in separate source trees share one version registry.
        runs = []
        for i in range(2):
            self.change()
            (self.root / "change.txt").write_text(str(i), encoding="utf-8")
            self.git("add", ".")
            self.git("commit", "-m", f"change {i}")
            run = p.Run(self.cfg, self.logs / str(i))
            run.state["sha"] = self.git("rev-parse", "HEAD")
            runs.append(run)
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            list(pool.map(lambda r: p.assign_version(r, "patch"), runs))
        entries = p.read_json(self.shared / REL["VERSIONS"])["entries"]
        self.assertEqual({e["version"] for e in entries}, {"0.1.1", "0.1.2"})

    def test_push_entry_requires_verified_run(self):
        # PYTHONPATH makes -m import the temporary copy (its ROOT is the temporary repository)
        env = dict(os.environ, PYTHONPATH=str(self.root / REL["PYTHON_PROJECT"] / "src"))
        proc = subprocess.run([sys.executable, "-m", "skim_search.diagnostics.one_shot.push",
                               "--config", str(self.config)], cwd=self.folder, capture_output=True, env=env)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn(b"independent stage requires --run-dir", proc.stderr)
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)

    def secured(self):
        self.prepare()
        with patch.dict(os.environ, {"PATH": str(UV.parent) + os.pathsep + os.environ["PATH"]}):
            p.execute_stage(self.run, "security")

    def test_push_blocked_by_security_policy(self):
        # a local user path in a commit to be pushed (rules/security.md SEC-LOCALPATH)
        bs = "\\"
        (self.root / "change.txt").write_text(f"log at C:{bs}Users{bs}" + "ali" + "ce" + f"{bs}x\n", encoding="utf-8")
        for stage in ("commit", "ci", "cd", "security"):
            p.execute_stage(self.run, stage, "patch" if stage == "commit" else None)
        with self.assertRaisesRegex(p.Failure, "security_policy blocked"):
            p.execute_stage(self.run, "push")
        self.assertEqual(self.run.state["stages"]["push"], "failed")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)

    def test_push_blocked_when_remote_changed_after_security(self):
        self.secured()
        other = self.folder / "other"
        self.shell(["git", "clone", "-q", str(self.remote), str(other)])
        subprocess.check_call(["git", "-C", str(other), "-c", "user.email=x@example.invalid", "-c", "user.name=x",
                               "commit", "-q", "--allow-empty", "-m", "concurrent"])
        subprocess.check_call(["git", "-C", str(other), "push", "-q", "origin", "master"])
        moved = self.git("ls-remote", "origin", "refs/heads/master").split()[0]
        with self.assertRaisesRegex(p.Failure, "remote branch changed"):
            p.execute_stage(self.run, "push")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], moved)

    def test_push_blocked_when_security_result_is_for_another_sha(self):
        self.secured()
        self.run.state["security"]["sha"] = "0" * 40
        with self.assertRaisesRegex(p.Failure, "security result missing or for another SHA"):
            p.execute_stage(self.run, "push")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)

    def test_keyboard_alert_once_per_run(self):
        calls = []
        self.run.cfg["keyboard_alert_lead_seconds"] = 0
        with patch.object(self.run, "command", side_effect=lambda *a, **k: calls.append(a) or (0, "toast")):
            p.keyboard_alert(self.run)
            p.keyboard_alert(self.run)
        self.assertEqual(len(calls), 1)
        self.assertEqual(self.run.state["keyboard_alert"]["method"], "toast")

    # ── failure issue (SSOT, no failure.json) ──
    def one_shot(self, run_dir, *extra):
        return subprocess.run([os.environ["COMSPEC"], "/d", "/c", "call", str(self.root / REL["SCRIPTS"] / "one-shot.cmd"),
                               "--config", str(self.config), "--run-dir", str(run_dir), "--bump", "patch", *extra],
                              cwd=self.folder, env=self.env, capture_output=True, timeout=180)

    def failure_issues(self):
        return sorted((self.root / REL["ISSUES"] / "backlog" / FAILURE_FAMILY).glob("*.md"))

    def test_failure_creates_one_masked_issue_and_updates_on_recurrence(self):
        # fixture values are assembled at runtime so this source never contains them
        token = "ghp" + "_" + "a" * 36
        home = "C:" + "\\" + "Users" + "\\" + "Alice"
        cfg = p.read_json(self.config)
        cfg["ci_commands"] = [[sys.executable, "-c", f"import sys; print({home + chr(92) + 'w'!r}, {token!r}); sys.exit(3)"]]
        p.write_json(self.config, cfg)
        self.change()
        first = self.one_shot(self.folder / "run1")
        self.assertEqual(first.returncode, 1)
        self.assertIn(b"failure issue created", first.stderr)
        issues = self.failure_issues()
        self.assertEqual(len(issues), 1)
        text = issues[0].read_text(encoding="utf-8")
        sha = self.git("rev-parse", "HEAD")
        self.assertIn(f"sha={sha} | stage=ci |", text)
        self.assertIn("종료 코드 3", text)
        self.assertNotIn("Alice", text)
        self.assertNotIn(token, text)
        self.assertFalse(list(self.folder.rglob("failure.json")))
        priority = (self.root / REL["PRIORITY"]).read_text(encoding="utf-8")
        self.assertEqual(priority.count(f"| {issues[0].stem} |"), 1)
        # same SHA + stage + item: the active issue gets a recurrence entry, no new file or row
        second = self.one_shot(self.folder / "run2")
        self.assertEqual(second.returncode, 1)
        self.assertIn(b"failure issue updated", second.stderr)
        self.assertEqual(self.failure_issues(), issues)
        text = issues[0].read_text(encoding="utf-8")
        self.assertEqual(text.count("- 재발 "), 1)
        self.assertEqual((self.root / REL["PRIORITY"]).read_text(encoding="utf-8").count(f"| {issues[0].stem} |"), 1)

    def test_failure_before_sha_and_save_failure(self):
        cfg = p.read_json(self.config)
        cfg.pop("remote")
        p.write_json(self.config, cfg)
        proc = self.one_shot(self.folder / "run-setup")
        self.assertEqual(proc.returncode, 1)
        [issue] = self.failure_issues()
        text = issue.read_text(encoding="utf-8")
        self.assertIn("sha=미확보 | stage=setup |", text)
        self.assertIn("미확보 (commit 단계 이전 실패)", text)
        # issue cannot be saved (backlog family path is a file): original failure and exit code stay
        shutil.rmtree(self.root / REL["ISSUES"] / "backlog" / FAILURE_FAMILY.parent)
        (self.root / REL["ISSUES"] / "backlog" / FAILURE_FAMILY.parent).write_text("blocker", encoding="utf-8")
        cfg["ci_commands"] = [[sys.executable, "-c", "raise SystemExit(4)"]]
        cfg["remote"] = "origin"
        p.write_json(self.config, cfg)
        self.change()
        proc = self.one_shot(self.folder / "run-unsaved")
        self.assertEqual(proc.returncode, 1)
        self.assertIn(b"failure issue not saved", proc.stderr)
        self.assertIn(b"FAIL:", proc.stderr)
        self.assertFalse(list(self.folder.rglob("failure.json")))

    def test_failure_kind_and_item_normalization(self):
        from skim_search.diagnostics.one_shot import failure_issue as fi
        self.assertEqual(fi.kind_of("command timed out: cargo; command-0123456789ab", "Failure"), "시간 초과")
        self.assertEqual(fi.kind_of("command failed: cargo (exit 101); command-0123456789ab", "Failure"), "종료 코드 101")
        self.assertEqual(fi.kind_of("tool missing", "FileNotFoundError"), "실행 불가")
        self.assertEqual(fi.kind_of("security result missing or for another SHA", "Failure"), "검증 실패 (Failure)")
        # run-specific command IDs, SHAs and absolute directories do not split the same failure item
        a = fi.item_of("command failed: " + "C:" + "\\tools\\cargo.exe (exit 101); command-0123456789ab at " + "a" * 40)
        b = fi.item_of("command failed: " + "D:" + "\\x\\y\\cargo.exe (exit 101); command-ba9876543210 at " + "b" * 40)
        self.assertEqual(a, b)

    def test_failure_issue_concurrent_and_closed_not_reopened(self):
        import threading
        from skim_search.diagnostics.one_shot import failure_issue as fi
        from skim_search.diagnostics import issue_ids
        run_dir = self.folder / "run-direct"
        run_dir.mkdir()
        args = dict(stage="ci", sha="b" * 40, sha_reason="", message="command failed: cargo (exit 101); command-0123456789ab",
                    exc_type="Failure", repro="one-shot.cmd", run_id="r", new_id=issue_ids.get_issue_id,
                    guard=lambda: p.lock(self.root / REL["ONE_SHOT_LOGS"] / ".failure-issue.lock", 30))
        results = []
        threads = [threading.Thread(target=lambda: results.append(fi.record(self.root, run_dir, **args))) for _ in range(4)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        self.assertEqual(sum(created for _, created in results), 1)
        [issue] = self.failure_issues()
        self.assertEqual(issue.read_text(encoding="utf-8").count("- 재발 "), 3)
        # once closed, a recurrence creates a new backlog issue that links the closed one
        closed = self.root / REL["ISSUES"] / "closed" / FAILURE_FAMILY / issue.name
        closed.parent.mkdir(parents=True)
        issue.rename(closed)
        path, created = fi.record(self.root, run_dir, **args)
        self.assertTrue(created)
        self.assertNotEqual(path.name, closed.name)
        self.assertIn(f"이전 기록: `{REL['ISSUES']}/closed/{FAILURE_FAMILY.as_posix()}/", path.read_text(encoding="utf-8"))
        self.assertNotIn("- 재발 ", closed.read_text(encoding="utf-8").split("- 재발 ", 1)[0] + "")


if __name__ == "__main__":
    unittest.main(verbosity=2)
