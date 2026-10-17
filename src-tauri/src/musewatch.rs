// Muse watcher — Win32 foreground polling via PowerShell, summarize with Claude Code CLI.
//
// Avoids new Rust dependencies (no windows-sys) by delegating Win32 P/Invoke to
// `powershell.exe`. Polls the foreground window every 5s when muse state == "watching",
// and on a new title+process pair, shells out to `claude -p` for a one-line summary.
// Results append to <vault>/Activity/<YYYY-MM-DD>.md so the user has a calm log.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc,
    },
    thread,
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::Config;

/// Suppress the Win32 console window when spawning child processes.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const POLL_INTERVAL_SECS: u64 = 1; // 1s tick — low cost (PowerShell is the only overhead)

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

pub fn spawn(cfg: Config) {
    thread::Builder::new()
        .name("muse-watcher".into())
        .spawn(move || run_loop(cfg, None))
        .expect("failed to spawn muse watcher thread");
}

/// Spawn the watcher with a Win32 event-hook receiver. The watcher reacts to
/// each foreground change in <50ms instead of waiting up to 1s for the poll.
/// The 1s poll still runs as a safety net for windows the hook misses (e.g.,
/// minimized-to-tray with no focus event).
pub fn spawn_with_hook(cfg: Config, hook_rx: Receiver<isize>) {
    thread::Builder::new()
        .name("muse-watcher".into())
        .spawn(move || run_loop(cfg, Some(hook_rx)))
        .expect("failed to spawn muse watcher thread");
}

pub fn stop() {
    STOP_FLAG.store(true, Ordering::SeqCst);
}

fn run_loop(cfg: Config, hook_rx: Option<Receiver<isize>>) {
    let activity_dir = cfg.vault_path().join("Activity");
    let _ = fs::create_dir_all(&activity_dir);
    let screenshots_dir = cfg.vault_path().join("Activity").join("screenshots");
    if cfg.muse.screenshots {
        let _ = fs::create_dir_all(&screenshots_dir);
    }
    // Ticks required to consider a title stable. If the user set a custom
    // debounce in config, honor it; otherwise fall back to 2 ticks (10s).
    let debounce_ticks = if cfg.muse.debounce_secs == 0 {
        0
    } else {
        (cfg.muse.debounce_secs / POLL_INTERVAL_SECS as u32).max(1)
    };
    let skip_processes: Vec<String> = cfg
        .muse
        .skip_processes
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let mut last_signature: Option<String> = None;
    let mut stable_ticks: u32 = 0;
    let mut last_recorded: Option<(String, String)> = None; // (process, normalized_title)
    while !STOP_FLAG.load(Ordering::SeqCst) {
        // Wait for either a Win32 event (sub-50ms when the foreground changes)
        // or the poll timer (1s) — whichever comes first. If the hook is
        // missing, the timer always wins.
        let _ = match hook_rx.as_ref() {
            Some(rx) => rx.recv_timeout(Duration::from_secs(POLL_INTERVAL_SECS)),
            None => {
                thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
                Ok(0)
            }
        };
        if read_state(&cfg) == "watching" {
            if let Some((title, process)) = read_foreground() {
                // Skip empty/blank process (would render as **** in markdown).
                let proc_lower = process.to_lowercase();
                if proc_lower.is_empty() {
                    last_signature = None;
                    stable_ticks = 0;
                    thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
                    continue;
                }
                // Skip configured processes (zoom, whatsapp, slack, ...).
                if skip_processes.iter().any(|p| p == &proc_lower) {
                    last_signature = Some(format!("__skip__{process}"));
                    stable_ticks = 0;
                    thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
                    continue;
                }
                let signature = format!("{process}|{title}");
                if last_signature.as_deref() == Some(&signature) {
                    stable_ticks = stable_ticks.saturating_add(1);
                } else {
                    stable_ticks = 0;
                    last_signature = Some(signature);
                }
                if stable_ticks == debounce_ticks {
                    // Smart dedup: if the last recorded entry is the same
                    // (process, normalized_title), don't re-record — just
                    // keep the same line. The on-disk log still ends up
                    // with one entry per task, not one per alt-tab.
                    let normalized = normalize_title(&title, &process);
                    if cfg.muse.smart_dedup {
                        if let Some((ref lp, ref lt)) = last_recorded {
                            if lp == &process && lt == &normalized {
                                last_recorded = Some((process.clone(), normalized));
                                stable_ticks = 0;
                                thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
                                continue;
                            }
                        }
                    }
                    let screenshot = if cfg.muse.screenshots {
                        capture_screenshot(&screenshots_dir, &title, &process)
                            .map(|name| screenshots_dir.join(name))
                    } else {
                        None
                    };
                    let summary = summarize_with_image(&title, &process, screenshot.as_deref());
                    if !summary.is_empty() && !summary.eq_ignore_ascii_case("skip") {
                        append_entry(&activity_dir, &title, &process, &summary, screenshot.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()));
                        last_recorded = Some((process.clone(), normalized));
                    }
                }
            } else {
                last_signature = None;
                stable_ticks = 0;
            }
        } else {
            last_signature = None;
            stable_ticks = 0;
        }
    }
}

/// Strip protocol noise (the leading "GitHub - " / "- YouTube" / "— Brave" etc.)
/// so we treat the same task as one entry even when the title's exact wording
/// shifts (e.g. "User/repo: PR review #1" vs "User/repo: PR review #2").
fn normalize_title(title: &str, _process: &str) -> String {
    let mut s = title.to_string();
    // Drop common trailing browser suffixes.
    for suffix in [" - Brave", " - Chrome", " - Edge", " - Firefox", " - YouTube"] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped.to_string();
        }
    }
    s.trim().to_string()
}

fn read_state(cfg: &Config) -> String {
    let path = muse_state_path(cfg);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("state").and_then(|s| s.as_str().map(|x| x.to_string())))
        .unwrap_or_else(|| "off".to_string())
}

fn muse_state_path(cfg: &Config) -> PathBuf {
    cfg.memory_dir()
        .parent()
        .map(|p| p.join("muse_state.json"))
        .unwrap_or_else(|| PathBuf::from("muse_state.json"))
}

fn read_foreground() -> Option<(String, String)> {
    // PowerShell snippet — uses Win32 P/Invoke to get the foreground window title
    // and its owning process name. Outputs two lines: title then process.
    // `GetModuleBaseName` returns empty for some windows (e.g. when OpenProcess
    // fails on a protected PID); fall back to `QueryFullProcessImageName` with
    // PROCESS_QUERY_LIMITED_INFORMATION (0x1000) before giving up.
    let script = r#"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class FG {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
  [DllImport("kernel32.dll")] public static extern IntPtr OpenProcess(uint dwDesiredAccess, bool bInheritHandle, uint dwProcessId);
  [DllImport("psapi.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern uint GetModuleBaseName(IntPtr hProcess, IntPtr hModule, StringBuilder lpBaseName, int nSize);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern uint QueryFullProcessImageName(IntPtr hProcess, uint dwFlags, StringBuilder lpExeName, ref uint lpdwSize);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr hObject);
}
"@
$h = [FG]::GetForegroundWindow()
if ($h -eq [IntPtr]::Zero) { exit 0 }
$sb = New-Object System.Text.StringBuilder 512
[void][FG]::GetWindowText($h, $sb, $sb.Capacity)
$title = $sb.ToString()
$pid = 0
[void][FG]::GetWindowThreadProcessId($h, [ref]$pid)
$proc = ''
$ph = [FG]::OpenProcess(0x1000, $false, $pid)
if ($ph -ne [IntPtr]::Zero) {
  $sb2 = New-Object System.Text.StringBuilder 256
  [void][FG]::GetModuleBaseName($ph, [IntPtr]::Zero, $sb2, $sb2.Capacity)
  $proc = $sb2.ToString().Trim()
  if ($proc.Length -eq 0) {
    $sz = 1024
    $sb3 = New-Object System.Text.StringBuilder $sz
    [void][FG]::QueryFullProcessImageName($ph, 0, $sb3, [ref]$sz)
    $path = $sb3.ToString()
    if ($path.Length -gt 0) {
      $proc = [System.IO.Path]::GetFileNameWithoutExtension($path)
    }
  }
  [void][FG]::CloseHandle($ph)
}
Write-Output $title
Write-Output $proc
"#;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut lines = text.lines();
    let title = lines.next()?.trim().to_string();
    let process = lines.next().unwrap_or("").trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some((title, process))
    }
}

fn summarize(title: &str, process: &str) -> String {
    summarize_with_image(title, process, None)
}

/// Ask Claude to summarize a foreground change. If a screenshot path is given
/// and `ANTHROPIC_API_KEY` is set, we hit the Anthropic Messages API directly
/// with a multimodal content block (image + text). Otherwise we fall back to
/// the `claude -p` CLI with no image.
fn summarize_with_image(title: &str, process: &str, screenshot: Option<&Path>) -> String {
    let prompt = format!(
        "You are a silent note-taker. The user switched to a window titled \"{title}\" (process: {process_name}). Reply with ONE concise line (max 120 chars) describing what they are likely working on. If uninformative (screensaver, lockscreen, blank), reply SKIP.",
        title = title,
        process_name = process,
    );
    if let (Some(path), Some(key)) = (screenshot, std::env::var("ANTHROPIC_API_KEY").ok()) {
        if let Ok(text) = call_anthropic_with_image(&key, &prompt, path) {
            return text;
        }
        // fall through to CLI on API error
    }
    let mut child = {
        let mut c = Command::new("claude");
        c.arg("-p")
            .arg(&prompt)
            .arg("--output-format")
            .arg("text")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        #[cfg(windows)]
        c.creation_flags(CREATE_NO_WINDOW);
        match c.spawn() {
            Ok(c) => c,
            Err(_) => return String::new(),
        }
    };
    let mut out = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = std::io::Read::read_to_string(&mut s, &mut out);
    }
    let _ = child.wait();
    out.lines().next().unwrap_or("").trim().to_string()
}

/// POST a multimodal message to the Anthropic Messages API. Uses
/// `claude-sonnet-4-5` by default; override with `MUSE_VISION_MODEL` env.
fn call_anthropic_with_image(api_key: &str, prompt: &str, png_path: &Path) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = std::fs::read(png_path).map_err(|e| e.to_string())?;
    let b64 = STANDARD.encode(&bytes);
    let model = std::env::var("MUSE_VISION_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5".to_string());
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 200,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": b64 } },
                { "type": "text", "text": prompt }
            ]
        }]
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let s = resp.status();
        let txt = resp.text().unwrap_or_default();
        return Err(format!("anthropic http {s}: {txt}"));
    }
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    // Walk the content[] array for the first text block.
    if let Some(blocks) = v.get("content").and_then(|c| c.as_array()) {
        for b in blocks {
            if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(text) = b.get("text").and_then(|t| t.as_str()) {
                    return Ok(text.lines().next().unwrap_or("").trim().to_string());
                }
            }
        }
    }
    Err("anthropic: no text block in response".to_string())
}

fn append_entry(
    activity_dir: &Path,
    title: &str,
    process: &str,
    summary: &str,
    screenshot: Option<&str>,
) {
    let date = current_date();
    let path = activity_dir.join(format!("{date}.md"));
    let ts = current_time_hhmm();
    let line = match screenshot {
        Some(s) => format!("- `{ts}` **{process}** — {title} — {summary} 📷 {s}\n"),
        None => format!("- `{ts}` **{process}** — {title} — {summary}\n"),
    };
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
    println!("Muse: {}", line.trim());
}

/// Capture the foreground window's bitmap into `<dir>/<HHMMSS>_<process>.png` via
/// `PrintWindow` (the GDI path — works even on DirectX/fullscreen apps that
/// `BitBlt` cannot reach). Returns the relative path on success so the markdown
/// line can link to it.
fn capture_screenshot(dir: &Path, _title: &str, process: &str) -> Option<String> {
    let safe_proc: String = process
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let stamp = {
        let now = time::OffsetDateTime::now_local()
            .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        format!("{:02}{:02}{:02}", now.hour(), now.minute(), now.second())
    };
    let filename = format!("{stamp}_{safe_proc}.png");
    let out_path = dir.join(&filename);
    let out_str = out_path.to_string_lossy().to_string();
    // PowerShell snippet: capture the foreground HWND with PrintWindow and save
    // a PNG. Uses System.Drawing for the bitmap; that needs STA which is the
    // default for powershell.exe -NoProfile.
    let script = format!(
        r#"
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
public class SW {{
  [DllImport(""user32.dll"")] public static extern IntPtr GetForegroundWindow();
  [DllImport(""user32.dll"")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport(""user32.dll"")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport(""user32.dll"", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport(""user32.dll"")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport(""kernel32.dll"")] public static extern IntPtr OpenProcess(uint a, bool i, uint p);
  [DllImport(""psapi.dll"", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern uint GetModuleBaseName(IntPtr h, IntPtr m, StringBuilder s, int n);
  [DllImport(""kernel32.dll"")] public static extern bool CloseHandle(IntPtr h);
  [StructLayout(LayoutKind.Sequential)] public struct RECT {{ public int L, T, R, B; }}
}}
"@
$h = [SW]::GetForegroundWindow()
if ($h -eq [IntPtr]::Zero) {{ exit 0 }}
$rect = New-Object SW+RECT
[void][SW]::GetWindowRect($h, [ref]$rect)
$w = $rect.R - $rect.L
$ht = $rect.B - $rect.T
if ($w -le 0 -or $ht -le 0) {{ exit 0 }}
$bmp = New-Object System.Drawing.Bitmap $w, $ht
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
$ok = [SW]::PrintWindow($h, $hdc, 2)  # PW_RENDERFULLCONTENT
$g.ReleaseHdc($hdc)
$g.Dispose()
if (-not $ok) {{ $bmp.Dispose(); exit 0 }}
$bmp.Save('{out_str}', [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
Write-Output 'ok'
"#
    );
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    if out.status.success() {
        Some(filename)
    } else {
        None
    }
}

fn current_date() -> String {
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}

fn current_time_hhmm() -> String {
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    format!("{:02}:{:02}", now.hour(), now.minute())
}

