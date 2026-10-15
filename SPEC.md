# Clipper — Specification (SPEC.md)

Consolidated specification for Clipper (Smart Clipboard / Watch-Later). This document is the canonical design reference for all implementation slices.

---

## 1. Goal

Clipper is a lightweight, local-first smart clipboard and watch-later system. It captures URLs, screenshots, photos, and arbitrary text, forwards them to a local LLM runner (Claude Code CLI) for extraction, summarization, and tag generation, stores enriched notes directly into an Obsidian vault as Markdown files, indexes them in a local SQLite database, and exposes an embedded web dashboard for browsing, filtering, and review.

The system runs as a single background desktop tray application built on Tauri 2 (Rust) serving a local web UI on `http://127.0.0.1:7137`.

---

## 2. Non-goals (v1)

The following capabilities are explicitly out of scope for v1:
- **ONNX classifier**: No local neural network inference runtime or classification models.
- **Daily digest engine**: No automated newsletter generation, daily rollups, or scheduled digest summaries.
- **Auto-archive**: No automatic deletion, expiration policies, or background archiving of clips.
- **Flutter app**: No native mobile application; mobile capture relies entirely on a lightweight LAN-paired Progressive Web App (PWA).
- **Multi-provider concurrent routing**: No dynamic fallback or parallel multi-model routing; Claude Code CLI is the sole active extraction engine in v1.
- **Cloud synchronization service**: No proprietary cloud backend or external user accounts; data is stored locally in SQLite and the user's Obsidian vault.

---

## 3. Personas

`[TBD — needs human]`
*Note: Not specified on the Hive board.*
- **Primary user**: Knowledge worker or researcher managing information intake across desktop and mobile who relies on an Obsidian vault as a second brain and needs zero-friction capture without manual formatting.
- Additional user personas and role specifications require human input (`[TBD — needs human]`).

---

## 4. Architecture diagram (ASCII)

```
[hotkey Ctrl+Shift+C] / [Win+Shift+S screenshot] / [drag-drop] / [phone PWA]
        ↓
[Clipper tray app :7137]  ← single Tauri binary
        ↓ append
[~/.clipper/queue/inbox.jsonl]
        ↓ batched drain
[claude -p with prompt from ~/.clipper/prompts/extract.md]
        ↓ for images: mcp__MiniMax__understand_image
[~/.clipper/staging/*.md]
        ↓ move
[Obsidian vault (auto-detect) / <config>]
        ↓ update
[index.db — rusqlite, FTS5]
        ↓ serve
[http://127.0.0.1:7137/ui dashboard]
```

### Flow Breakdown
1. **Ingress**: Capture via global hotkey (`Ctrl+Shift+C`), OS screenshot (`Win+Shift+S`), drag-and-drop, or phone PWA.
2. **Tray Service**: Single Tauri 2 binary listening on loopback port 7137; appends payload to `~/.clipper/queue/inbox.jsonl`.
3. **Queue Drainer**: Background worker batches items and invokes `claude -p` using prompt `~/.clipper/prompts/extract.md`.
4. **Vision Path**: For images and screenshots, extraction delegates to the MiniMax MCP tool (`mcp__MiniMax__understand_image`).
5. **Staging & Vault Write**: Generated Markdown files land in `~/.clipper/staging/` before moving to the Obsidian vault directory.
6. **Indexing & Serving**: `index.db` (SQLite via `rusqlite` with FTS5) indexes metadata; local HTTP server serves the dashboard at `/ui`.

---

## 5. Data model (markdown frontmatter schema + sqlite tables)

### Markdown Frontmatter Schema
Each captured clip is persisted into the Obsidian vault as a standalone `.md` file containing YAML frontmatter:

```yaml
---
id: "clip-20261015-120000-a1b2"
title: "Article or Clip Title"
source_url: "https://example.com/post"
capture_type: "url" # url | screenshot | photo | text
captured_at: "2026-10-15T12:00:00Z"
tags:
  - engineering
  - rust
status: "inbox" # inbox | library | archived
summary: "Single paragraph core summary of the clip content."
---
```

The document body below the frontmatter contains structured Markdown:
- Key Takeaways (bullet points)
- Extracted Content / Image Caption
- Original Context or Source Notes

### SQLite Tables (`index.db`)
Stored in `~/.clipper/index.db` using `rusqlite` with FTS5 full-text indexing enabled:

```sql
CREATE TABLE IF NOT EXISTS clips (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    source_url TEXT,
    capture_type TEXT NOT NULL CHECK(capture_type IN ('url', 'screenshot', 'photo', 'text')),
    captured_at TEXT NOT NULL,
    file_path TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'inbox' CHECK(status IN ('inbox', 'library', 'archived')),
    summary TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS clip_tags (
    clip_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (clip_id, tag),
    FOREIGN KEY (clip_id) REFERENCES clips(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
    id UNINDEXED,
    title,
    summary,
    content,
    tokenize = 'porter unicode61'
);
```

---

## 6. Capture modes (URL, screenshot, photo, text)

1. **URL Capture**:
   - Triggered via global desktop hotkey `Ctrl+Shift+C` or browser extension.
   - Tauri reads the clipboard; if text matches a valid URL pattern, an entry is appended to `inbox.jsonl`.
2. **Screenshot Capture**:
   - Triggered via `Win+Shift+S` or desktop clip hotkey.
   - Image binary is written to temporary storage; item references image path in `inbox.jsonl`.
   - Processed through `claude -p` leveraging `mcp__MiniMax__understand_image` for OCR and visual captioning.
3. **Photo Capture**:
   - Captured on mobile devices via the lightweight phone PWA.
   - PWA uploads image to Tauri desktop receiver over local LAN via `POST /api/upload`.
   - Appended to queue and processed through MiniMax MCP vision pipeline.
4. **Text Capture**:
   - Highlighted text or direct clipboard strings that are not raw URLs.
   - Sent to `inbox.jsonl` with `capture_type = 'text'` for semantic tagging, title generation, and vault archiving.

---

## 7. Provider config

Configuration file located at `~/.clipper/config.toml`. Claude Code is the sole active provider for v1; alternative provider templates remain commented out.

```toml
[provider]
active = "claude"

[provider.claude]
bin = "claude"
args = ["-p"]
prompt_file = "~/.clipper/prompts/extract.md"
model = "claude-3-7-sonnet"

# [provider.codex]
# bin = "codex"
# args = ["exec"]
# model = "gpt-4o"

# [provider.agy]
# bin = "agy"
# args = ["run"]
# model = "gemini-2.5-pro"

# [provider.opencode]
# bin = "opencode"
# args = ["pipe"]
# model = "local-llama"

[storage]
vault_path = "auto"
db_path = "~/.clipper/index.db"

[server]
host = "127.0.0.1"
port = 7137
```

---

## 8. Prompt template

The prompt below is deployed to `~/.clipper/prompts/extract.md` and passed directly to `claude -p` when draining `inbox.jsonl`:

```markdown
You are Clipper's extraction engine. You convert raw captured items into structured Markdown notes for an Obsidian vault.

INPUT:
You will receive a JSON payload with `capture_type`, `content`, `source_url`, and optional `image_path`.
- If an image path is present, use tool `mcp__MiniMax__understand_image` to inspect visual elements, OCR text, and descriptive details.
- If a URL is present, extract key points, main subject, and context.

OUTPUT REQUIREMENTS:
Output ONLY valid Markdown with strict YAML frontmatter. No surrounding chat commentary or conversational markers.

Required YAML Frontmatter:
---
id: "<YYYYMMDD-HHMMSS-RANDOM>"
title: "<Concise, descriptive title (under 80 characters)>"
source_url: "<URL or empty>"
capture_type: "<url|screenshot|photo|text>"
captured_at: "<ISO-8601 timestamp>"
tags:
  - <tag1>
  - <tag2>
status: "inbox"
summary: "<1-2 sentence core overview>"
---

Body Format:
# <Title>

## Key Takeaways
- <Bullet point 1>
- <Bullet point 2>
- <Bullet point 3>

## Content & Notes
<Formatted body text, transcribed notes, or image descriptions>

## References
- Source: <source_url or "Local Capture">
```

---

## 9. Dashboard wireframe (ASCII)

Embedded web dashboard served by the Tauri binary at `http://127.0.0.1:7137/ui` using HTML, Alpine.js, and Pico.css:

```
+-------------------------------------------------------------------------------+
|  CLIPPER  [Inbox (12)]  [Library]  [Today (4)]           [QR Pair] [Settings] |
+-------------------------------------------------------------------------------+
|  Filter: [ All Types v ]   Search: [ Search clips (FTS5)...        ] [Refresh]|
+-------------------------------------------------------------------------------+
|  CLIP LIST (Inbox / Today / Library)        |  CLIP DETAIL PREVIEW            |
|                                            |                                 |
|  * [URL] Building Resilient Agents         |  ### Building Resilient Agents  |
|    tags: #agents #architecture             |  URL: https://example.com/post  |
|    captured: 10 mins ago                   |  Status: [Move to Library]      |
|                                            |  Tags: #agents #architecture    |
|  * [IMG] System Architecture Diagram       |                                 |
|    tags: #design #tauri                    |  Summary:                       |
|    captured: 1 hour ago                    |  Analysis of multi-agent hives  |
|                                            |  and file-based coordination.   |
|  * [TXT] Rust rusqlite snippet             |                                 |
|    tags: #rust #database                   |  Key Takeaways:                 |
|    captured: Today 09:15                   |  - Single writer pattern        |
|                                            |  - FTS5 for fast recall         |
|  * [PHO] Whiteboard Sprint Plan            |                                 |
|    tags: #meeting #notes                   |  [Open in Obsidian] [Delete]    |
+-------------------------------------------------------------------------------+
|  Status: Daemon running on :7137 | SQLite indexed | Vault: ~/Obsidian/Vault    |
+-------------------------------------------------------------------------------+
```

---

## 10. Build slices 1-5 with acceptance criteria per slice

### Slice 1: Core Daemon & Ingestion Pipeline
- **Deliverable**: Tauri scaffold + system tray + global hotkey `Ctrl+Shift+C` + clipboard URL reader + JSONL queue + Claude Code drainer + vault writer.
- **Acceptance Criteria**: Working `clipper.exe` runs in background; pressing `Ctrl+Shift+C` with a URL in clipboard appends to `inbox.jsonl`, invokes `claude -p`, and outputs 5 valid `.md` files in the Obsidian vault.

### Slice 2: Web Dashboard & Indexing
- **Deliverable**: Web dashboard (Inbox / Library / Today panes) + SQLite index (`rusqlite` + FTS5) + QR pair endpoint.
- **Acceptance Criteria**: Open dashboard from tray icon menu at `http://127.0.0.1:7137/ui`, view saved clips, perform full-text search across notes, and filter by tag or capture type.

### Slice 3: Phone PWA & LAN Pairing
- **Deliverable**: Phone Progressive Web App (3 HTML files) + LAN pair flow.
- **Acceptance Criteria**: Scan QR code on desktop dashboard from mobile phone browser, open PWA over LAN, submit URL or photo, and observe processed note appear in desktop vault and dashboard.

### Slice 4: Screenshot Capture & Vision Path
- **Deliverable**: Screenshot capture (`Win+Shift+S` hook) + MiniMax MCP vision pipeline (`mcp__MiniMax__understand_image`).
- **Acceptance Criteria**: Trigger screenshot capture, store image asset, forward to MiniMax MCP vision tool, and produce synthesized Markdown note with image caption and tags in vault.

### Slice 5: Browser Extension
- **Deliverable**: Manifest V3 browser extension with one-click share button.
- **Acceptance Criteria**: Clicking extension icon or right-click context menu on any web page sends active URL and page title to desktop port 7137 and successfully queues clip.

---

## 11. Open questions

1. `[TBD — needs human]`: **Obsidian Vault Auto-Detection**
   - Determination of fallback logic if Obsidian default configuration (`%APPDATA%\obsidian\obsidian.json`) is not found or multiple vaults exist.
2. `[TBD — needs human]`: **Queue Drain Strategy**
   - Whether queue drain triggers immediately on hotkey press (debounced), periodically via timer interval (e.g. every 60s), or via user manual trigger on dashboard.
3. `[TBD — needs human]`: **LAN Pairing Authentication**
   - Whether the QR code should encode a temporary bearer token for the mobile PWA session or permit open LAN writes on port 7137.
4. `[TBD — needs human]`: **Image Binary Storage Location**
   - Whether binary assets (screenshots, photos) should be stored in an `assets/` subfolder inside the Obsidian vault or kept privately in `~/.clipper/assets/`.
