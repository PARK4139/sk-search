"""Headless verification; no git push, desktop input or production publication.

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

import pipeline as p

UV = next((a / "3rd_party/pk_system/uv.exe" for a in p.ROOT.parents if (a / "3rd_party/pk_system/uv.exe").exists()), None)
GITLEAKS = next((a / "3rd_party/security/gitleaks.exe" for a in p.ROOT.parents if (a / "3rd_party/security/gitleaks.exe").exists()), None)


class PipelineTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="one shot tests ")
        self.folder = Path(self.tmp.name)
        self.root = self.folder / "repo with spaces"
        self.root.mkdir()
        self.shared = self.folder / "shared"
        self.shared.mkdir()
        self.logs = self.root / "ref/actual/logs/one-shot/run"
        self.root.joinpath("cores").mkdir()
        shutil.copytree(p.HERE, self.root / "cores/scripts/one_shot", ignore=shutil.ignore_patterns("__pycache__"))
        for name in ("one-shot.cmd", "one-shot.ps1", "one_shot.py"):
            shutil.copy2(p.ROOT / name, self.root / name)
        shutil.copytree(p.ROOT / "cores/tests/py", self.root / "cores/tests/py",
                        ignore=shutil.ignore_patterns(".venv", "__pycache__"))
        (self.root / ".gitignore").write_text("cores/target/\nref/actual/\n__pycache__/\n*.pyc\n", encoding="utf-8")
        self.artifact = self.root / "cores/target/release/skim-search.cmd"
        builder = self.root / "cores/builder.py"
        builder.write_text(
            "import os\nfrom pathlib import Path\np=Path('target/release/skim-search.cmd')\n"
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

    def test_full_wrappers_stop_before_push_and_preserve_remote(self):
        self.change()
        proc = subprocess.run([os.environ["COMSPEC"], "/d", "/c", "call", str(self.root / "one-shot.cmd"),
                               "--config", str(self.config), "--run-dir", str(self.logs)],
                              cwd=self.folder, env=self.env, capture_output=True, timeout=120)
        self.assertEqual(proc.returncode, 0, proc.stdout.decode(errors="replace") + proc.stderr.decode(errors="replace"))
        state = p.read_json(self.logs / "state.json")
        self.assertEqual(state["stages"], {s: "passed" for s in p.STAGES})
        self.assertEqual(state["assignment"]["version"], "0.1.1")
        self.assertEqual(state["push"], "disabled")
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)
        self.assertTrue(Path(state["release"]).exists())
        report = p.read_json(self.logs / "security-report.json")
        self.assertEqual(report["result"], "passed")
        self.assertEqual(report["sha"], state["sha"])

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
        self.assertFalse((self.shared / "skim-search/releases").exists())

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
        entries = p.read_json(self.shared / "skim-search/versions.json")["entries"]
        self.assertEqual({e["version"] for e in entries}, {"0.1.1", "0.1.2"})

    def test_push_entry_is_always_disabled(self):
        proc = subprocess.run([sys.executable, str(self.root / "cores/scripts/one_shot/push.py"),
                               "--config", str(self.config)], cwd=self.folder, capture_output=True)
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn(b"push is disabled", proc.stderr)
        self.assertEqual(self.git("ls-remote", "origin", "refs/heads/master").split()[0], self.base)


if __name__ == "__main__":
    unittest.main(verbosity=2)
