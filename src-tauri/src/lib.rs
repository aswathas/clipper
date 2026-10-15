// Clipper — Tauri 2 backend.
//
// Boot order: resolve config + vault, build the tray, register the global
// hotkey, then start the queue drainer in the background.

use std::sync::{Arc, Mutex};

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub mod capture;
pub mod clock;
pub mod config;
pub mod db;
pub mod drainer;
pub mod frontmatter;
pub mod prompts;
pub mod queue;
pub mod screenshot;
pub mod selfcheck;
pub mod server;
pub mod vault;
pub mod vaultwatch;

use capture::CaptureOutcome;
use config::Config;
use drainer::Drainer;
use vault::Detected;

/// Global capture hotkey for URLs.
const CAPTURE_HOTKEY: &str = "Ctrl+Shift+C";
/// Global capture hotkey for clipboard images.
const SCREENSHOT_HOTKEY: &str = "Ctrl+Shift+X";
/// Tray icon id, used to look the tray up for notifications.
const TRAY_ID: &str = "clipper";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Resolve the config, scanning for a vault only on first run.
fn boot() -> Result<(Config, Detected), Box<dyn std::error::Error>> {
    let path = config::config_path();

    if path.exists() {
        let cfg = Config::load(&path)?;
        cfg.ensure_dirs()?;
        println!("Clipper: config {}", path.display());
        // Re-use the persisted vault; do not re-scan.
        let detected = Detected {
            path: cfg.vault_path(),
            candidates: Vec::new(),
            created: false,
        };
        Ok((cfg, detected))
    } else {
        let detected = vault::detect();
        let cfg = Config::for_vault(&detected.path);
        cfg.ensure_dirs()?;
        cfg.save(&path)?;
        println!("Clipper: wrote config {}", path.display());
        Ok((cfg, detected))
    }
}

/// Surface a message in the console and on the tray tooltip.
fn notify_user(app: &AppHandle, message: &str) {
    println!("{message}");
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(message));
    }
}

fn read_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut clipboard| clipboard.get_text().ok())
}

/// The hotkey handler: clipboard URL -> deduped queue append.
fn on_capture(app: &AppHandle, cfg: &Config, lock: &Mutex<()>) {
    let Some(text) = read_clipboard() else {
        return;
    };

    match capture::capture(cfg, lock, &text) {
        Ok(CaptureOutcome::Queued { id, .. }) => {
            println!("clip captured: {} (queued {})", text.trim(), id);
        }
        Ok(CaptureOutcome::AlreadySaved { host, .. }) => {
            notify_user(app, &format!("Already saved: {host}"));
        }
        Ok(CaptureOutcome::NotAUrl) => {}
        Err(e) => eprintln!("capture failed: {e}"),
    }
}

/// The screenshot hotkey handler: clipboard image -> staging PNG -> queue.
fn on_capture_image(app: &AppHandle, cfg: &Config, lock: &Mutex<()>) {
    let Some((width, height, bytes)) = screenshot::clipboard_image() else {
        notify_user(app, "Clipboard has no image");
        return;
    };

    let _guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match screenshot::save_and_queue(cfg, width, height, bytes, "screenshot") {
        Ok(Some(id)) => println!("screenshot captured: {width}x{height} (queued {id})"),
        Ok(None) => notify_user(app, "Clipboard image was not usable"),
        Err(e) => eprintln!("screenshot failed: {e}"),
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
            // ---- config + vault, before the tray exists ----
            let (cfg, detected) = boot()?;
            println!("Clipper: vault {}", cfg.vault_path().display());
            let lock = Arc::new(Mutex::new(()));

            // ---- system tray: Show dashboard / Settings / Quit ----
            let show_i = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

            let mut tray = TrayIconBuilder::with_id(TRAY_ID)
                .menu(&menu)
                .tooltip("Clipper")
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

            if detected.created {
                notify_user(
                    app.handle(),
                    &format!("Clipper: created vault at {}", detected.path.display()),
                );
            }

            // ---- global hotkeys: Ctrl+Shift+C (URL), Ctrl+Shift+X (image) ----
            let hotkey_cfg = cfg.clone();
            let hotkey_lock = Arc::clone(&lock);
            app.global_shortcut()
                .on_shortcut(CAPTURE_HOTKEY, move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        on_capture(app, &hotkey_cfg, &hotkey_lock);
                    }
                })?;

            let image_cfg = cfg.clone();
            let image_lock = Arc::clone(&lock);
            app.global_shortcut()
                .on_shortcut(SCREENSHOT_HOTKEY, move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        on_capture_image(app, &image_cfg, &image_lock);
                    }
                })?;

            // ---- background drainer ----
            let drainer = Arc::new(Drainer::new(cfg.clone(), Arc::clone(&lock))?);
            tauri::async_runtime::spawn(async move {
                drainer.run_forever().await;
            });

            // ---- SQLite index, kept in step with the vault ----
            let index_db: db::Db = Arc::new(Mutex::new(db::open(
                &config::state_dir().join("index.db"),
            )?));
            let indexer = Arc::new(vaultwatch::Indexer::new(
                cfg.clone(),
                Arc::clone(&index_db),
            ));
            tauri::async_runtime::spawn(async move {
                indexer.watch().await;
            });

            // ---- REST API on [server] bind/port ----
            // Bind synchronously so a taken port fails loudly here rather than
            // silently moving to another one.
            let listener = match server::bind(&cfg) {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!(
                        "Clipper: cannot bind {}:{} — {e}. Another program (or another Clipper) may be using that port.",
                        cfg.server.bind, cfg.server.port
                    );
                    return Err(Box::new(e));
                }
            };
            let api_state = server::AppState {
                cfg: cfg.clone(),
                db: Arc::clone(&index_db),
                queue_lock: Arc::clone(&lock),
            };
            tauri::async_runtime::spawn(async move {
                server::serve(listener, api_state).await;
            });

            println!(
                "Clipper: dashboard at http://{}:{}",
                cfg.server.bind, cfg.server.port
            );

            println!(
                "Clipper: tray ready, hotkeys {CAPTURE_HOTKEY} (url) and {SCREENSHOT_HOTKEY} (image) registered"
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
