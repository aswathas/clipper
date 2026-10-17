// Audio capture for Muse — local Whisper transcription of call audio.
//
// When `cfg.muse.record_calls` is true and the foreground process is in
// `cfg.muse.calls_processes`, the watcher asks `start_capture()` to spin up
// a worker that records 30-second windows, transcribes them with the local
// `whisper-cli` from ggml-base.en, and appends a one-line summary to
// `<vault>/Calls/<YYYY-MM-DD>.md`.
//
// All audio stays on the machine. The PowerShell recorder shells out to
// `ffmpeg` for WASAPI loopback capture; if ffmpeg is missing, we log a
// one-time error to the Calls log and stop recording for this session.

#![cfg(windows)]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use time::OffsetDateTime;

use crate::config::Config;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const WINDOW_SECS: u64 = 30;

/// Handle to the audio capture worker. Drop to stop.
pub struct CaptureHandle {
    stop: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl CaptureHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Start the audio capture worker. The worker loops every 30 s: if the
/// foreground process is in `calls_processes`, it records one window,
/// transcribes, and appends to the Calls log; otherwise it sleeps.
pub fn start_capture(cfg: Config) -> CaptureHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_inner = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("audiocapture".into())
        .spawn(move || run(cfg, stop_inner))
        .expect("failed to spawn audiocapture");
    CaptureHandle {
        stop,
        join: Some(join),
    }
}

fn run(cfg: Config, stop: Arc<AtomicBool>) {
    let cache = cfg.vault_path().join(".cache").join("calls");
    let _ = fs::create_dir_all(&cache);
    let calls_dir = cfg.vault_path().join("Calls");
    let _ = fs::create_dir_all(&calls_dir);
    let calls_procs: Vec<String> = cfg
        .muse
        .calls_processes
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let whisper_cli = cfg.muse.transcription_cli.clone();
    let whisper_model = PathBuf::from(&cfg.muse.transcription_model);
    let mut recording = false;
    let mut recorded_note: bool = false;
    while !stop.load(Ordering::SeqCst) {
        let fg = read_foreground_process();
        let on_call = fg
            .as_ref()
            .map(|p| calls_procs.iter().any(|cp| p.eq_ignore_ascii_case(cp)))
            .unwrap_or(false);
        if !cfg.muse.record_calls || !on_call {
            if recording {
                println!("audiocapture: call ended, idle");
                recording = false;
            }
            thread::sleep(Duration::from_secs(2));
            continue;
        }
        // First tick after a call begins: log a session header.
        if !recording {
            println!(
                "audiocapture: call foreground={}; recording 30s windows",
                fg.as_deref().unwrap_or("?")
            );
            recording = true;
            recorded_note = append_session_header(&calls_dir, fg.as_deref().unwrap_or("?"));
        }
        let stamp = {
            let n = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            format!(
                "{:04}{:02}{:02}_{:02}{:02}{:02}",
                n.year(),
                n.month() as u8,
                n.day(),
                n.hour(),
                n.minute(),
                n.second()
            )
        };
        let wav = cache.join(format!("{stamp}.wav"));
        if record_window(&wav, WINDOW_SECS) {
            if let Some(txt) = transcribe(&whisper_cli, &whisper_model, &wav) {
                if !txt.trim().is_empty() {
                    append_transcript(&calls_dir, &txt);
                }
            }
            let _ = fs::remove_file(&wav);
        } else {
            // Recorder failed (likely no ffmpeg). Mark unavailable so we
            // don't spam the log every 30 s.
            if !recorded_note {
                append_capture_unavailable(&calls_dir);
                recorded_note = true;
            }
            thread::sleep(Duration::from_secs(WINDOW_SECS));
            continue;
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn read_foreground_process() -> Option<String> {
    let script = r#"
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class FG2 {
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  public static extern uint QueryFullProcessImageName(IntPtr h, uint f, StringBuilder s, ref uint n);
  [DllImport("kernel32.dll")] public static extern IntPtr OpenProcess(uint a, bool i, uint p);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr h);
}
"@
$h = [FG2]::GetForegroundWindow()
if ($h -eq [IntPtr]::Zero) { exit 0 }
$pid = 0
[void][FG2]::GetWindowThreadProcessId($h, [ref]$pid)
$ph = [FG2]::OpenProcess(0x1000, $false, $pid)
if ($ph -eq [IntPtr]::Zero) { exit 0 }
$sz = 1024
$sb = New-Object System.Text.StringBuilder $sz
[void][FG2]::QueryFullProcessImageName($ph, 0, $sb, [ref]$sz)
$path = $sb.ToString()
[void][FG2]::CloseHandle($ph)
if ($path.Length -eq 0) { exit 0 }
[System.IO.Path]::GetFileNameWithoutExtension($path)
"#;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Record `secs` of WASAPI loopback audio to `out_wav` using ffmpeg.
/// Returns false on failure (ffmpeg missing, mic permission denied, etc.).
fn record_window(out_wav: &Path, secs: u64) -> bool {
    let out_str = out_wav.to_string_lossy().replace('\'', "''");
    let script = format!(
        r#"$f = Get-Command ffmpeg -ErrorAction SilentlyContinue
if (-not $f) {{ exit 2 }}
& ffmpeg -hide_banner -loglevel error -f wasapi -i ''@Communications'' -t {secs} -ac 1 -ar 16000 -y ''{out_str}'' 2>$null
exit $LASTEXITCODE"#,
        secs = secs,
        out_str = out_str,
    );
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    match cmd.output() {
        Ok(o) => o.status.success() && out_wav.exists(),
        Err(_) => false,
    }
}

/// Transcribe a WAV file via the local `whisper-cli`. Returns the transcript
/// (joined across segments) or None on any failure.
fn transcribe(cli: &str, model: &Path, wav: &Path) -> Option<String> {
    if !Path::new(cli).exists() || !model.exists() || !wav.exists() {
        return None;
    }
    let out_dir = wav.parent()?;
    let base = wav.file_stem()?.to_string_lossy().to_string();
    let mut cmd = Command::new(cli);
    cmd.arg("-m")
        .arg(model)
        .arg("-f")
        .arg(wav)
        .arg("-l")
        .arg("en")
        .arg("-otxt")
        .arg("-of")
        .arg(out_dir.join(&base))
        .arg("--no-timestamps")
        .arg("--threads")
        .arg("4");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    if let Ok(status) = cmd.status() {
        if !status.success() {
            return None;
        }
    } else {
        return None;
    }
    let txt = out_dir.join(format!("{base}.txt"));
    let body = fs::read_to_string(&txt).ok()?;
    let _ = fs::remove_file(&txt);
    Some(body)
}

fn calls_path(dir: &Path) -> PathBuf {
    let n = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    dir.join(format!(
        "{:04}-{:02}-{:02}.md",
        n.year(),
        n.month() as u8,
        n.day()
    ))
}

fn append_session_header(dir: &Path, process: &str) -> bool {
    let path = calls_path(dir);
    let n = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let ts = format!("{:02}:{:02}", n.hour(), n.minute());
    let line = format!("\n## {ts} · {process}\n\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        return f.write_all(line.as_bytes()).is_ok();
    }
    false
}

fn append_transcript(dir: &Path, txt: &str) {
    let path = calls_path(dir);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(txt.as_bytes());
        let _ = f.write_all(b"\n\n");
    }
}

fn append_capture_unavailable(dir: &Path) {
    let path = calls_path(dir);
    let line = "_audio capture unavailable: install ffmpeg and enable `transcription_cli` in `[muse]`_\n\n";
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}
