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

/// FTS5 MATCH wants a phrase; wrap the user's text so their punctuation cannot
/// be read as query syntax, doubling any inner quotes.
fn fts_literal(raw: &str) -> String {
    format!("\"{}\"", raw.replace('"', "\"\""))
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
