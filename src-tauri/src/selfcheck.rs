//! HAG-3 self-check, runnable from the shipped binary:
//!
//! ```text
//! clipper.exe --hag3-smoke
//! ```
//!
//! Exits 0 when every check passes, 1 otherwise. The provider is faked with
//! `cmd /c ...`, so it needs no network and never runs `claude`.
//!
//! Why a flag on the real binary instead of a `tests/` target: a Rust test
//! harness binary linking the Tauri stack cannot be loaded on this
//! windows-gnu toolchain (`STATUS_ENTRYPOINT_NOT_FOUND`), while `clipper.exe`
//! itself loads and runs. This exercises the same code on a binary that works.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde_json::{json, Value};

use crate::{
    capture::{self, CaptureOutcome},
    config::Config,
    drainer::Drainer,
    memory::Memory,
    queue,
};

/// A throwaway [`Memory`] for checks that are not about the persona files.
fn scratch_memory() -> Arc<Memory> {
    Arc::new(Memory {
        soul: "# Soul\n(scratch)\n".to_string(),
        user: "# User\n(scratch)\n".to_string(),
        memory: "# Memory\n(scratch)\n".to_string(),
    })
}

/// An isolated `~/.clipper` + vault.
struct Fixture {
    cfg: Config,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("clipper-hag3-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let mut cfg = Config::default();
        let s = |p: PathBuf| p.to_string_lossy().into_owned();
        cfg.vault.path = s(root.join("Vault"));
        cfg.vault.inbox_subdir = "Inbox".into();
        cfg.queue.dir = s(root.join("queue"));
        cfg.queue.staging_dir = s(root.join("staging"));
        cfg.prompts.extract = s(root.join("prompts").join("extract.md"));
        cfg.queue.batch_size = 5;
        cfg.queue.timeout_s = 30;
        cfg.ensure_dirs().expect("fixture dirs");

        Self { cfg }
    }

    /// A provider that prints the contents of `list` (one path per line).
    fn printing_provider(&self, list: &Path) -> Config {
        let mut cfg = self.cfg.clone();
        cfg.provider.claude.command = "cmd".into();
        cfg.provider.claude.args = vec![
            "/c".into(),
            "type".into(),
            list.to_string_lossy().into_owned(),
        ];
        cfg
    }
}

fn sample_note(index: usize) -> String {
    format!(
        "---\n\
         type: article\n\
         source: example.com\n\
         url: https://example.com/post-{index}\n\
         saved_at: 2026-01-0{index}T00:00:00Z\n\
         title: Post {index}\n\
         summary: Summary {index}.\n\
         tags: [test]\n\
         ---\n\n\
         # Post {index}\n\n\
         Summary {index}.\n"
    )
}

/// Pull `title` and `url` out of a note's YAML frontmatter.
fn frontmatter_title_url(text: &str) -> Option<(String, String)> {
    let rest = text.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];

    let mut title = None;
    let mut url = None;
    for line in block.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("title:") {
            title = Some(value.trim().to_string());
        }
        if let Some(value) = line.strip_prefix("url:") {
            url = Some(value.trim().to_string());
        }
    }
    Some((title?, url?))
}

/// 1. Three queued URLs -> three notes with valid frontmatter in the vault.
fn three_urls_become_three_notes() -> Result<(), String> {
    let fixture = Fixture::new("happy");
    let lock = Arc::new(Mutex::new(()));

    for index in 1..=3 {
        let outcome =
            capture::capture(&fixture.cfg, &lock, &format!("https://example.com/post-{index}"))
                .map_err(|e| format!("capture: {e}"))?;
        if !matches!(outcome, CaptureOutcome::Queued { .. }) {
            return Err(format!("expected Queued, got {outcome:?}"));
        }
    }
    let queued = queue::read_lines(&fixture.cfg.inbox_jsonl())
        .map_err(|e| e.to_string())?
        .len();
    if queued != 3 {
        return Err(format!("expected 3 queued lines, got {queued}"));
    }

    // Stand in for the provider: write the notes it would have produced, then
    // have a real process print their paths.
    let notes: Vec<PathBuf> = (1..=3)
        .map(|index| fixture.cfg.staging_dir().join(format!("note-{index}.md")))
        .collect();
    for (offset, path) in notes.iter().enumerate() {
        fs::write(path, sample_note(offset + 1)).map_err(|e| e.to_string())?;
    }
    let list = fixture.cfg.staging_dir().join("list.txt");
    let listing = notes
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\r\n");
    fs::write(&list, listing).map_err(|e| e.to_string())?;

    let cfg = fixture.printing_provider(&list);
    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock), scratch_memory())
        .map_err(|e| e.to_string())?;
    let outcome = drainer.run_once().map_err(|e| e.to_string())?;

    if outcome.picked != 3 {
        return Err(format!("picked {}, expected 3", outcome.picked));
    }
    if let Some(reason) = &outcome.reason {
        return Err(format!("batch deadlettered: {reason}"));
    }
    if outcome.written.len() != 3 {
        return Err(format!("wrote {}, expected 3", outcome.written.len()));
    }

    let mut filed: Vec<PathBuf> = fs::read_dir(cfg.vault_inbox())
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|entry| entry.path())
        .collect();
    filed.sort();
    if filed.len() != 3 {
        return Err(format!("{} notes in vault, expected 3", filed.len()));
    }
    for path in &filed {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (title, url) = frontmatter_title_url(&text)
            .ok_or_else(|| format!("no title/url frontmatter in {}", path.display()))?;
        if title.is_empty() || !url.starts_with("https://") {
            return Err(format!("weak frontmatter in {}: {title} / {url}", path.display()));
        }
    }

    let left = queue::read_lines(&cfg.inbox_jsonl()).map_err(|e| e.to_string())?.len();
    let archived = queue::read_lines(&cfg.archive_jsonl()).map_err(|e| e.to_string())?.len();
    if left != 0 {
        return Err(format!("{left} lines left in inbox.jsonl"));
    }
    if archived != 3 {
        return Err(format!("{archived} archived, expected 3"));
    }
    if cfg.processing_jsonl().exists() {
        return Err("inbox.processing was not cleaned up".into());
    }
    Ok(())
}

/// 2. A URL whose hash is already archived is skipped.
fn already_archived_url_is_skipped() -> Result<(), String> {
    let fixture = Fixture::new("dedupe");
    let lock = Arc::new(Mutex::new(()));

    let url = "https://example.com/already-here";
    let hash = queue::url_hash(url);
    queue::append_json_line(
        &fixture.cfg.archive_jsonl(),
        &serde_json::json!({ "id": "pre-existing", "hash": hash }),
    )
    .map_err(|e| e.to_string())?;

    match capture::capture(&fixture.cfg, &lock, url).map_err(|e| e.to_string())? {
        CaptureOutcome::AlreadySaved { host, .. } => {
            if host != "example.com" {
                return Err(format!("host was {host}"));
            }
        }
        other => return Err(format!("expected AlreadySaved, got {other:?}")),
    }

    let queued = queue::read_lines(&fixture.cfg.inbox_jsonl())
        .map_err(|e| e.to_string())?
        .len();
    if queued != 0 {
        return Err(format!("duplicate reached the queue ({queued} lines)"));
    }

    // A different URL still gets through.
    let fresh =
        capture::capture(&fixture.cfg, &lock, "https://example.com/brand-new").map_err(|e| e.to_string())?;
    if !matches!(fresh, CaptureOutcome::Queued { .. }) {
        return Err(format!("fresh URL was not queued: {fresh:?}"));
    }
    Ok(())
}

/// 2b. Non-URL clipboard text is ignored.
fn non_url_is_ignored() -> Result<(), String> {
    let fixture = Fixture::new("noturl");
    let lock = Arc::new(Mutex::new(()));

    let outcome =
        capture::capture(&fixture.cfg, &lock, "just some words").map_err(|e| e.to_string())?;
    if outcome != CaptureOutcome::NotAUrl {
        return Err(format!("expected NotAUrl, got {outcome:?}"));
    }
    let queued = queue::read_lines(&fixture.cfg.inbox_jsonl())
        .map_err(|e| e.to_string())?
        .len();
    if queued != 0 {
        return Err(format!("non-URL reached the queue ({queued} lines)"));
    }
    Ok(())
}

/// 3. Garbage provider stdout deadletters the batch.
fn garbage_stdout_deadletters() -> Result<(), String> {
    let fixture = Fixture::new("badout");
    let lock = Arc::new(Mutex::new(()));

    capture::capture(&fixture.cfg, &lock, "https://example.com/dead").map_err(|e| e.to_string())?;

    let mut cfg = fixture.cfg.clone();
    cfg.provider.claude.command = "cmd".into();
    cfg.provider.claude.args = vec!["/c".into(), "echo".into(), "not-a-real-path".into()];

    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock), scratch_memory())
        .map_err(|e| e.to_string())?;
    let outcome = drainer.run_once().map_err(|e| e.to_string())?;

    if outcome.reason.as_deref() != Some("bad-stdout") {
        return Err(format!("reason was {:?}", outcome.reason));
    }
    if !outcome.written.is_empty() {
        return Err("notes were filed despite bad stdout".into());
    }

    let dead = queue::read_lines(&cfg.deadletter_jsonl()).map_err(|e| e.to_string())?;
    if dead.len() != 1 {
        return Err(format!("{} deadletter lines, expected 1", dead.len()));
    }
    let parsed: serde_json::Value =
        serde_json::from_str(&dead[0]).map_err(|e| e.to_string())?;
    if parsed["reason"] != "bad-stdout" {
        return Err(format!("deadletter reason was {}", parsed["reason"]));
    }
    let filed = fs::read_dir(cfg.vault_inbox()).map_err(|e| e.to_string())?.count();
    if filed != 0 {
        return Err(format!("{filed} notes filed despite deadletter"));
    }
    Ok(())
}

/// Run every HAG-3 check; returns a process exit code.
pub fn run_hag3() -> i32 {
    let checks: [(&str, fn() -> Result<(), String>); 4] = [
        (
            "three_urls_become_three_notes_with_frontmatter",
            three_urls_become_three_notes,
        ),
        ("already_archived_url_is_skipped", already_archived_url_is_skipped),
        ("non_url_is_ignored", non_url_is_ignored),
        ("garbage_stdout_deadletters_the_batch", garbage_stdout_deadletters),
    ];

    println!("hag3 self-check");
    let mut failed = 0;
    for (name, check) in checks {
        match check() {
            Ok(()) => println!("  PASS {name}"),
            Err(message) => {
                failed += 1;
                println!("  FAIL {name}: {message}");
            }
        }
    }

    if failed == 0 {
        println!("hag3 self-check: OK ({} checks)", checks.len());
        0
    } else {
        println!("hag3 self-check: {failed} FAILED");
        1
    }
}

// ========================================================== HAG-4 self-check

/// HAG-4: SQLite index, FTS5 search, and the REST API.
pub fn run_hag4() -> i32 {
    println!("hag4 self-check");
    let mut failed = 0;
    match hag4_rest_api() {
        Ok(()) => println!("  PASS hag4_rest_api"),
        Err(message) => {
            failed += 1;
            println!("  FAIL hag4_rest_api: {message}");
        }
    }

    if failed == 0 {
        println!("hag4 self-check: OK (1 check)");
        0
    } else {
        println!("hag4 self-check: {failed} FAILED");
        1
    }
}

/// Index three notes by *watching* the vault, serve the API on a random port,
/// and exercise every route over real HTTP.
fn hag4_rest_api() -> Result<(), String> {
    use crate::{db, server, vaultwatch};

    let root = std::env::temp_dir().join(format!("clipper-hag4-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let mut cfg = Config::default();
    let temp = |p: PathBuf| p.to_string_lossy().into_owned();
    cfg.vault.path = temp(root.join("Vault"));
    // Every path must be inside the temp root, or the check writes into the
    // user's real ~/.clipper.
    cfg.queue.dir = temp(root.join("queue"));
    cfg.queue.staging_dir = temp(root.join("staging"));
    cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
    cfg.server.bind = "127.0.0.1".into();
    cfg.server.port = 0;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;

    // Three notes. Only the first mentions "rust", so q=rust must return one.
    let notes: [(&str, &str, &str, &str); 3] = [
        ("aaa11111", "Rust ownership explained", "https://rust.example/1", "rust"),
        ("bbb22222", "Go routines", "https://go.example/2", "go"),
        ("ccc33333", "Writing notes", "https://notes.example/3", "notes"),
    ];

    let index_db: db::Db = Arc::new(Mutex::new(
        db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
    ));
    let indexer = Arc::new(vaultwatch::Indexer::new(cfg.clone(), Arc::clone(&index_db)));

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let state = server::AppState {
        cfg: cfg.clone(),
        db: Arc::clone(&index_db),
        queue_lock: Arc::new(Mutex::new(())),
        token: Arc::new(std::sync::RwLock::new("hag4-unused".to_string())),
        lan_ip: "127.0.0.1".into(),
    };

    let ready = indexer.ready_flag();

    // Server + watcher on their own runtime so this thread can block on HTTP.
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(async move {
            tokio::spawn(indexer.watch());
            server::serve(listener, state).await;
        });
    });

    wait_for_server(port)?;

    // Wait for the startup scan to finish *before* writing, otherwise the scan
    // would do the indexing and this would not prove the file-watch path.
    let mut watching = false;
    for _ in 0..100 {
        if ready.load(std::sync::atomic::Ordering::SeqCst) {
            watching = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if !watching {
        return Err("the vault watcher never became ready".into());
    }

    // The vault is empty at this point, so anything indexed from here on proves
    // the live file-watch path.
    for (id, title, url, tag) in notes {
        let body = format!(
            "---\ntype: article\nsource: example.com\ntitle: {title}\nurl: {url}\n\
             saved_at: 2026-01-01T12:00:00Z\nsummary: A summary.\ntags: [{tag}]\n---\n\n# {title}\n"
        );
        fs::write(
            cfg.vault_inbox().join(format!("2026-01-01_1200_clipboard_{id}.md")),
            body,
        )
        .map_err(|e| e.to_string())?;
    }

    // Give the watcher its 1s to ingest (it debounces 500ms).
    std::thread::sleep(Duration::from_millis(1500));

    // ---- GET /api/items ----
    let (status, body) = http(port, "GET", "/api/items", None)?;
    if status != 200 {
        return Err(format!("GET /api/items -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| format!("{e}: {body}"))?;
    let items = payload["items"].as_array().cloned().unwrap_or_default();
    if items.len() != 3 {
        return Err(format!("expected 3 items, got {} ({body})", items.len()));
    }
    if payload["total"].as_i64() != Some(3) {
        return Err(format!("total = {}", payload["total"]));
    }
    for field in ["id", "url", "saved_at", "status", "tags"] {
        if items[0].get(field).is_none() {
            return Err(format!("item missing `{field}`: {}", items[0]));
        }
    }
    if items[0]["status"] != "inbox" {
        return Err(format!("freshly indexed status = {}", items[0]["status"]));
    }
    if items[0]["tags"][0] != "rust" && items[0]["tags"][0] != "go" && items[0]["tags"][0] != "notes" {
        return Err(format!("tags did not round-trip: {}", items[0]["tags"]));
    }

    // ---- GET /api/items/:id ----
    let (status, body) = http(port, "GET", "/api/items/aaa11111", None)?;
    if status != 200 {
        return Err(format!("GET /api/items/aaa11111 -> {status} {body}"));
    }
    let one: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if one["title"] != "Rust ownership explained" {
        return Err(format!("title = {}", one["title"]));
    }
    let (status, _) = http(port, "GET", "/api/items/nope", None)?;
    if status != 404 {
        return Err(format!("unknown id -> {status}, expected 404"));
    }

    // ---- GET /api/items?q=rust ----
    let (status, body) = http(port, "GET", "/api/items?q=rust", None)?;
    if status != 200 {
        return Err(format!("GET /api/items?q=rust -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let hits = payload["items"].as_array().map(Vec::len).unwrap_or(0);
    if hits != 1 {
        return Err(format!("q=rust returned {hits}, expected 1 ({body})"));
    }
    if payload["items"][0]["id"] != "aaa11111" {
        return Err(format!("q=rust matched {}", payload["items"][0]["id"]));
    }

    // ---- GET /api/today ----
    let (status, body) = http(port, "GET", "/api/today", None)?;
    if status != 200 {
        return Err(format!("GET /api/today -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let today = payload["items"].as_array().map(Vec::len).unwrap_or(0);
    if today > 3 {
        return Err(format!("today returned {today}, expected <= 3"));
    }
    if today != 3 {
        return Err(format!("today returned {today}, expected 3 inbox items ({body})"));
    }

    // ---- POST status, valid and invalid ----
    let (status, body) = http(
        port,
        "POST",
        "/api/items/aaa11111/status",
        Some(r#"{"status":"done"}"#),
    )?;
    if status != 200 {
        return Err(format!("POST status -> {status} {body}"));
    }
    let (status, _) = http(
        port,
        "POST",
        "/api/items/aaa11111/status",
        Some(r#"{"status":"bogus"}"#),
    )?;
    if status != 400 {
        return Err(format!("invalid status -> {status}, expected 400"));
    }
    let (_, body) = http(port, "GET", "/api/items?status=done", None)?;
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if payload["total"].as_i64() != Some(1) {
        return Err(format!("status filter total = {}", payload["total"]));
    }

    // ---- POST tags ----
    let (status, body) = http(
        port,
        "POST",
        "/api/items/bbb22222/tags",
        Some(r#"{"tags":["alpha","beta"]}"#),
    )?;
    if status != 200 {
        return Err(format!("POST tags -> {status} {body}"));
    }
    let (_, body) = http(port, "GET", "/api/items/bbb22222", None)?;
    let item: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if item["tags"].as_array().map(Vec::len) != Some(2) {
        return Err(format!("tags = {}", item["tags"]));
    }
    let (status, body) = http(port, "GET", "/api/items?tag=beta", None)?;
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if payload["total"].as_i64() != Some(1) {
        return Err(format!("tag filter total = {} ({status})", payload["total"]));
    }

    // ---- POST /api/clips ----
    let (status, body) = http(
        port,
        "POST",
        "/api/clips",
        Some(r#"{"url":"https://manual.example/9"}"#),
    )?;
    if status != 200 {
        return Err(format!("POST /api/clips -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if payload["status"] != "queued" {
        return Err(format!("clip status = {}", payload["status"]));
    }
    if payload["id"].as_str().map(str::is_empty).unwrap_or(true) {
        return Err("clip returned no id".into());
    }
    let queued = queue::read_lines(&cfg.inbox_jsonl()).map_err(|e| e.to_string())?;
    if queued.len() != 1 {
        return Err(format!("inbox.jsonl has {} lines, expected 1", queued.len()));
    }
    let (status, _) = http(port, "POST", "/api/clips", Some(r#"{"url":"not a url"}"#))?;
    if status != 400 {
        return Err(format!("bad url -> {status}, expected 400"));
    }

    // ---- GET /api/stats ----
    let (status, body) = http(port, "GET", "/api/stats", None)?;
    if status != 200 {
        return Err(format!("GET /api/stats -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if payload["total"].as_i64() != Some(3) {
        return Err(format!("stats.total = {}", payload["total"]));
    }
    if payload["by_type"]["article"].as_i64() != Some(3) {
        return Err(format!("stats.by_type = {}", payload["by_type"]));
    }
    if payload.get("stale_30d").is_none() {
        return Err("stats is missing stale_30d".into());
    }

    // ---- healthz + the dashboard the API serves ----
    let (status, body) = http(port, "GET", "/healthz", None)?;
    if status != 200 || body.trim() != "ok" {
        return Err(format!("/healthz -> {status} {body:?}"));
    }
    let (status, body) = http(port, "GET", "/", None)?;
    if status != 200 {
        return Err(format!("GET / -> {status}"));
    }
    if !body.to_ascii_lowercase().contains("<html") {
        return Err("dashboard response did not look like HTML".into());
    }

    Ok(())
}

/// One blocking HTTP request over a real socket.
fn http(port: u16, method: &str, path: &str, body: Option<&str>) -> Result<(u16, String), String> {
    http_with_headers(port, method, path, body, &[])
}

fn http_with_headers(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra: &[(&str, &str)],
) -> Result<(u16, String), String> {
    let (status, head, bytes) = http_full(port, method, path, extra, body)?;
    let body = String::from_utf8_lossy(&bytes).into_owned();
    let body = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        dechunk(&body)
    } else {
        body
    };
    Ok((status, body))
}

/// Like [`http`], but keeps the body as bytes (for the PNG) and returns the
/// raw header block so headers and content types can be checked.
fn http_bytes(port: u16, path: &str) -> Result<(u16, String, Vec<u8>), String> {
    http_full(port, "GET", path, &[], None)
}

/// The one implementation: status, raw header block, raw body bytes.
fn http_full(
    port: u16,
    method: &str,
    path: &str,
    extra: &[(&str, &str)],
    body: Option<&str>,
) -> Result<(u16, String, Vec<u8>), String> {
    use std::io::{Read, Write};

    let mut stream =
        std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok();

    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    for (name, value) in extra {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    if let Some(body) = body {
        request.push_str("Content-Type: application/json\r\n");
        request.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    request.push_str("\r\n");
    if let Some(body) = body {
        request.push_str(body);
    }

    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;

    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .unwrap_or(0);
    let (head, body) = raw.split_at(split);
    let head = String::from_utf8_lossy(head).into_owned();
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);

    Ok((status, head, body.to_vec()))
}

/// Minimal HTTP/1.1 chunked decoding, in case a body arrives without Content-Length.
fn dechunk(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    loop {
        let Some(position) = rest.find("\r\n") else { break };
        let Ok(size) = usize::from_str_radix(rest[..position].trim(), 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        let start = position + 2;
        if rest.len() < start + size {
            break;
        }
        out.push_str(&rest[start..start + size]);
        rest = &rest[start + size..];
        if let Some(stripped) = rest.strip_prefix("\r\n") {
            rest = stripped;
        }
    }
    out
}

fn wait_for_server(port: u16) -> Result<(), String> {
    for _ in 0..50 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!("server on 127.0.0.1:{port} never came up"))
}

// ========================================================== HAG-6 self-check

/// HAG-6: clipboard image -> staging PNG -> queue -> drainer -> vault.
pub fn run_hag6() -> i32 {
    println!("hag6 self-check");
    let mut failed = 0;
    match hag6_image_capture() {
        Ok(()) => println!("  PASS hag6_image_capture"),
        Err(message) => {
            failed += 1;
            println!("  FAIL hag6_image_capture: {message}");
        }
    }

    if failed == 0 {
        println!("hag6 self-check: OK (1 check)");
        0
    } else {
        println!("hag6 self-check: {failed} FAILED");
        1
    }
}

fn hag6_image_capture() -> Result<(), String> {
    use crate::{frontmatter, screenshot};

    let fixture = Fixture::new("hag6");
    let lock = Arc::new(Mutex::new(()));

    // A 1x1 red RGBA image, through the real save-and-queue path the hotkey uses.
    let id = screenshot::save_and_queue(&fixture.cfg, 1, 1, vec![255, 0, 0, 255], "screenshot")
        .map_err(|e| e.to_string())?
        .ok_or("save_and_queue refused a valid 1x1 image")?;

    let queued = queue::read_lines(&fixture.cfg.inbox_jsonl()).map_err(|e| e.to_string())?;
    if queued.len() != 1 {
        return Err(format!("{} queue lines, expected 1", queued.len()));
    }
    let entry: Value = serde_json::from_str(&queued[0]).map_err(|e| e.to_string())?;
    if entry["type"] != "image" {
        return Err(format!("queue type = {}", entry["type"]));
    }
    if entry["source"] != "screenshot" {
        return Err(format!("queue source = {}", entry["source"]));
    }
    if entry["id"] != id.as_str() {
        return Err("queued id does not match the returned id".into());
    }

    let image_path = entry["path"].as_str().ok_or("queue entry carries no path")?;
    let raw = fs::read(image_path).map_err(|e| format!("{image_path}: {e}"))?;
    if raw.len() < 8 || &raw[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("staged file is not a PNG".into());
    }

    // Stand in for the vision provider: the caption it would have written.
    let caption = fixture.cfg.staging_dir().join("caption.md");
    fs::write(
        &caption,
        "---\ntype: image\nsource: screenshot\nurl: \n\
         saved_at: 2026-01-01T00:00:00Z\ntitle: A red square\n\
         summary: One red pixel.\ntags: [image]\n---\n\n# A red square\n\nOne red pixel.\n",
    )
    .map_err(|e| e.to_string())?;
    let list = fixture.cfg.staging_dir().join("img-list.txt");
    fs::write(&list, caption.display().to_string()).map_err(|e| e.to_string())?;

    let mut cfg = fixture.cfg.clone();
    cfg.provider.image.command = Some("cmd".into());
    cfg.provider.image.args = Some(vec![
        "/c".into(),
        "type".into(),
        list.to_string_lossy().into_owned(),
    ]);
    cfg.provider.image.prompt_hint =
        "Use mcp__MiniMax__understand_image to caption the image at <path>.".into();

    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock), scratch_memory())
        .map_err(|e| e.to_string())?;
    let outcome = drainer.run_once().map_err(|e| e.to_string())?;

    if let Some(reason) = &outcome.reason {
        return Err(format!("batch deadlettered: {reason}"));
    }
    if outcome.written.len() != 1 {
        return Err(format!("wrote {}, expected 1", outcome.written.len()));
    }

    let filed = &outcome.written[0];
    let text = fs::read_to_string(filed).map_err(|e| e.to_string())?;
    let meta = frontmatter::parse(&text).ok_or("filed note has no frontmatter")?;
    if meta.get("type") != Some("image") {
        return Err(format!("filed type = {:?}", meta.get("type")));
    }
    if meta.get("source") != Some("screenshot") {
        return Err(format!("filed source = {:?}", meta.get("source")));
    }
    if fs::read_dir(cfg.vault_inbox()).map_err(|e| e.to_string())?.count() != 1 {
        return Err("vault Inbox does not hold exactly one note".into());
    }

    // The id must round-trip through the filename, or the HAG-4 indexer would
    // treat a re-index as a new item.
    let name = filed
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("filed note has no filename")?;
    let (parsed_id, parsed_source) =
        frontmatter::parse_note_filename(name).ok_or("filename did not parse")?;
    if parsed_id != id {
        return Err(format!("filename id {parsed_id} != queue id {id}"));
    }
    if parsed_source != "screenshot" {
        return Err(format!("filename source = {parsed_source}"));
    }

    Ok(())
}

// ========================================================== HAG-7 self-check

const HAG7_TOKEN: &str = "test-token-0123456789abcdefghijkl";

/// HAG-7: phone pairing — the PWA, the QR, and the token-gated `/save`.
pub fn run_hag7() -> i32 {
    println!("hag7 self-check");
    let mut failed = 0;
    match hag7_pair_and_save() {
        Ok(()) => println!("  PASS hag7_pair_and_save"),
        Err(message) => {
            failed += 1;
            println!("  FAIL hag7_pair_and_save: {message}");
        }
    }

    if failed == 0 {
        println!("hag7 self-check: OK (1 check)");
        0
    } else {
        println!("hag7 self-check: {failed} FAILED");
        1
    }
}

fn hag7_pair_and_save() -> Result<(), String> {
    use crate::{db, server};
    use std::sync::RwLock;

    let root = std::env::temp_dir().join(format!("clipper-hag7-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let mut cfg = Config::default();
    let temp = |p: PathBuf| p.to_string_lossy().into_owned();
    cfg.vault.path = temp(root.join("Vault"));
    cfg.queue.dir = temp(root.join("queue"));
    cfg.queue.staging_dir = temp(root.join("staging"));
    cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
    cfg.server.bind = "127.0.0.1".into();
    cfg.server.port = 0;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;

    let index_db: db::Db = Arc::new(Mutex::new(
        db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
    ));
    let state = server::AppState {
        cfg: cfg.clone(),
        db: index_db,
        queue_lock: Arc::new(Mutex::new(())),
        token: Arc::new(RwLock::new(HAG7_TOKEN.to_string())),
        lan_ip: "127.0.0.1".into(),
    };

    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    std::thread::spawn(move || {
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        runtime.block_on(server::serve(listener, state));
    });
    wait_for_server(port)?;

    // ---- GET /pair: the phone's input page ----
    let (status, body) = http(port, "GET", "/pair", None)?;
    if status != 200 {
        return Err(format!("GET /pair -> {status}"));
    }
    if !body.contains("<input") {
        return Err("the pairing page has no input element".into());
    }
    if !body.contains("/pwa/app.js") {
        return Err("the pairing page does not load app.js".into());
    }

    // ---- the PWA assets ----
    let (status, body) = http(port, "GET", "/pwa/app.js", None)?;
    if status != 200 || !body.contains("/save") {
        return Err(format!("GET /pwa/app.js -> {status} ({} bytes)", body.len()));
    }
    let (status, body) = http(port, "GET", "/pwa/manifest.json", None)?;
    if status != 200 || !body.contains("\"name\": \"Clipper\"") {
        return Err(format!("GET /pwa/manifest.json -> {status}"));
    }

    // ---- GET /pair.png: a real PNG ----
    let (status, head, bytes) = http_bytes(port, "/pair.png")?;
    if status != 200 {
        return Err(format!("GET /pair.png -> {status}"));
    }
    if !head.to_ascii_lowercase().contains("image/png") {
        return Err("pair.png did not come back as image/png".into());
    }
    if bytes.len() < 100 {
        return Err(format!("pair.png was only {} bytes", bytes.len()));
    }
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("pair.png is not a PNG".into());
    }

    // ---- POST /save without / with a wrong token ----
    let clip = r#"{"url":"https://phone.example/1"}"#;
    let (status, _) = http(port, "POST", "/save", Some(clip))?;
    if status != 401 {
        return Err(format!("/save with no token -> {status}, expected 401"));
    }
    let (status, _) = http(port, "POST", "/save?token=nope", Some(clip))?;
    if status != 401 {
        return Err(format!("/save with a wrong token -> {status}, expected 401"));
    }
    if !queue::read_lines(&cfg.inbox_jsonl())
        .map_err(|e| e.to_string())?
        .is_empty()
    {
        return Err("an unauthorised /save still reached the queue".into());
    }

    // ---- POST /save with the token in the query string ----
    let (status, body) = http(port, "POST", &format!("/save?token={HAG7_TOKEN}"), Some(clip))?;
    if status != 200 {
        return Err(format!("/save?token=… -> {status} {body}"));
    }
    let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    if payload["status"] != "queued" {
        return Err(format!("status = {}", payload["status"]));
    }
    if payload["id"].as_str().map(str::is_empty).unwrap_or(true) {
        return Err("no id came back from /save".into());
    }

    // ---- POST /save with the token in the header ----
    let (status, body) = http_with_headers(
        port,
        "POST",
        "/save",
        Some(r#"{"url":"https://phone.example/2"}"#),
        &[("X-Clipper-Token", HAG7_TOKEN)],
    )?;
    if status != 200 {
        return Err(format!("/save with the header -> {status} {body}"));
    }

    let queued = queue::read_lines(&cfg.inbox_jsonl()).map_err(|e| e.to_string())?;
    if queued.len() != 2 {
        return Err(format!("{} queue lines, expected 2", queued.len()));
    }
    let entry: Value = serde_json::from_str(&queued[0]).map_err(|e| e.to_string())?;
    if entry["source"] != "phone" {
        return Err(format!("queue source = {}", entry["source"]));
    }

    Ok(())
}

// ========================================================= HAG-10 self-check

/// HAG-10 (Slice A1): the soul / user / memory files, and their injection into
/// every Claude prompt.
pub fn run_hag10() -> i32 {
    println!("hag10 self-check");
    let mut failed = 0;
    match hag10_memory_injection() {
        Ok(()) => println!("  PASS hag10_memory_injection"),
        Err(message) => {
            failed += 1;
            println!("  FAIL hag10_memory_injection: {message}");
        }
    }

    if failed == 0 {
        println!("hag10 self-check: OK (1 check)");
        0
    } else {
        println!("hag10 self-check: {failed} FAILED");
        1
    }
}

fn hag10_memory_injection() -> Result<(), String> {
    use crate::memory;

    let root = std::env::temp_dir().join(format!("clipper-hag10-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    // ---- 1-2. first run seeds the three files from the embedded defaults ----
    let dir = memory::memory_dir(&root);
    let seeded = memory::load_now(&dir).map_err(|e| e.to_string())?;
    for (name, expected) in [
        (memory::SOUL, memory::default_soul()),
        (memory::USER, memory::default_user()),
        (memory::MEMORY, memory::default_memory()),
    ] {
        let path = dir.join(format!("{name}.md"));
        if !path.is_file() {
            return Err(format!("{name}.md was not seeded at {}", path.display()));
        }
        let actual = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if actual != expected {
            return Err(format!("{name}.md does not match the embedded default"));
        }
    }
    if seeded.soul != memory::default_soul() || seeded.memory != memory::default_memory() {
        return Err("the loaded parts do not match the defaults".into());
    }

    // ---- 3. the user edits user.md ----
    let edited = memory::default_user().replace("expertise: []", "expertise: [rust]");
    if edited == memory::default_user() {
        return Err("could not build the edited user.md".into());
    }
    memory::save_part_at(&dir, memory::USER, &edited).map_err(|e| e.to_string())?;

    // ---- 4. a reload picks the edit up, and the header carries all three ----
    let reloaded = memory::load_now(&dir).map_err(|e| e.to_string())?;
    let header = memory::render_header(&reloaded);
    if !header.starts_with("# Soul") {
        return Err(format!(
            "header does not start with '# Soul', got {:?}",
            header.chars().take(40).collect::<String>()
        ));
    }
    for needle in ["# Soul", "# User", "# Memory", "rust"] {
        if !header.contains(needle) {
            return Err(format!("header is missing {needle:?}"));
        }
    }

    // ---- and the prompt the drainer really hands to the provider ----
    let mut cfg = Config::default();
    let temp = |p: PathBuf| p.to_string_lossy().into_owned();
    cfg.vault.path = temp(root.join("Vault"));
    cfg.queue.dir = temp(root.join("queue"));
    cfg.queue.staging_dir = temp(root.join("staging"));
    cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
    cfg.queue.timeout_s = 60;
    cfg.ensure_dirs().map_err(|e| e.to_string())?;

    let captured = root.join("captured-prompt.md");
    let list = cfg.staging_dir().join("hag10-list.txt");
    let note = cfg.staging_dir().join("hag10-note.md");
    fs::write(
        &note,
        "---\ntype: article\nsource: example.com\nurl: https://example.com/x\n\
         saved_at: 2026-01-01T00:00:00Z\ntitle: Memory check\nsummary: s.\ntags: [t]\n---\n\n# Memory check\n",
    )
    .map_err(|e| e.to_string())?;
    fs::write(&list, note.display().to_string()).map_err(|e| e.to_string())?;

    // A provider that records the prompt it receives on stdin, then reports the
    // note path it would have written.
    let script = format!(
        "$p = [Console]::In.ReadToEnd(); Set-Content -Path '{}' -Value $p; Get-Content '{}'",
        captured.display(),
        list.display()
    );
    cfg.provider.claude.command = "powershell".into();
    cfg.provider.claude.args = vec!["-NoProfile".into(), "-Command".into(), script];

    let lock = Arc::new(Mutex::new(()));
    capture::capture(&cfg, &lock, "https://example.com/x").map_err(|e| e.to_string())?;

    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock), Arc::new(reloaded))
        .map_err(|e| e.to_string())?;
    let outcome = drainer.run_once().map_err(|e| e.to_string())?;
    if let Some(reason) = &outcome.reason {
        return Err(format!("batch deadlettered: {reason}"));
    }
    if outcome.written.len() != 1 {
        return Err(format!("wrote {}, expected 1", outcome.written.len()));
    }

    let prompt = fs::read_to_string(&captured)
        .map_err(|e| format!("the provider recorded no prompt: {e}"))?;

    if !prompt.starts_with("# Soul") {
        return Err(format!(
            "prompt does not start with '# Soul', got {:?}",
            prompt.chars().take(60).collect::<String>()
        ));
    }

    // Order matters: persona, then user, then memory, then the task brief.
    let at = |needle: &str| prompt.find(needle).unwrap_or(usize::MAX);
    let (soul, user, mem, task) = (
        at("# Soul"),
        at("# User"),
        at("# Memory"),
        at("You are Clipper's extractor"),
    );
    if !(soul < user && user < mem && mem < task) {
        return Err(format!(
            "prompt parts are out of order: soul={soul} user={user} memory={mem} task={task}"
        ));
    }
    if !prompt.contains("expertise: [rust]") {
        return Err("the edited expertise never reached the prompt".into());
    }

    Ok(())
}

// ================================================ HAG-12 integration gates

/// Cross-cutting contracts that every later slice (S4-S8) builds on.
///
/// Also reachable from `tests/integration_api.rs` on a toolchain where
/// `cargo test` can load a Tauri-linked harness; here it runs as
/// `clipper.exe --integration-smoke`.
pub fn integration_api() -> i32 {
    use crate::{db, server, vaultwatch};

    println!("integration api gates");
    let mut failed = 0;
    let mut total = 0;

    macro_rules! gate {
        ($name:expr, $body:expr) => {{
            total += 1;
            match $body {
                Ok(()) => println!("  PASS {}", $name),
                Err(e) => {
                    failed += 1;
                    println!("  FAIL {}: {}", $name, e);
                }
            }
        }};
    }

    // ---- a temp home with three indexed notes and a live server ----
    let root = std::env::temp_dir().join(format!("clipper-hag12-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let setup = (|| -> Result<(u16, std::net::SocketAddr), String> {
        let mut cfg = Config::default();
        let temp = |p: PathBuf| p.to_string_lossy().into_owned();
        cfg.vault.path = temp(root.join("Vault"));
        cfg.queue.dir = temp(root.join("queue"));
        cfg.queue.staging_dir = temp(root.join("staging"));
        cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
        cfg.server.bind = "127.0.0.1".into();
        cfg.server.port = 0;
        cfg.ensure_dirs().map_err(|e| e.to_string())?;

        let index_db: db::Db = Arc::new(Mutex::new(
            db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
        ));
        let indexer = Arc::new(vaultwatch::Indexer::new(cfg.clone(), Arc::clone(&index_db)));
        let ready = indexer.ready_flag();

        let listener =
            std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;

        let state = server::AppState {
            cfg: cfg.clone(),
            db: Arc::clone(&index_db),
            queue_lock: Arc::new(Mutex::new(())),
            token: Arc::new(std::sync::RwLock::new("hag12".to_string())),
            lan_ip: "127.0.0.1".into(),
        };

        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                tokio::spawn(indexer.watch());
                server::serve(listener, state).await;
            });
        });
        wait_for_server(addr.port())?;

        let mut watching = false;
        for _ in 0..100 {
            if ready.load(std::sync::atomic::Ordering::SeqCst) {
                watching = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !watching {
            return Err("the vault watcher never became ready".into());
        }

        for (id, title, url, tag) in [
            ("int11111", "Rust ownership explained", "https://rust.example/1", "rust"),
            ("int22222", "Go routines", "https://go.example/2", "go"),
            ("int33333", "Notes on notes", "https://notes.example/3", "notes"),
        ] {
            let body = format!(
                "---\ntype: article\nsource: example.com\ntitle: {title}\nurl: {url}\n\
                 saved_at: 2026-01-01T12:00:00Z\nsummary: A summary.\ntags: [{tag}]\n---\n\n# {title}\n"
            );
            fs::write(
                cfg.vault_inbox()
                    .join(format!("2026-01-01_1200_clipboard_{id}.md")),
                body,
            )
            .map_err(|e| e.to_string())?;
        }
        std::thread::sleep(Duration::from_millis(1500));
        Ok((addr.port(), addr))
    })();

    let (port, addr) = match setup {
        Ok(value) => value,
        Err(e) => {
            println!("  FAIL setup: {e}");
            return 1;
        }
    };

    // 1. saved_at is Unix seconds, as a JSON number
    gate!("saved_at_is_unix_seconds", (|| -> Result<(), String> {
        let (status, body) = http(port, "GET", "/api/items", None)?;
        if status != 200 {
            return Err(format!("GET /api/items -> {status}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let value = payload["items"][0]
            .get("saved_at")
            .ok_or("the item has no saved_at")?;
        let seconds = value
            .as_i64()
            .ok_or_else(|| format!("saved_at is not an integer: {value}"))?;
        if !(1_500_000_000..4_000_000_000).contains(&seconds) {
            return Err(format!("saved_at {seconds} is not Unix seconds"));
        }
        Ok(())
    })());

    // 2. the FTS5 external-content triggers stay in sync across I/U/D
    gate!("fts5_triggers_stay_in_sync", (|| -> Result<(), String> {
        let conn = db::open_in_memory().map_err(|e| e.to_string())?;
        let mut item = db::Item {
            id: "fts1".into(),
            url: "https://x/1".into(),
            kind: Some("article".into()),
            source: Some("clipboard".into()),
            title: Some("Rust ownership".into()),
            summary: Some("borrow checker".into()),
            vault_path: None,
            saved_at: 1_700_000_000,
            status: "inbox".into(),
            tags: vec!["rust".into()],
        };
        db::upsert(&conn, &item).map_err(|e| e.to_string())?;

        let count = |needle: &str| -> Result<i64, String> {
            db::list(
                &conn,
                &db::Query {
                    limit: 10,
                    offset: 0,
                    q: Some(needle.to_string()),
                    ..Default::default()
                },
            )
            .map(|(_, total)| total)
            .map_err(|e| e.to_string())
        };

        if count("rust")? != 1 {
            return Err("an inserted row never reached the FTS index".into());
        }

        item.title = Some("Go routines".into());
        item.tags = vec!["go".into()];
        db::upsert(&conn, &item).map_err(|e| e.to_string())?;
        if count("rust")? != 0 {
            return Err("the old term still matches after an update".into());
        }
        if count("go")? != 1 {
            return Err("the new term does not match after an update".into());
        }

        conn.execute("DELETE FROM items WHERE id = 'fts1'", [])
            .map_err(|e| e.to_string())?;
        if count("go")? != 0 {
            return Err("a deleted row is still searchable".into());
        }
        Ok(())
    })());

    // 3. the drainer/indexer contract: items.type is populated from the note
    gate!("indexer_populates_items_type", (|| -> Result<(), String> {
        let (_, body) = http(port, "GET", "/api/items", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let items = payload["items"].as_array().ok_or("no items array")?;
        if items.len() != 3 {
            return Err(format!("expected 3 indexed items, got {} ({body})", items.len()));
        }
        for item in items {
            if item["type"].as_str() != Some("article") {
                return Err(format!("type is {} for {}", item["type"], item["id"]));
            }
        }
        Ok(())
    })());

    // 4. CORS: allowed origins get the header, everyone else does not
    gate!("cors_allowlist_is_locked", (|| -> Result<(), String> {
        let (_, allowed, _) =
            http_full(port, "GET", "/healthz", &[("Origin", "http://127.0.0.1:7137")], None)?;
        if !allowed.to_ascii_lowercase().contains("access-control-allow-origin") {
            return Err("the allow-listed origin got no CORS header".into());
        }
        for origin in ["http://evil.example", "http://192.168.1.3:7137"] {
            let (_, denied, _) =
                http_full(port, "GET", "/healthz", &[("Origin", origin)], None)?;
            if denied.to_ascii_lowercase().contains("access-control-allow-origin") {
                return Err(format!("{origin} was granted CORS"));
            }
        }
        Ok(())
    })());

    // 5. the API is loopback-only by default
    gate!("server_binds_loopback", (|| -> Result<(), String> {
        let default_bind = Config::default().server.bind;
        if default_bind != "127.0.0.1" {
            return Err(format!("the default bind is {default_bind}, not 127.0.0.1"));
        }
        if !addr.ip().is_loopback() {
            return Err(format!("listening on {}", addr.ip()));
        }
        Ok(())
    })());

    // 6. hostile FTS input cannot 500 the API
    gate!("fts_query_input_is_escaped", (|| -> Result<(), String> {
        for probe in ["%22unbalanced", "title%3A%28", "a%20OR%20b%20%22", "*", "%28%29", "-"] {
            let (status, body) = http(port, "GET", &format!("/api/items?q={probe}"), None)?;
            if status != 200 {
                return Err(format!("q={probe} -> {status} {body}"));
            }
        }
        let (_, body) = http(port, "GET", "/api/items?q=%22Rust%20ownership%22", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["total"].as_i64() != Some(1) {
            return Err(format!("a quoted phrase search returned {}", payload["total"]));
        }
        Ok(())
    })());

    if failed == 0 {
        println!("integration api gates: OK ({total} gates)");
        0
    } else {
        println!("integration api gates: {failed} of {total} FAILED");
        1
    }
}

// ========================================================= HAG-14 self-check

/// HAG-14 (S4): Skills CRUD — files are the truth, the table is a cache.
pub fn run_hag14() -> i32 {
    use crate::{db, routes_skills, server};

    println!("hag14 self-check");
    let mut failed = 0;
    let mut total = 0;

    macro_rules! gate {
        ($name:expr, $body:expr) => {{
            total += 1;
            match $body {
                Ok(()) => println!("  PASS {}", $name),
                Err(e) => {
                    failed += 1;
                    println!("  FAIL {}: {}", $name, e);
                }
            }
        }};
    }

    let root = std::env::temp_dir().join(format!("clipper-hag14-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let setup = (|| -> Result<(u16, Config, db::Db), String> {
        let mut cfg = Config::default();
        let temp = |p: PathBuf| p.to_string_lossy().into_owned();
        cfg.vault.path = temp(root.join("Vault"));
        cfg.queue.dir = temp(root.join("queue"));
        cfg.queue.staging_dir = temp(root.join("staging"));
        cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
        cfg.skills.dir = temp(root.join("skills"));
        cfg.server.bind = "127.0.0.1".into();
        cfg.server.port = 0;
        cfg.ensure_dirs().map_err(|e| e.to_string())?;

        // A skill the user dropped in by hand: the file store is the truth.
        fs::write(
            cfg.skills_dir().join("manual.md"),
            "---\ntitle: Manual\n---\n\n# Manual skill\n",
        )
        .map_err(|e| e.to_string())?;

        let index_db: db::Db = Arc::new(Mutex::new(
            db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
        ));
        let synced = routes_skills::sync(&cfg, &index_db).map_err(|e| e.to_string())?;
        if synced != 1 {
            return Err(format!("startup scan indexed {synced} skills, expected 1"));
        }

        let listener =
            std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let state = server::AppState {
            cfg: cfg.clone(),
            db: Arc::clone(&index_db),
            queue_lock: Arc::new(Mutex::new(())),
            token: Arc::new(std::sync::RwLock::new("hag14".to_string())),
            lan_ip: "127.0.0.1".into(),
        };
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(server::serve(listener, state));
        });
        wait_for_server(addr.port())?;
        Ok((addr.port(), cfg, index_db))
    })();

    let (port, cfg, index_db) = match setup {
        Ok(value) => value,
        Err(e) => {
            println!("  FAIL setup: {e}");
            return 1;
        }
    };
    let skills_dir = cfg.skills_dir();

    // 1. a file on disk is listed, with a token estimate
    gate!("files_on_disk_are_listed", (|| -> Result<(), String> {
        let (status, body) = http(port, "GET", "/api/skills", None)?;
        if status != 200 {
            return Err(format!("GET /api/skills -> {status} {body}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["total"].as_i64() != Some(1) {
            return Err(format!("expected 1 skill, got {} ({body})", payload["total"]));
        }
        let skill = &payload["skills"][0];
        if skill["name"] != "manual" {
            return Err(format!("name = {}", skill["name"]));
        }
        if skill["title"] != "Manual" {
            return Err(format!("title = {}", skill["title"]));
        }
        if skill["token_estimate"].as_i64().unwrap_or(0) <= 0 {
            return Err(format!("token_estimate = {}", skill["token_estimate"]));
        }
        Ok(())
    })());

    // 2. PUT writes the file and indexes it
    gate!("put_creates_a_skill", (|| -> Result<(), String> {
        let (status, body) = http(
            port,
            "PUT",
            "/api/skills/demo",
            Some(r#"{"body_md":"---\ntitle: Demo\n---\n\n# Demo\n\nSome body text.\n"}"#),
        )?;
        if status != 200 {
            return Err(format!("PUT -> {status} {body}"));
        }
        if !skills_dir.join("demo.md").is_file() {
            return Err("the skill file was not written".into());
        }
        let (_, body) = http(port, "GET", "/api/skills", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["total"].as_i64() != Some(2) {
            return Err(format!("expected 2 skills, got {}", payload["total"]));
        }
        Ok(())
    })());

    // 3. markdown renders; embedded raw HTML is dropped
    gate!("raw_html_is_stripped", (|| -> Result<(), String> {
        let body = json!({
            "body_md": "---\ntitle: Unsafe\n---\n\n# Heading\n\n<script>alert(1)</script>\n\nSafe paragraph.\n"
        })
        .to_string();
        let (status, _) = http(port, "PUT", "/api/skills/unsafe", Some(&body))?;
        if status != 200 {
            return Err(format!("PUT /api/skills/unsafe -> {status}"));
        }
        let (status, body) = http(port, "GET", "/api/skills/unsafe", None)?;
        if status != 200 {
            return Err(format!("GET -> {status}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let html = payload["html"].as_str().ok_or("no html in the payload")?;
        if !html.contains("<h1") {
            return Err(format!("markdown was not rendered: {html}"));
        }
        if html.to_ascii_lowercase().contains("<script") {
            return Err(format!("raw HTML survived rendering: {html}"));
        }
        if !payload["body_md"].as_str().unwrap_or("").contains("<script>") {
            return Err("the stored body should keep the original markdown".into());
        }
        Ok(())
    })());

    // 4. a name can never escape the skills directory
    gate!("path_traversal_is_rejected", (|| -> Result<(), String> {
        for name in ["..evil", "a..b", "bad name", "bad%2Fname"] {
            let (status, _) = http(
                port,
                "PUT",
                &format!("/api/skills/{name}"),
                Some(r#"{"body_md":"x"}"#),
            )?;
            if status != 400 && status != 404 {
                return Err(format!("PUT {name} -> {status}, expected 400/404"));
            }
        }
        // Nothing escaped.
        let escaped = root.join("evil.md");
        if escaped.exists() {
            return Err(format!("a write escaped to {}", escaped.display()));
        }
        if root.parent().map(|p| p.join("evil.md").exists()).unwrap_or(false) {
            return Err("a write escaped the temp dir".into());
        }
        Ok(())
    })());

    // 5. DELETE removes both the file and the row
    gate!("delete_removes_file_and_row", (|| -> Result<(), String> {
        let (status, body) = http(port, "DELETE", "/api/skills/demo", None)?;
        if status != 200 {
            return Err(format!("DELETE -> {status} {body}"));
        }
        if skills_dir.join("demo.md").exists() {
            return Err("the file is still on disk".into());
        }
        let (_, body) = http(port, "GET", "/api/skills", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["total"].as_i64() != Some(2) {
            return Err(format!("expected 2 skills left, got {}", payload["total"]));
        }
        let (status, _) = http(port, "DELETE", "/api/skills/demo", None)?;
        if status != 404 {
            return Err(format!("second DELETE -> {status}, expected 404"));
        }
        Ok(())
    })());

    // 6. rescan picks up a hand-written file and drops a deleted one
    gate!("rescan_tracks_the_directory", (|| -> Result<(), String> {
        fs::write(skills_dir.join("hand.md"), "# Hand written\n").map_err(|e| e.to_string())?;
        let (status, body) = http(port, "POST", "/api/skills/rescan", Some("{}"))?;
        if status != 200 {
            return Err(format!("rescan -> {status} {body}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["scanned"].as_i64() != Some(3) {
            return Err(format!("rescan scanned {} ({body})", payload["scanned"]));
        }

        fs::remove_file(skills_dir.join("manual.md")).map_err(|e| e.to_string())?;
        let (_, body) = http(port, "POST", "/api/skills/rescan", Some("{}"))?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["scanned"].as_i64() != Some(2) {
            return Err(format!("rescan scanned {} after a delete", payload["scanned"]));
        }
        let (_, body) = http(port, "GET", "/api/skills/manual", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload.get("name").is_some() {
            return Err("a deleted file is still served".into());
        }
        Ok(())
    })());

    // 7. the index really is rebuilt from disk
    gate!("index_matches_the_directory", (|| -> Result<(), String> {
        let conn = index_db.lock().unwrap_or_else(|p| p.into_inner());
        let names: Vec<String> = crate::db::list_skills(&conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|skill| skill.name)
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        if sorted != vec!["hand".to_string(), "unsafe".to_string()] {
            return Err(format!("table holds {names:?}"));
        }
        Ok(())
    })());

    if failed == 0 {
        println!("hag14 self-check: OK ({total} gates)");
        0
    } else {
        println!("hag14 self-check: {failed} of {total} FAILED");
        1
    }
}

// ========================================================= HAG-15 self-check

/// HAG-15 (S5): memory/soul over HTTP, with If-Match optimistic concurrency.
pub fn run_hag15() -> i32 {
    use crate::{db, routes_memory, server};

    println!("hag15 self-check");
    let mut failed = 0;
    let mut total = 0;

    macro_rules! gate {
        ($name:expr, $body:expr) => {{
            total += 1;
            match $body {
                Ok(()) => println!("  PASS {}", $name),
                Err(e) => {
                    failed += 1;
                    println!("  FAIL {}: {}", $name, e);
                }
            }
        }};
    }

    let root = std::env::temp_dir().join(format!("clipper-hag15-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let setup = (|| -> Result<(u16, Config, db::Db), String> {
        let mut cfg = Config::default();
        let temp = |p: PathBuf| p.to_string_lossy().into_owned();
        cfg.vault.path = temp(root.join("Vault"));
        cfg.queue.dir = temp(root.join("queue"));
        cfg.queue.staging_dir = temp(root.join("staging"));
        cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
        cfg.skills.dir = temp(root.join("skills"));
        cfg.memory.dir = temp(root.join("memory"));
        cfg.server.bind = "127.0.0.1".into();
        cfg.server.port = 0;
        cfg.ensure_dirs().map_err(|e| e.to_string())?;
        crate::memory::load_now(&cfg.memory_dir()).map_err(|e| e.to_string())?;

        let index_db: db::Db = Arc::new(Mutex::new(
            db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
        ));
        let listener =
            std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let state = server::AppState {
            cfg: cfg.clone(),
            db: Arc::clone(&index_db),
            queue_lock: Arc::new(Mutex::new(())),
            token: Arc::new(std::sync::RwLock::new("hag15".to_string())),
            lan_ip: "127.0.0.1".into(),
        };
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(server::serve(listener, state));
        });
        wait_for_server(addr.port())?;
        Ok((addr.port(), cfg, index_db))
    })();

    let (port, cfg, index_db) = match setup {
        Ok(value) => value,
        Err(e) => {
            println!("  FAIL setup: {e}");
            return 1;
        }
    };
    let user_path = cfg.memory_dir().join("user.md");

    // 1. all three parts are listed
    gate!("memory_parts_are_listed", (|| -> Result<(), String> {
        let (status, body) = http(port, "GET", "/api/memory", None)?;
        if status != 200 {
            return Err(format!("GET /api/memory -> {status} {body}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let parts = payload["parts"].as_array().ok_or("no parts array")?;
        if parts.len() != 3 {
            return Err(format!("expected 3 parts, got {} ({body})", parts.len()));
        }
        for part in parts {
            if part["etag"].as_str().unwrap_or("").len() != 16 {
                return Err(format!("bad etag for {}: {}", part["part"], part["etag"]));
            }
            if part["bytes"].as_i64().unwrap_or(0) <= 0 {
                return Err(format!("{} reports 0 bytes", part["part"]));
            }
        }
        Ok(())
    })());

    // 2. a matching If-Match writes and returns the new version
    gate!("matching_etag_writes", (|| -> Result<(), String> {
        let (_, body) = http(port, "GET", "/api/memory/user", None)?;
        let before: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let etag = before["etag"].as_str().unwrap_or("").to_string();

        let edited = before["content"]
            .as_str()
            .unwrap_or("")
            .replace("expertise: []", "expertise: [rust]");
        let request = json!({ "content": edited, "if_match": etag }).to_string();
        let (status, body) = http(port, "PUT", "/api/memory/user", Some(&request))?;
        if status != 200 {
            return Err(format!("PUT -> {status} {body}"));
        }
        let after: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if after["etag"].as_str() == Some(etag.as_str()) {
            return Err("the etag did not change after an edit".into());
        }
        let on_disk = fs::read_to_string(&user_path).map_err(|e| e.to_string())?;
        if !on_disk.contains("expertise: [rust]") {
            return Err("the file on disk was not updated".into());
        }
        Ok(())
    })());

    // 3. the gate that matters: a stale If-Match must not lose the update
    gate!("stale_etag_is_rejected_412", (|| -> Result<(), String> {
        let (_, body) = http(port, "GET", "/api/memory/user", None)?;
        let current: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let current_etag = current["etag"].as_str().unwrap_or("").to_string();
        let before = fs::read_to_string(&user_path).map_err(|e| e.to_string())?;

        let request =
            json!({ "content": "clobbered", "if_match": "0000000000000000" }).to_string();
        let (status, body) = http(port, "PUT", "/api/memory/user", Some(&request))?;
        if status != 412 {
            return Err(format!("stale PUT -> {status}, expected 412 ({body})"));
        }
        let after = fs::read_to_string(&user_path).map_err(|e| e.to_string())?;
        if after != before {
            return Err("a rejected write still changed the file".into());
        }
        if !body.contains(&current_etag) {
            return Err(format!("412 did not report the current etag: {body}"));
        }
        Ok(())
    })());

    // 4. no If-Match is a deliberate force
    gate!("missing_if_match_forces", (|| -> Result<(), String> {
        let request = json!({ "content": "---\nexpertise: [forced]\n---\n\nforced\n" }).to_string();
        let (status, body) = http(port, "PUT", "/api/memory/user", Some(&request))?;
        if status != 200 {
            return Err(format!("forced PUT -> {status} {body}"));
        }
        Ok(())
    })());

    // 5. only the three known parts are addressable
    gate!("unknown_part_is_rejected", (|| -> Result<(), String> {
        for part in ["bogus", "soul.md", "..", "user.md"] {
            let (status, _) = http(port, "GET", &format!("/api/memory/{part}"), None)?;
            if status != 400 && status != 404 {
                return Err(format!("GET {part} -> {status}, expected 400/404"));
            }
        }
        Ok(())
    })());

    // 6. the soul is surfaced with parsed settings
    gate!("soul_exposes_parsed_settings", (|| -> Result<(), String> {
        let (status, body) = http(port, "GET", "/api/soul", None)?;
        if status != 200 {
            return Err(format!("GET /api/soul -> {status} {body}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let settings = &payload["settings"];
        if settings["tone"] != "concise, technical" {
            return Err(format!("tone = {}", settings["tone"]));
        }
        if settings["language"] != "en" {
            return Err(format!("language = {}", settings["language"]));
        }
        if settings["max_summary_words"].as_i64() != Some(25) {
            return Err(format!("max_summary_words = {}", settings["max_summary_words"]));
        }
        let avoid = settings["avoid"].as_array().cloned().unwrap_or_default();
        if avoid.len() != 4 {
            return Err(format!("avoid = {}", settings["avoid"]));
        }
        Ok(())
    })());

    // 7. the cache row matches what was written
    gate!("cache_tracks_writes", (|| -> Result<(), String> {
        let (_, body) = http(port, "GET", "/api/memory/user", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let etag = payload["etag"].as_str().unwrap_or("").to_string();

        let conn = index_db.lock().unwrap_or_else(|p| p.into_inner());
        let row = crate::db::memory_file(&conn, &user_path.to_string_lossy())
            .map_err(|e| e.to_string())?
            .ok_or("no memory_file row for user.md")?;
        if row.1 != etag {
            return Err(format!("cache hash {} != etag {}", row.1, etag));
        }
        Ok(())
    })());

    let _ = routes_memory::PARTS;

    if failed == 0 {
        println!("hag15 self-check: OK ({total} gates)");
        0
    } else {
        println!("hag15 self-check: {failed} of {total} FAILED");
        1
    }
}

// ========================================================= HAG-16 self-check

/// Drain a subscriber until an event of `kind` shows up, or give up.
fn wait_for_event(
    rx: &mut tokio::sync::broadcast::Receiver<crate::events::Event>,
    kind: &str,
    ms: u64,
) -> Option<crate::events::Event> {
    let deadline = std::time::Instant::now() + Duration::from_millis(ms);
    while std::time::Instant::now() < deadline {
        match rx.try_recv() {
            Ok(event) if event.kind == kind => return Some(event),
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                std::thread::sleep(Duration::from_millis(25))
            }
            Err(_) => return None,
        }
    }
    None
}

/// Read just the first bytes of a response. SSE streams never end, so this
/// cannot wait for EOF the way [`http`] does.
fn http_read_some(
    port: u16,
    path: &str,
    want: usize,
    ms: u64,
) -> Result<(u16, String, String), String> {
    use std::io::{Read, Write};

    let mut stream =
        std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_millis(ms)))
        .ok();

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept: text/event-stream\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut raw = Vec::new();
    let mut buffer = [0u8; 512];
    while raw.len() < want {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&buffer[..n]),
            // A timeout is expected: the stream stays open.
            Err(_) => break,
        }
    }

    let text = String::from_utf8_lossy(&raw).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    Ok((status, head.to_string(), body.to_string()))
}

/// A note row, for seeding the index without going through the vault.
fn seed_item(id: &str, kind: &str, tags: &[String]) -> crate::db::Item {
    crate::db::Item {
        id: id.to_string(),
        url: format!("https://example.com/{id}"),
        kind: Some(kind.to_string()),
        source: Some("clipboard".to_string()),
        title: Some(format!("Note {id}")),
        summary: Some("A summary.".to_string()),
        vault_path: None,
        saved_at: 1_700_000_000,
        status: "inbox".to_string(),
        tags: tags.to_vec(),
    }
}

/// HAG-16 (S6): self-improvement — events, topics, patterns, and the SSE hub.
pub fn run_hag16() -> i32 {
    use crate::{db, events, server};

    println!("hag16 self-check");
    let mut failed = 0;
    let mut total = 0;

    macro_rules! gate {
        ($name:expr, $body:expr) => {{
            total += 1;
            match $body {
                Ok(()) => println!("  PASS {}", $name),
                Err(e) => {
                    failed += 1;
                    println!("  FAIL {}: {}", $name, e);
                }
            }
        }};
    }

    let root = std::env::temp_dir().join(format!("clipper-hag16-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);

    let setup = (|| -> Result<(u16, Config, db::Db), String> {
        let mut cfg = Config::default();
        let temp = |p: PathBuf| p.to_string_lossy().into_owned();
        cfg.vault.path = temp(root.join("Vault"));
        cfg.queue.dir = temp(root.join("queue"));
        cfg.queue.staging_dir = temp(root.join("staging"));
        cfg.prompts.extract = temp(root.join("prompts").join("extract.md"));
        cfg.skills.dir = temp(root.join("skills"));
        cfg.memory.dir = temp(root.join("memory"));
        cfg.server.bind = "127.0.0.1".into();
        cfg.server.port = 0;
        cfg.ensure_dirs().map_err(|e| e.to_string())?;

        let index_db: db::Db = Arc::new(Mutex::new(
            db::open(&root.join("index.db")).map_err(|e| e.to_string())?,
        ));
        let listener =
            std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let state = server::AppState {
            cfg: cfg.clone(),
            db: Arc::clone(&index_db),
            queue_lock: Arc::new(Mutex::new(())),
            token: Arc::new(std::sync::RwLock::new("hag16".to_string())),
            lan_ip: "127.0.0.1".into(),
        };
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(server::serve(listener, state));
        });
        wait_for_server(addr.port())?;
        Ok((addr.port(), cfg, index_db))
    })();

    let (port, cfg, index_db) = match setup {
        Ok(value) => value,
        Err(e) => {
            println!("  FAIL setup: {e}");
            return 1;
        }
    };

    // 1. a reported event lands in the table and comes back in the summary
    gate!("events_are_recorded", (|| -> Result<(), String> {
        let (status, body) = http(
            port,
            "POST",
            "/api/events",
            Some(r#"{"kind":"clip_viewed","payload":{"id":"abc"}}"#),
        )?;
        if status != 200 {
            return Err(format!("POST /api/events -> {status} {body}"));
        }
        let (_, body) = http(port, "GET", "/api/self/summary", None)?;
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if payload["event_count"].as_i64() != Some(1) {
            return Err(format!("event_count = {} ({body})", payload["event_count"]));
        }
        if payload["recent_events"][0]["kind"] != "clip_viewed" {
            return Err(format!("recent kind = {}", payload["recent_events"][0]["kind"]));
        }
        Ok(())
    })());

    // 2. recompute derives topics and patterns from the index
    gate!("recompute_derives_topics_and_patterns", (|| -> Result<(), String> {
        {
            let conn = index_db.lock().unwrap_or_else(|p| p.into_inner());
            for (id, kind, tags) in [
                ("n1", "article", vec!["rust".to_string(), "systems".to_string()]),
                ("n2", "article", vec!["rust".to_string()]),
                ("n3", "video", vec!["go".to_string()]),
            ] {
                db::upsert(&conn, &seed_item(id, kind, &tags)).map_err(|e| e.to_string())?;
            }
        }

        let (status, body) = http(port, "POST", "/api/self/recompute", Some("{}"))?;
        if status != 200 {
            return Err(format!("recompute -> {status} {body}"));
        }
        let payload: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        let topics = payload["topics"].as_array().cloned().unwrap_or_default();
        if topics.is_empty() {
            return Err("no topics were derived".into());
        }
        if topics[0]["label"] != "rust" || topics[0]["weight"].as_f64() != Some(2.0) {
            return Err(format!("top topic = {} ({body})", topics[0]));
        }
        let patterns = payload["patterns"].as_array().cloned().unwrap_or_default();
        if patterns.is_empty() {
            return Err("no patterns were derived".into());
        }
        if !patterns[0]["rule_md"].as_str().unwrap_or("").contains("rust") {
            return Err(format!("first pattern = {}", patterns[0]["rule_md"]));
        }
        Ok(())
    })());

    // 3. recompute is idempotent: derived tables are replaced, never appended
    gate!("recompute_is_idempotent", (|| -> Result<(), String> {
        let (_, first) = http(port, "POST", "/api/self/recompute", Some("{}"))?;
        let (_, second) = http(port, "POST", "/api/self/recompute", Some("{}"))?;
        let a: Value = serde_json::from_str(&first).map_err(|e| e.to_string())?;
        let b: Value = serde_json::from_str(&second).map_err(|e| e.to_string())?;
        if a["topics"].as_array().map(Vec::len) != b["topics"].as_array().map(Vec::len) {
            return Err("the topic count drifted between runs".into());
        }
        if a["topics"].as_array().map(Vec::len) != Some(3) {
            return Err(format!("expected 3 topics (rust, systems, go), got {}", a["topics"]));
        }
        if a["patterns"].as_array().map(Vec::len) != b["patterns"].as_array().map(Vec::len) {
            return Err("the pattern count drifted between runs".into());
        }
        Ok(())
    })());

    // 4. the hub reaches live subscribers
    gate!("hub_broadcasts_to_subscribers", (|| -> Result<(), String> {
        let mut rx = events::hub().subscribe();
        let (status, _) = http(
            port,
            "POST",
            "/api/events",
            Some(r#"{"kind":"search_used"}"#),
        )?;
        if status != 200 {
            return Err(format!("POST /api/events -> {status}"));
        }
        let event = wait_for_event(&mut rx, "user_event", 3000)
            .ok_or("no user_event reached the subscriber")?;
        if event.payload["kind"] != "search_used" {
            return Err(format!("payload = {}", event.payload));
        }
        Ok(())
    })());

    // 5. indexing a note feeds the stream
    gate!("vaultwatch_feeds_the_stream", (|| -> Result<(), String> {
        let vault = cfg.vault_inbox();
        fs::create_dir_all(&vault).map_err(|e| e.to_string())?;
        fs::write(
            vault.join("2026-01-01_1200_clipboard_stream1.md"),
            "---\ntype: article\nsource: example.com\nurl: https://example.com/s\n\
             saved_at: 2026-01-01T12:00:00Z\ntitle: Streamed\ntags: [events]\n---\n\n# Streamed\n",
        )
        .map_err(|e| e.to_string())?;

        let mut rx = events::hub().subscribe();
        let indexer = crate::vaultwatch::Indexer::new(cfg.clone(), Arc::clone(&index_db));
        let indexed = indexer.ingest_dir().map_err(|e| e.to_string())?;
        if indexed != 1 {
            return Err(format!("ingested {indexed} notes, expected 1"));
        }
        let event = wait_for_event(&mut rx, "note_indexed", 3000)
            .ok_or("indexing did not emit note_indexed")?;
        if event.payload["id"] != "stream1" {
            return Err(format!("payload = {}", event.payload));
        }
        Ok(())
    })());

    // 6. the SSE route really speaks text/event-stream
    gate!("sse_route_streams", (|| -> Result<(), String> {
        let (status, head, body) = http_read_some(port, "/api/events", 64, 4000)?;
        if status != 200 {
            return Err(format!("GET /api/events -> {status}"));
        }
        if !head.to_ascii_lowercase().contains("text/event-stream") {
            return Err(format!("content-type was not event-stream: {head}"));
        }
        if !body.contains("event: hello") {
            return Err(format!("no hello frame: {body:?}"));
        }
        Ok(())
    })());


    if failed == 0 {
        println!("hag16 self-check: OK ({total} gates)");
        0
    } else {
        println!("hag16 self-check: {failed} of {total} FAILED");
        1
    }
}
