//! Keeps the SQLite index in step with the vault's Inbox folder.

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, UNIX_EPOCH},
};

use crate::{clock, config::Config, db, frontmatter};

/// Watches the vault and writes notes into the index.
pub struct Indexer {
    cfg: Config,
    db: db::Db,
    /// Set once the watcher is installed and the startup scan has finished.
    ready: Arc<AtomicBool>,
}

impl Indexer {
    pub fn new(cfg: Config, db: db::Db) -> Self {
        Self {
            cfg,
            db,
            ready: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Flips to true once [`Indexer::watch`] is actually watching. Lets callers
    /// (and the self-check) avoid racing the startup scan.
    pub fn ready_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.ready)
    }

    /// Index one note. Returns its id when the file looked like a Clipper note.
    pub fn ingest_file(&self, path: &Path) -> std::io::Result<Option<String>> {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Ok(None);
        };
        // The id lives in the filename, which is what makes re-indexing idempotent.
        let Some((id, source)) = frontmatter::parse_note_filename(file_name) else {
            return Ok(None);
        };

        let text = fs::read_to_string(path)?;
        let meta = frontmatter::parse(&text).unwrap_or_default();

        let saved_at = meta
            .get("saved_at")
            .and_then(frontmatter::parse_iso8601)
            .or_else(|| {
                fs::metadata(path)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
            })
            .unwrap_or_else(clock::now_unix);

        let item = db::Item {
            id: id.clone(),
            url: meta.get("url").unwrap_or_default().to_string(),
            kind: meta.get("type").map(str::to_string),
            source: meta.get("source").map(str::to_string).or(Some(source)),
            title: meta.get("title").map(str::to_string),
            summary: meta.get("summary").map(str::to_string),
            vault_path: Some(path.to_string_lossy().into_owned()),
            saved_at,
            status: "inbox".to_string(),
            tags: meta.tags(),
        };

        let conn = self.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        db::upsert(&conn, &item).map_err(std::io::Error::other)?;
        drop(conn);

        // Feed the self-improvement stream: every note that lands is an event.
        crate::events::hub().emit(
            "note_indexed",
            serde_json::json!({ "id": id, "title": item.title, "type": item.kind }),
        );

        Ok(Some(id))
    }

    /// Index every `.md` already in the vault's Inbox.
    pub fn ingest_dir(&self) -> std::io::Result<usize> {
        let Ok(entries) = fs::read_dir(self.cfg.vault_inbox()) else {
            return Ok(0);
        };
        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if is_markdown(&path) && self.ingest_file(&path)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Watch the vault Inbox, debouncing the burst a single save produces.
    pub async fn watch(self: Arc<Self>) {
        let dir = self.cfg.vault_inbox();
        let _ = fs::create_dir_all(&dir);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    for path in event.paths {
                        let _ = tx.send(path);
                    }
                }
            })
            .ok();
        if let Some(watcher) = watcher.as_mut() {
            use notify::Watcher;
            let _ = watcher.watch(&dir, notify::RecursiveMode::NonRecursive);
        }

        if let Ok(count) = self.ingest_dir() {
            if count > 0 {
                println!("indexer: {count} existing note(s) indexed");
            }
        }

        self.ready.store(true, Ordering::SeqCst);

        loop {
            let Some(first) = rx.recv().await else { return };
            let mut changed: HashSet<PathBuf> = HashSet::new();
            changed.insert(first);

            // Coalesce everything that lands within 500ms.
            loop {
                match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                    Ok(Some(path)) => {
                        changed.insert(path);
                    }
                    Ok(None) => return,
                    Err(_) => break,
                }
            }

            let mut indexed = 0;
            for path in changed {
                if !is_markdown(&path) || !path.is_file() {
                    continue;
                }
                match self.ingest_file(&path) {
                    Ok(Some(_)) => indexed += 1,
                    Ok(None) => {}
                    Err(e) => eprintln!("indexer: {}: {e}", path.display()),
                }
            }
            if indexed > 0 {
                println!("indexer: {indexed} note(s) indexed");
            }
        }
    }
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}
