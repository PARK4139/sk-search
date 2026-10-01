# e2e on the real release exe via UI Automation (no synthetic in-process calls).
# Evidence: ref/actual/screenshot/frames/*.png, ref/actual/logs/e2e-ui.log, ref/actual/logs/skim-search.log
param([switch]$NoBuild, [switch]$SystemOpen)
$ErrorActionPreference = "Stop"
$root = "%USERPROFILE%\Downloads\CavemanDrive\backlog\sk_search"
$shots = "$root\ref\actual\screenshot\frames"
$out = "$root\ref\actual\logs\e2e-ui.log"
$applog = "$root\ref\actual\logs\skim-search.log"
$exe = "$root\cores\target\release\skim-search.exe"
# cargo prints progress on stderr; run via cmd so PowerShell 5.1 does not turn it into an error
if (-not $NoBuild) { Push-Location "$root\cores"; cmd /c "cargo build --release 2>&1" | Out-Null; $b = $LASTEXITCODE; Pop-Location; if ($b -ne 0) { "BUILD_FAILED"; exit 1 } }

Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
}
public static class D {
  [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr h, int id);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr after, string cls, string title);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, string l);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@
[W]::SetProcessDPIAware() | Out-Null

function Log($m) { $line = "$(Get-Date -Format 'HH:mm:ss.fff') $m"; $line; Add-Content -Encoding UTF8 $out $line }
function Shot($hwnd, $name) {
  $r = New-Object W+RECT; [W]::GetWindowRect($hwnd, [ref]$r) | Out-Null
  $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
  $g = [System.Drawing.Graphics]::FromImage($bmp); $hdc = $g.GetHdc()
  [W]::PrintWindow($hwnd, $hdc, 2) | Out-Null; $g.ReleaseHdc($hdc); $g.Dispose()
  $bmp.Save("$shots\$name.png", [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
  Log "screenshot $name.png"
}
function Find($el, $name) {
  $c = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $name)
  $el.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $c)
}
function SetValue($el, $name, $value) {
  $e = Find $el $name
  if ($null -eq $e) { throw "element not found: $name" }
  $p = $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
  $p.SetValue($value)
  Log "set $name = '$value'"
}
function Invoke($el, $name) {
  $e = Find $el $name
  if ($null -eq $e) { throw "element not found: $name" }
  $e.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Log "invoke $name"
}
function GetValue($el, $name) {
  $e = Find $el $name
  if ($null -eq $e) { return $null }
  try { return $e.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value } catch { return $e.Current.Name }
}
$script:logMark = 0
function MarkLog() { $script:logMark = (Get-Content $applog -Encoding UTF8 | Measure-Object -Line).Lines }
# Lines appended to the app log since the last MarkLog.
function NewLog() { Get-Content $applog -Encoding UTF8 | Select-Object -Skip $script:logMark }
function WaitLog($pattern, $timeoutMs = 10000) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
    $hit = NewLog | Where-Object { $_ -match $pattern } | Select-Object -Last 1
    if ($hit) { return $hit }
    Start-Sleep -Milliseconds 50
  }
  return $null
}
function Search($a, $q) {
  MarkLog
  SetValue $a.el "query-input" $q
  $hit = WaitLog ("\[toast\] push .*title=검색 완료 body=" + [regex]::Escape($q.Trim()) + " ·")
  if (-not $hit) { throw "search not done: $q" }
  Log "done '$q': $hit"
  Start-Sleep -Milliseconds 200
}
# Sends keys only when skim-search is the foreground window (never type into the user's apps).
function Keys($a, [byte[]]$vks) {
  if ([W]::GetForegroundWindow() -ne $a.hwnd) { Log "SKIP keys $($vks -join ','): skim-search not foreground"; return $false }
  foreach ($k in $vks) { [W]::keybd_event($k, 0, 0, [UIntPtr]::Zero) }
  [array]::Reverse($vks); foreach ($k in $vks) { [W]::keybd_event($k, 0, 2, [UIntPtr]::Zero) }
  Log "keys $($vks -join ',')"
  return $true
}

# ── fixture workspace (common test workspace mirror) ──
$ws = Join-Path $env:TEMP "skim-search-e2e-ws"
if (Test-Path $ws) { Remove-Item -Recurse -Force $ws }
$files = @{
  "src\auth\login.ts" = "import { db } from '../lib/db'`nexport async function login(email: string) {`n  const result = await loginWithToken(token)`n  logger.info('login success')`n  throw new Error('login failed')`n}`n"
  "src\api\auth.ts" = "import { login } from '../auth/login'`nexport const auth = { login }`n"
  "src\auth\logout.ts" = "export function logout() {}`n// logout handler`n"
  "test\login.test.ts" = "describe('login', () => {})`nit('calls logout', () => logout())`n"
  "docs\login-guide.md" = "# Login Guide`nUse the login command to authenticate.`n## Logout`nRun logout when the session must end.`nTODO: document SSO login behavior.`n"
  "docs\install.md" = "# Installation`nTODO: add screenshots`n"
  "README.md" = "# my-project`nTODO: write readme`nlogin and logout supported`n"
  "config.json" = "{ `"login`": true }`n"
  "docs\sample file.md" = "TODO: sample with spaces`n"
}
foreach ($k in $files.Keys) { $p = Join-Path $ws $k; New-Item -ItemType Directory -Force (Split-Path $p) | Out-Null; [IO.File]::WriteAllText($p, $files[$k]) }
# more lines so the preview scrolls to the match
$long = (1..80 | ForEach-Object { "// filler $_" }) -join "`n"
[IO.File]::WriteAllText((Join-Path $ws "src\long.ts"), "$long`nexport const loginTarget = 1`n$long`n")

$tmp = Join-Path $env:TEMP "skim-search-e2e-cfg"; New-Item -ItemType Directory -Force $tmp | Out-Null
$bin = Join-Path $tmp "bin"; New-Item -ItemType Directory -Force $bin | Out-Null
Set-Content -Encoding ascii "$bin\code.cmd" "@echo %* > `"%~dp0code-args.txt`""
Set-Content -Encoding ascii "$bin\cursor.cmd" "@echo %* > `"%~dp0cursor-args.txt`""

function Settings($editor) {
  $cfg = @{ editor = $editor; recent_roots = @($ws) } | ConvertTo-Json
  [IO.File]::WriteAllText("$tmp\settings.json", $cfg)
}

function Launch() {
  $env:SKIM_SEARCH_SETTINGS = "$tmp\settings.json"
  $env:PATH = "$bin;" + $env:PATH
  $p = Start-Process -FilePath $exe -PassThru
  $hwnd = [IntPtr]::Zero; $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($hwnd -eq [IntPtr]::Zero -and $sw.ElapsedMilliseconds -lt 15000) { Start-Sleep -Milliseconds 100; $hwnd = [W]::FindWindow([NullString]::Value, "skim-search") }
  if ($hwnd -eq [IntPtr]::Zero) { throw "window not found" }
  [W]::ShowWindow($hwnd, 9) | Out-Null
  Start-Sleep -Milliseconds 500
  $el = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
  Log "launched pid=$($p.Id) hwnd=$hwnd"
  return @{ p = $p; hwnd = $hwnd; el = $el }
}
function Close($a) {
  [W]::PostMessage($a.hwnd, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
  if (-not $a.p.WaitForExit(5000)) { Stop-Process $a.p -Force; Log "FORCED EXIT" } else { Log "exit code $($a.p.ExitCode)" }
}

[IO.File]::WriteAllText((Join-Path $ws "open-test.txt"), "systemopen marker`n")

Set-Content -Encoding UTF8 $out "# e2e_ui $(Get-Date -Format s)"

# ── run 1: editor=vscode (fake code.cmd) ──
Settings "vscode"
$a = Launch

# A. layout / results / preview / status
Search $a "login"
Shot $a.hwnd "e2e_login"

# B. preview scrolls to a deep match and highlights it
Search $a "loginTarget"
Log "preview-loc=$(GetValue $a.el 'preview-loc') preview-path=$(GetValue $a.el 'preview-path')"
Shot $a.hwnd "e2e_scroll_highlight"

# C. latency: 10 x type 'login' (release build, local SSD)
$lat = @()
for ($i = 0; $i -lt 10; $i++) {
  SetValue $a.el "query-input" ""
  Start-Sleep -Milliseconds 100
  Search $a "login"
  $gen = (NewLog | Where-Object { $_ -match 'first_result_ms' } | Select-Object -Last 1) -replace '.*generation=(\d+).*', '$1'
  $vals = @{}
  foreach ($k in 'coalesced_ms', 'rg_spawn_ms', 'first_result_ms', 'completed_ms') {
    $line = NewLog | Where-Object { $_ -match "generation=$gen $k=" } | Select-Object -Last 1
    $vals[$k] = [double]($line -replace ".*$k=([0-9.]+).*", '$1')
  }
  $lat += [pscustomobject]$vals
  Log ("latency gen={0} coalesced={1} rg_spawn={2} first_result={3} completed={4}" -f $gen, $vals.coalesced_ms, $vals.rg_spawn_ms, $vals.first_result_ms, $vals.completed_ms)
}
foreach ($k in 'coalesced_ms', 'rg_spawn_ms', 'first_result_ms', 'completed_ms') {
  $m = $lat | Measure-Object -Property $k -Average -Maximum -Minimum
  Log ("latency summary {0}: avg={1:N1} min={2:N1} max={3:N1}" -f $k, $m.Average, $m.Minimum, $m.Maximum)
}

# D. markdown edit in preview (single text layer, dirty mark)
Search $a "TODO ext:md"
$path = GetValue $a.el "preview-path"
$text = GetValue $a.el "preview-editor"
SetValue $a.el "preview-editor" ($text + "TODO: edited in e2e`n")
Start-Sleep -Milliseconds 300
Log "after edit preview-path=$(GetValue $a.el 'preview-path')"
Shot $a.hwnd "e2e_md_edit"

# E. Ctrl+S from keyboard (only if foreground)
$editor = Find $a.el "preview-editor"; $editor.SetFocus(); Start-Sleep -Milliseconds 200
MarkLog
if (Keys $a ([byte[]](0x11, 0x53))) {
  $saved = WaitLog "\[preview_editor\] saved path="
  Log "ctrl+s: $saved"
}

# F. toast stack: 4 searches → max 3 visible; plus a warning
foreach ($q in "logout", "TODO", "auth", "login") { Search $a $q }
MarkLog
SetValue $a.el "search-root-input" (Join-Path $ws "README.md")
$warn = WaitLog "\[toast\] push .*kind=warning"
Log "warning toast: $warn"
Start-Sleep -Milliseconds 300
Shot $a.hwnd "e2e_toasts"
SetValue $a.el "search-root-input" $ws
Start-Sleep -Milliseconds 500

# G1. open file → fake code.cmd
Search $a "login"
Remove-Item "$bin\code-args.txt" -ErrorAction SilentlyContinue
Invoke $a.el "open-file-button"
Start-Sleep -Milliseconds 1500
Log "code args: $(Get-Content "$bin\code-args.txt" -ErrorAction SilentlyContinue)"

# I. Esc hides, Ctrl+Shift+F shows + focuses query
Search $a "auth"
[W]::PostMessage($a.hwnd, 0x0100, [IntPtr]0x1B, [IntPtr]0x00010001) | Out-Null   # WM_KEYDOWN VK_ESCAPE
[W]::PostMessage($a.hwnd, 0x0101, [IntPtr]0x1B, [IntPtr]0xC0010001) | Out-Null   # WM_KEYUP
Start-Sleep -Milliseconds 500
Log "after Esc visible=$([W]::IsWindowVisible($a.hwnd))"
MarkLog
[W]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero); [W]::keybd_event(0x10, 0, 0, [UIntPtr]::Zero); [W]::keybd_event(0x46, 0, 0, [UIntPtr]::Zero)
[W]::keybd_event(0x46, 0, 2, [UIntPtr]::Zero); [W]::keybd_event(0x10, 0, 2, [UIntPtr]::Zero); [W]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)
$hk = WaitLog "\[shortcut\] window shown and query focused" 5000
Start-Sleep -Milliseconds 300
Log "after hotkey visible=$([W]::IsWindowVisible($a.hwnd)) foreground=$([W]::GetForegroundWindow() -eq $a.hwnd) query='$(GetValue $a.el 'query-input')' log=$hk"
Shot $a.hwnd "e2e_after_hotkey"

# J. folder picker: cancel keeps root, select changes root
$pick = Join-Path $ws "docs"
function PickerDialog() {
  $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty, $a.p.Id)
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 8000) {
    $wins = [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
    foreach ($w in $wins) { if ($w.Current.ClassName -eq "#32770") { return $w } }
    Start-Sleep -Milliseconds 200
  }
  return $null
}
# Common file dialog controls: filename Edit id 1152, OK Button id 1.
function ById($el, $id, $type) {
  $c1 = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty, $id)
  $c2 = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, $type)
  $el.FindFirst([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.AndCondition($c1, $c2)))
}
$before = GetValue $a.el "search-root-input"
Invoke $a.el "browse-button"
$dlg = PickerDialog
if ($dlg) {
  Log "picker shown: '$($dlg.Current.Name)'"
  $dlg.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern).Close()   # = Cancel
  Start-Sleep -Milliseconds 500
  Log "after cancel root='$(GetValue $a.el 'search-root-input')' unchanged=$((GetValue $a.el 'search-root-input') -eq $before)"
  Invoke $a.el "browse-button"
  Start-Sleep -Milliseconds 800
  $dlg = PickerDialog
  # The dialog controls are only visible to UIA as Win32 panes: use Win32 messages.
  # id 1152 = "폴더:" field (ComboBoxEx → ComboBox → Edit), id 1 = "폴더 선택" button.
  $dh = [IntPtr]$dlg.Current.NativeWindowHandle
  $field = [D]::GetDlgItem($dh, 1152)
  $edit = $field
  foreach ($cls in "ComboBox", "Edit") { $c = [D]::FindWindowEx($edit, [IntPtr]::Zero, $cls, $null); if ($c -ne [IntPtr]::Zero) { $edit = $c } }
  [D]::SendMessage($edit, 0x000C, [IntPtr]::Zero, $pick) | Out-Null          # WM_SETTEXT
  Log "folder field hwnd=$field edit=$edit set='$pick'"
  [D]::PostMessage([D]::GetDlgItem($dh, 1), 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null   # BM_CLICK 폴더 선택
  Start-Sleep -Milliseconds 800
  Log "after select root='$(GetValue $a.el 'search-root-input')' expected='$pick'"
} else { Log "picker NOT found" }
Close $a

# ── run 2: editor=cursor ──
Settings "cursor"
$a = Launch
Search $a "login"
Remove-Item "$bin\cursor-args.txt" -ErrorAction SilentlyContinue
Invoke $a.el "open-file-button"
Start-Sleep -Milliseconds 1500
Log "cursor args: $(Get-Content "$bin\cursor-args.txt" -ErrorAction SilentlyContinue)"
Close $a

# ── run 3: editor=system + real Explorer ──
Settings "system"
$a = Launch
# System Default opens the file in the user's default app, which may be an app the user
# is working in (e.g. VS Code). Opt-in only (-SystemOpen), and NEVER close or kill any
# window/process this script did not start (closed/process/e2e-closed-user-vscode).
if ($SystemOpen) {
  Search $a "systemopen"
  MarkLog
  Invoke $a.el "open-file-button"
  $r = WaitLog "\[external_open\] open_file editor=system" 5000
  Log "system default: $r"
  $sw = [Diagnostics.Stopwatch]::StartNew(); $opened = $null
  while (-not $opened -and $sw.ElapsedMilliseconds -lt 8000) {
    Start-Sleep -Milliseconds 300
    $opened = Get-Process | Where-Object { $_.MainWindowTitle -like "*open-test*" } | Select-Object -First 1
  }
  if ($opened) { Log "system default window: $($opened.ProcessName) '$($opened.MainWindowTitle)' (left open on purpose)" } else { Log "system default window NOT found" }
} else { Log "SKIP system default open (pass -SystemOpen to run; opens the file in the user's default app)" }

$shell = New-Object -ComObject Shell.Application
$beforeWins = @($shell.Windows() | ForEach-Object { $_.HWND })
Invoke $a.el "open-parent-button"
$sw = [Diagnostics.Stopwatch]::StartNew(); $ex = $null
while (-not $ex -and $sw.ElapsedMilliseconds -lt 8000) {
  Start-Sleep -Milliseconds 300
  $ex = $shell.Windows() | Where-Object { $beforeWins -notcontains $_.HWND -and $_.Document.Folder.Self.Path -eq $ws } | Select-Object -First 1
}
if ($ex) {
  Start-Sleep -Milliseconds 500
  $sel = @($ex.Document.SelectedItems() | ForEach-Object { $_.Path })
  Log "explorer folder=$($ex.Document.Folder.Self.Path) selected=$($sel -join ';')"
  $ex.Quit()
} else { Log "explorer window NOT found" }
Close $a
Log "e2e_ui finished"
