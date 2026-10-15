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

use serde_json::Value;

use crate::{
    capture::{self, CaptureOutcome},
    config::Config,
    drainer::Drainer,
    queue,
};

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
    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock)).map_err(|e| e.to_string())?;
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

    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock)).map_err(|e| e.to_string())?;
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
    use std::io::{Read, Write};

    let mut stream =
        std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok();

    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
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
    let raw = String::from_utf8_lossy(&raw).into_owned();

    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw.as_str(), ""));
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let body = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
        dechunk(body)
    } else {
        body.to_string()
    };

    Ok((status, body))
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

    let drainer = Drainer::new(cfg.clone(), Arc::clone(&lock)).map_err(|e| e.to_string())?;
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
