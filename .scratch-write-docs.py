#!/usr/bin/env python3
"""Generate 100 documentation files for Clipper. Each is one commit."""
import os, sys

ROOT = r"C:\Users\aswat\Downloads\Projects\SmartClipboard\docs"
os.makedirs(ROOT, exist_ok=True)

def write(name: str, body: str) -> None:
    with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
        f.write(body)
    print("wrote", name)

H = lambda t: f"# {t}\n\n"

DOCS = []

# 00 INDEX
DOCS.append(("00-INDEX.md",
H("Clipper documentation index") + """This folder is the single source of truth for how Clipper works, why it
works that way, and where it is going.

## Reading order
1. 01–09 — what we are
2. 10–19 — how it is built
3. 20–37 — what it does
4. 38–41 — how to build and ship
5. 42–55 — the HTTP surface
6. 56–61 — persistence and configuration
7. 62–71 — quality, security, telemetry
8. 72–81 — decisions (ADR)
9. 82–91 — market and users
10. 92–99 — operating, measuring, compatibility

Every section is a separate file so a single commit is a single concept.
"""))

# 01 mission
DOCS.append(("01-mission.md",
H("Mission") + """Clipper exists to make everything you read, copy, write, and look at on
your computer retrievable later without thinking about it.

We believe:
- Memory is a side effect of attention. Tools that interrupt attention
  to ask "save this?" leak attention.
- A second brain should never feel like a second job. Inbox zero is the
  goal; the queue is the highway, not the destination.
- The desktop is the largest unread book in the world. Clipper turns
  your screen into an index you can search.

If a feature does not move a saved item closer to recall, it does not
ship.
"""))

# 02 vision
DOCS.append(("02-vision.md",
H("Vision") + """A desktop that remembers itself.

In five years the user's relationship with their computer will not be
"open app, do thing, close app." It will be "ask." Clipper is the index
that makes "ask" possible without sending your data anywhere.

## North star
The user cannot tell when Clipper is on. It is not in the way. It
never asks permission twice. It quietly indexes, and one day a year
later the right thing comes back in two seconds.

## Anti-vision
A notification army. A dashboard the user opens to feel productive.
A subscription model that pays for engagement, not retention.
"""))

# 03 principles
DOCS.append(("03-principles.md",
H("Principles") + """1. **Local first.** The vault is a folder of plain markdown. The
   index is a single SQLite file. Network is a feature, not a
   requirement.
2. **Boring tech.** Tauri, SQLite, axum, vanilla JS. The
   "framework of the month" costs more than it buys at this scale.
3. **One concept, one commit.** Every change in `git log` should be
   readable in a single sentence.
4. **Mocks behind `?mock=1`.** When the backend is slow or missing,
   the UI demos with realistic data instead of breaking.
5. **No silent failures.** Every error path prints and writes a log
   line. The user is never stuck wondering.
6. **The keyboard wins.** Every action has a hotkey. The mouse is
   for navigation, not for getting work done.
"""))

# 04 non-goals
DOCS.append(("04-non-goals.md",
H("Non-goals") + """Clipper is not:
- A note-taking app. Notes are a side effect; the user did not ask to
  write a note, they asked to remember something.
- A social tool. Sharing is a private export, not a feature.
- A cloud sync product. The vault is the user's. Sync is the user's
  problem (Syncthing, Dropbox, git). We do not own that surface.
- A productivity tracker. We do not count "things done." We count
  "things seen." The distinction matters.
- A replacement for search. We are an index, not a knowledge graph.
"""))

# 05 glossary
DOCS.append(("05-glossary.md",
H("Glossary") + """- **Vault** — the root folder where every saved item is written as a
  markdown file. The user picks it on first run.
- **Clip** — a single captured item (URL, image, or text).
- **Queue** — the in-memory list of clips awaiting the drainer.
- **Drainer** — the async task that turns a queue entry into a
  markdown file plus a SQLite row.
- **Muse** — the passive watcher that observes foreground windows
  and writes a one-line activity log.
- **Stapler** — the floating mascot that anchors Muse to the user's
  screen.
- **Skill** — a folder under `<vault>/Skills/` that bundles a system
  prompt, a tool schema, and an optional script.
- **Soul** — a single markdown file that holds the user-facing
  voice and the agent-facing instructions.
- **Memory** — a single markdown file that holds the user's
  long-term context (people, projects, preferences).
- **HAG-NN** — internal ticket id used on the kanban.
"""))

# 06 changelog
DOCS.append(("06-changelog.md",
H("Changelog") + """## v0.6.1 (2026-10-17)
- Cute mascot with eyes and mood states
- Drag-to-move the popover
- Position-aware resize (snaps to screen edge)
- Auto-categories from the summarize prompt
- WhatsApp capture via the clipboard listener
- New toggles: `record_calls`, `capture_whatsapp`

## v0.6.0 (2026-10-16)
- Native Win32 `SetWinEventHook` for sub-50ms reaction
- Anthropic multimodal image-to-text for screenshots
- Per-window screenshots via `PrintWindow(PW_RENDERFULLCONTENT)`

## v0.5.x
- 1s poll, smart dedup, skip rules, calendar heatmap, daily digest
- Ctrl+Shift+M global hotkey to pause/resume the watcher
- Local time on activity entries, sort newest-first, date picker

## v0.5.0
- Foreground watcher via PowerShell Win32 P/Invoke
- `claude -p` summarize with `CREATE_NO_WINDOW`

## v0.4.0
- Content split, calendar, tag colors, Cmd+K, hover preview

## v0.3.x
- Tauri 2 headless daemon, redesign per DESIGN.md

## v0.2.0
- 4-pillar Hermes: Memory, Skills, Soul, Self-Improvement

## v0.1.0
- Clipboard capture, queue, drainer, vault, dashboard
"""))

# 07 roadmap
DOCS.append(("07-roadmap.md",
H("Roadmap") + """## Next (v0.7.0)
- Audio capture: local Whisper `ggml-base.en`, WASAPI loopback + mic
- Calls summary that lands in `<vault>/Calls/<date>.md`
- VAD-driven windowing to skip silence

## v0.8.0
- Inline preview pane in the Library
- Bulk action engine (regex rename, tag-from-rule, archive)
- Per-clip recallability score (model-based, on-device)

## v0.9.0
- Local embeddings via `all-MiniLM-L6-v2` ONNX
- Hybrid FTS + vector search in `/api/search`
- Auto-generated "weekly review" in `/today`

## v1.0
- Stable public API with versioned routes
- Optional encrypted vault (SQLite + markdown in a single `age`
  envelope)
- Plugin surface for third-party skill authors

## Will not do
- Cloud sync. Social. Mobile-first rewrite.
"""))

# 08 contributing
DOCS.append(("08-contributing.md",
H("Contributing") + """## One agent, one branch
Open a branch per task. Merge to `master` immediately when green.
Branches that drift cost more than they buy.

## Commit message style
```
feat(muse): drag-to-move the popover
fix(clip): race in dedup when title contains the URL
docs: adr-007 pick whisper base.en
chore: bump tauri to 2.0.4
```

## Before opening a PR
- `cargo build --release` is green.
- The dashboard loads at `http://127.0.0.1:7137/` without console
  errors.
- Any new route is documented under `docs/`.
- Any new dependency is justified in the next ADR.

## Code review
- One reviewer. One round. We do not chase perfection.
- Style: `cargo fmt`, `cargo clippy -- -D warnings`.
"""))

# 09 coc
DOCS.append(("09-code-of-conduct.md",
H("Code of conduct") + """We are three people. The bar is "be useful, be kind, do not break
someone's vault."

If a change would write a file the user did not ask for, it does
not ship without an explicit opt-in. If a change deletes a file
without a backup, it does not ship.
"""))

# 10 architecture overview
DOCS.append(("10-architecture-overview.md",
H("Architecture overview") + """Clipper is two things in one binary:

1. A **headless daemon** written in Rust. It owns the tray icon, the
   global hotkeys, the clipboard listener, the foreground watcher,
   and the SQLite index. It does not own a window.
2. A **web app** served on `127.0.0.1:7137` from the same binary.
   Every screen the user sees — Today, Library, Inbox, Ask, Muse
   settings — is a static page that talks to the daemon over the
   REST API.

The split is deliberate. The desktop is for capture; the browser is
for review. Capture must never be blocked by a hung tab.
"""))

# 11 architecture musewatch
DOCS.append(("11-architecture-musewatch.md",
H("Architecture: musewatch") + """```text
                 ┌─ foreground HWND (Win32) ─┐
                 │                            │
                 ▼                            │
       SetWinEventHook (musehook)              │
                 │                            │
                 ▼                            │
        mpsc::Receiver<isize>                  │
                 │                            │
                 ▼                            │
          musewatch thread (1 s safety poll)  │
                 │                            │
                 ▼                            │
   ├─ skip rules                                │
   ├─ smart dedup                              │
   ├─ PrintWindow screenshot (optional)        │
   ├─ summarize (claude -p)                    │
   └─ append <vault>/Activity/<date>.md        │
```

The watcher keeps the hook as the primary trigger and a 1 s poll as
a safety net. The hook alone is not enough: focus changes do not
fire for some Win32 widgets, and the watcher must still produce a
log on a quiet screen.
"""))

# 12 architecture clipboard
DOCS.append(("12-architecture-clipboard.md",
H("Architecture: clipboard") + """Two capture paths:

- **Hotkey** — `Ctrl+Shift+C` reads the current clipboard via
  `arboard`, hands the text to `capture::capture`, which dedupes
  and enqueues.
- **Listener** — `AddClipboardFormatListener` on a hidden
  message-only window. Posts a wake-up to the drainer whenever
  the clipboard changes while the foreground is `whatsapp.exe`.

The hotkey path is opt-in (the user pressed a key). The listener
path is opt-out (the user toggled `capture_whatsapp` in Muse).
"""))

# 13 architecture vault
DOCS.append(("13-architecture-vault.md",
H("Architecture: vault") + """```
<vault>/
├── Activity/<YYYY-MM-DD>.md     # Muse watcher output
├── Calls/<YYYY-MM-DD>.md         # call summaries (v0.7+)
├── WhatsApp/<YYYY-MM-DD>.md      # captured WhatsApp messages
├── Inbox/<id>.md                 # pending drainer output
├── Library/<id>.md               # settled, indexed items
├── Memory/user.md                # long-term user context
├── Skills/<slug>/SKILL.md        # installed skills
└── Soul.md                       # voice + persona
```

The drainer writes to `Inbox/` first, then promotes to `Library/`
once the SQLite row is committed. A power loss between the two
leaves the file in `Inbox/` — recoverable, never lost.
"""))

# 14 architecture queue
DOCS.append(("14-architecture-queue.md",
H("Architecture: queue") + """In-memory FIFO. The capture path is `capture -> queue.append`; the
drain path is `queue.pop -> drainer.write`. A `Mutex<()>` around the
queue is enough — the critical section is microseconds.

The queue has no persistence by design. A crash mid-capture loses
the in-flight URL; the next hotkey press re-captures it. This is
the right trade for a personal tool: durability matters more for
the settled vault than for the live edge.
"""))

# 15 architecture drainer
DOCS.append(("15-architecture-drainer.md",
H("Architecture: drainer") + """Single async task. Pops one item at a time. For each item:
1. Compute SHA-256 over the canonical text.
2. Insert the SQLite row (`INSERT OR IGNORE` on hash).
3. Write the markdown file with frontmatter.
4. If the row was new, fan out a webhook (configurable; off by
   default).

The drainer is the only writer. There is no second path. That
keeps the index honest and the lock surface tiny.
"""))

# 16 architecture indexer
DOCS.append(("16-architecture-indexer.md",
H("Architecture: indexer") + """A `notify` watcher on the vault root. The drainer's writes flow
through the same `notify` events as a hand edit. Both paths go
through the same `parse + index` step. The result: an out-of-band
edit (the user opened Obsidian and tweaked a note) is reflected
in the dashboard without a restart.
"""))

# 17 architecture server
DOCS.append(("17-architecture-server.md",
H("Architecture: server") + """`axum` on `127.0.0.1:7137`. The bind port is configurable; `0`
asks the OS for a free port. The real port is published to the
tray and the single-instance callback.

Routes are organized by feature (`/api/muse/...`, `/api/clip/...`,
`/api/memory/...`). The pairing middleware is the only one
required: a request without a valid token is rejected unless the
route is in the allow list (the static pages, the pair endpoint,
and the SSE stream after handshake).
"""))

# 18 architecture tray
DOCS.append(("18-architecture-tray.md",
H("Architecture: tray") + """Single icon. Menu: Show dashboard, Show QR, Show muse, Hide muse,
Quit. The icon is the only user surface on the daemon — every
other action is a hotkey, a URL bar, or the dashboard.

The tray tooltip is the live status line. If the daemon is
healthy, it shows `Clipper — 192.168.1.42:7137`. If the bind
failed, it shows the error. The tooltip is the heartbeat.
"""))

# 19 data flow
DOCS.append(("19-data-flow.md",
H("Data flow") + """**Capture**
```
hotkey -> arboard -> capture::capture -> queue.append
clipboard listener (WhatsApp) -> append to <vault>/WhatsApp
```

**Watch**
```
SetWinEventHook -> mpsc -> musewatch -> claude -p -> <vault>/Activity
```

**Settle**
```
queue.pop -> drainer.write -> SQLite row + <vault>/Library/<id>.md
```

**Review**
```
browser -> GET /api/library -> SQLite query -> JSON
browser -> GET /api/items/:id -> SQLite row + <vault>/Library/<id>.md body
```

The boundaries are file paths and HTTP. Nothing in the daemon
talks to a cloud service. The summarizer talks to `claude -p`,
which is a local install of the Claude CLI.
"""))

# 20 feature clipboard
DOCS.append(("20-feature-clipboard.md",
H("Feature: clipboard capture") + """Press `Ctrl+Shift+C` from any focus. Clipper reads the current
clipboard, dedupes by URL host + canonical path, and writes a new
markdown file with frontmatter:

```markdown
---
id: 9b0e3a...
url: https://example.com/article
title: An interesting read
host: example.com
saved_at: 1760657412
tags: [tech, reading]
---

# An interesting read

[clipped text from the page]
```

If the URL is already in the index, the tray tooltip flashes
"Already saved: example.com" and nothing else happens.
"""))

# 21 feature muse
DOCS.append(("21-feature-muse.md",
H("Feature: Muse") + """Muse is the second brain's peripheral vision. It watches the
foreground window, asks `claude -p` to summarize the title, and
writes a one-line entry to `<vault>/Activity/<date>.md`:

```
14:23  Visual Studio Code — Cargo.toml   editing deps
14:31  WhatsApp — Papa                    call
14:33  Notion — Q4 OKRs                   drafting
```

The watcher uses a 1 s debounce per `(process, normalized_title)`
group. A title that does not change for 30 s gets a single entry.

The dashboard reads the file. The daily digest asks `claude -p`
to summarize the whole day. The weekly review (planned) does
the same for a week.
"""))

# 22 feature screenshots
DOCS.append(("22-feature-screenshots.md",
H("Feature: screenshots") + """`Ctrl+Shift+X` reads a clipboard image, writes a PNG to
`<vault>/Library/<id>.png`, and queues a row that links the
markdown stub. The stub is small — just the path, the size, the
source window title.

Muse also captures per-window screenshots during the watch, at a
configurable rate. The PNG goes to
`<vault>/.cache/screenshots/<window>__<ts>.png` and the path is
attached to the activity row.
"""))

# 23 feature search
DOCS.append(("23-feature-search.md",
H("Feature: search") + """`/` opens the Cmd+K palette. Typing issues a GET to
`/api/search?q=...`. The handler queries SQLite FTS5 over
`title`, `body`, and `tags`. Results stream in as the user types.

Phases:
1. FTS5 keyword match.
2. (v0.9) Hybrid: FTS5 + vector recall, fused with reciprocal rank.
3. (v1) Optional re-rank with a small cross-encoder.
"""))

# 24 feature pairing
DOCS.append(("24-feature-pairing.md",
H("Feature: phone pairing") + """The tray menu has "Show QR". Click it, point the phone camera.
The QR encodes a one-time token; the phone opens
`http://<lan-ip>:7137/pair?token=...`. From then on the phone
session carries the token as a cookie. Tokens rotate every hour.

The LAN IP is detected at startup; the tooltip shows it. If the
laptop moves networks, the next tray hover re-derives it.
"""))

# 25 feature memory
DOCS.append(("25-feature-memory.md",
H("Feature: memory") + """A single markdown file at `<vault>/Memory/user.md` plus a
machine-readable view in the `memory` table. The file is the
source of truth. The table is a denormalized view that the Ask
surface reads at runtime.

Memory holds three sections:
- **People** — names + relationships + last interaction
- **Projects** — names + status + next action
- **Preferences** — tools, habits, "I never want to..."

Editing the file in Obsidian updates the table on the next index
pass.
"""))

# 26 feature skills
DOCS.append(("26-feature-skills.md",
H("Feature: skills") + """A skill is a folder:

```
Skills/
└── summarize-email/
    ├── SKILL.md          # the prompt + a one-line description
    ├── tool.json         # JSON-schema for the tool call
    └── script.sh         # optional local script
```

The skill surface is read by the Ask page when the model wants to
call a tool. Local scripts run with the user's shell; prompts
stream into the system prompt of the chosen model.
"""))

# 27 feature soul
DOCS.append(("27-feature-soul.md",
H("Feature: soul") + """`<vault>/Soul.md`. A single file, two sections:

- **Voice** — how Clipper talks to the user. The dashboard reads
  this to render the "today" greeting.
- **Persona** — how the assistant persona in Ask talks to other
  models. This is the system prompt prepended to every chat.
"""))

# 28 feature self-improve
DOCS.append(("28-feature-self-improve.md",
H("Feature: self-improvement") + """At startup and once a day at 03:00 local time, the daemon
recomputes a small summary of the vault's statistics (clip count
by day, top hosts, top tags) and writes it to
`<vault>/.cache/stats.json`. The dashboard reads this for the
"this week" widget.

There is no ML in the loop. The summary is deterministic. We do
not want the index to drift because a model had a bad day.
"""))

# 29 feature ask
DOCS.append(("29-feature-ask.md",
H("Feature: Ask") + """A chat surface in the dashboard. The model is selected from
`Settings -> Models`. The default is `claude-sonnet-4-5`.

Streams over Server-Sent Events. The client uses `getReader` to
read chunks. Sources are collapsible; the user can see exactly
which file the model read.

The model receives:
1. The system prompt from `Soul.md`.
2. The memory context from the `memory` table.
3. The user's question.
4. The set of available skills.
"""))

# 30 feature library
DOCS.append(("30-feature-library.md",
H("Feature: library") + """The browsable grid. Every settled clip is a card. The card shows
the title, the host, the saved date, and a type-color accent
bar. The decay bar on the bottom shows how stale the item is
(green for fresh, amber for a month, red for a year).

Filter by host, by tag, by date. Cmd+K jumps to search.
"""))

# 31 feature inbox
DOCS.append(("31-feature-inbox.md",
H("Feature: inbox") + """The pre-library view. Anything still in `<vault>/Inbox/` shows
up here. The user triages: read, archive, tag, send to library.
Bulk select with shift-click. Keyboard:
- `j`/`k` — next/prev
- `space` — toggle select
- `a` — archive
- `l` — promote to library
- `t` — add tag
"""))

# 32 feature today
DOCS.append(("32-feature-today.md",
H("Feature: today") + """The landing page. Three sections:
1. **Greeting** — pulled from `Soul.md` and the time of day.
2. **This week** — clip count + sparkline + top hosts.
3. **For you** — five items the indexer thinks the user wants
   back (most-tagged, most-recent, most-saved-this-week).

The greeting changes tone at 06:00, 12:00, 18:00, 22:00.
"""))

# 33 feature detail
DOCS.append(("33-feature-detail.md",
H("Feature: detail slide-over") + """Click any card. The right-side slide-over shows the full markdown
body, the frontmatter, the tags, the source URL, and the back
links (other items that link here).

Edit the body in place. The save is `PUT /api/items/:id` with
`If-Match: <mtime>` for optimistic concurrency. 412 means
someone else (you, in another tab) saved first.
"""))

# 34 feature cmdk
DOCS.append(("34-feature-cmdk.md",
H("Feature: command palette") + """`/` opens it. `>` opens the action palette instead. Both are the
same surface with two different filters: `>` shows actions
(archive, tag, export), `/` shows search.

The palette is keyboard-only by design. The mouse is a fallback.
"""))

# 35 feature connections
DOCS.append(("35-feature-connections.md",
H("Feature: connections") + """The slide-over panel that lists every frontmatter field in the
current selection plus the items that share any field with it.
Two items that share three tags are drawn as a thicker line on
the graph; the graph is rendered as inline SVG, no library.
"""))

# 36 feature whatsapp
DOCS.append(("36-feature-whatsapp.md",
H("Feature: WhatsApp capture") + """Toggle `capture_whatsapp` in Muse settings. While on, a hidden
message-only window listens for `WM_CLIPBOARDUPDATE`. When the
foreground process is `whatsapp.exe` and the clipboard changed,
the listener appends to `<vault>/WhatsApp/<date>.md`:

```markdown
## 14:31 · Papa

> don't forget the keys

---
```

The contact name is read from the foreground window title. The
listener uses `AddClipboardFormatListener` (Windows 8+).
"""))

# 37 feature calls
DOCS.append(("37-feature-calls.md",
H("Feature: call capture (v0.7)") + """Toggle `record_calls` in Muse settings. When a known call
process (Zoom, Teams, Slack huddle, Discord) is foregrounded,
the watcher opens a WASAPI loopback + mic stream, downsamples
to 16 kHz mono, and buffers 30 s windows. Each window is
transcribed by `ggml-base.en` (local, on-device) and appended
to `<vault>/Calls/<date>.md`.

The transcription is local. No audio ever leaves the machine.
"""))

# 38 build
DOCS.append(("38-build.md",
H("Build") + """```
cd src-tauri
cargo build --release
./target/release/clipper.exe
```

The first build is slow (~6 min) because of `tauri` + `windows-sys`
+ `rusqlite` + `reqwest`. Subsequent builds are incremental.

Cross-compile is not supported. Clipper is a Windows desktop
product; macOS and Linux are out of scope.
"""))

# 39 dev loop
DOCS.append(("39-dev-loop.md",
H("Dev loop") + """```
cargo run --release          # one terminal
code .                       # editor
http://127.0.0.1:7137/       # browser
```

Changes to a Rust file trigger a rebuild and a daemon restart. The
single-instance plugin brings the dashboard up in the browser
when the new build takes over the port. Hot-reload of the static
pages is `F5`.

If the build is locked (`clipper.exe` is the current process):
`taskkill //F //IM clipper.exe //T` (Git Bash mangles `/F`; use the
double-slash form).
"""))

# 40 release
DOCS.append(("40-release-process.md",
H("Release process") + """1. Bump `version` in `src-tauri/Cargo.toml`.
2. `cargo build --release` — confirm green.
3. Smoke test: launch, capture a URL, watch Muse, open dashboard.
4. `git tag vX.Y.Z` and `git push --tags`.
5. Write `RELEASE_NOTES_vX.Y.Z.md` with the diff summary.
6. Post the announcement.

We do not have a CI build. The maintainer runs the build, the
maintainer signs the tag. Until there is a real team, automation
adds more risk than it removes.
"""))

# 41 packaging
DOCS.append(("41-packaging.md",
H("Packaging") + """Out of scope for v0.x. The Tauri build emits a single
`clipper.exe`. Distributing the binary + a stub installer is
enough for the personal use case.

When packaging lands it will be `tauri build` + a WiX bundle.
"""))

# 42 api overview
DOCS.append(("42-api-overview.md",
H("API overview") + """All routes are under `/api/`. The static dashboard is at `/`. The
pairing QR is at `/qr`.

Routes return JSON. Errors return `{ "error": "<message>" }` with
the right status code. The pairing middleware allows `/`, `/qr`,
`/pair`, and the SSE stream without a token.

Versioning is by URL prefix; `v0` is implicit.
"""))

# 43 api clip
DOCS.append(("43-api-clip.md",
H("API: clip") + """- `POST /api/clip` — `{ url, title? }` returns `{ id, status }`.
- `GET /api/clips?from=&to=&host=&tag=&limit=` — list.
- `GET /api/clips/:id` — single row.
- `DELETE /api/clips/:id` — archive (soft).

`POST` dedupes by SHA-256 of the canonical URL. If the row exists,
the response is `200` with the existing id, not `201`.
"""))

# 44 api search
DOCS.append(("44-api-search.md",
H("API: search") + """- `GET /api/search?q=&limit=` — FTS5 results.
- `GET /api/search?q=&mode=hybrid` — FTS5 + vector (v0.9).

Results are sorted by `bm25` score. `limit` defaults to 20, max
100.
"""))

# 45 api muse
DOCS.append(("45-api-muse.md",
H("API: muse (root)") + """- `GET /api/muse/state` — current state (`off` | `watching` |
  `paused`) + last 30 entries from today's activity log.
- `POST /api/muse/state` — `{ "state": "watching" }` toggles the
  watcher.

The watcher itself runs in the daemon process; this endpoint is
the only way the dashboard talks to it.
"""))

# 46 api muse activity
DOCS.append(("46-api-muse-activity.md",
H("API: muse/activity") + """- `GET /api/muse/activity?date=YYYY-MM-DD&limit=N` — entries
  from the day's markdown, newest first.

The response is a JSON array of:
```json
{
  "time": "14:23",
  "title": "Cargo.toml",
  "process": "Code",
  "summary": "editing deps",
  "category": "dev",
  "tags": ["rust", "clipper"]
}
```

`date` defaults to today (local). `limit` defaults to 200.
"""))

# 47 api muse heatmap
DOCS.append(("47-api-muse-heatmap.md",
H("API: muse/heatmap") + """- `GET /api/muse/heatmap?days=84` — daily counts for the last N
  days. The dashboard renders this as an SVG grid.

Response:
```json
{ "days": [
  { "date": "2026-10-01", "count": 47 },
  ...
] }
```

`days` defaults to 84 (12 weeks). Max 365.
"""))

# 48 api muse digest
DOCS.append(("48-api-muse-digest.md",
H("API: muse/digest") + """- `GET /api/muse/digest?date=YYYY-MM-DD` — the day's one-line
  digest as written by `claude -p`. Cached for 6 hours.

The endpoint shells out to `claude -p` with
`CREATE_NO_WINDOW`. On a fresh install the first call may take
2-3 s while the CLI warms up.
"""))

# 49 api muse config
DOCS.append(("49-api-muse-config.md",
H("API: muse/config") + """- `GET /api/muse/config` — current `MuseCfg`.
- `POST /api/muse/config` — partial update. Unknown fields are
  rejected with `400`.

Fields: `skip_processes`, `smart_dedup`, `debounce_secs`,
`screenshots`, `record_calls`, `capture_whatsapp`.
"""))

# 50 api memory
DOCS.append(("50-api-memory.md",
H("API: memory") + """- `GET /api/memory` — JSON view of `<vault>/Memory/user.md`.
- `PUT /api/memory` — full replace, with `If-Match` mtime check.
- `GET /api/memory/people` — list of `People` entries.
- `POST /api/memory/people` — add a person.

The file is the source of truth. The table is a cache.
"""))

# 51 api skills
DOCS.append(("51-api-skills.md",
H("API: skills") + """- `GET /api/skills` — list installed skills.
- `GET /api/skills/:slug` — full skill body.
- `POST /api/skills/:slug/install` — register in the index.
- `DELETE /api/skills/:slug` — unregister (folder stays).
"""))

# 52 api chat
DOCS.append(("52-api-chat.md",
H("API: chat (SSE)") + """- `POST /api/chat/stream` — `{ "messages": [...] }` returns a
  text/event-stream.

Events:
- `message` — token delta
- `tool_call` — model wants to invoke a tool
- `tool_result` — local result for the call
- `done` — stream finished
- `error` — terminal error

The model is selected from `Settings -> Models`.
"""))

# 53 api pairing
DOCS.append(("53-api-pairing.md",
H("API: pairing") + """- `GET /api/pair/qr` — SVG QR for the current token.
- `POST /api/pair` — `{ "token": "..." }` returns the session
  cookie.

The token rotates every hour. A session is valid until the next
rotation; the rotation middleware re-issues a new cookie on the
next request.
"""))

# 54 api soul
DOCS.append(("54-api-soul.md",
H("API: soul") + """- `GET /api/soul` — current `Soul.md` (voice + persona).
- `PUT /api/soul` — replace. The body is markdown; the server
  splits on the first `##` heading.

The file is the source of truth; the API is a thin wrapper.
"""))

# 55 api self
DOCS.append(("55-api-self.md",
H("API: self (improvement)") + """- `POST /api/self/run` — kick the index recompute now.
- `GET /api/self/last` — last run timestamp + summary.

Used by the dashboard's "Refresh" button and the nightly cron.
"""))

# 56 db schema
DOCS.append(("56-db-schema.md",
H("Database schema") + """```sql
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
"""))

# 57 db fts
DOCS.append(("57-db-fts.md",
H("Database: FTS5") + """SQLite FTS5 is enough for personal-scale search. 10k clips is
under 50 MB of index. Reindex on every write is fine.

Triggers keep the FTS table in step:
```sql
CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
  INSERT INTO clips_fts(rowid, title, body, host, tags)
  VALUES (new.rowid, new.title, new.body, new.host, new.tags);
END;
```

`bm25` is the ranking function. We do not enable the trigram
tokenizer; English + a small ASCII set is the entire corpus.
"""))

# 58 db vault index
DOCS.append(("58-db-vault-index.md",
H("Database: vault index") + """The drainer is not the only writer to the vault. The user can open
Obsidian and rename a file. The index must follow.

The indexer (`vaultwatch`) is a `notify` watcher on the vault
root. Every change re-parses the touched file, upserts the row,
and refreshes the FTS index. The drainer is the fast path; the
indexer is the safety net.
"""))

# 59 config muse
DOCS.append(("59-config-muse.md",
H("Config: muse") + """```toml
[muse]
skip_processes = ["LockApp.exe", "ShellExperienceHost.exe"]
smart_dedup = true
debounce_secs = 30
screenshots = true
record_calls = false
capture_whatsapp = false
```

`skip_processes` is matched by exact basename. `smart_dedup`
groups by `(process, normalized_title)`. `debounce_secs` is the
quiet period before a new title is logged.
"""))

# 60 config server
DOCS.append(("60-config-server.md",
H("Config: server") + """```toml
[server]
bind = "127.0.0.1"
port = 7137
```

`port = 0` means "ask the OS for a free port". The chosen port is
written to the tray tooltip and the single-instance callback so
the dashboard URL is always right.
"""))

# 61 config hotkeys
DOCS.append(("61-config-hotkeys.md",
H("Config: hotkeys") + """Built-in:

- `Ctrl+Shift+C` — capture URL from clipboard
- `Ctrl+Shift+X` — capture image from clipboard
- `Ctrl+Shift+M` — pause/resume the Muse watcher

These are constants in `lib.rs`. Configurable hotkeys are
post-v1; the surface is small and the user has not asked.
"""))

# 62 testing strategy
DOCS.append(("62-testing-strategy.md",
H("Testing strategy") + """- **Unit** — every public function in `musewatch`, `drainer`,
  `capture`, `indexer`. No mocks of stdlib.
- **Integration** — boot the daemon in `--mock` mode and run a
  shell script that hits every route.
- **Manual smoke** — the maintainer runs the dashboard for 10
  minutes before any release.

Coverage target: 70% on `src-tauri/src`. The UI is not measured.
"""))

# 63 testing unit
DOCS.append(("63-testing-unit.md",
H("Testing: unit") + """```bash
cd src-tauri
cargo test --lib
```

The watcher tests use a fake `mpsc` and assert on the markdown
output. The drainer tests use a `tempfile` vault and assert on
the SQLite row.
"""))

# 64 testing integration
DOCS.append(("64-testing-integration.md",
H("Testing: integration") + """```bash
cd src-tauri
cargo build --release
./target/release/clipper.exe --smoke &
SERVER=http://127.0.0.1:7137
curl -s $SERVER/api/health
curl -s -X POST $SERVER/api/clip -d '{"url":"https://example.com"}'
curl -s "$SERVER/api/search?q=example"
```

`--smoke` skips the foreground watcher and the tray. Routes
return mock data behind `?mock=1` when the daemon is in smoke
mode.
"""))

# 65 security overview
DOCS.append(("65-security-overview.md",
H("Security overview") + """Threat model: a malicious file on the user's disk and a malicious
web page in the browser. The daemon trusts neither.

- The static pages are served with a strict CSP that forbids
  inline scripts and remote origins.
- Markdown from the vault is rendered through DOMPurify. User
  content can be markdown; it cannot be script.
- WASAPI loopback captures the default render device. There is
  no synthetic sink.
- The pairing token is bound to the LAN IP. A request from a
  different origin gets a 403.
"""))

# 66 security privacy
DOCS.append(("66-security-privacy.md",
H("Security: privacy") + """Nothing leaves the machine. The Claude CLI is a local install;
the audio capture is local Whisper. The single network egress is
the user's choice (e.g. they may turn on an outbound webhook
that posts a JSON row to their own server).

When a feature wants the network, the user is told. The default
is off.
"""))

# 67 security permissions
DOCS.append(("67-security-permissions.md",
H("Security: permissions") + """Windows capabilities the daemon requests:
- **Microphone** — only when `record_calls = true`. Required for
  capturing the user's voice on a call.
- **Accessibility** — `SetWinEventHook` does not require it on
  Windows 10+; we still request it because some UWP apps need it
  to fire events.
- **Filesystem** — read/write the vault folder. No broad access.

The Tauri capabilities are listed in
`src-tauri/capabilities/main.json`. We do not request any other
Windows capability.
"""))

# 68 troubleshooting
DOCS.append(("68-troubleshooting.md",
H("Troubleshooting") + """**Port 7137 is busy.** Edit `<config>/config.toml`, change
`server.port` to `0`, restart. The new port is in the tray
tooltip.

**The dashboard shows 0 items.** The vault may be empty. Press
`Ctrl+Shift+C` with a URL in the clipboard. If the tray tooltip
does not flash, the hotkey is owned by another app.

**The Muse watcher is silent.** Check `Ctrl+Shift+M` did not
pause it. Open Muse settings, confirm `skip_processes` does not
include the foreground app.

**A window pops up every second.** A previous Clipper build is
still running. `taskkill //F //IM clipper.exe //T`.
"""))

# 69 faq
DOCS.append(("69-faq.md",
H("FAQ") + """**Why headless?** A desktop app that owns a window has to fight
the window manager. A daemon that serves a URL gets the user's
browser for free, with their extensions and their fonts.

**Why markdown?** The user owns the file. They can grep, rsync,
edit in any editor. The drainer writes the file, the indexer
reads it. There is no second source of truth.

**Why SQLite?** One file. The user can copy it. `sqlite3` is on
every dev machine. FTS5 is built in. Vector recall is a v0.9
addition.

**Why `claude -p` instead of the SDK?** The CLI is what the
user already has installed. The shell-out is `CREATE_NO_WINDOW`
so there is no popup. The SDK is a post-v1 consideration.
"""))

# 70 error codes
DOCS.append(("70-error-codes.md",
H("Error codes") + """- `400` — bad input (schema mismatch, unknown field)
- `401` — missing pairing token
- `403` — token wrong or rotated
- `404` — id not found
- `409` — duplicate hash (dedup)
- `412` — `If-Match` mtime stale
- `422` — markdown parse error
- `500` — daemon panic; the response body has the cause

Every 500 is also written to `<vault>/.cache/clipper.log`.
"""))

# 71 observability
DOCS.append(("71-observability.md",
H("Observability") + """- Logs: `println!` in the daemon, visible in the launching
  terminal. The packaged build writes to
  `%APPDATA%/Clipper/clipper.log`.
- Stats: `<vault>/.cache/stats.json` — daily counts, top hosts,
  top tags. Refreshed at startup and at 03:00 local.
- Errors: tray tooltip flashes the message; the dashboard's
  `/api/self/last` exposes the last index run.

There is no remote telemetry. There is no event upload.
"""))

# 72 ADR 001
DOCS.append(("72-adr-001-tauri.md",
H("ADR 001: Tauri 2") + """**Status:** accepted.

**Context.** A Windows desktop product needs a shell. The
candidates are Electron, Tauri, WinUI, and a hand-rolled C++
tray. The product is small; the team is small.

**Decision.** Tauri 2.

**Consequences.** We get a system tray, global hotkeys, and a
small static binary. The webview is WebView2, which is already
on every Windows 11 machine. The downside is the Tauri build
graph, which is large but well-understood.

**Alternatives.** Electron (larger binary, easier JS), WinUI
(Windows-only, more code, no webview).
"""))

# 73 ADR 002
DOCS.append(("73-adr-002-headless.md",
H("ADR 002: headless daemon") + """**Status:** accepted.

**Context.** The product has a tray, hotkeys, a watcher, and a
dashboard. The dashboard is a web app. Should the desktop app
also own a window?

**Decision.** No. The daemon is headless. The tray's "Show
dashboard" opens the browser at `http://127.0.0.1:7137/`.

**Consequences.** One less surface to maintain. The browser
becomes the user's window, with their extensions and their zoom
level. The downside is the user has to know the port; mitigated
by the tray tooltip and the QR.

**Alternatives.** A Tauri WebviewWindow (more memory, more
processes, more bugs).
"""))

# 74 ADR 003
DOCS.append(("74-adr-003-sse.md",
H("ADR 003: SSE for chat") + """**Status:** accepted.

**Context.** The chat surface streams tokens from the model.
Candidates are WebSocket, SSE, and long-poll.

**Decision.** Server-Sent Events.

**Consequences.** One-way, plain HTTP, works through every
proxy, easy to debug with `curl`. The downside is no
client-to-server streaming beyond the initial POST; we do not
need it for now.

**Alternatives.** WebSocket (heavier, harder to proxy), long-poll
(works but is slow).
"""))

# 75 ADR 004
DOCS.append(("75-adr-004-sqlite-fts.md",
H("ADR 004: SQLite + FTS5") + """**Status:** accepted.

**Context.** We need a search index over a personal vault. The
candidates are SQLite FTS5, Tantivy (Rust), and a hosted
search.

**Decision.** SQLite + FTS5 + the bundled `rusqlite` crate.

**Consequences.** One file, no service, fast. The downside is
the lack of a vector index; deferred to v0.9 with `sqlite-vec`.

**Alternatives.** Tantivy (great, but a separate process and a
new build dep), hosted (privacy, latency).
"""))

# 76 ADR 005
DOCS.append(("76-adr-005-markdown-vault.md",
H("ADR 005: markdown vault") + """**Status:** accepted.

**Context.** The vault holds every saved item. The choices are
a single SQLite blob, a folder of JSON, or a folder of markdown
with frontmatter.

**Decision.** Markdown with frontmatter.

**Consequences.** The user can grep, edit, and version the
vault. The drainer writes the file; the indexer reads it. The
downside is the parser; mitigated by `pulldown-cmark`.

**Alternatives.** JSON (lossy, ugly), SQLite blob (opaque to
the user).
"""))

# 77 ADR 006
DOCS.append(("77-adr-006-muse-markdown.md",
H("ADR 006: Muse writes markdown") + """**Status:** accepted.

**Context.** The Muse watcher emits one entry per focus change.
The storage choice is a SQLite log, a JSON-lines file, or a
markdown log.

**Decision.** One markdown file per day, append-only.

**Consequences.** The user can read the log in any editor. The
dashboard reads the file with a tiny parser. The downside is
the parse cost; negligible at 200 lines per day.
"""))

# 78 ADR 007
DOCS.append(("78-adr-007-whisper-base.md",
H("ADR 007: local Whisper, base.en") + """**Status:** accepted.

**Context.** The audio capture feature needs a speech-to-text
model. Candidates are `whisper-rs` with `ggml-base.en`, Vosk,
and a cloud API.

**Decision.** `ggml-base.en` via `whisper-rs`.

**Consequences.** 141 MB on disk, 376 MB peak RSS, WER 2.07%
on librispeech-asr clean. CPU floor at idle is 0. RTF 0.24 on a
Ryzen 7 5700X. No network. MIT license.

**Alternatives.** Vosk (faster, weaker on accented speech), a
cloud API (privacy, latency, cost).
"""))

# 79 ADR 008
DOCS.append(("79-adr-008-loopback-only.md",
H("ADR 008: WASAPI loopback, not a virtual cable") + """**Status:** accepted.

**Context.** Capturing the user's call audio needs the OS to
hand us the speaker buffer. The choices are WASAPI loopback on
the default render device, a virtual-audio-cable sink, or a
kernel driver.

**Decision.** WASAPI loopback, no synthetic device.

**Consequences.** The capture is on whatever the user picked as
their output. The downside is device switching; mitigated by
subscribing to `IMMNotificationClient` events (v0.8).

**Alternatives.** Virtual cable (more setup, more failure modes),
kernel driver (privileged, fragile).
"""))

# 80 ADR 009
DOCS.append(("80-adr-009-anthropic-summary.md",
H("ADR 009: Claude CLI for summary") + """**Status:** accepted.

**Context.** The watcher summarizes a foreground window title.
The choices are the Anthropic SDK, the Claude CLI, or a local
model.

**Decision.** `claude -p` with `CREATE_NO_WINDOW`.

**Consequences.** The user already has the CLI installed for
Ask. The shell-out is two flags; no SDK wiring. The downside
is the cold-start cost (2-3 s); the watcher is debounced, so
this is invisible.

**Alternatives.** SDK (more code, no behavior gain), local model
(privacy wins, accuracy cost).
"""))

# 81 ADR 010
DOCS.append(("81-adr-010-skip-processes.md",
H("ADR 010: skip-processes allowlist") + """**Status:** accepted.

**Context.** The watcher would otherwise log every focus
change, including the lock screen and shell windows.

**Decision.** A configurable `skip_processes` list. Defaults:
`LockApp.exe`, `ShellExperienceHost.exe`.

**Consequences.** The user can grow the list as they find more
noise. The watcher checks the basename, not the path.
"""))

# 82 market overview
DOCS.append(("82-market-overview.md",
H("Market overview") + """The personal-knowledge-management (PKM) market is crowded:

- **Capture**: Raycast, Notion Web Clipper, Raindrop.io
- **Note-taking**: Obsidian, Logseq, Notion, Bear
- **Read-later**: Readwise Reader, Pocket, Matter
- **Memory / recall**: Reflect, Heptabase, Mem

Clipper sits between the first two: a clipper that respects the
local-first ethos of Obsidian, the keyboard-first ethos of
Raycast, and the design ethos of Heptabase.

We are not trying to replace any one of them. We are trying to
make the average user's existing tools (their browser, their
editor) more useful by adding a quiet layer of recall.
"""))

# 83 competitors
DOCS.append(("83-market-competitors.md",
H("Market: competitors") + """| Product         | Local | Open | Capture | Recall | Price    |
|-----------------|-------|------|---------|--------|----------|
| Raycast         | No    | No   | Yes     | No     | $8/mo    |
| Notion Clipper  | No    | No   | Yes     | Yes    | $10/mo   |
| Raindrop        | No    | No   | Yes     | No     | $5/mo    |
| Obsidian        | Yes   | No   | No      | Yes    | $8/mo    |
| Logseq          | Yes   | Yes  | No      | Yes    | Free     |
| Reflect         | No    | No   | No      | Yes    | $10/mo   |
| Heptabase       | No    | No   | No      | Yes    | $8/mo    |
| Mem             | No    | No   | No      | Yes    | $15/mo   |
| Readwise Reader | No    | No   | Yes     | Yes    | $8/mo    |
| **Clipper**     | Yes   | Yes  | Yes     | Yes    | Free     |
"""))

# 84 raycast
DOCS.append(("84-market-raycast.md",
H("Market: Raycast") + """Raycast is the keyboard-first launcher. It has a clipper, a
window manager, an extension store, and a community. It is
excellent for power users. It is not local-first, the extensions
are sandboxed in JavaScript, and the free tier is deliberately
crippled.

Clipper's UX debts to Raycast: the global hotkey, the Cmd+K
palette, the tray-style menu, the markdown everywhere.
"""))

# 85 obsidian
DOCS.append(("85-market-obsidian.md",
H("Market: Obsidian") + """Obsidian is the user's editor. It owns the file; we own the
capture. We integrate by writing markdown to the same folder.

The integration is "format only." The user does not need a
plugin. The drainer writes frontmatter that Obsidian understands;
Obsidian links resolve naturally in the dashboard.

We will never replace Obsidian. We are the inbox to its library.
"""))

# 86 readwise
DOCS.append(("86-market-readwise.md",
H("Market: Readwise") + """Readwise is the recall layer. It pulls highlights from Kindle,
Twitter, podcasts, and YouTube and resurfaces them.

Clipper is the *capture* layer for the things Readwise does not
see: the URLs, the code, the half-written thoughts in a chat
window. The two products complement each other; a user with
both has both ends of the loop.
"""))

# 87 notion
DOCS.append(("87-market-notion.md",
H("Market: Notion") + """Notion is a workspace. Clipper is a memory. Notion's web clipper
captures into a Notion page; ours captures into the local
markdown vault.

The difference matters when Notion is down, when the user does
not want a Notion account, or when the user wants to keep the
note on the laptop that wrote it.
"""))

# 88 spotlight
DOCS.append(("88-market-spotlight.md",
H("Market: Spotlight") + """Spotlight is the OS-level search. It indexes the file metadata,
not the file body. It does not capture.

Clipper is what feeds Spotlight. We write a folder of markdown
that Spotlight can find. The user can also search through our
dashboard, which sees the body.
"""))

# 89 market sizing
DOCS.append(("89-market-sizing.md",
H("Market: sizing") + """Bottom-up:

- Obsidian: ~1M paid users (publicly stated)
- Raycast: ~1M active users (publicly stated)
- Readwise: ~200k paid users (publicly stated)

The addressable market for "local-first personal capture" is the
intersection of these: keyboard-first users who already pay for
software and care about their data. Estimate: 200-500k people
globally.

We are not chasing that number. We are building the product we
want. If the number follows, great; if not, we still have the
product.
"""))

# 90 personas
DOCS.append(("90-market-personas.md",
H("Market: personas") + """- **The writer** — drafts in Obsidian, wants a single key to
  save anything they read. Their pain is the lost article.
- **The developer** — has 200 tabs open. Wants a single key to
  save the one that mattered. Their pain is the lost link.
- **The student** — captures lectures and readings. Wants to
  find them again. Their pain is the lost note.
- **The founder** — captures competitor pages, customer quotes,
  and their own thoughts. Wants to see the pattern later. Their
  pain is the lost thread.
"""))

# 91 positioning
DOCS.append(("91-market-positioning.md",
H("Market: positioning") + """**For** keyboard-first people who already pay for software,
**who** lose more than they should to their own memory,
**Clipper** is a local-first capture layer for the desktop,
**that** writes to the same folder their editor already reads,
**unlike** Raycast (not local) or Readwise (not capture),
**we** never send your data anywhere.
"""))

# 92 user stories
DOCS.append(("92-user-stories.md",
H("User stories") + """- As a writer, I want to save a paragraph from any page with
  one key, so I can find it later.
- As a developer, I want to capture a Stack Overflow answer in
  the same place as my notes, so I do not lose it to a browser
  refresh.
- As a student, I want my lecture links and reading links in
  one searchable place, so exam week is faster.
- As a founder, I want every customer quote I ever copied to
  surface when I plan a launch, so the language is real.
- As a privacy user, I want my clipboard and screen watched by
  something on my own machine, so I do not have to trust a
  cloud.
"""))

# 93 onboarding
DOCS.append(("93-onboarding.md",
H("Onboarding") + """1. Install. The tray icon appears.
2. Click the tray. Pick a vault folder. The dashboard opens.
3. Press `Ctrl+Shift+C` with a URL in the clipboard. The
   dashboard updates in 1 s.
4. Open Muse settings. Toggle "Watch the foreground." Watch a
   few apps. Open the Muse page; the activity log is live.
5. Press `Ctrl+Shift+X` with an image in the clipboard. The
   image lands in the library.

Total time: 90 s. The user never sees a setup wizard.
"""))

# 94 power user tips
DOCS.append(("94-power-user-tips.md",
H("Power user tips") + """- **Mute a process.** Add `Teams.exe` to `skip_processes` to
  skip the focus spam from a call.
- **Two windows at once.** Open the dashboard in two browsers.
  Both see the same state.
- **Edit the markdown.** The dashboard will pick up your edit
  on the next index pass (≤ 5 s).
- **Tag from the keyboard.** In the inbox, press `t`, type,
  Enter. The tag is applied to the selected rows.
- **Export the vault.** `rsync` it to a NAS. The format is
  portable across machines.
"""))

# 95 keyboard shortcuts
DOCS.append(("95-keyboard-shortcuts.md",
H("Keyboard shortcuts") + """**Global**
- `Ctrl+Shift+C` — capture URL
- `Ctrl+Shift+X` — capture image
- `Ctrl+Shift+M` — Muse pause/resume

**Dashboard**
- `/` — search
- `>` — actions
- `j` / `k` — next / prev
- `space` — toggle select
- `a` — archive
- `l` — promote to library
- `t` — tag
- `e` — edit
- `?` — show shortcuts
"""))

# 96 metrics
DOCS.append(("96-metrics.md",
H("Metrics") + """What we measure:
- **Daily active launches** — how often the tray is clicked.
- **Clips per day** — the median and the tail.
- **Time to first recall** — minutes from launch to first
  dashboard open.
- **Time to first search** — minutes from launch to first
  `/api/search` hit.
- **Recalled per week** — unique clips viewed in the library.

We do not measure:
- Signups. There is no signup.
- DAU. There is no server.
- Engagement. We are a tool, not a feed.
"""))

# 97 known issues
DOCS.append(("97-known-issues.md",
H("Known issues") + """- **The watcher misses UWP apps** that do not fire Win32
  foreground events. Mitigated by the 1 s safety poll.
- **WhatsApp contact name** is read from the window title,
  which is "WhatsApp" while the chat list is focused and the
  contact name while a chat is open. The captured entry uses
  the latter.
- **Long titles** are truncated to 200 chars in the markdown
  log. The full title is in the SQLite row.
- **Audio device switching** during a call drops the loopback
  stream. Will be fixed in v0.8 with `IMMNotificationClient`.
- **Cold Claude CLI** takes 2-3 s. The watcher is debounced, so
  this is invisible during normal use.
"""))

# 98 compatibility
DOCS.append(("98-compatibility.md",
H("Compatibility") + """- **OS**: Windows 11 23H2 or later. Windows 10 22H2 may
  work but is not tested.
- **WebView2**: bundled with Windows 11. On Windows 10 it is a
  one-time install.
- **Claude CLI**: optional. Without it, the summarizer is a
  no-op and the watcher still writes titles.
- **Microphone**: required only for `record_calls`.
- **Disk**: 80 MB for the binary, 200 MB for the cache
  (screenshots + Whisper weights).
"""))

# 99 license
DOCS.append(("99-license.md",
H("License") + """MIT.

Copyright (c) 2026 Aswath Asokan.

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without
restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, sublicense, and/or
sell copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.
"""))

with open(os.path.join(ROOT, "00-INDEX.md"), "w", encoding="utf-8") as f:
    pass
for name, body in DOCS:
    with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
        f.write(body)
print(f"wrote {len(DOCS)} files")
