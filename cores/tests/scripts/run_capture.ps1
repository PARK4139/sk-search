param([string]$Exe, [string]$Out, [string]$Title = "skim-search", [int]$WaitMs = 3000)
# Launch exe, find its top-level window by title, capture to PNG, close via WM_CLOSE, report exit code.
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@
[W]::SetProcessDPIAware() | Out-Null
$p = Start-Process -FilePath $Exe -PassThru
$hwnd = [IntPtr]::Zero
$deadline = (Get-Date).AddSeconds(15)
while ($hwnd -eq [IntPtr]::Zero -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200; $hwnd = [W]::FindWindow([NullString]::Value, $Title) }
if ($hwnd -eq [IntPtr]::Zero) { "NO_WINDOW"; Stop-Process $p -Force; exit 1 }
[W]::ShowWindow($hwnd, 9) | Out-Null   # SW_RESTORE
[W]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds $WaitMs
$r = New-Object W+RECT
[W]::GetWindowRect($hwnd, [ref]$r) | Out-Null
$w = $r.R - $r.L; $h = $r.B - $r.T
$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
# PrintWindow (PW_RENDERFULLCONTENT=2): captures this window only, even if covered by others.
$hdc = $g.GetHdc(); $ok = [W]::PrintWindow($hwnd, $hdc, 2); $g.ReleaseHdc($hdc)
if (-not $ok) { "PRINTWINDOW_FAILED" }
New-Item -ItemType Directory -Force (Split-Path $Out) | Out-Null
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
"CAPTURED ${w}x${h} -> $Out"
[W]::PostMessage($hwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null   # WM_CLOSE
if (-not $p.WaitForExit(5000)) { "NO_GRACEFUL_EXIT"; Stop-Process $p -Force; exit 1 }
"EXIT_CODE $($p.ExitCode)"
