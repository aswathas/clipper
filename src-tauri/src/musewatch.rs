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

const POLL_INTERVAL_SECS: u64 = 5;
const DEBOUNCE_TICKS: u32 = 2; // 10s of stable title before summarize

static STOP_FLAG: AtomicBool = AtomicBool::new(false);

pub fn spawn(cfg: Config) {
    thread::Builder::new()
        .name("muse-watcher".into())
        .spawn(move || run_loop(cfg))
        .expect("failed to spawn muse watcher thread");
}

pub fn stop() {
    STOP_FLAG.store(true, Ordering::SeqCst);
}

fn run_loop(cfg: Config) {
    let activity_dir = cfg.vault_path().join("Activity");
    let _ = fs::create_dir_all(&activity_dir);
    let mut last_signature: Option<String> = None;
    let mut stable_ticks: u32 = 0;
    while !STOP_FLAG.load(Ordering::SeqCst) {
        if read_state(&cfg) == "watching" {
            if let Some((title, process)) = read_foreground() {
                let signature = format!("{process}|{title}");
                if last_signature.as_deref() == Some(&signature) {
                    stable_ticks = stable_ticks.saturating_add(1);
                } else {
                    stable_ticks = 0;
                    last_signature = Some(signature);
                }
                if stable_ticks == DEBOUNCE_TICKS {
                    let summary = summarize(&title, &process);
                    if !summary.is_empty() && !summary.eq_ignore_ascii_case("skip") {
                        append_entry(&activity_dir, &title, &process, &summary);
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
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
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
    let prompt = format!(
        "You are a silent note-taker. The user switched to a window titled \"{title}\" (process: {process_name}). Reply with ONE concise line (max 120 chars) describing what they are likely working on. If uninformative (screensaver, lockscreen, blank), reply SKIP.",
        title = title,
        process_name = process,
    );
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

fn append_entry(activity_dir: &Path, title: &str, process: &str, summary: &str) {
    let date = current_date();
    let path = activity_dir.join(format!("{date}.md"));
    let ts = current_time_hhmm();
    let line = format!("- `{ts}` **{process}** — {title} — {summary}\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
    println!("Muse: {}", line.trim());
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

