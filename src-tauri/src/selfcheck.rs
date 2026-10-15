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
};

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

/// Run every check; returns a process exit code.
pub fn run() -> i32 {
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
