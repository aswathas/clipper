//! The SQLite index: schema, upsert, and the read side the REST API uses.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::{params, Connection, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};

/// Shared handle to the index.
pub type Db = Arc<Mutex<Connection>>;

/// Statuses the API accepts.
pub const STATUSES: [&str; 5] = ["inbox", "reading", "done", "archived", "deleted"];

const COLUMNS: &str = "items.id, items.url, items.type, items.source, items.title, \
                       items.summary, items.vault_path, items.saved_at, items.status, items.tags";

/// One indexed note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub vault_path: Option<String>,
    /// Unix seconds.
    pub saved_at: i64,
    pub status: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Filters for [`list`].
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub limit: i64,
    pub offset: i64,
    pub kind: Option<String>,
    pub tag: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
}

/// Open (creating if needed) the index and apply the schema.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let conn = Connection::open(path)?;
    // journal_mode returns a row, so it has to go through execute_batch.
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory index (used by the self-checks).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    migrate(&conn)?;
    Ok(conn)
}

/// The schema. `items_fts` is an external-content FTS5 table over `items`,
/// kept in sync by the three standard triggers.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
CREATE TABLE IF NOT EXISTS items (
  id TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  type TEXT,
  source TEXT,
  title TEXT,
  summary TEXT,
  vault_path TEXT,
  saved_at INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'inbox',
  tags TEXT
);
CREATE INDEX IF NOT EXISTS items_saved_at_idx ON items(saved_at DESC);
CREATE INDEX IF NOT EXISTS items_type_idx ON items(type);
CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
  title, summary, tags, url,
  content='items', content_rowid='rowid'
);
CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
  INSERT INTO items_fts(rowid, title, summary, tags, url)
  VALUES (new.rowid, new.title, new.summary, new.tags, new.url);
END;
CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, title, summary, tags, url)
  VALUES ('delete', old.rowid, old.title, old.summary, old.tags, old.url);
END;
CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
  INSERT INTO items_fts(items_fts, rowid, title, summary, tags, url)
  VALUES ('delete', old.rowid, old.title, old.summary, old.tags, old.url);
  INSERT INTO items_fts(rowid, title, summary, tags, url)
  VALUES (new.rowid, new.title, new.summary, new.tags, new.url);
END;

-- Skills (S4). The markdown files under ~/.clipper/skills are the source of
-- truth; this table is a cache rebuilt from them.
CREATE TABLE IF NOT EXISTS skills (
  name TEXT PRIMARY KEY,
  source TEXT,
  body_md TEXT NOT NULL DEFAULT '',
  frontmatter_json TEXT,
  token_estimate INTEGER NOT NULL DEFAULT 0,
  last_modified INTEGER NOT NULL DEFAULT 0
);

-- Memory/soul parts (S5): an mtime + hash cache backing If-Match.
CREATE TABLE IF NOT EXISTS memory_file (
  path TEXT PRIMARY KEY,
  mtime INTEGER NOT NULL DEFAULT 0,
  sha256 TEXT NOT NULL DEFAULT ''
);

-- Self-improvement (S6).
CREATE TABLE IF NOT EXISTS user_event (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts INTEGER NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT
);
CREATE INDEX IF NOT EXISTS user_event_ts_idx ON user_event(ts DESC);
CREATE TABLE IF NOT EXISTS user_topic (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  label TEXT NOT NULL UNIQUE,
  weight REAL NOT NULL DEFAULT 0,
  last_seen INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS user_pattern (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  rule_md TEXT NOT NULL,
  confidence REAL NOT NULL DEFAULT 0,
  added_at INTEGER NOT NULL DEFAULT 0
);
"#,
    )
}

/// Insert or update a note. `status` is deliberately *not* overwritten, so a
/// status the user set in the dashboard survives a re-index.
pub fn upsert(conn: &Connection, item: &Item) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO items (id, url, type, source, title, summary, vault_path, saved_at, status, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
           url = excluded.url,
           type = excluded.type,
           source = excluded.source,
           title = excluded.title,
           summary = excluded.summary,
           vault_path = excluded.vault_path,
           saved_at = excluded.saved_at,
           tags = excluded.tags",
        params![
            item.id,
            item.url,
            item.kind,
            item.source,
            item.title,
            item.summary,
            item.vault_path,
            item.saved_at,
            item.status,
            tags_to_json(&item.tags),
        ],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Item>> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM items WHERE items.id = ?1"),
        params![id],
        row_to_item,
    )
    .optional()
}

/// Filtered, paged listing plus the unpaged total.
pub fn list(conn: &Connection, query: &Query) -> rusqlite::Result<(Vec<Item>, i64)> {
    let mut join = String::new();
    let mut wheres: Vec<String> = Vec::new();
    let mut args: Vec<String> = Vec::new();

    if let Some(q) = query.q.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        join.push_str(" JOIN items_fts ON items_fts.rowid = items.rowid");
        wheres.push("items_fts MATCH ?".into());
        args.push(fts_literal(q));
    }
    if let Some(kind) = &query.kind {
        wheres.push("items.type = ?".into());
        args.push(kind.clone());
    }
    if let Some(status) = &query.status {
        wheres.push("items.status = ?".into());
        args.push(status.clone());
    }
    if let Some(tag) = &query.tag {
        wheres.push("EXISTS (SELECT 1 FROM json_each(items.tags) WHERE json_each.value = ?)".into());
        args.push(tag.clone());
    }

    let where_clause = if wheres.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", wheres.join(" AND "))
    };

    let bound: Vec<&dyn ToSql> = args.iter().map(|arg| arg as &dyn ToSql).collect();

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM items{join}{where_clause}"),
        bound.as_slice(),
        |row| row.get(0),
    )?;

    let limit = query.limit.clamp(1, 500);
    let offset = query.offset.max(0);
    let mut paged: Vec<&dyn ToSql> = bound.clone();
    paged.push(&limit);
    paged.push(&offset);

    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM items{join}{where_clause} ORDER BY items.saved_at DESC LIMIT ? OFFSET ?"
    ))?;
    let rows = statement.query_map(paged.as_slice(), row_to_item)?;
    let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok((items, total))
}

/// The v1 "today" heuristic: newest unread items.
pub fn today(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<Item>> {
    let mut statement = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM items WHERE items.status = 'inbox' ORDER BY items.saved_at DESC LIMIT ?1"
    ))?;
    let rows = statement.query_map(params![limit], row_to_item)?;
    rows.collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total: i64,
    pub by_type: std::collections::BTreeMap<String, i64>,
    pub by_status: std::collections::BTreeMap<String, i64>,
    pub stale_30d: i64,
}

pub fn stats(conn: &Connection) -> rusqlite::Result<Stats> {
    let total = conn.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?;

    let mut by_type = std::collections::BTreeMap::new();
    let mut statement = conn.prepare("SELECT COALESCE(type, 'unknown'), COUNT(*) FROM items GROUP BY 1")?;
    for row in statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))? {
        let (key, count) = row?;
        by_type.insert(key, count);
    }

    let mut by_status = std::collections::BTreeMap::new();
    let mut statement = conn.prepare("SELECT status, COUNT(*) FROM items GROUP BY 1")?;
    for row in statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))? {
        let (key, count) = row?;
        by_status.insert(key, count);
    }

    let cutoff = crate::clock::now_unix() - 30 * 24 * 60 * 60;
    let stale_30d = conn.query_row(
        "SELECT COUNT(*) FROM items WHERE status = 'inbox' AND saved_at < ?1",
        params![cutoff],
        |row| row.get(0),
    )?;

    Ok(Stats {
        total,
        by_type,
        by_status,
        stale_30d,
    })
}

/// Returns false when no such item exists.
pub fn set_status(conn: &Connection, id: &str, status: &str) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE items SET status = ?2 WHERE id = ?1",
        params![id, status],
    )?;
    Ok(changed > 0)
}

pub fn set_tags(conn: &Connection, id: &str, tags: &[String]) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE items SET tags = ?2 WHERE id = ?1",
        params![id, tags_to_json(tags)],
    )?;
    Ok(changed > 0)
}

fn tags_to_json(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

// ------------------------------------------------------------------- skills
/// A skill file, mirrored into the index (S4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub body_md: String,
    pub token_estimate: i64,
    pub last_modified: i64,
}

pub fn upsert_skill(conn: &Connection, skill: &Skill) -> rusqlite::Result<()> {
    let frontmatter = serde_json::json!({
        "title": skill.title,
        "description": skill.description,
    })
    .to_string();

    conn.execute(
        "INSERT INTO skills (name, source, body_md, frontmatter_json, token_estimate, last_modified)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(name) DO UPDATE SET
           source = excluded.source,
           body_md = excluded.body_md,
           frontmatter_json = excluded.frontmatter_json,
           token_estimate = excluded.token_estimate,
           last_modified = excluded.last_modified",
        params![
            skill.name,
            skill.source,
            skill.body_md,
            frontmatter,
            skill.token_estimate,
            skill.last_modified,
        ],
    )?;
    Ok(())
}

pub fn list_skills(conn: &Connection) -> rusqlite::Result<Vec<Skill>> {
    let mut statement = conn.prepare(
        "SELECT name, source, body_md, frontmatter_json, token_estimate, last_modified
         FROM skills ORDER BY name",
    )?;
    let rows = statement.query_map([], row_to_skill)?;
    rows.collect()
}

pub fn get_skill(conn: &Connection, name: &str) -> rusqlite::Result<Option<Skill>> {
    conn.query_row(
        "SELECT name, source, body_md, frontmatter_json, token_estimate, last_modified
         FROM skills WHERE name = ?1",
        params![name],
        row_to_skill,
    )
    .optional()
}

pub fn delete_skill(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM skills WHERE name = ?1", params![name])? > 0)
}

fn row_to_skill(row: &rusqlite::Row<'_>) -> rusqlite::Result<Skill> {    let frontmatter: Option<String> = row.get(3)?;
    let parsed: serde_json::Value = frontmatter
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(Skill {
        name: row.get(0)?,
        source: row.get(1)?,
        body_md: row.get(2)?,
        title: parsed.get("title").and_then(|v| v.as_str()).map(str::to_string),
        description: parsed
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        token_estimate: row.get(4)?,
        last_modified: row.get(5)?,
    })
}

// -------------------------------------------------------------- memory files

/// Record a memory/soul part's mtime and hash.
pub fn upsert_memory_file(
    conn: &Connection,
    path: &str,
    mtime: i64,
    sha256: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO memory_file (path, mtime, sha256) VALUES (?1, ?2, ?3)
         ON CONFLICT(path) DO UPDATE SET mtime = excluded.mtime, sha256 = excluded.sha256",
        params![path, mtime, sha256],
    )?;
    Ok(())
}

/// The cached `(mtime, sha256)` for a part, if it has been seen.
pub fn memory_file(conn: &Connection, path: &str) -> rusqlite::Result<Option<(i64, String)>> {
    conn.query_row(
        "SELECT mtime, sha256 FROM memory_file WHERE path = ?1",
        params![path],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
}

// ----------------------------------------------------------- self-improvement

/// Something the user did.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub id: i64,
    pub ts: i64,
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// A topic Clipper has noticed the user cares about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTopic {
    pub label: String,
    pub weight: f64,
    pub last_seen: i64,
}

/// A rule derived from the user's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPattern {
    pub rule_md: String,
    pub confidence: f64,
    pub added_at: i64,
}

pub fn record_event(
    conn: &Connection,
    ts: i64,
    kind: &str,
    payload: &serde_json::Value,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO user_event (ts, kind, payload_json) VALUES (?1, ?2, ?3)",
        params![ts, kind, payload.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn event_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM user_event", [], |row| row.get(0))
}

pub fn recent_events(conn: &Connection, limit: i64) -> rusqlite::Result<Vec<UserEvent>> {
    let mut statement = conn.prepare(
        "SELECT id, ts, kind, COALESCE(payload_json, '{}') FROM user_event
         ORDER BY ts DESC, id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![limit], |row| {
        let payload: String = row.get(3)?;
        Ok(UserEvent {
            id: row.get(0)?,
            ts: row.get(1)?,
            kind: row.get(2)?,
            payload: serde_json::from_str(&payload).unwrap_or_else(|_| serde_json::json!({})),
        })
    })?;
    rows.collect()
}

pub fn list_topics(conn: &Connection) -> rusqlite::Result<Vec<UserTopic>> {
    let mut statement = conn.prepare(
        "SELECT label, weight, last_seen FROM user_topic ORDER BY weight DESC, label ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(UserTopic {
            label: row.get(0)?,
            weight: row.get(1)?,
            last_seen: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn list_patterns(conn: &Connection) -> rusqlite::Result<Vec<UserPattern>> {
    let mut statement = conn.prepare(
        "SELECT rule_md, confidence, added_at FROM user_pattern ORDER BY confidence DESC, id ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(UserPattern {
            rule_md: row.get(0)?,
            confidence: row.get(1)?,
            added_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

/// Recompute replaces the derived tables wholesale — they are pure functions of
/// the index, so there is nothing to merge.
pub fn replace_topics(conn: &mut Connection, topics: &[UserTopic]) -> rusqlite::Result<()> {
    let transaction = conn.transaction()?;
    transaction.execute("DELETE FROM user_topic", [])?;
    for topic in topics {
        transaction.execute(
            "INSERT INTO user_topic (label, weight, last_seen) VALUES (?1, ?2, ?3)",
            params![topic.label, topic.weight, topic.last_seen],
        )?;
    }
    transaction.commit()
}

pub fn replace_patterns(conn: &mut Connection, patterns: &[UserPattern]) -> rusqlite::Result<()> {
    let transaction = conn.transaction()?;
    transaction.execute("DELETE FROM user_pattern", [])?;
    for pattern in patterns {
        transaction.execute(
            "INSERT INTO user_pattern (rule_md, confidence, added_at) VALUES (?1, ?2, ?3)",
            params![pattern.rule_md, pattern.confidence, pattern.added_at],
        )?;
    }
    transaction.commit()
}

/// Every `(type, tags_json)` pair, for deriving topics.
pub fn item_type_tags(conn: &Connection) -> rusqlite::Result<Vec<(Option<String>, String)>> {
    let mut statement = conn.prepare("SELECT type, COALESCE(tags, '[]') FROM items")?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

/// FTS5 MATCH wants a phrase; wrap the user's text so their punctuation cannot
/// be read as query syntax, doubling any inner quotes.
fn fts_literal(raw: &str) -> String {
    format!("\"{}\"", raw.replace('"', "\"\""))
}

// ----------------------------------------------------------- Zettelkasten links

/// Items related to `id` by shared tags (Zettelkasten principle: atomic notes
/// cross-link by concept, not by chronology). Excludes the source id itself.
/// Up to `limit` items, ranked by overlap count then recency.
pub fn related_by_tags(
    conn: &Connection,
    id: &str,
    limit: i64,
) -> rusqlite::Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "WITH src AS (SELECT tags FROM items WHERE id = ?1)
         SELECT i.id, i.url, i.type, i.source, i.title, i.summary,
                i.vault_path, i.saved_at, i.status, i.tags,
                (SELECT COUNT(*) FROM json_each(src.tags) je
                  WHERE je.value IN (SELECT value FROM json_each(i.tags))) AS overlap
         FROM items i, src
         WHERE i.id != ?1
           AND EXISTS (
             SELECT 1 FROM json_each(src.tags) je2
             WHERE je2.value IN (SELECT value FROM json_each(i.tags))
           )
         ORDER BY overlap DESC, i.saved_at DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![id, limit], |row| {
        let tags_raw: Option<String> = row.get(9)?;
        Ok(Item {
            id: row.get(0)?,
            url: row.get(1)?,
            kind: row.get(2)?,
            source: row.get(3)?,
            title: row.get(4)?,
            summary: row.get(5)?,
            vault_path: row.get(6)?,
            saved_at: row.get(7)?,
            status: row.get(8)?,
            tags: tags_raw
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .unwrap_or_default(),
        })
    })?;
    rows.collect()
}

/// Backlinks: items whose markdown body mentions `[[<id>]]`. Walks every
/// indexed note's `vault_path`, extracts wikilinks, and returns the ones
/// pointing at `id`. Limited to `limit` results, newest first.
pub fn backlinks(conn: &Connection, id: &str, limit: i64) -> rusqlite::Result<Vec<Item>> {
    let mut stmt = conn.prepare(
        "SELECT id, url, type, source, title, summary, vault_path, saved_at, status, tags
         FROM items WHERE id != ?1 AND vault_path IS NOT NULL
         ORDER BY saved_at DESC",
    )?;
    let rows = stmt.query_map(params![id], |row| {
        let tags_raw: Option<String> = row.get(9)?;
        Ok(Item {
            id: row.get(0)?,
            url: row.get(1)?,
            kind: row.get(2)?,
            source: row.get(3)?,
            title: row.get(4)?,
            summary: row.get(5)?,
            vault_path: row.get(6)?,
            saved_at: row.get(7)?,
            status: row.get(8)?,
            tags: tags_raw
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .unwrap_or_default(),
        })
    })?;
    let needle = format!("[[{}]]", id);
    let mut out: Vec<Item> = Vec::new();
    for item in rows {
        let item = item?;
        if let Some(path) = item.vault_path.as_deref() {
            if let Ok(body) = std::fs::read_to_string(path) {
                if body.contains(&needle) {
                    out.push(item);
                    if out.len() as i64 >= limit {
                        break;
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Extract `[[id]]` wikilinks from a markdown body. Used by the indexer to
/// harvest user-authored links and by the related-items heuristic.
pub fn extract_wikilinks(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("]]") {
            let inner = after[..end].trim();
            if !inner.is_empty() && !inner.contains('[') && !inner.contains(']') {
                out.push(inner.to_string());
            }
            rest = &after[end + 2..];
        } else {
            break;
        }
    }
    out
}

fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    let tags_raw: Option<String> = row.get(9)?;
    Ok(Item {
        id: row.get(0)?,
        url: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        kind: row.get(2)?,
        source: row.get(3)?,
        title: row.get(4)?,
        summary: row.get(5)?,
        vault_path: row.get(6)?,
        saved_at: row.get(7)?,
        status: row.get(8)?,
        tags: tags_raw
            .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
            .unwrap_or_default(),
    })
}
