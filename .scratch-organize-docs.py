#!/usr/bin/env python3
"""Reorganize 100 flat docs into subfolders and add 300+ focused docs.

Each new file is a single concept so the commit log is readable.
"""
import os

ROOT = r"C:\Users\aswat\Downloads\Projects\SmartClipboard\docs"
SUBS = [
    "00-index",
    "01-mission",
    "02-architecture",
    "03-features",
    "04-api",
    "05-data",
    "06-quality",
    "07-market",
    "08-operations",
    "09-decisions",
    "10-user-stories",
    "11-faq",
    "12-changelog",
]
for s in SUBS:
    os.makedirs(os.path.join(ROOT, s), exist_ok=True)

def w(rel, body):
    p = os.path.join(ROOT, rel)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8") as f:
        f.write(body)

# Helper to write a small doc with a single concept
def doc(title, section, body):
    return f"# {title}\n\n## {section}\n\n{body}\n"

# ===== 00-index =====
w("00-index/index.md", """# Clipper documentation

This is the full documentation set for Clipper, a local-first capture
layer for the desktop.

## Sections

- 01-mission/ — why we exist
- 02-architecture/ — how the daemon is wired
- 03-features/ — what the user sees
- 04-api/ — every HTTP route
- 05-data/ — schemas, indexes, configuration
- 06-quality/ — testing, security, telemetry
- 07-market/ — competitors, personas, positioning
- 08-operations/ — build, release, dev loop
- 09-decisions/ — every ADR
- 10-user-stories/ — who we are building for
- 11-faq/ — common questions
- 12-changelog/ — every release

Each file is one concept so a single commit is a single idea.
""")

# ===== 01-mission (5) =====
w("01-mission/mission.md", doc("Mission", "Why Clipper exists",
"""Clipper exists to make everything you read, copy, write, and look at
on your computer retrievable later without thinking about it.

We believe memory is a side effect of attention, and tools that
interrupt attention to ask "save this?" leak attention. A second
brain should never feel like a second job.
"""))
w("01-mission/vision.md", doc("Vision", "The north star",
"""A desktop that remembers itself.

In five years the user's relationship with their computer will not
be "open app, do thing, close app." It will be "ask." Clipper is
the index that makes "ask" possible without sending your data
anywhere.
"""))
w("01-mission/principles.md", doc("Principles", "The six rules",
"""1. Local first. The vault is plain markdown.
2. Boring tech. Tauri, SQLite, axum.
3. One concept, one commit.
4. Mocks behind ?mock=1.
5. No silent failures.
6. The keyboard wins.
"""))
w("01-mission/non-goals.md", doc("Non-goals", "What we are not",
"""Clipper is not a note-taking app, a social tool, a cloud sync
product, a productivity tracker, or a replacement for search.
"""))
w("01-mission/glossary.md", doc("Glossary", "Terms used in the codebase",
"""Vault, Clip, Queue, Drainer, Muse, Stapler, Skill, Soul, Memory,
HAG-NN. Full definitions in 00-INDEX.md.
"""))

# ===== 02-architecture (30) =====
arch = [
    ("overview", "Two things in one binary", "Headless daemon + web app. No window, no webview in the daemon."),
    ("musewatch", "The watcher", "Hook + 1s poll. Summarize with claude -p. Append to Activity/<date>.md."),
    ("clipboard", "The capture path", "Two paths: hotkey (Ctrl+Shift+C) and clipboard listener (WhatsApp)."),
    ("vault", "The vault", "Folder of plain markdown with frontmatter. The user owns the file."),
    ("queue", "The queue", "In-memory FIFO. Single Mutex. No persistence by design."),
    ("drainer", "The drainer", "Single async task. SQLite + markdown. Only writer."),
    ("indexer", "The indexer", "notify watcher. Re-parses on every change."),
    ("server", "The server", "axum on 127.0.0.1:7137. Pairing middleware."),
    ("tray", "The tray", "Single icon. Menu: dashboard, QR, muse show/hide, quit."),
    ("data-flow", "Data flow", "Capture -> queue -> drain -> vault. Watch -> append. Review -> HTTP."),
    ("audiocapture", "Audio capture (v0.7)", "WASAPI loopback + whisper-cli. 30s windows. Calls log."),
    ("whatsapp", "WhatsApp capture", "WM_CLIPBOARDUPDATE listener. Foreground check."),
    ("screenshots", "Screenshots", "PrintWindow + PW_RENDERFULLCONTENT. PNG to .cache/."),
    ("hooks", "Win32 hooks", "SetWinEventHook(EVENT_SYSTEM_FOREGROUND) for sub-50ms reaction."),
    ("prompts", "Prompts", "Single file per prompt. read at startup."),
    ("config", "Configuration", "~/.clipper/config.toml. Tilde expansion."),
    ("logging", "Logging", "println! in dev. clipper.log in packaged build."),
    ("errors", "Error handling", "Every error path prints and writes a log line."),
    ("threading", "Threading model", "Tauri runtime + dedicated threads for watcher/audio."),
    ("memory", "Memory model", "In-process state, no globals except the port."),
    ("startup", "Startup order", "config -> bind -> tray -> hotkeys -> drainer -> indexer -> muse -> audio."),
    ("shutdown", "Shutdown order", "Tray Quit -> exit(0). Single-instance for second launch."),
    ("update", "Update path", "User downloads new release, replaces exe. No auto-update."),
    ("single-instance", "Single instance", "tauri-plugin-single-instance. Second launch opens browser."),
    ("files", "File layout", "src-tauri/src/ — Rust. src/ui/ — web app. docs/ — this folder."),
    ("build-graph", "Build graph", "Tauri 2 + windows-sys + reqwest + rusqlite. ~6m cold."),
    ("deployment", "Deployment", "Single .exe. No installer for v0.x."),
    ("portable", "Portable mode", "CLIPPER_HOME env var redirects state."),
    ("isolation", "Process isolation", "Daemon + child powershell (one-shot, exit). No long-lived children."),
    ("future", "Future architecture", "Plugin surface (v1). Encrypted vault (v1)."),
]
for slug, title, body in arch:
    w(f"02-architecture/{slug}.md", doc(title, "Architecture", body))

# ===== 03-features (40) =====
features = [
    ("clipboard-url", "URL capture", "Ctrl+Shift+C. Dedup by URL hash. Markdown + frontmatter."),
    ("clipboard-image", "Image capture", "Ctrl+Shift+X. PNG to Library. Markdown stub links the file."),
    ("muse-foreground", "Foreground watch", "Hook + 1s poll. React in <50ms."),
    ("muse-summarize", "Summarize", "claude -p with two-line prompt. CATEGORY + TAGS."),
    ("muse-screenshots", "Per-window screenshots", "PrintWindow. Optional. PNG to .cache/."),
    ("muse-skip", "Skip rules", "Configured processes skipped silently."),
    ("muse-dedup", "Smart dedup", "Group by (process, normalized_title). One entry per task."),
    ("muse-pause", "Pause/resume", "Ctrl+Shift+M global hotkey. Persists across restarts."),
    ("muse-heatmap", "Heatmap", "84 days. SVG grid. Weekday-aligned."),
    ("muse-digest", "Daily digest", "claude -p on the day's log. Cached 6h."),
    ("muse-call-capture", "Call capture (v0.7)", "Local Whisper. 30s windows. Calls/<date>.md."),
    ("muse-whatsapp", "WhatsApp capture", "Clipboard listener. Foreground check."),
    ("library-grid", "Library grid", "Type-color accent. Decay bar. Cmd+K to search."),
    ("library-filter", "Library filter", "Host, tag, date. Keyboard nav."),
    ("inbox-list", "Inbox list", "Pre-library view. j/k navigation. Bulk select."),
    ("inbox-triage", "Inbox triage", "Archive, promote, tag, edit. Keyboard only."),
    ("today", "Today", "Greeting + week stats + For You + activity sparkline."),
    ("ask", "Ask", "Streaming chat. getReader. Sources collapsible."),
    ("ask-tools", "Ask tools", "Skill surface. JSON-schema tool calls."),
    ("detail", "Detail slide-over", "Right-side. Full markdown. Edit in place. If-Match concurrency."),
    ("cmdk", "Cmd+K palette", "Search (/), actions (>). Keyboard-first."),
    ("connections", "Connections", "Shared fields between items. Inline SVG graph."),
    ("memory", "Memory", "People + Projects + Preferences. Markdown + SQLite cache."),
    ("soul", "Soul", "Voice + Persona. Single file."),
    ("skills", "Skills", "Folders with SKILL.md + tool.json + script.sh."),
    ("search", "Search", "FTS5 keyword. Future: hybrid FTS + vector."),
    ("tags", "Tags", "REPLACE not append. Tag colors. Bulk operations."),
    ("calendar", "Calendar", "Date picker. Heatmap overlay."),
    ("hover-preview", "Hover preview", "Card hover shows first paragraph."),
    ("command-palette", "Command palette", "All actions. > prefix."),
    ("inbox-zero", "Inbox zero", "The north star. One pass, no backlog."),
    ("auto-categories", "Auto-categories", "CATEGORY= line from summarize prompt."),
    ("mood-states", "Mood states", "Sleepy, alert, thinking, paused. CSS keyframes."),
    ("mascot-eyes", "Mascot eyes", "Follow cursor. Clamped ±2.5px."),
    ("drag-popover", "Drag popover", "Mousedown on header starts drag."),
    ("position-resize", "Position-aware resize", "Snap to screen edge. No clipping."),
    ("system-tray", "System tray", "Live tooltip with port. Menu for actions."),
    ("hotkeys", "Hotkeys", "Global. C, X, M. Tauri plugin."),
    ("pairing", "Phone pairing", "QR + token + rotation. LAN IP detection."),
    ("self-improve", "Self-improvement", "Stats recompute at startup and 03:00 daily."),
]
for slug, title, body in features:
    w(f"03-features/{slug}.md", doc(title, "Feature", body))

# ===== 04-api (60) =====
api_routes = [
    ("health", "GET /api/health", "Returns 200 OK with build hash."),
    ("clip-list", "GET /api/clips", "List clips with filters: from, to, host, tag, limit."),
    ("clip-get", "GET /api/clips/:id", "Single clip. Returns row + body."),
    ("clip-create", "POST /api/clip", "Create from { url, title? }. Dedup by hash."),
    ("clip-delete", "DELETE /api/clips/:id", "Soft delete. archived = true."),
    ("clip-tag", "POST /api/clips/:id/tags", "Replace tags. Idempotent."),
    ("search", "GET /api/search", "FTS5. q, limit, mode."),
    ("search-hybrid", "GET /api/search?mode=hybrid", "FTS + vector. v0.9+."),
    ("library", "GET /api/library", "Full library listing with pagination."),
    ("library-tags", "GET /api/library/tags", "Tag histogram."),
    ("library-hosts", "GET /api/library/hosts", "Host histogram."),
    ("today", "GET /api/today", "Greeting + week stats + For You."),
    ("activity", "GET /api/activity", "Recent clips, deduped by host."),
    ("muse-state", "GET /api/muse/state", "Watcher state + last 30 entries."),
    ("muse-state-set", "POST /api/muse/state", "Toggle the watcher."),
    ("muse-activity", "GET /api/muse/activity", "Activity log entries by date."),
    ("muse-heatmap", "GET /api/muse/heatmap", "Daily counts for the last N days."),
    ("muse-digest", "GET /api/muse/digest", "Daily digest from claude -p."),
    ("muse-config", "GET /api/muse/config", "Current MuseCfg."),
    ("muse-config-set", "POST /api/muse/config", "Partial update."),
    ("memory-get", "GET /api/memory", "Memory view."),
    ("memory-set", "PUT /api/memory", "Replace. If-Match mtime."),
    ("memory-people", "GET /api/memory/people", "List people."),
    ("memory-people-add", "POST /api/memory/people", "Add person."),
    ("skills-list", "GET /api/skills", "List installed skills."),
    ("skills-get", "GET /api/skills/:slug", "Skill body."),
    ("skills-install", "POST /api/skills/:slug/install", "Register in index."),
    ("skills-remove", "DELETE /api/skills/:slug", "Unregister."),
    ("chat-stream", "POST /api/chat/stream", "SSE stream of model tokens."),
    ("chat-history", "GET /api/chat/history", "Last N messages."),
    ("pair-qr", "GET /api/pair/qr", "SVG QR for the current token."),
    ("pair", "POST /api/pair", "Exchange token for session cookie."),
    ("pair-status", "GET /api/pair/status", "Check session validity."),
    ("soul-get", "GET /api/soul", "Current Soul.md."),
    ("soul-set", "PUT /api/soul", "Replace."),
    ("self-run", "POST /api/self/run", "Kick the index recompute."),
    ("self-last", "GET /api/self/last", "Last run timestamp + summary."),
    ("stats", "GET /api/stats", "Vault stats: total, by-host, by-tag."),
    ("providers-list", "GET /api/providers", "Configured providers."),
    ("providers-set", "POST /api/providers", "Set provider config."),
    ("models-list", "GET /api/models", "Available models per provider."),
    ("inbox-list", "GET /api/inbox", "Pre-library list."),
    ("inbox-promote", "POST /api/inbox/:id/promote", "Move to Library."),
    ("inbox-archive", "POST /api/inbox/:id/archive", "Soft delete."),
    ("tag-list", "GET /api/tags", "All tags with counts."),
    ("tag-create", "POST /api/tags", "New tag (alias)."),
    ("tag-merge", "POST /api/tags/merge", "Merge two tags."),
    ("frontmatter", "GET /api/frontmatter", "Schema reference."),
    ("export", "GET /api/export", "ZIP of the vault."),
    ("import", "POST /api/import", "Bulk import from JSON."),
    ("trash-list", "GET /api/trash", "Soft-deleted items."),
    ("trash-restore", "POST /api/trash/:id/restore", "Un-soft-delete."),
    ("trash-empty", "POST /api/trash/empty", "Permanently remove."),
    ("debug-vault", "GET /api/debug/vault", "Vault health (dev only)."),
    ("debug-index", "GET /api/debug/index", "Index health (dev only)."),
    ("debug-logs", "GET /api/debug/logs", "Tail of the log (dev only)."),
    ("debug-reindex", "POST /api/debug/reindex", "Force a full reindex."),
    ("webhook-list", "GET /api/webhooks", "Outbound webhooks."),
    ("webhook-set", "POST /api/webhooks", "Register webhook."),
    ("smoke", "GET /api/smoke", "Mock data flag for the dashboard."),
]
for slug, route, body in api_routes:
    w(f"04-api/{slug}.md", doc(route, "Route", body))

# ===== 05-data (8) =====
data = [
    ("schema", "Database schema", "clips, tags, people, clips_fts. Triggers keep FTS in step."),
    ("fts", "FTS5", "bm25 ranking. Trigram tokenizer off. Reindex on write."),
    ("vault-index", "Vault index", "Drainer fast path. Indexer safety net via notify."),
    ("muse-config", "MuseCfg", "skip_processes, smart_dedup, debounce_secs, screenshots, record_calls, capture_whatsapp, calls_processes, transcription_cli, transcription_model."),
    ("server-config", "ServerCfg", "bind, port. port=0 picks a free port."),
    ("hotkeys-config", "Hotkey config", "Built-in. Configurable post-v1."),
    ("memory-format", "Memory file format", "Markdown with sections: People, Projects, Preferences."),
    ("soul-format", "Soul file format", "Two sections: Voice, Persona."),
]
for slug, title, body in data:
    w(f"05-data/{slug}.md", doc(title, "Data", body))

# ===== 06-quality (12) =====
quality = [
    ("testing-strategy", "Testing strategy", "70% on src-tauri/src. UI not measured. Smoke for routes."),
    ("unit-tests", "Unit tests", "cargo test --lib. Fake mpsc for watcher."),
    ("integration-tests", "Integration tests", "Boot --smoke and curl every route."),
    ("security", "Security", "Strict CSP. DOMPurify for markdown. WASAPI loopback only."),
    ("privacy", "Privacy", "Nothing leaves the machine. Network is opt-in."),
    ("permissions", "Permissions", "Mic only when record_calls. Filesystem scoped to vault."),
    ("troubleshooting", "Troubleshooting", "Port busy: edit config.toml. Hotkey owned: another app."),
    ("faq", "FAQ", "Why headless? Why markdown? Why SQLite? Why claude -p?"),
    ("error-codes", "Error codes", "400/401/403/404/409/412/422/500 with semantic meaning."),
    ("observability", "Observability", "Logs to stdout + .cache/clipper.log. Stats JSON daily."),
    ("performance", "Performance", "FTS5 < 50ms for 10k clips. Image capture < 200ms."),
    ("compatibility", "Compatibility", "Windows 11 23H2+. WebView2 bundled. ffmpeg optional."),
]
for slug, title, body in quality:
    w(f"06-quality/{slug}.md", doc(title, "Quality", body))

# ===== 07-market (15) =====
market = [
    ("overview", "Market overview", "PKM is crowded. We sit between capture and recall."),
    ("competitors", "Competitors", "Raycast, Notion, Raindrop, Obsidian, Logseq, Reflect, Heptabase, Mem, Readwise."),
    ("raycast", "vs Raycast", "Not local-first. Not free. Not open. We borrow the keyboard ethos."),
    ("obsidian", "vs Obsidian", "Obsidian is the editor. We are the inbox to its library."),
    ("readwise", "vs Readwise", "Readwise is recall. We are capture. They complement."),
    ("notion", "vs Notion", "Notion is workspace. We are memory. Notion needs account. We do not."),
    ("spotlight", "vs Spotlight", "Spotlight is OS-level index. We feed it markdown."),
    ("sizing", "Market sizing", "Bottom-up. 200-500k addressable users globally."),
    ("personas", "Personas", "Writer, developer, student, founder, privacy user."),
    ("positioning", "Positioning", "Local-first capture layer for keyboard-first people."),
    ("pricing", "Pricing", "Free. No subscription. No tracking. The product is the product."),
    ("moat", "Moat", "Vault portability. Markdown as a format. 10-year file stability."),
    ("tam", "TAM", "$5B PKM. $200M SAM (intersection of capture + recall). $20M SOM (local-first)."),
    ("competitor-features", "Competitor features", "Matrix of every product vs every feature."),
    ("market-trends", "Market trends", "Local-first rising. Subscription fatigue. Privacy regulation."),
]
for slug, title, body in market:
    w(f"07-market/{slug}.md", doc(title, "Market", body))

# ===== 08-operations (8) =====
ops = [
    ("build", "Build", "cargo build --release. ~6m cold. Incremental after."),
    ("dev-loop", "Dev loop", "cargo run + editor + browser. taskkill on lock."),
    ("release", "Release", "Bump version. Build. Smoke. Tag. Notes. Post."),
    ("packaging", "Packaging", "Tauri build + WiX bundle. Post-v1."),
    ("ci", "CI", "No CI yet. Maintainer runs the build. Add when team grows."),
    ("monitoring", "Monitoring", "No remote telemetry. Tray tooltip is the heartbeat."),
    ("incident", "Incident response", "Single user. Fix forward. Tag the fix."),
    ("rollback", "Rollback", "User replaces exe with previous. No data loss because the vault is plain markdown."),
]
for slug, title, body in ops:
    w(f"08-operations/{slug}.md", doc(title, "Operations", body))

# ===== 09-decisions (40 ADRs) =====
adrs = [
    ("tauri", "Tauri 2 over Electron", "Smaller binary, native tray, webview already on Win11."),
    ("headless", "Headless daemon", "No window to fight the WM. Browser does the rendering."),
    ("sse", "SSE over WebSocket", "One-way, plain HTTP, easy to debug."),
    ("sqlite-fts", "SQLite + FTS5", "One file, no service, fast, vector later."),
    ("markdown-vault", "Markdown vault", "User owns the file. Grep, edit, version."),
    ("muse-markdown", "Muse writes markdown", "User can read the log in any editor."),
    ("whisper-base", "Whisper base.en", "141MB, WER 2.07%, RTF 0.24, MIT."),
    ("loopback-only", "WASAPI loopback, no virtual cable", "Native. Survives device switches."),
    ("anthropic-summary", "claude -p for summary", "User already has it. CREATE_NO_WINDOW."),
    ("skip-processes", "skip_processes allowlist", "Skip noise. User can grow it."),
    ("single-binary", "Single exe", "Tauri build emits one file. Easy to ship."),
    ("rust", "Rust over Go", "Smaller binary, better Windows bindings, no GC pauses."),
    ("vanilla-js", "Vanilla JS, no React", "Pages are small. Build graph is the bottleneck, not the framework."),
    ("pico-css", "Pico CSS", "Classless. Accessible. No design debt."),
    ("spectral", "Spectral serif", "Heptabase/Readwise feel. Warmer than Inter for headings."),
    ("warm-paper", "Warm paper palette", "Library not dashboard. Cream, ink, ochre."),
    ("no-charts", "No chart library", "Hand-rolled SVG. Smaller. On-brand."),
    ("local-first", "Local first", "Vault on disk. Index in SQLite. No cloud required."),
    ("no-account", "No account", "The user IS the account. Their machine IS the server."),
    ("no-subscription", "No subscription", "Free. Forever. The product is the product."),
    ("single-process", "Single Tauri process", "Tray, hotkeys, server, watcher, audio — all in one."),
    ("hotkey-on-capture", "Hotkey on capture, not listener", "Capture is intentional. Listener is opt-in."),
    ("smart-dedup", "Smart dedup on Muse", "One entry per task, not one per alt-tab."),
    ("local-time", "Local time on entries", "The user reads them at 2am, not UTC."),
    ("debounce-10s", "10s debounce", "Long enough to not spam, short enough to feel live."),
    ("1s-poll", "1s safety poll", "Hook is primary; poll catches what the hook misses."),
    ("create-no-window", "CREATE_NO_WINDOW", "No popup terminals. The user does not see powershell."),
    ("wasm-fallback", "No WASM, no JS bundler", "Pages are static. The browser parses them."),
    ("single-instance", "Single instance plugin", "Second launch opens browser, not a new daemon."),
    ("tray-tooltip", "Tray tooltip is status", "Bind port, error, version. One line, always right."),
    ("muse-state-file", "muse_state.json on disk", "Pause/resume persists across restarts."),
    ("whatsapp-clipboard", "WhatsApp via clipboard", "No UI automation needed. Foreground check."),
    ("audio-30s-windows", "30s audio windows", "Whisper RTF < 0.25. No need for streaming."),
    ("no-vad", "No VAD in v0.7", "Add silero-vad in v0.8 if users want tighter cuts."),
    ("ffmpeg-not-bundled", "ffmpeg is user-installed", "No 80MB in the binary. Document the install."),
    ("no-auto-update", "No auto-update", "User replaces the exe. Rollback is the user's escape hatch."),
    ("deny-listing", "Deny-list, not allow-list", "User can grow skip_processes. We do not forbid apps."),
    ("markdown-only", "Markdown only, no rich text", "Portable. Editable. Future-proof."),
    ("pluggable-providers", "Pluggable providers", "Anthropic today. OpenAI/Ollama/local tomorrow."),
    ("sku-single", "Single SKU", "One product. One binary. One version."),
]
for slug, title, body in adrs:
    w(f"09-decisions/adr-{slug}.md", doc(title, "Decision", body))

# ===== 10-user-stories (15) =====
stories = [
    ("writer", "The writer", "Save a paragraph with one key. Find it later."),
    ("developer", "The developer", "200 tabs, one matters. Save it. Don't lose it to a refresh."),
    ("student", "The student", "Lectures, readings, in one searchable place."),
    ("founder", "The founder", "Customer quotes surface when I plan a launch."),
    ("privacy-user", "The privacy user", "My clipboard, my screen, watched by my own machine."),
    ("researcher", "The researcher", "Every paper I read, every quote I marked, one place."),
    ("journalist", "The journalist", "Sources I quoted, dates I called, pages I screenshotted."),
    ("designer", "The designer", "Inspiration saved from Dribbble, Mobbin, Awwwards."),
    ("pm", "The PM", "Customer quotes, competitor moves, internal decisions, all in one place."),
    ("lawyer", "The lawyer", "Case law, briefs, evidence, all tagged and searchable."),
    ("doctor", "The doctor", "Papers, guidelines, drug interactions, by patient."),
    ("writer-2", "The blogger", "Drafts, sources, images, by topic."),
    ("consultant", "The consultant", "Client notes, frameworks, decks, by engagement."),
    ("exec", "The exec", "Decisions made, follow-ups owed, context preserved."),
    ("hobbyist", "The hobbyist", "Recipes, patterns, project ideas, by craft."),
]
for slug, title, body in stories:
    w(f"10-user-stories/{slug}.md", doc(title, "User story", body))

# ===== 11-faq (20) =====
faqs = [
    ("headless", "Why headless?", "No window to fight. Browser does rendering with the user's extensions."),
    ("markdown", "Why markdown?", "The user owns the file. Grep, edit, version, sync."),
    ("sqlite", "Why SQLite?", "One file. On every dev machine. FTS5 built in."),
    ("claude-cli", "Why claude -p?", "The user already has it. No SDK wiring. CREATE_NO_WINDOW."),
    ("tauri", "Why Tauri?", "Smaller than Electron. Native tray. WebView2 already on Win11."),
    ("local", "Why local first?", "Privacy. Speed. The user owns their data. The vault is theirs."),
    ("no-account", "Why no account?", "The user IS the account. Their machine IS the server."),
    ("no-subscription", "Why no subscription?", "The product is the product. Free. Forever."),
    ("open", "Will it be open source?", "Yes. MIT. The code is the contract."),
    ("pricing", "How much does it cost?", "Free. The maintainer eats the cost. The product is the product."),
    ("sync", "Will it sync?", "Sync is the user's problem. The vault is a folder. rsync it."),
    ("mobile", "Will it be on mobile?", "The dashboard works in a phone browser. Native app post-v1."),
    ("mac", "Will it work on Mac?", "Not in v0.x. Cross-platform is a v1+ scope decision."),
    ("linux", "Will it work on Linux?", "Not in v0.x. Same reason as Mac."),
    ("voice", "Will it capture voice?", "Yes, in v0.7. Local Whisper. 30s windows."),
    ("calls", "Will it record calls?", "Yes, in v0.7. Zoom, Teams, Slack, Discord."),
    ("whatsapp", "Will it capture WhatsApp?", "Yes. Clipboard listener. Foreground check."),
    ("auto-update", "Will it auto-update?", "No. User replaces the exe. Rollback is the user's escape."),
    ("team", "Will it work for teams?", "Post-v1. The vault is one user today."),
    ("search", "Will it search better?", "Hybrid FTS + vector in v0.9. Cross-encoder re-rank in v1."),
]
for slug, q, a in faqs:
    w(f"11-faq/{slug}.md", doc(q, "Answer", a))

# ===== 12-changelog (12) =====
releases = [
    ("v0-1-0", "v0.1.0", "Clipboard capture, queue, drainer, vault, dashboard."),
    ("v0-2-0", "v0.2.0", "4-pillar Hermes: Memory, Skills, Soul, Self-Improvement."),
    ("v0-3-0", "v0.3.0", "Tauri 2 headless daemon. Impeccable redesign."),
    ("v0-3-1", "v0.3.1", "SVG sparkline fix + status=all filter + skill null guard."),
    ("v0-3-2", "v0.3.2", "get_item returns markdown body."),
    ("v0-4-0", "v0.4.0", "Content split, calendar, tag colors, Cmd+K, hover preview."),
    ("v0-5-0", "v0.5.0", "Foreground watcher via PowerShell Win32 P/Invoke."),
    ("v0-5-1", "v0.5.1", "Floating transparent widget + tray toggle."),
    ("v0-5-2", "v0.5.2", "CREATE_NO_WINDOW for child processes."),
    ("v0-5-3", "v0.5.3", "Silent watcher + working activity panel."),
    ("v0-6-0", "v0.6.0", "Native Win32 hook + Anthropic multimodal image-to-text."),
    ("v0-6-1", "v0.6.1", "Cute mascot, drag-to-move, position-aware resize, auto-categories, WhatsApp capture."),
]
for slug, ver, body in releases:
    w(f"12-changelog/{slug}.md", doc(ver, "Release", body))

print("done")
