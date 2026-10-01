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

from .. import security_policy as sp

FAMILY = Path("diagnostics") / "one-shot-failure"
KEY_PREFIX = "- 실패 키: "
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
        return "시간 초과"
    if code := re.search(r"\(exit (-?\d+)\)", message):
        return f"종료 코드 {code.group(1)}"
    return "실행 불가" if exc_type in ("FileNotFoundError", "PermissionError", "OSError") else f"검증 실패 ({exc_type})"


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


KEY = re.compile(r"^- 실패 키: `sha=(?P<sha>\S+) \| stage=(?P<stage>\S+) \| item=(?P<item>.*)`$", re.M)


def _git_same_source(root: Path):
    """Two SHAs are the same source when they differ only in issue records / generated evidence
    (the previous failure issue itself is committed by the next run)."""
    import subprocess

    def same(a: str, b: str) -> bool:
        r = subprocess.run(["git", "diff", "--quiet", a, b, "--", ".", ":(exclude)issues", ":(exclude)ref/actual"],
                           cwd=root, capture_output=True)
        return r.returncode == 0
    return same


def _active(root: Path, sha: str, stage: str, item: str, states: tuple[str, ...], same_source) -> Path | None:
    for state in states:
        for path in sorted((root / "issues" / state / FAMILY).glob("*.md")):
            found = KEY.search(path.read_text(encoding="utf-8"))
            if not found or found["stage"] != stage or found["item"] != item:
                continue
            old = found["sha"]
            if old == sha or ("미확보" not in (old, sha) and same_source(old, sha)):
                return path
    return None


def _add_priority(root: Path, issue_id: str, reason: str) -> None:
    path = root / "issues" / "priority.md"
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True) if path.exists() else \
        ["| 우선순위 | UUID | 근거 |\n", "|---|---|---|\n"]
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
    key_line = f"{KEY_PREFIX}`sha={sha or '미확보'} | stage={stage} | item={item}`"
    when = datetime.now().astimezone().isoformat(timespec="seconds")
    evidence = _relative(root, run_dir) + (f"/{command}.stderr.log, {command}.stdout.log" if command else "")
    occurrence = f"- 발생 {when}: run `{run_id}`, SHA {sha or '미확보'}, {kind_of(inner, exc_type)}, 증거 `{evidence}`"
    key_sha = sha or "미확보"

    existing = _active(root, key_sha, stage, item, ("backlog", "working"), same_source)
    if existing:
        text = existing.read_text(encoding="utf-8")
        head, _, rest = text.partition("# expected result")
        existing.write_text(head.rstrip("\n") + "\n" + occurrence.replace("- 발생", "- 재발", 1) + "\n\n# expected result" + rest, encoding="utf-8")
        return existing, False

    previous = _active(root, key_sha, stage, item, ("closed",), same_source)
    issue_id = new_id(root)
    path = root / "issues" / "backlog" / FAMILY / f"{issue_id}.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    summary = mask(tail(run_dir, command)) or mask(inner)
    sha_text = sha or f"미확보 ({sha_reason})"
    lines = [
        "# title",
        f"one-shot {stage} 단계 실패: {item[:60]}",
        "",
        "# pre-condition",
        f"- 대상 전체 SHA: {sha_text}",
        f"- 실행 ID: `{run_id}`",
        "",
        "# steps",
        "근거: rules/one-shot.md#단계와-실패-처리 / 발생: one-shot 자동 기록",
        f"1. `{repro}`",
        "",
        "# actual result",
        key_line,
        f"- 실패 단계·항목: {stage} / {mask(inner)}",
        "- 오류 요약 (마스킹):",
        "```text",
        summary,
        "```",
        "- 원인: 미확인",
        "- 조치: 없음",
        "- 재검증: 없음",
    ]
    if previous:
        lines.append(f"- 이전 기록: `{_relative(root, previous)}` (closed, 재개하지 않음)")
    lines += [
        occurrence,
        "",
        "# expected result",
        f"- 같은 SHA에서 one-shot {stage} 단계가 통과한다. 해결·재검증 후 이 파일을 closed로 옮기고 priority 행을 제거한다.",
        "",
        "# label",
        "SQA_sk_0_0_0",
        "",
        "# environment",
        f"OS: {sys.platform}",
        "hostname: TBD",
        "",
        "# 담당자",
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")
    _add_priority(root, issue_id, f"one-shot 실패 자동 기록: {stage} / run {run_id}")
    return path, True
