"""Shared helpers for skim-search verification scripts.

- paths (repository, 3rd_party tools, evidence locations)
- Log: UTF-8 log file under ref/actual/logs + console echo
- Checker: CHECK PASS/WARN/FAIL lines and the process exit code
- AppLog: read the app runtime log (ref/actual/logs/skim-search.log) since a mark
- Win32 via ctypes: windows, keyboard, messages, PrintWindow capture
- App: launch / find window / close only the process this script started
"""

from __future__ import annotations

import ctypes
import ctypes.wintypes as wt
import os
import re
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

from .paths import (
    APP_LOG, CORES, EXE_DEBUG, EXE_RELEASE, LOGS, REPO, RG, SHOTS, SK,
    TEMP, THIRD_PARTY, WINDOW_TITLE,
)

# console output in UTF-8 regardless of the code page (log files are always UTF-8)
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        _stream.reconfigure(encoding="utf-8", errors="replace")

def cargo_build(release: bool) -> bool:
    args = ["cargo", "build"] + (["--release"] if release else [])
    return subprocess.run(args, cwd=CORES, capture_output=True).returncode == 0


# ── logging / checks ───────────────────────────────────────────────────


class Log:
    def __init__(self, name: str, header: str = ""):
        LOGS.mkdir(parents=True, exist_ok=True)
        self.path = LOGS / name
        self.path.write_text(f"# {header or name} {datetime.now().isoformat(timespec='seconds')}\n", encoding="utf-8")

    def __call__(self, msg: str) -> None:
        line = f"{datetime.now().strftime('%H:%M:%S.%f')[:-3]} {msg}"
        print(line, flush=True)
        with self.path.open("a", encoding="utf-8") as f:
            f.write(line + "\n")


class Checker:
    """Collects verification results. `issue` is the sub family path the check belongs to."""

    def __init__(self, log: Log):
        self.log = log
        self.passes = self.warns = self.fails = 0

    def __call__(self, issue: str, ok: bool, detail: str, warn: bool = False) -> bool:
        if ok:
            self.passes += 1
            self.log(f"CHECK PASS [{issue}] {detail}")
        elif warn:
            self.warns += 1
            self.log(f"CHECK WARN [{issue}] {detail}")
        else:
            self.fails += 1
            self.log(f"CHECK FAIL [{issue}] {detail}")
        return ok

    def finish(self, name: str) -> int:
        self.log(f"{name} summary: pass={self.passes} warn={self.warns} fail={self.fails}")
        return 1 if self.fails else 0


def pct(sorted_values: list[float], p: float) -> float:
    i = max(0, min(len(sorted_values) - 1, int(-(-len(sorted_values) * p // 1)) - 1))
    return sorted_values[i]


class AppLog:
    """Lines appended to the app runtime log since the last mark()."""

    def __init__(self) -> None:
        self.offset = 0
        self.mark()

    def mark(self) -> None:
        self.offset = APP_LOG.stat().st_size if APP_LOG.exists() else 0

    def new(self) -> list[str]:
        if not APP_LOG.exists():
            return []
        with APP_LOG.open("rb") as f:
            f.seek(self.offset)
            data = f.read()
        return data.decode("utf-8", errors="replace").splitlines()

    def last(self, pattern: str) -> str | None:
        rx = re.compile(pattern)
        hits = [l for l in self.new() if rx.search(l)]
        return hits[-1] if hits else None

    def wait(self, pattern: str, timeout: float = 10.0) -> str | None:
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            hit = self.last(pattern)
            if hit:
                return hit
            time.sleep(0.05)
        return None


# ── Win32 (ctypes) ─────────────────────────────────────────────────────

user32 = ctypes.WinDLL("user32", use_last_error=True)
gdi32 = ctypes.WinDLL("gdi32", use_last_error=True)
user32.SetProcessDPIAware()

user32.FindWindowW.restype = wt.HWND
user32.FindWindowW.argtypes = [wt.LPCWSTR, wt.LPCWSTR]
user32.FindWindowExW.restype = wt.HWND
user32.FindWindowExW.argtypes = [wt.HWND, wt.HWND, wt.LPCWSTR, wt.LPCWSTR]
user32.GetDlgItem.restype = wt.HWND
user32.GetDlgItem.argtypes = [wt.HWND, ctypes.c_int]
user32.GetForegroundWindow.restype = wt.HWND
user32.SendMessageW.argtypes = [wt.HWND, wt.UINT, wt.WPARAM, wt.LPCWSTR]
user32.PostMessageW.argtypes = [wt.HWND, wt.UINT, wt.WPARAM, wt.LPARAM]
user32.GetWindowRect.argtypes = [wt.HWND, ctypes.POINTER(wt.RECT)]
user32.IsWindowVisible.argtypes = [wt.HWND]
user32.ShowWindow.argtypes = [wt.HWND, ctypes.c_int]
user32.GetWindowThreadProcessId.argtypes = [wt.HWND, ctypes.POINTER(wt.DWORD)]
user32.GetClassNameW.argtypes = [wt.HWND, wt.LPWSTR, ctypes.c_int]
user32.PrintWindow.argtypes = [wt.HWND, wt.HDC, wt.UINT]
user32.GetDC.restype = wt.HDC
user32.GetDC.argtypes = [wt.HWND]
user32.ReleaseDC.argtypes = [wt.HWND, wt.HDC]
gdi32.CreateCompatibleDC.restype = wt.HDC
gdi32.CreateCompatibleDC.argtypes = [wt.HDC]
gdi32.CreateCompatibleBitmap.restype = wt.HBITMAP
gdi32.CreateCompatibleBitmap.argtypes = [wt.HDC, ctypes.c_int, ctypes.c_int]
gdi32.SelectObject.restype = wt.HGDIOBJ
gdi32.SelectObject.argtypes = [wt.HDC, wt.HGDIOBJ]
gdi32.DeleteObject.argtypes = [wt.HGDIOBJ]
gdi32.DeleteDC.argtypes = [wt.HDC]
gdi32.GetDIBits.argtypes = [wt.HDC, wt.HBITMAP, wt.UINT, wt.UINT, ctypes.c_void_p, ctypes.c_void_p, wt.UINT]

WM_CLOSE, WM_KEYDOWN, WM_KEYUP, WM_SETTEXT, BM_CLICK = 0x0010, 0x0100, 0x0101, 0x000C, 0x00F5
SW_RESTORE = 9
VK_CONTROL, VK_SHIFT, VK_ESCAPE = 0x11, 0x10, 0x1B
KEYEVENTF_KEYUP = 2


def find_window(title: str = WINDOW_TITLE) -> int:
    return user32.FindWindowW(None, title) or 0


def foreground() -> int:
    return user32.GetForegroundWindow() or 0


def visible(hwnd: int) -> bool:
    return bool(user32.IsWindowVisible(hwnd))


def post(hwnd: int, msg: int, wparam: int = 0, lparam: int = 0) -> None:
    user32.PostMessageW(hwnd, msg, wparam, lparam)


def key_combo(*vks: int) -> None:
    """Global key injection (keybd_event). Callers must ensure where the keys go."""
    for vk in vks:
        user32.keybd_event(vk, 0, 0, 0)
    for vk in reversed(vks):
        user32.keybd_event(vk, 0, KEYEVENTF_KEYUP, 0)


def window_pid(hwnd: int) -> int:
    pid = wt.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
    return pid.value


def class_name(hwnd: int) -> str:
    buf = ctypes.create_unicode_buffer(256)
    user32.GetClassNameW(hwnd, buf, 256)
    return buf.value


def top_windows_of(pid: int) -> list[int]:
    out: list[int] = []
    proto = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)

    def cb(hwnd, _):
        if window_pid(hwnd) == pid:
            out.append(hwnd)
        return True

    user32.EnumWindows(proto(cb), 0)
    return out


def capture(hwnd: int, png: Path) -> tuple[int, int]:
    """PrintWindow(PW_RENDERFULLCONTENT): this window only, even if covered by other windows."""
    from PIL import Image

    r = wt.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(r))
    w, h = r.right - r.left, r.bottom - r.top
    screen = user32.GetDC(None)
    mem = gdi32.CreateCompatibleDC(screen)
    bmp = gdi32.CreateCompatibleBitmap(screen, w, h)
    old = gdi32.SelectObject(mem, bmp)
    user32.PrintWindow(hwnd, mem, 2)

    class BITMAPINFOHEADER(ctypes.Structure):
        _fields_ = [
            ("biSize", wt.DWORD), ("biWidth", wt.LONG), ("biHeight", wt.LONG), ("biPlanes", wt.WORD),
            ("biBitCount", wt.WORD), ("biCompression", wt.DWORD), ("biSizeImage", wt.DWORD),
            ("biXPelsPerMeter", wt.LONG), ("biYPelsPerMeter", wt.LONG), ("biClrUsed", wt.DWORD),
            ("biClrImportant", wt.DWORD),
        ]

    bih = BITMAPINFOHEADER(ctypes.sizeof(BITMAPINFOHEADER), w, -h, 1, 32, 0, 0, 0, 0, 0, 0)
    buf = ctypes.create_string_buffer(w * h * 4)
    gdi32.GetDIBits(mem, bmp, 0, h, buf, ctypes.byref(bih), 0)
    gdi32.SelectObject(mem, old)
    gdi32.DeleteObject(bmp)
    gdi32.DeleteDC(mem)
    user32.ReleaseDC(None, screen)
    png.parent.mkdir(parents=True, exist_ok=True)
    Image.frombuffer("RGBA", (w, h), buf.raw, "raw", "BGRA", 0, 1).convert("RGB").save(png)
    return w, h


# ── app process ────────────────────────────────────────────────────────


class App:
    """A skim-search process started by this script. Only this process/window is ever closed."""

    def __init__(self, exe: Path, env: dict[str, str] | None = None, timeout: float = 15.0):
        self.proc = subprocess.Popen([str(exe)], env={**os.environ, **(env or {})})
        end = time.monotonic() + timeout
        self.hwnd = 0
        while time.monotonic() < end:
            hwnd = find_window()
            if hwnd and window_pid(hwnd) == self.proc.pid:
                self.hwnd = hwnd
                break
            time.sleep(0.1)
        if not self.hwnd:
            self.kill()
            raise RuntimeError(f"window '{WINDOW_TITLE}' of pid {self.proc.pid} not found")
        user32.ShowWindow(self.hwnd, SW_RESTORE)
        time.sleep(0.5)

    def close(self, timeout: float = 5.0) -> int | None:
        """WM_CLOSE to our window; returns the exit code (None if it had to be killed)."""
        if self.proc.poll() is None:
            post(self.hwnd, WM_CLOSE)
            try:
                return self.proc.wait(timeout)
            except subprocess.TimeoutExpired:
                self.kill()
                return None
        return self.proc.returncode

    def kill(self) -> None:
        if self.proc.poll() is None:
            self.proc.kill()
            self.proc.wait()
