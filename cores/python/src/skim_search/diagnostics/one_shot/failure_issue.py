"""One-shot failure -> one active backlog issue (the failure SSOT; no failure.json).

Same full SHA + stage + failure item updates the active backlog/working issue instead of
creating another. Closed issues are never reopened; a recurrence gets a new issue that links them.
"""
from __future__ import annotations

from contextlib import nullcontext
import re
import sys
from datetime import datetime
from pathlib import Path

from skim_search import REL

from .. import security_policy as sp

FAMILY = Path("diagnostics") / "one-shot-failure"
KEY_PREFIX = "- Failure match: "
COMMAND_ID = re.compile(r"command-[0-9a-f]{12}")
TAIL_LINES = 20


def mask(text: str) -> str:
    """Account names, secrets and personal e-mail never reach the issue."""
    text = sp.LOCALPATH.sub(lambda m: m.group(0).replace(m.group(1), "<user>"), text)
    for _, pattern in sp.SECRET_PATTERNS:
        text = pattern.sub("<secret>", text)
    return sp.EMAIL.sub(lambda m: m.group(0) if any(a.search(m.group(0)) for a in sp.EMAIL_ALLOW) else "<email>", text)


def innermost(run_dir: Path, message: str) -> tuple[str, str | None]:
    """Follow nested wrapper failures (`FAIL: ...` in each command's stderr) to the command that failed."""
    command, seen = None, set()
    while (found := COMMAND_ID.search(message)) and found.group(0) not in seen:
        seen.add(found.group(0))
        command = found.group(0)
        stderr = run_dir / f"{command}.stderr.log"
        lines = re.findall(r"^FAIL: (.*?); logs: ", stderr.read_text(encoding="utf-8", errors="replace"), re.M) if stderr.exists() else []
        if not lines:
            break
        message = lines[-1]
    return message, command


def tail(run_dir: Path, command: str | None) -> str:
    if not command:
        return ""
    out = []
    for suffix in ("stderr", "stdout"):
        path = run_dir / f"{command}.{suffix}.log"
        if path.exists():
            out += [l for l in path.read_text(encoding="utf-8", errors="replace").splitlines() if l.strip()]
    return "\n".join(out[-TAIL_LINES:])


def kind_of(message: str, exc_type: str) -> str:
    if "timed out" in message:
        return "timeout"
    if code := re.search(r"\(exit (-?\d+)\)", message):
        return f"exit code {code.group(1)}"
    return "not runnable" if exc_type in ("FileNotFoundError", "PermissionError", "OSError") else f"check failed ({exc_type})"


def item_of(message: str) -> str:
    item = COMMAND_ID.sub("command", message)
    item = re.sub(r"\b[0-9a-f]{7,40}\b", "<sha>", item)
    item = re.sub(r"[A-Za-z]:\\[^\s;)]*\\", "", item)  # keep only the executable name of absolute paths
    return mask(item).strip()[:160]


def _relative(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return mask(str(path))


KEY = re.compile(r"^- Failure match: `sha=(?P<sha>\S+) \| stage=(?P<stage>\S+) \| item=(?P<item>.*)`$", re.M)


def _git_same_source(root: Path):
    """Two SHAs are the same source when they differ only in issue records / generated evidence
    (the previous failure issue itself is committed by the next run)."""
    import subprocess

    def same(a: str, b: str) -> bool:
        r = subprocess.run(["git", "diff", "--quiet", a, b, "--", ".", ":(exclude)" + REL["ISSUES"], ":(exclude)" + REL["EVIDENCE"]],
                           cwd=root, capture_output=True)
        return r.returncode == 0
    return same


def _active(root: Path, sha: str, stage: str, item: str, states: tuple[str, ...], same_source) -> Path | None:
    for state in states:
        for path in sorted((root / REL["ISSUES"] / state / FAMILY).glob("*.md")):
            found = KEY.search(path.read_text(encoding="utf-8"))
            if not found or found["stage"] != stage or found["item"] != item:
                continue
            old = found["sha"]
            if old == sha or ("unknown" not in (old, sha) and same_source(old, sha)):
                return path
    return None


def _add_priority(root: Path, issue_id: str, reason: str) -> None:
    path = root / REL["PRIORITY"]
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True) if path.exists() else \
        ["| Priority | UUID | Source |\n", "|---|---|---|\n"]
    if any(f"| {issue_id} |" in l for l in lines):
        return
    row = f"| Normal | {issue_id} | {reason} |\n"
    # after the last Critical/High/Normal row (rows are sorted by grade), else after the header
    anchor = max((i for i, l in enumerate(lines) if l.startswith(("| Critical ", "| High ", "| Normal ", "|---"))), default=len(lines) - 1)
    lines.insert(anchor + 1, row)
    path.write_text("".join(lines), encoding="utf-8")


def record(root: Path, run_dir: Path, *, stage: str, sha: str | None, sha_reason: str, message: str,
           exc_type: str, repro: str, run_id: str, new_id, guard=None, same_source=None) -> tuple[Path, bool]:
    """Create or update the failure issue. Returns (path, created). `guard` serializes writers."""
    with guard() if guard else nullcontext():
        return _record(root, run_dir, stage=stage, sha=sha, sha_reason=sha_reason, message=message,
                       exc_type=exc_type, repro=repro, run_id=run_id, new_id=new_id,
                       same_source=same_source or _git_same_source(Path(root)))


def _record(root, run_dir, *, stage, sha, sha_reason, message, exc_type, repro, run_id, new_id, same_source):
    root, run_dir = Path(root), Path(run_dir)
    inner, command = innermost(run_dir, message)
    item = item_of(inner)
    key_line = f"{KEY_PREFIX}`sha={sha or 'unknown'} | stage={stage} | item={item}`"
    when = datetime.now().astimezone().isoformat(timespec="seconds")
    evidence = _relative(root, run_dir) + (f"/{command}.stderr.log, {command}.stdout.log" if command else "")
    occurrence = f"- Occurred {when}: run `{run_id}`, SHA {sha or 'unknown'}, {kind_of(inner, exc_type)}, evidence `{evidence}`"
    key_sha = sha or "unknown"

    existing = _active(root, key_sha, stage, item, ("backlog", "working"), same_source)
    if existing:
        text = existing.read_text(encoding="utf-8")
        head, _, rest = text.partition("# expected result")
        existing.write_text(head.rstrip("\n") + "\n" + occurrence.replace("- Occurred", "- Recurred", 1) + "\n\n# expected result" + rest, encoding="utf-8")
        return existing, False

    previous = _active(root, key_sha, stage, item, ("closed",), same_source)
    issue_id = new_id(root)
    path = root / REL["ISSUES"] / "backlog" / FAMILY / f"{issue_id}.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    summary = mask(tail(run_dir, command)) or mask(inner)
    sha_text = sha or f"unknown ({sha_reason})"
    lines = [
        "# title",
        f"one-shot {stage} stage failed: {item[:60]}",
        "",
        "# pre-condition",
        f"- Target full SHA: {sha_text}",
        f"- Run ID: `{run_id}`",
        "",
        "# steps",
        "Source: rules/one-shot.md#stages-and-failure-handling / Origin: recorded automatically by one-shot",
        f"1. `{repro}`",
        "",
        "# actual result",
        key_line,
        f"- Failed stage / item: {stage} / {mask(inner)}",
        "- Error summary (masked):",
        "```text",
        summary,
        "```",
        "- Cause: Unconfirmed",
        "- Action: None",
        "- Re-verification: None",
    ]
    if previous:
        lines.append(f"- Previous record: `{_relative(root, previous)}` (closed, not reopened)")
    lines += [
        occurrence,
        "",
        "# expected result",
        f"- The one-shot {stage} stage passes for the same SHA. After the fix is verified, move this file to closed and remove its priority row.",
        "",
        "# label",
        "SQA_sk_0_0_0",
        "",
        "# environment",
        f"OS: {sys.platform}",
        "hostname: TBD",
        "",
        "# assignee",
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")
    _add_priority(root, issue_id, f"one-shot failure recorded automatically: {stage} / run {run_id}")
    return path, True
