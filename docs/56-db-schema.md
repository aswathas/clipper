# Database schema

```sql
CREATE TABLE clips (
  id            TEXT PRIMARY KEY,
  url           TEXT,
  title         TEXT,
  host          TEXT,
  saved_at      INTEGER NOT NULL,
  body_path     TEXT NOT NULL,
  deleted_at    INTEGER,
  hash          TEXT UNIQUE
);

CREATE TABLE tags (
  clip_id       TEXT NOT NULL,
  tag           TEXT NOT NULL,
  PRIMARY KEY (clip_id, tag)
);

CREATE TABLE people (
  id            INTEGER PRIMARY KEY,
  name          TEXT NOT NULL,
  relation      TEXT,
  last_seen     INTEGER
);

CREATE VIRTUAL TABLE clips_fts USING fts5(
  title, body, host, tags,
  content='clips', content_rowid='rowid'
);
```

The schema is in `src-tauri/src/db.rs`.
