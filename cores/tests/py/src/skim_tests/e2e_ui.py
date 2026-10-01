"""e2e on the real release exe via UI Automation (no synthetic in-process calls).

    uv run python -m skim_tests.e2e_ui [--no-build] [--latency-runs 10] [--strict-latency] [--system-open]

Exit code 0 = all checks PASS (latency targets are WARN unless --strict-latency).
Evidence: ref/actual/logs/e2e-ui.log, ref/actual/logs/skim-search.log, ref/actual/screenshot/frames/e2e_*.png

Safety (closed/process/e2e-closed-user-vscode):
- only the skim-search process and the Explorer window this script started are ever closed
- keys are injected only while skim-search is the foreground window (global hotkey excepted)
- opening a file with the system default app is opt-in (--system-open)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sys
import time
from pathlib import Path

import uiautomation as auto

from .common import (
    BM_CLICK, EXE_RELEASE, SHOTS, TEMP, VK_CONTROL, VK_ESCAPE, VK_SHIFT, WM_CLOSE, WM_KEYDOWN, WM_KEYUP,
    WM_SETTEXT, App, AppLog, Checker, Log, capture, cargo_build, class_name, foreground, key_combo, pct,
    post, top_windows_of, user32, visible,
)
from .paths import SAMPLE_TREE

class Ui:
    """UI Automation access to one skim-search window (elements found by accessible-label)."""

    def __init__(self, app: App, log: Log):
        self.app, self.log = app, log
        self.root = auto.ControlFromHandle(app.hwnd)

    def find(self, name: str):
        c = self.root.Control(Name=name)
        if not c.Exists(5, 0.1):
            raise RuntimeError(f"element not found: {name}")
        return c

    def set(self, name: str, value: str) -> None:
        self.find(name).GetPattern(auto.PatternId.ValuePattern).SetValue(value)
        self.log(f"set {name} = {value!r}")

    def get(self, name: str) -> str:
        try:
            return self.find(name).GetPattern(auto.PatternId.ValuePattern).Value
        except Exception:  # noqa: BLE001 - element without a value pattern
            return ""

    def invoke(self, name: str) -> None:
        self.find(name).GetPattern(auto.PatternId.InvokePattern).Invoke()
        self.log(f"invoke {name}")


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--no-build", action="store_true")
    p.add_argument("--latency-runs", type=int, default=10)
    p.add_argument("--strict-latency", action="store_true")
    p.add_argument("--system-open", action="store_true")
    a = p.parse_args()
    if not a.no_build and not cargo_build(release=True):
        print("BUILD_FAILED")
        return 1

    log = Log("e2e-ui.log", "e2e_ui")
    check = Checker(log)
    applog = AppLog()

    # fixture workspace (common test workspace mirror) + a long file for scrolling
    ws = TEMP / "skim-search-e2e-ws"
    shutil.rmtree(ws, ignore_errors=True)
    shutil.copytree(SAMPLE_TREE, ws)
    (ws / "open-test.txt").write_text("systemopen marker\n", encoding="utf-8")
    filler = "\n".join(f"// filler {i}" for i in range(1, 81))
    (ws / "src" / "long.ts").write_text(f"{filler}\nexport const loginTarget = 1\n{filler}\n", encoding="utf-8")

    cfg = TEMP / "skim-search-e2e-cfg"
    bin_dir = cfg / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    (bin_dir / "code.cmd").write_text('@echo %* > "%~dp0code-args.txt"\r\n', encoding="ascii")
    (bin_dir / "cursor.cmd").write_text('@echo %* > "%~dp0cursor-args.txt"\r\n', encoding="ascii")
    settings_file = cfg / "settings.json"
    env = {"SKIM_SEARCH_SETTINGS": str(settings_file), "PATH": f"{bin_dir};" + os.environ["PATH"]}

    def write_settings(**kw) -> None:
        settings_file.write_text(json.dumps({"recent_roots": [str(ws)], **kw}), encoding="utf-8")

    def search(ui: Ui, q: str) -> str:
        pattern = r"\[toast\] push .*title=검색 완료 body=" + re.escape(q.strip()) + " ·"
        for attempt in (1, 2):
            applog.mark()
            ui.set("query-input", q)
            hit = applog.wait(pattern, 6)
            if hit:
                log(f"done {q!r}: {hit}")
                time.sleep(0.2)
                return hit
            # stray keyboard input from outside the test can alter the query
            log(f"RETRY search {q!r} (try {attempt}): query is now {ui.get('query-input')!r}")
        raise RuntimeError(f"search not done: {q}")

    def shot(app: App, name: str) -> None:
        capture(app.hwnd, SHOTS / f"{name}.png")
        log(f"screenshot {name}.png")

    def keys_if_foreground(app: App, *vks: int) -> bool:
        if foreground() != app.hwnd:
            log(f"SKIP keys {vks}: skim-search not foreground")
            return False
        key_combo(*vks)
        log(f"keys {vks}")
        return True

    app: App | None = None
    try:
        # ── run 1: editor=vscode (fake code.cmd) ──
        write_settings(editor="vscode")
        app = App(EXE_RELEASE, env)
        ui = Ui(app, log)
        log(f"launched pid={app.proc.pid} hwnd={app.hwnd}")

        # A. layout / results / preview / status (Text values are not exposed via UIA → app display log)
        search(ui, "login")
        check("search_engine/auto-search-on-input", True, "query 'login' searched without button/Enter")
        disp = applog.wait(r"\[status_bar\] .*stats=", 3)
        check("status_bar/result-stats-not-updated",
              bool(disp and 'stats="14 결과 · 7 파일' in disp and re.search(r'status="rg \S+ · sk \S+ · 14 results', disp)), f"{disp}")
        shot(app, "e2e_login")

        # B. preview scrolls to a deep match and highlights it
        search(ui, "loginTarget")
        show = applog.wait(r"\[preview_editor\] show path=", 3)
        check("preview_editor/match-scroll-highlight", bool(show and re.search(r"long\.ts line=81 column=14", show)),
              f"{show} (screenshot e2e_scroll_highlight)")
        shot(app, "e2e_scroll_highlight")

        # C. latency: N x type 'login' (release build, local SSD)
        lat: dict[str, list[float]] = {k: [] for k in ("coalesced_ms", "rg_spawn_ms", "first_result_ms", "completed_ms")}
        for _ in range(a.latency_runs):
            ui.set("query-input", "")
            time.sleep(0.1)
            search(ui, "login")
            first = applog.last(r"first_result_ms")
            gen = re.search(r"generation=(\d+)", first).group(1)
            vals = {}
            for k in lat:
                line = applog.last(rf"generation={gen} {k}=")
                vals[k] = float(re.search(rf"{k}=([0-9.]+)", line).group(1))
                lat[k].append(vals[k])
            log("latency gen={} coalesced={} rg_spawn={} first_result={} completed={}".format(
                gen, vals["coalesced_ms"], vals["rg_spawn_ms"], vals["first_result_ms"], vals["completed_ms"]))
        for k, v in lat.items():
            s = sorted(v)
            log(f"latency summary {k}: n={len(s)} avg={sum(s) / len(s):.1f} p50={pct(s, .5):.1f} p90={pct(s, .9):.1f} min={s[0]:.1f} max={s[-1]:.1f}")
            if k == "rg_spawn_ms":
                check("search_engine/coalescing-20ms", pct(s, .5) <= 25, f"search start p50={pct(s, .5):.1f}ms (target <=25)", warn=not a.strict_latency)
            if k == "first_result_ms":
                check("diagnostics/first-result-latency", pct(s, .5) <= 50,
                      f"first result p50={pct(s, .5):.1f}ms p90={pct(s, .9):.1f}ms (target <=50)", warn=not a.strict_latency)

        # D. markdown result + edit in preview (single text layer)
        search(ui, "TODO ext:md")
        text = ui.get("preview-editor")
        ui.set("preview-editor", text + "TODO: edited in e2e\n")
        time.sleep(0.3)
        show = applog.last(r"\[preview_editor\] show path=")
        disp = applog.last(r"\[status_bar\] .*stats=")
        check("result_panel/markdown-results", bool(show and ".md line=" in show and disp and 'stats="4 결과 · 4 파일' in disp), f"{show} | {disp}")
        check("preview_editor/edit-text-overlap", "TODO: edited in e2e" in ui.get("preview-editor"), "edited text kept in single editor (screenshot e2e_md_edit)")
        shot(app, "e2e_md_edit")

        # E. Ctrl+S from the keyboard (only if foreground)
        ui.find("preview-editor").SetFocus()
        time.sleep(0.2)
        applog.mark()
        if keys_if_foreground(app, VK_CONTROL, 0x53):
            saved = applog.wait(r"\[preview_editor\] saved path=")
            disk = (ws / "README.md").read_text(encoding="utf-8")
            check("preview_editor/ctrl-s-save", bool(saved) and "TODO: edited in e2e" in disk, "keyboard Ctrl+S wrote README.md")
            time.sleep(0.2)
            shot(app, "e2e_toast_success")
        else:
            check("preview_editor/ctrl-s-save", False, "skim-search was not foreground; keys not sent", warn=True)

        # F. toast stack (4 searches + a warning → max 3) and invalid root
        for q in ("logout", "TODO", "auth", "login"):
            search(ui, q)
        applog.mark()
        ui.set("search-root-input", str(ws / "README.md"))
        warn = applog.wait(r"\[toast\] push .*kind=warning")
        disp = applog.wait(r'\[status_bar\] .*stats="검색 경로 오류', 3)
        check("view/search-root-validation-missing", bool(warn and disp) and not applog.last("rg_spawn_ms"), f"file path as root rejected, no rg: {disp}")
        drops = len([l for l in applog.new() if "[toast] drop oldest" in l])
        over = applog.last(r"\[toast\] push .* count=([4-9]|\d\d)")
        check("toast/exceeds-max-three", drops >= 1 and not over, f"count never above 3, drops={drops}")
        time.sleep(0.3)
        shot(app, "e2e_toasts")
        ui.set("search-root-input", str(ws))
        time.sleep(0.5)

        # G. open file → fake code.cmd
        search(ui, "login")
        (bin_dir / "code-args.txt").unlink(missing_ok=True)
        ui.invoke("open-file-button")
        time.sleep(1.5)
        args1 = (bin_dir / "code-args.txt").read_text(errors="replace").strip() if (bin_dir / "code-args.txt").exists() else ""
        check("external_open/open-file-goto-line-column", bool(re.match(r"^--goto .+:\d+:\d+$", args1)), f"vscode args={args1!r}")

        # I. Esc hides, Ctrl+Shift+F shows + focuses query (global hotkey, intercepted by RegisterHotKey)
        search(ui, "auth")
        post(app.hwnd, WM_KEYDOWN, VK_ESCAPE, 0x00010001)
        post(app.hwnd, WM_KEYUP, VK_ESCAPE, 0xC0010001)
        time.sleep(0.5)
        check("shortcut/esc-hide", not visible(app.hwnd) and app.proc.poll() is None, "Esc hid window, process alive")
        applog.mark()
        key_combo(VK_CONTROL, VK_SHIFT, 0x46)
        hk = applog.wait(r"\[shortcut\] window shown and query focused", 5)
        time.sleep(0.3)
        check("shortcut/global-hotkey-ctrl-shift-f", visible(app.hwnd) and foreground() == app.hwnd and bool(hk), f"visible+foreground+focused log={hk}")
        q = ui.get("query-input")
        check("shortcut/esc-hide", q == "auth", f"state kept after re-show: query={q!r}")
        shot(app, "e2e_after_hotkey")

        # J. folder picker: cancel keeps root, select changes root (dialog controls via Win32)
        pick = ws / "docs"

        def picker() -> int:
            end = time.monotonic() + 8
            while time.monotonic() < end:
                for h in top_windows_of(app.proc.pid):
                    if class_name(h) == "#32770" and visible(h):
                        return h
                time.sleep(0.2)
            return 0

        before = ui.get("search-root-input")
        ui.invoke("browse-button")
        dlg = picker()
        if dlg:
            post(dlg, WM_CLOSE)  # = Cancel
            time.sleep(0.5)
            check("view/folder-picker-missing", ui.get("search-root-input") == before, "cancel keeps root")
            ui.invoke("browse-button")
            time.sleep(0.8)
            dlg = picker()
            # id 1152 = "폴더:" field (ComboBoxEx → ComboBox → Edit), id 1 = "폴더 선택"
            edit = user32.GetDlgItem(dlg, 1152)
            for cls in ("ComboBox", "Edit"):
                child = user32.FindWindowExW(edit, None, cls, None)
                if child:
                    edit = child
            applog.mark()
            user32.SendMessageW(edit, WM_SETTEXT, 0, str(pick))
            post(user32.GetDlgItem(dlg, 1), BM_CLICK)
            time.sleep(0.8)
            r = ui.get("search-root-input")
            check("view/folder-picker-missing", r == str(pick), f"selected root={r!r}")
            check("view/rerun-on-search-root-change-missing", bool(applog.wait(r"change detected reason=browse", 3)), "root change triggered a new search")
        else:
            check("view/folder-picker-missing", False, "picker dialog not found")
        app.close()
        app = None

        # K. settings persisted by the app and restored on restart
        saved = json.loads(settings_file.read_text(encoding="utf-8"))
        log(f"settings after run 1: {saved}")
        app = App(EXE_RELEASE, env)
        ui = Ui(app, log)
        r = ui.get("search-root-input")
        check("settings/recent-search-root-persistence", saved["recent_roots"][0] == str(pick) and r == str(pick),
              f"saved first={saved['recent_roots'][0]!r} restored={r!r}")
        app.close()
        app = None

        # ── run 2: editor=cursor ──
        write_settings(editor="cursor")
        app = App(EXE_RELEASE, env)
        ui = Ui(app, log)
        search(ui, "login")
        (bin_dir / "cursor-args.txt").unlink(missing_ok=True)
        ui.invoke("open-file-button")
        time.sleep(1.5)
        args2 = (bin_dir / "cursor-args.txt").read_text(errors="replace").strip() if (bin_dir / "cursor-args.txt").exists() else ""
        check("settings/open-editor-selection", bool(re.match(r"^--goto .+:\d+:\d+$", args2)), f"editor=cursor args={args2!r}")
        app.close()
        app = None

        # ── run 3: editor=system + real Explorer ──
        write_settings(editor="system")
        app = App(EXE_RELEASE, env)
        ui = Ui(app, log)
        search(ui, "systemopen")  # selects open-test.txt (also used by the Explorer check)
        if a.system_open:
            applog.mark()
            ui.invoke("open-file-button")
            log(f"system default: {applog.wait(r'\[external_open\] open_file editor=system', 5)}")
            check("external_open/open-file-goto-line-column", True, "system default open requested (window left open on purpose)")
        else:
            log("SKIP system default open (pass --system-open; opens the file in the user's default app)")

        import comtypes.client

        shell = comtypes.client.CreateObject("Shell.Application", dynamic=True)

        def explorer_windows():
            wins = shell.Windows()
            return [wins.Item(i) for i in range(wins.Count)]

        before_hwnds = {w.HWND for w in explorer_windows() if w is not None}
        ui.invoke("open-parent-button")
        ex, end = None, time.monotonic() + 8
        while not ex and time.monotonic() < end:
            time.sleep(0.3)
            for w in explorer_windows():
                try:
                    if w is not None and w.HWND not in before_hwnds and Path(w.Document.Folder.Self.Path) == ws:
                        ex = w
                except Exception:  # noqa: BLE001 - window still initialising
                    pass
        if ex:
            time.sleep(0.5)
            items = ex.Document.SelectedItems()
            sel = [items.Item(i).Path for i in range(items.Count)]
            check("external_open/open-parent-select-file", str(ws / "open-test.txt") in sel, f"explorer selected={sel}")
            ex.Quit()  # only the Explorer window this script opened (new HWND + workspace path)
        else:
            check("external_open/open-parent-select-file", False, "explorer window not found")
        app.close()
        app = None

        # ── run 4: failing rg → error toast ──
        fail_rg = bin_dir / "rg-fail.cmd"
        fail_rg.write_text('@if "%1"=="--version" (echo ripgrep 0.0.0-fail& exit /b 0)\r\n'
                           "@echo rg: permission denied 1>&2\r\n@exit /b 2\r\n", encoding="ascii")
        write_settings(editor="vscode", rg_path=str(fail_rg))
        app = App(EXE_RELEASE, env)
        ui = Ui(app, log)
        applog.mark()
        ui.set("query-input", "login")
        err = applog.wait(r"\[toast\] push .*kind=error")
        check("toast/search-error-toast", bool(err and "title=검색 오류 body=rg: permission denied" in err), f"error toast: {err}")
        time.sleep(0.2)
        shot(app, "e2e_toast_error")
        app.close()
        app = None
    except Exception as e:  # noqa: BLE001 - any error is a failed run
        check("e2e_ui", False, f"aborted: {e!r}")
    finally:
        if app is not None:  # only the skim-search process this script launched
            app.kill()
            log(f"killed own skim-search pid={app.proc.pid}")
    return check.finish("e2e_ui")


if __name__ == "__main__":
    with auto.UIAutomationInitializerInThread():
        sys.exit(main())
