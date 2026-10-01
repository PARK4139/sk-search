"""Pre-push repository policy check (rules/security.md).

Checks only what will be pushed: commits in `<remote>/<branch>..<ref>` (full history of
`<ref>` when the remote branch does not exist yet). Local-only refs (stash, rewritten
history) are never scanned.

Blocking rules
  SEC-SECRET     token / private key / password patterns in added lines (supplements gitleaks)
  SEC-LOCALPATH  local user paths (C:\\Users\\<name>, /c/Users/<name>) in added lines
  SEC-EMAIL      non-allowlisted e-mail in commit metadata or added lines
  SEC-PATH       files that must not be published (logs, venv, build output, .env, user settings)
  SEC-TOOL       the check itself could not run (git error, remote unreachable)
Warnings
  WARN-LARGE     added blob larger than 5 MB

Exit code: 0 = no blocking finding, 1 = blocked, 2 = check could not run (SEC-TOOL).
Report: ref/actual/logs/security/policy-<UTC time>.json + .log (UTF-8, secrets masked).

    security_policy.cmd [--remote origin] [--branch main] [--ref HEAD] [--repo PATH]
    security_policy.cmd --self-test
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass, field
from datetime import date, datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]  # cores/python/src/skim_search/diagnostics -> repository
EXCEPTIONS_FILE = ROOT / "configs" / "security-exceptions.json"
LARGE_BYTES = 5 * 1024 * 1024

SECRET_PATTERNS = [
    ("github-token", re.compile(r"\b(gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{40,})\b")),
    ("slack-token", re.compile(r"\bxox[abposr]-[A-Za-z0-9-]{10,}\b")),
    ("aws-access-key", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("google-api-key", re.compile(r"\bAIza[0-9A-Za-z_\-]{35}\b")),
    ("openai-key", re.compile(r"\bsk-(proj-)?[A-Za-z0-9_\-]{32,}\b")),
    ("private-key", re.compile(r"-----BEGIN (RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY")),
    ("assigned-secret", re.compile(
        r"(?i)\b(password|passwd|secret|api[_-]?key|access[_-]?token|auth[_-]?token)\b\s*[:=]\s*['\"]([^'\"\s]{8,})['\"]")),
]
# user directory followed by a concrete name (placeholders like <name>, %USERPROFILE%, {user} pass;
# Windows shared profiles such as C:\Users\Public are not accounts and pass)
LOCALPATH = re.compile(
    r"(?i)(?:\b[a-z]:[\\/]{1,2}|/[a-z]/|/home/)users?[\\/]{1,2}"
    r"(?![<%{$]|(?:public|default|default user|all users)(?=[\\/'\"`\s]|$))"
    r"([A-Za-z0-9._ -]+?)(?=[\\/'\"`\s]|$)")
EMAIL = re.compile(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b")
EMAIL_ALLOW = [
    re.compile(r"(?i)^[A-Za-z0-9._%+\-]+@users\.noreply\.github\.com$"),
    re.compile(r"(?i)^noreply@(github\.com|anthropic\.com)$"),
    # reserved for documentation / tests (RFC 2606, RFC 6761)
    re.compile(r"(?i)@(example\.(com|org|net)|localhost|[A-Za-z0-9.\-]+\.(invalid|test|example))$"),
]
FORBIDDEN_PATHS = [
    "ref/actual/logs/*", "*/.venv/*", ".venv/*", "*/target/*", "target/*", "*/__pycache__/*", "__pycache__/*",
    "*.env", ".env", "*/.env.*", "settings.json", "*/skim-search/settings.json",
]
# files allowed even though they match a forbidden pattern
FORBIDDEN_ALLOW = ["ref/actual/logs/.gitignore"]


@dataclass
class Finding:
    rule: str
    severity: str  # block | warn
    commit: str
    path: str
    line: int | None
    detail: str
    excepted_by: str | None = None


@dataclass
class Report:
    repo: str
    remote: str
    branch: str
    ref_sha: str = ""
    base_sha: str | None = None
    range: str = ""
    commits: list[str] = field(default_factory=list)
    findings: list[Finding] = field(default_factory=list)
    exceptions_applied: list[str] = field(default_factory=list)
    scope: str = "commits to be pushed: metadata, added lines, added/modified paths and sizes"
    not_covered: list[str] = field(default_factory=lambda: [
        "dependency vulnerabilities (SEC-VULN: cargo-audit / pip-audit in the pipeline security stage)",
        "secrets in encodings the patterns do not recognise (gitleaks in the pipeline security stage)",
        "binary file contents",
    ])
    result: str = ""
    started: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat(timespec="seconds"))


class ToolError(Exception):
    pass


def mask(text: str) -> str:
    text = text.strip()
    return (text[:4] + "…" + f"({len(text)} chars)") if len(text) > 4 else "…"


def git(repo: Path, *args: str, check: bool = True, text: bool = True) -> str:
    r = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=text,
                       encoding="utf-8" if text else None, errors="replace" if text else None)
    if check and r.returncode != 0:
        raise ToolError(f"git {' '.join(args)} failed: {(r.stderr or '').strip()[:300]}")
    return r.stdout


def push_range(repo: Path, remote: str, branch: str, ref: str) -> tuple[str, str | None, list[str]]:
    ref_sha = git(repo, "rev-parse", "--verify", f"{ref}^{{commit}}").strip()
    heads = subprocess.run(["git", "-C", str(repo), "ls-remote", "--heads", remote, branch],
                           capture_output=True, text=True, timeout=120)
    if heads.returncode != 0:
        raise ToolError(f"remote '{remote}' unreachable: {heads.stderr.strip()[:300]}")
    remote_sha = heads.stdout.split()[0] if heads.stdout.strip() else None
    if remote_sha:
        fetched = subprocess.run(["git", "-C", str(repo), "fetch", "--quiet", remote, branch],
                                 capture_output=True, text=True, timeout=300)
        if fetched.returncode != 0:
            raise ToolError(f"fetch {remote}/{branch} failed: {fetched.stderr.strip()[:300]}")
        rev_range = f"{remote_sha}..{ref_sha}"
    else:
        rev_range = ref_sha  # new remote branch: everything reachable from ref
    commits = git(repo, "rev-list", "--reverse", rev_range).split()
    return ref_sha, remote_sha, commits


def load_exceptions(path: Path) -> list[dict]:
    if not path.exists():
        return []
    data = json.loads(path.read_text(encoding="utf-8"))
    out = []
    for e in data.get("exceptions", []):
        missing = {"id", "rule", "path", "reason", "expires"} - e.keys()
        if missing:
            raise ToolError(f"exception {e.get('id', '?')} missing {sorted(missing)}")
        if date.fromisoformat(e["expires"]) >= date.today():
            out.append(e)
    return out


def scan_commit(repo: Path, sha: str, findings: list[Finding]) -> None:
    meta = git(repo, "show", "-s", "--format=%an%x00%ae%x00%cn%x00%ce", sha).strip().split("\x00")
    for role, email in (("author", meta[1]), ("committer", meta[3])):
        if not any(a.search(email) for a in EMAIL_ALLOW):
            findings.append(Finding("SEC-EMAIL", "block", sha, "(commit metadata)", None, f"{role} email {mask(email)}"))

    for line in git(repo, "diff-tree", "-r", "--root", "--no-commit-id", "-M", "--name-status", sha).splitlines():
        parts = line.split("\t")
        status, path = parts[0], parts[-1]
        if status.startswith("D"):
            continue
        if path not in FORBIDDEN_ALLOW and any(fnmatch.fnmatch(path, p) or fnmatch.fnmatch("/" + path, "*/" + p) for p in FORBIDDEN_PATHS):
            findings.append(Finding("SEC-PATH", "block", sha, path, None, "path must not be published"))
        size = int(git(repo, "cat-file", "-s", f"{sha}:{path}").strip() or 0)
        if size > LARGE_BYTES:
            findings.append(Finding("WARN-LARGE", "warn", sha, path, None, f"{size / 1048576:.1f} MB"))

    # added lines only (removals cannot leak); binary diffs are skipped by git
    path, lineno = "", 0
    for raw in git(repo, "show", "--format=", "--unified=0", "--no-color", "--no-ext-diff", sha).splitlines():
        if raw.startswith("+++ "):
            path = raw[6:] if raw.startswith("+++ b/") else raw[4:]
            continue
        if raw.startswith("@@"):
            m = re.search(r"\+(\d+)", raw)
            lineno = int(m.group(1)) - 1 if m else 0
            continue
        if not raw.startswith("+") or raw.startswith("+++"):
            continue
        lineno += 1
        text = raw[1:]
        for name, rx in SECRET_PATTERNS:
            for m in rx.finditer(text):
                findings.append(Finding("SEC-SECRET", "block", sha, path, lineno, f"{name}: {mask(m.group(m.lastindex or 0))}"))
        for m in LOCALPATH.finditer(text):
            findings.append(Finding("SEC-LOCALPATH", "block", sha, path, lineno, f"user path '{m.group(0)[:3]}…/{mask(m.group(1))}'"))
        for m in EMAIL.finditer(text):
            if not any(a.search(m.group(0)) for a in EMAIL_ALLOW):
                findings.append(Finding("SEC-EMAIL", "block", sha, path, lineno, f"email {mask(m.group(0))}"))


def apply_exceptions(report: Report, exceptions: list[dict]) -> None:
    for f in report.findings:
        for e in exceptions:
            if e["rule"] == f.rule and fnmatch.fnmatch(f.path, e["path"]):
                f.excepted_by = e["id"]
                report.exceptions_applied.append(f"{e['id']}: {e['rule']} {e['path']} ({e['reason']}, expires {e['expires']})")
                break


def check(repo: Path, remote: str, branch: str, ref: str, exceptions_file: Path) -> Report:
    report = Report(repo=repo.name, remote=remote, branch=branch)  # name only: no local user path in reports
    try:
        report.ref_sha, report.base_sha, report.commits = push_range(repo, remote, branch, ref)
        report.range = f"{report.base_sha or '(new branch)'}..{report.ref_sha}"
        for sha in report.commits:
            scan_commit(repo, sha, report.findings)
        apply_exceptions(report, load_exceptions(exceptions_file))
    except (ToolError, subprocess.TimeoutExpired, OSError, ValueError) as e:
        report.findings.append(Finding("SEC-TOOL", "block", report.ref_sha, "", None, str(e)))
    blocked = [f for f in report.findings if f.severity == "block" and not f.excepted_by]
    tool = any(f.rule == "SEC-TOOL" for f in blocked)
    report.result = "error" if tool else ("blocked" if blocked else "no blocking findings (see not_covered)")
    return report


def write_report(report: Report, out_dir: Path) -> Path:
    out_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = out_dir / f"policy-{stamp}.json"
    path.write_text(json.dumps(asdict(report), ensure_ascii=False, indent=2), encoding="utf-8")
    lines = [f"range={report.range} commits={len(report.commits)} result={report.result}"]
    lines += [f"{f.rule} {f.severity}{' (excepted ' + f.excepted_by + ')' if f.excepted_by else ''} "
              f"{f.commit[:8]} {f.path}{':' + str(f.line) if f.line else ''} {f.detail}" for f in report.findings]
    path.with_suffix(".log").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return path


def exit_code(report: Report) -> int:
    return {"error": 2, "blocked": 1}.get(report.result, 0)


# ── self test: temporary repositories only, never the real origin ──────────


def self_test() -> int:
    failures = 0
    # synthetic fixtures are assembled at runtime so this source file itself passes the policy check
    token = "ghp_" + "a1B2c3D4e5F6g7H8i9J0" * 2
    bs = "\\"
    user_win = f"C:{bs}Users{bs}" + "ali" + "ce"
    user_bash = "/c/Users/" + "ali" + "ce"
    user_bob = f"C:{bs}Users{bs}" + "b" + "ob"
    at = "@"
    personal = "alice" + at + "gmail.com"
    corp = "alice" + at + "corp.io"
    gh_noreply = "noreply" + at + "github.com"

    def expect(name: str, report: Report, code: int, rules: set[str]) -> None:
        nonlocal failures
        got = {f.rule for f in report.findings if f.severity == "block" and not f.excepted_by}
        raw_leak = token in json.dumps(asdict(report))
        ok = exit_code(report) == code and got == rules and not raw_leak
        failures += not ok
        print(f"{'PASS' if ok else 'FAIL'} {name}: exit={exit_code(report)} rules={sorted(got)} raw_secret_in_report={raw_leak}")

    with tempfile.TemporaryDirectory(prefix="sk-sec-") as tmp:
        tmp = Path(tmp)
        remote, repo = tmp / "remote.git", tmp / "work"
        subprocess.run(["git", "init", "-q", "--bare", "-b", "main", str(remote)], check=True)
        subprocess.run(["git", "init", "-q", "-b", "main", str(repo)], check=True)
        for k, v in (("user.name", "t"), ("user.email", "t@users.noreply.github.com"), ("core.autocrlf", "false")):
            git(repo, "config", k, v)
        git(repo, "remote", "add", "origin", str(remote))
        none = tmp / "none.json"

        def commit(files: dict[str, str], msg: str, email: str | None = None) -> None:
            for rel, content in files.items():
                p = repo / rel
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text(content, encoding="utf-8")
            git(repo, "add", "-A", "-f", *files)  # -f: also stage ignored files (simulates a forced add)
            env = ["-c", f"user.email={email}"] if email else []
            git(repo, *env, "commit", "-q", "-m", msg)

        def run() -> Report:
            return check(repo, "origin", "main", "HEAD", none)

        def reset_to_remote() -> None:
            git(repo, "reset", "-q", "--hard", "origin/main")

        commit({"README.md": "docs at %USERPROFILE%\\Downloads and C:\\Users\\<name>\\x\n"
                             "e2e at C:\\Users\\Public\\skim-search-e2e and C:\\Users\\Default\\x\n"}, "clean")
        expect("new branch, clean, placeholders and shared profiles allowed", run(), 0, set())
        git(repo, "push", "-q", "origin", "main")
        expect("nothing to push", run(), 0, set())

        commit({"a.txt": f"token = {token}\n"}, "secret")
        expect("SEC-SECRET token", run(), 1, {"SEC-SECRET"})
        commit({"a.txt": "removed\n"}, "remove secret")
        expect("secret removed later is still in history", run(), 1, {"SEC-SECRET"})
        reset_to_remote()

        commit({"b.md": f"log at {user_win}{bs}Downloads{bs}x and {user_bash}/y\n"}, "path")
        expect("SEC-LOCALPATH", run(), 1, {"SEC-LOCALPATH"})
        reset_to_remote()

        commit({"c.txt": "ok\n"}, "personal email", email=personal)
        expect("SEC-EMAIL metadata", run(), 1, {"SEC-EMAIL"})
        reset_to_remote()
        commit({"c.txt": f"contact {corp}, {gh_noreply}, fixture{at}example.invalid\n"}, "email in file")
        expect("SEC-EMAIL content (noreply allowed)", run(), 1, {"SEC-EMAIL"})
        reset_to_remote()

        commit({"ref/actual/logs/run.log": "x\n", "ref/actual/logs/.gitignore": "*\n"}, "logs")
        expect("SEC-PATH logs (.gitignore allowed)", run(), 1, {"SEC-PATH"})
        reset_to_remote()

        # stash holding a secret is local only → not scanned
        (repo / "d.txt").write_text(f"{token}\n", encoding="utf-8")
        git(repo, "stash", "push", "-u", "-q")
        commit({"e.txt": "clean\n"}, "clean after stash")
        expect("local stash with secret is not in push range", run(), 0, set())
        reset_to_remote()

        commit({"b.md": f"{user_bob}{bs}x\n"}, "path with exception")
        exc = tmp / "exc.json"
        exc.write_text(json.dumps({"exceptions": [
            {"id": "EX-1", "rule": "SEC-LOCALPATH", "path": "b.md", "reason": "test", "expires": "2999-12-31"}]}), encoding="utf-8")
        expect("valid exception", check(repo, "origin", "main", "HEAD", exc), 0, set())
        exc.write_text(json.dumps({"exceptions": [
            {"id": "EX-2", "rule": "SEC-LOCALPATH", "path": "b.md", "reason": "test", "expires": "2000-01-01"}]}), encoding="utf-8")
        expect("expired exception ignored", check(repo, "origin", "main", "HEAD", exc), 1, {"SEC-LOCALPATH"})
        reset_to_remote()

        git(repo, "remote", "set-url", "origin", str(tmp / "missing.git"))
        expect("remote unreachable → SEC-TOOL", run(), 2, {"SEC-TOOL"})

    print(f"self-test failures={failures}")
    return 1 if failures else 0


def main() -> int:
    for s in (sys.stdout, sys.stderr):
        if hasattr(s, "reconfigure"):
            s.reconfigure(encoding="utf-8", errors="replace")
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--repo", type=Path, default=ROOT)
    p.add_argument("--remote", default="origin")
    p.add_argument("--branch", default="main")
    p.add_argument("--ref", default="HEAD")
    p.add_argument("--exceptions", type=Path, default=EXCEPTIONS_FILE)
    p.add_argument("--report-dir", type=Path, default=ROOT / "ref" / "actual" / "logs" / "security")
    p.add_argument("--self-test", action="store_true")
    a = p.parse_args()
    if a.self_test:
        return self_test()
    if not shutil.which("git"):
        print("SEC-TOOL git not found", file=sys.stderr)
        return 2
    report = check(a.repo.resolve(), a.remote, a.branch, a.ref, a.exceptions)
    path = write_report(report, a.report_dir)
    print(path.with_suffix(".log").read_text(encoding="utf-8"), end="")
    print(f"report: {path}")
    return exit_code(report)


if __name__ == "__main__":
    raise SystemExit(main())
