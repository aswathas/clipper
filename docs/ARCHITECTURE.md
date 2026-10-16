# Clipper Architecture

## One-liner

A Tauri 2 desktop binary that captures anything via a hotkey, runs it through Claude Code CLI to produce a structured markdown note, and serves a local web dashboard over `http://127.0.0.1:7137`.

## Process layout

```
+---------------------+        +-------------------------+
|  Tray / Hotkey      |  POST  |  Tauri 2 binary         |
|  (single-instance)  | -----> |  (Rust + axum + SQLite)|
+---------------------+        +-------------------------+
                                        |
                                        | spawns `claude -p`
                                        v
                                +---------------+
                                |  Claude Code  |
                                |  CLI          |
                                +---------------+
                                        |
                                        v
        +-------------------+   +-------------------+   +-------------------+
        |  ~/Documents/     |   |  SQLite + FTS5    |   |  http://127.0.0.1: |
        |  ClipperVault/     |   |  (single file     |   |  7137 web         |
        |  Inbox/*.md        |   |   in app data)    |   |  dashboard        |
        +-------------------+   +-------------------+   +-------------------+
```

## Components

### Tauri daemon (`src-tauri/`)

- `lib.rs` — tray menu, single-instance plugin, global shortcut, plugin-opener for `http://127.0.0.1:7137` in default browser
- `server.rs` — axum routes for `/api/items`, `/api/today`, `/api/skills`, `/api/clips`, `/api/stats`, `/healthz`, `/pair`, `/save`, `/api/items/{id}`
- `db.rs` — rusqlite connection, FTS5 virtual table, schema migrations, query helpers
- `drainer.rs` — spawns `claude -p <prompt_file>`, tolerates real Claude stdout (parses paths under staging, scans for `.md` outputs)
- `frontmatter.rs` — split YAML frontmatter from body, parse frontmatter fields
- `config.rs` — TOML config (provider, vault path, server port, log level, defaults)
- `seed/` — embedded `soul.md`, `user.md`, `memory.md` defaults
- `selfcheck.rs` — `--<n>-smoke` flag handlers, one per slice

### Frontend (`src/ui/`)

- `index.html` — Alpine.js templates, 6 pages, 8 inline SVG type icons
- `app.js` — page factories (`pageToday`, `pageLibrary`, `pageInbox`, `pageAsk`, `pageSkills`, `pageSettings`), `clipperShell`, `api(path)` with `?mock=1` fallback
- `styles.css` — warm-paper palette tokens, Spectral + Inter + JetBrains Mono, library aesthetic
- `build.rs` — walks `src/ui/` at compile time, generates a route table, embeds via `include_bytes!`

### Vault layout (`~/Documents/ClipperVault/`)

- `Inbox/<timestamp>_<source>_<uuid>.md` — captured items
- `Library/` — manually archived items (future)
- `Settings.toml` — user config (currently lives in `~/.config/clipper/`)
- `Activity/<YYYY-MM-DD>.md` — Muse daily notes (v0.5+)

## Data flow

1. Hotkey → `create_clip` route → item row in `queue` table
2. Drainer polls queue → spawns `claude -p` with prompt including source URL, source type, frontmatter template
3. Claude writes `<id>.md` to vault staging area
4. Drainer reads the file, validates frontmatter, moves to `vault_path`
5. Item row in `items` table with `vault_path` set
6. Dashboard `/api/today` returns recent items; `/api/items/{id}` returns the full body split from frontmatter

## Build

- Toolchain: Rust GNU (x86_64-pc-windows-gnu), Node 18+
- `cargo build --release --manifest-path src-tauri/Cargo.toml`
- `npx tauri build --no-bundle` (Tauri config aware)
- Output: `src-tauri/target/release/clipper.exe`

## Test

- `powershell -File scripts/smoke.ps1` — 22 end-to-end checks
- `./clipper.exe --hag<n>-smoke` — per-slice self-checks (11 currently, +3 in v0.5)
- `cargo test` not viable on windows-gnu (Tauri test harness doesn't link)

## Roadmap

- v0.5 — Muse (passive watcher), mass import (Pocket / Chrome / Notion / JSON), 5 UI ideas, Obsidian-style wikilinks
- v0.6 — Smart captures: hotkey → screenshot + voice context → typed cards (shopping / watchlist / recipe)
- v0.7 — Proactive assistant with screen context
