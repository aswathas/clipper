//! The JSONL queue on disk: append, rotate, and the dedupe index.

use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// True when the clipboard text looks like a web URL.
pub fn looks_like_url(text: &str) -> bool {
    let t = text.trim().to_ascii_lowercase();
    t.starts_with("http://") || t.starts_with("https://")
}

/// Best-effort hostname for user-facing messages.
pub fn host_of(url: &str) -> String {
    let trimmed = url.trim();
    let after_scheme = trimmed.split_once("://").map(|(_, rest)| rest).unwrap_or(trimmed);
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    host.strip_prefix("www.").unwrap_or(host).to_string()
}

/// `sha256(url)[..16]`, hex.
pub fn url_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.trim().as_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    hex.get(..16).unwrap_or(&hex).to_string()
}

/// One captured item in `inbox.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub id: String,
    pub url: String,
    pub source: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub saved_at: String,
}

/// Append one JSON object as a line, fsync'd so a polling reader sees it.
pub fn append_json_line<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut line = serde_json::to_string(value).map_err(io::Error::other)?;
    line.push('\n');
    file.write_all(line.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

/// Read a JSONL file into trimmed, non-empty lines. Missing file -> empty.
pub fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect())
}

/// Replace a file's contents atomically (write a sibling temp file, then rename).
pub fn write_lines_atomic(path: &Path, lines: &[String]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut body = String::new();
    for line in lines {
        body.push_str(line);
        body.push('\n');
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)
}

/// Every `hash` already recorded in `archive.jsonl`.
pub fn archived_hashes(archive: &Path) -> io::Result<HashSet<String>> {
    let mut set = HashSet::new();
    for line in read_lines(archive)? {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(hash) = value.get("hash").and_then(|h| h.as_str()) {
                set.insert(hash.to_string());
            }
        }
    }
    Ok(set)
}
