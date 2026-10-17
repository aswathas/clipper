// WhatsApp capture — when enabled, the watcher monitors the Windows clipboard
// for text changes. If the foreground window belongs to `whatsapp.exe` at the
// moment the user copies, we save the new text plus the contact name (the
// window title) to `<vault>/WhatsApp/<YYYY-MM-DD>.md`.
//
// The clipboard listener uses the Win32 `AddClipboardFormatListener` API
// (Windows 8+). A hidden message-only window receives `WM_CLIPBOARDUPDATE`
// whenever the clipboard changes. The thread runs an std message loop and
// reads `CF_UNICODETEXT` whenever the foreground is WhatsApp.

#![cfg(windows)]

use std::{
    fs::{self, OpenOptions},
    io::Write,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        OnceLock,
    },
    thread,
    time::Duration,
};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::DataExchange::{AddClipboardFormatListener, RemoveClipboardFormatListener},
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
        TranslateMessage, CS_HREDRAW, CS_VREDRAW, MSG, WNDCLASSEXW,
    },
};

const WM_CLIPBOARDUPDATE: u32 = 0x031D;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CLASS_NAME: &str = "ClipperWhatsAppListener";

static ENABLED: AtomicBool = AtomicBool::new(false);
static TX: OnceLock<Sender<()>> = OnceLock::new();

/// Spawn the clipboard monitor thread. The watcher is told to stop via the
/// returned `Receiver<()>`; sending on it just unblocks the message loop
/// without affecting the OS-level listener (which lives until process exit).
pub fn spawn(enabled: bool, vault: PathBuf) {
    ENABLED.store(enabled, Ordering::SeqCst);
    if !enabled {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    TX.set(tx).ok();
    thread::Builder::new()
        .name("whatsapp-clipboard".into())
        .spawn(move || run(rx, vault))
        .expect("failed to spawn whatsapp listener");
}

pub fn stop() {
    if let Some(tx) = TX.get() {
        let _ = tx.send(());
    }
}

fn run(rx: Receiver<()>, vault: PathBuf) {
    unsafe {
        let class_name: Vec<u16> = CLASS_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: std::ptr::null_mut(),
            lpszClassName: class_name.as_ptr(),
            ..std::mem::zeroed()
        };
        RegisterClassExW(&wc);
        let hwnd: HWND = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if hwnd.is_null() {
            eprintln!("whatsapp: CreateWindowExW failed");
            return;
        }
        if AddClipboardFormatListener(hwnd) == 0 {
            eprintln!("whatsapp: AddClipboardFormatListener failed");
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        loop {
            // Pump messages. Exit when the watcher drops the sender.
            if rx.recv_timeout(Duration::from_millis(50)).is_ok() {
                break;
            }
            while PeekMessageW_alt(&mut msg, hwnd, 0, 0, 1) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        RemoveClipboardFormatListener(hwnd);
    }
    let _ = vault; // currently unused; capture is routed to the same Activity dir.
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        handle_clipboard_update();
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn handle_clipboard_update() {
    // Read CF_UNICODETEXT (or fall back to CF_TEXT). If the foreground is
    // WhatsApp, append a one-line entry to the day's WhatsApp log.
    let text = read_clipboard_text();
    let Some(text) = text else { return };
    if text.trim().is_empty() {
        return;
    }
    if !is_whatsapp_foreground() {
        return;
    }
    let Some(contact) = foreground_window_title() else { return };
    append_whatsapp_entry(&contact, &text);
}

fn is_whatsapp_foreground() -> bool {
    let script = r#"
$h = (Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class FG { [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow(); }
'@ -PassThru)::GetForegroundWindow()
$pid = 0
$tid = (Get-Process | Where-Object { $_.MainWindowHandle -eq $h }).Id
if (-not $tid) { exit 0 }
$proc = (Get-Process -Id $tid).ProcessName
if ($proc -like '*hatsapp*') { 'yes' } else { 'no' }
"#;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let Ok(out) = cmd.output() else { return false };
    String::from_utf8_lossy(&out.stdout).contains("yes")
}

fn foreground_window_title() -> Option<String> {
    let script = r#"
$h = (Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class FG { [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n); }
'@ -PassThru)::GetWindowText((Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public class GF { [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow(); }
'@ -PassThru)::GetForegroundWindow(), (New-Object System.Text.StringBuilder 512), 512)
"#;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let Ok(out) = cmd.output() else { return None };
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn read_clipboard_text() -> Option<String> {
    let script = r#"Add-Type -AssemblyName PresentationCore; [Windows.Clipboard]::GetText()"#;
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let out = cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() || s == "Cannot open clipboard" { None } else { Some(s) }
}

fn append_whatsapp_entry(contact: &str, text: &str) {
    let vault = match std::env::var("CLIPPER_VAULT") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };
    let dir = vault.join("WhatsApp");
    let _ = fs::create_dir_all(&dir);
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let date = format!("{:04}-{:02}-{:02}", now.year(), now.month() as u8, now.day());
    let ts = format!("{:02}:{:02}", now.hour(), now.minute());
    let path = dir.join(format!("{date}.md"));
    let safe_contact: String = contact
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let body = format!("## {ts} · {safe_contact}\n\n> {text}\n\n---\n\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(body.as_bytes());
    }
}

// Wrapper so the call site doesn't have to import PeekMessageW from windows-sys.
#[cfg(windows)]
unsafe fn PeekMessageW_alt(msg: *mut MSG, hwnd: HWND, min: u32, max: u32, remove: u32) -> i32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::PeekMessageW;
    PeekMessageW(msg, hwnd, min, max, remove)
}
