// Clipper — Tauri 2 backend.
//
// The desktop app is **headless**: no webview, no window. It is the capture and
// ingest daemon — tray icon, global hotkeys, queue, drainer, and the axum server
// that hosts the entire UI. The tray's "Show dashboard" opens that UI in the
// user's browser.
//
// Boot order: resolve config + vault, bind the API port, build the tray, register
// the global hotkeys, then start the drainer, indexer and self-improvement tasks.

use std::sync::{Arc, Mutex};

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::WebviewWindowBuilder,
    AppHandle, Manager, WebviewUrl,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub mod capture;
pub mod clock;
pub mod config;
pub mod db;
pub mod drainer;
pub mod events;
pub mod frontmatter;
pub mod memory;
pub mod pairing;
pub mod prompts;
pub mod queue;
pub mod routes_chat;
pub mod routes_memory;
pub mod routes_pairing;
pub mod routes_providers;
pub mod routes_self;
pub mod routes_skills;
pub mod routes_soul;
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
/// The port the API actually bound. Tray handlers and the single-instance
/// callback need it to build the dashboard URL; it is set once during setup.
static LIVE_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(7137);

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

/// Where the app serves its own dashboard.
///
/// The desktop app is headless: this URL *is* the product's UI, and it opens in
/// the user's browser rather than in a Tauri window.
fn dashboard_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}/")
}

/// Open a URL in the user's default browser.
///
/// The webview is gone, so this replaces every "show a window" path: the opener
/// plugin hands the URL to the operating system.
fn open_in_browser(url: &str) {
    match tauri_plugin_opener::open_url(url, None::<&str>) {
        Ok(()) => println!("Clipper: opened {url}"),
        Err(e) => eprintln!("Clipper: could not open {url}: {e}"),
    }
}

fn toggle_muse_window(app: &AppHandle, want_visible: bool) {
    if let Some(win) = app.get_webview_window("muse") {
        if want_visible {
            let _ = win.show();
            let _ = win.set_focus();
        } else {
            let _ = win.hide();
        }
        return;
    }
    // Create the floating widget for the first time.
    if !want_visible {
        return;
    }
    // Bottom-right anchor on the primary monitor.
    let (anchor_x, anchor_y) = primary_monitor_work_area(app)
        .map(|r| (r.0 + r.2 - 252.0, r.1 + r.3 - 84.0))
        .unwrap_or((100.0, 100.0));
    let url = WebviewUrl::App("muse.html".into());
    let res = WebviewWindowBuilder::new(app, "muse", url)
        .title("Clipper Muse")
        .inner_size(240.0, 72.0)
        .position(anchor_x, anchor_y)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .transparent(true)
        .build();
    match res {
        Ok(_w) => println!("Clipper: muse window created"),
        Err(e) => eprintln!("Clipper: could not create muse window: {e}"),
    }
}

fn primary_monitor_work_area(app: &AppHandle) -> Option<(f64, f64, f64, f64)> {
    let mon = app.primary_monitor().ok().flatten()?;
    let pos = mon.position();
    let size = mon.size();
    let scale = mon.scale_factor();
    Some((pos.x as f64 / scale, pos.y as f64 / scale, size.width as f64 / scale, size.height as f64 / scale))
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
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            // Headless: a second launch just brings the dashboard up in the browser.
            open_in_browser(&dashboard_url(
                LIVE_PORT.load(std::sync::atomic::Ordering::SeqCst),
            ));
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // ---- config + vault, before the tray exists ----
            let (cfg, detected) = boot()?;
            println!("Clipper: vault {}", cfg.vault_path().display());
            let lock = Arc::new(Mutex::new(()));

            // ---- bind the API port first, so the real port is known before the
            //      tray and the dashboard window are built ----
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
            // `[server] port = 0` means "pick a free port", so ask the socket.
            let port = listener
                .local_addr()
                .map(|addr| addr.port())
                .unwrap_or(cfg.server.port);

            // ---- phone pairing: LAN address + a rotating session token ----
            let token = Arc::new(std::sync::RwLock::new(pairing::new_token()));
            let lan_ip = pairing::lan_ip();
            println!("Clipper: phone pairing on http://{lan_ip}:{port}/pair (token is in the QR)");

            // ---- system tray: Show dashboard / Show QR / Show muse / Hide muse / Quit ----
            let show_i = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
            let qr_i = MenuItem::with_id(app, "qr", "Show QR", true, None::<&str>)?;
            let muse_show_i = MenuItem::with_id(app, "muse_show", "Show muse", true, None::<&str>)?;
            let muse_hide_i = MenuItem::with_id(app, "muse_hide", "Hide muse", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &qr_i, &muse_show_i, &muse_hide_i, &quit_i])?;

            let tray_port = port;
            LIVE_PORT.store(tray_port, std::sync::atomic::Ordering::SeqCst);
            let mut tray = TrayIconBuilder::with_id(TRAY_ID)
                .menu(&menu)
                .tooltip(format!("Clipper — {lan_ip}:{tray_port}"))
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => open_in_browser(&dashboard_url(tray_port)),
                    "qr" => open_in_browser(&format!("http://127.0.0.1:{tray_port}/qr")),
                    "muse_show" => toggle_muse_window(app, true),
                    "muse_hide" => toggle_muse_window(app, false),
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

            // ---- the second brain: soul / user / memory ----
            let memory = Arc::new(memory::load_now(&cfg.memory_dir())?);
            println!(
                "Clipper: memory loaded from {}",
                cfg.memory_dir().display()
            );

            // ---- background drainer ----
            let drainer = Arc::new(Drainer::new(
                cfg.clone(),
                Arc::clone(&lock),
                Arc::clone(&memory),
            )?);
            tauri::async_runtime::spawn(async move {
                drainer.run_forever().await;
            });

            // ---- SQLite index, kept in step with the vault ----
            let index_db: db::Db = Arc::new(Mutex::new(db::open(
                &config::state_dir().join("index.db"),
            )?));

            // ---- skills are files; the table is a cache of them ----
            match routes_skills::sync(&cfg, &index_db) {
                Ok(count) => println!("Clipper: {count} skill(s) indexed"),
                Err(e) => eprintln!("Clipper: skills scan failed: {e}"),
            }

            let indexer = Arc::new(vaultwatch::Indexer::new(
                cfg.clone(),
                Arc::clone(&index_db),
            ));
            tauri::async_runtime::spawn(async move {
                indexer.watch().await;
            });

            // ---- self-improvement: recompute at startup, then daily ----
            tauri::async_runtime::spawn(routes_self::nightly(Arc::clone(&index_db)));

            // ---- REST API on the port bound above ----
            let api_state = server::AppState {
                cfg: cfg.clone(),
                db: Arc::clone(&index_db),
                queue_lock: Arc::clone(&lock),
                token: Arc::clone(&token),
                lan_ip: lan_ip.clone(),
            };
            tauri::async_runtime::spawn(async move {
                server::serve(listener, api_state).await;
            });

            // ---- keep the pairing token fresh ----
            let rotation_token = Arc::clone(&token);
            tauri::async_runtime::spawn(async move {
                pairing::rotate_forever(rotation_token).await;
            });

            println!(
                "Clipper: dashboard at http://{}:{port}",
                cfg.server.bind
            );

            println!(
                "Clipper: tray ready, hotkeys {CAPTURE_HOTKEY} (url) and {SCREENSHOT_HOTKEY} (image) registered"
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
