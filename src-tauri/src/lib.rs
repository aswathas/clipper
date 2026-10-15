// Clipper — Tauri 2 backend (HAG-2 scaffold).
//
// Scope for this slice: system tray + global hotkey + a clipboard read that
// prints URL-shaped text to the console. The JSONL queue, vault writer and
// Claude dispatch arrive in HAG-3.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Global capture hotkey.
const CAPTURE_HOTKEY: &str = "Ctrl+Shift+C";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// True when the clipboard text looks like a web URL.
fn looks_like_url(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("http://") || t.starts_with("https://")
}

/// Read the clipboard and print it when it looks like a URL.
/// HAG-3 replaces this stub with a queue append.
fn capture_clipboard() {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.get_text() {
            Ok(text) => {
                if looks_like_url(&text) {
                    println!("clip captured: {}", text.trim());
                }
            }
            Err(e) => eprintln!("clipboard: read failed: {e}"),
        },
        Err(e) => eprintln!("clipboard: unavailable: {e}"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Single instance must be registered before any other plugin.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // ---- system tray: Show dashboard / Settings / Quit ----
            let show_i = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => println!("tray: Show dashboard (placeholder)"),
                    "settings" => println!("tray: Settings (placeholder)"),
                    "quit" => {
                        println!("tray: Quit");
                        app.exit(0);
                    }
                    other => println!("tray: unhandled menu id: {other}"),
                });

            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            tray.build(app)?;

            // ---- global hotkey: Ctrl+Shift+C ----
            app.global_shortcut()
                .on_shortcut(CAPTURE_HOTKEY, |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        capture_clipboard();
                    }
                })?;

            println!("Clipper: tray ready, global hotkey {CAPTURE_HOTKEY} registered");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
