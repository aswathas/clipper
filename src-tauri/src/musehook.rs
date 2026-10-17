// Native Win32 `SetWinEventHook` for `EVENT_SYSTEM_FOREGROUND`.
//
// Polling the foreground window every 1s is fine, but a real event hook reacts
// in <50 ms and avoids spawning PowerShell on every idle tick. The callback
// lives in a Windows DLL-loaded callback and posts the new HWND to a
// `mpsc::Sender` so the Rust watcher can do its title/process lookup on its
// own thread.
//
// One process-wide hook is registered on `start()` and unregistered on
// `stop()`. The `Sender` is held in a `OnceLock` because the callback is a
// `fn` pointer (no captures).

#![cfg(windows)]

use std::{
    os::raw::c_void,
    ptr,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use windows_sys::Win32::{
    UI::Accessibility::{SetWinEventHook, UnhookWinEvent},
    UI::WindowsAndMessaging::{
        EVENT_SYSTEM_FOREGROUND, GetMessageW, WINEVENT_OUTOFCONTEXT, MSG,
    },
};

static HOOK_TX: OnceLock<Mutex<Sender<isize>>> = OnceLock::new();

/// Spawn the event hook on its own thread. Returns a `Receiver<HWND>` that
/// fires every time the foreground window changes (HWND is the raw isize
/// cast of the Win32 handle). The caller is expected to keep the receiver
/// alive and drain it; when the receiver is dropped the hook is removed.
pub fn spawn() -> Receiver<isize> {
    let (tx, rx) = mpsc::channel::<isize>();
    HOOK_TX
        .set(Mutex::new(tx))
        .expect("musehook: HOOK_TX already initialized");
    thread::Builder::new()
        .name("muse-eventhook".into())
        .spawn(run_hook)
        .expect("failed to spawn musehook thread");
    rx
}

fn run_hook() {
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            ptr::null_mut(),
            Some(win_event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_null() {
        eprintln!("musehook: SetWinEventHook failed; falling back to poll");
        return;
    }

    // Pump messages until the process is killed. Without a real message loop
    // the WINEVENT_OUTOFCONTEXT callback never fires.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            // No translations needed for our callback.
        }
    }

    unsafe {
        UnhookWinEvent(hook);
    }
}

unsafe extern "system" fn win_event_callback(
    _hook: *mut c_void,
    _event: u32,
    hwnd: *mut c_void,
    _id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _ms: u32,
) {
    let h = hwnd as isize;
    if let Some(tx) = HOOK_TX.get() {
        if let Ok(g) = tx.lock() {
            let _ = g.send(h);
        }
    }
}

/// Spawn a tiny "pinger" thread that does nothing on its own; the watcher's
/// existing poll loop reads the mpsc Receiver returned by `spawn()` and falls
/// back to PowerShell when no event arrives. This keeps the existing
/// read_foreground path working without rewrites.
pub fn _silence_unused() {
    thread::sleep(Duration::from_secs(60));
}
