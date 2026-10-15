# Clipper

Smart clipboard / watch-later for Windows. A hotkey captures anything, Claude Code turns
it into structured notes in your Obsidian vault, and a local dashboard shows it all.

One Tauri binary. No cloud, no account, no telemetry.

---

## Features

- **Clipboard URL capture** — press `Ctrl+Shift+C` in any app. A URL is deduped
  (SHA-256 of the URL, first 16 hex chars, checked against `archive.jsonl`) and queued.
  An already-saved URL reports `Already saved: <host>` and is skipped.
- **Clipboard image capture** — press `Ctrl+Shift+X` (use `Win+Shift+S` first). The
  image is written to `~/.clipper/staging/img_<unix_ts>.png` and queued as `type: image`.
- **Smart classification via the Claude Code CLI** — a background drainer batches queued
  items, builds a prompt from `~/.clipper/prompts/extract.md`, and runs `claude -p`. It
  gets back one markdown path per item carrying `type`, `title`, a one-sentence
  `summary`, and up to five `tags`.
- **Your Obsidian vault is the source of truth** — notes land as
  `<vault>/Inbox/YYYY-MM-DD_HHMM_<source>_<id>.md` with YAML frontmatter. Clipper
  auto-detects the vault; if you have none it creates `~/Documents/ClipperVault`.
- **Local dashboard** — `http://127.0.0.1:7137/` with Today / Inbox / Library panes and
  full-text search, backed by a SQLite + FTS5 index that mirrors the vault.
- **Phone PWA pair over LAN** — tray → *Show QR*, scan it, and the phone gets a no-build
  web app that posts links to the desktop.
- **Local only** — no cloud, no account, no telemetry. A JSONL queue, one SQLite file,
  and markdown in your vault.
- **MIT licensed.**

> **Scope of this release.** `SPEC.md` designs four capture modes (URL, screenshot,
> photo, free text); this release wires **two** — URL and screenshot. Photo and free-text
> capture are designed but not implemented yet. The tray's *Settings* item is still a
> placeholder.

---

## Install

### Prerequisites

- **Windows 10/11 with the WebView2 runtime.** Included on Windows 11; on Windows 10
  install it from Microsoft.
- **Rust 1.99+ with MinGW-w64 (GCC 16+) and the `x86_64-pc-windows-gnu` target.**
  Clipper builds with the GNU toolchain, so **no Visual Studio C++ build tools are
  needed**:

  ```powershell
  winget install --id Rustlang.Rustup -e
  winget install --id BrechtSanders.WinLibs.POSIX.UCRT -e   # MinGW-w64: gcc, ld, ar
  rustup toolchain install stable-x86_64-pc-windows-gnu
  rustup default stable-x86_64-pc-windows-gnu
  ```

  Then point cargo at the MinGW linker — `~/.cargo/config.toml`:

  ```toml
  [target.x86_64-pc-windows-gnu]
  linker = "x86_64-w64-mingw32-gcc.exe"
  ar = "ar.exe"
  ```

  > **winget id note:** `BrechtSanders.WinLibs.POSIX.UCRT` is the package that works.
  > The `niXman.mingw-w64-gcc` id does not exist in the winget repository.

- **Node.js 20+** — only to run the Tauri CLI. Not a runtime dependency of the built app.
- **Claude Code CLI on `PATH`** (`claude --version` works). This is what performs the
  extraction; without it the queue will deadletter.

### Build

```powershell
git clone <repo>
cd SmartClipboard
npm install
npx tauri build
```

Produces `target\release\clipper.exe`, plus installers in `target\release\bundle\`:
an `.msi` and an NSIS `-setup.exe`.

### First run

1. Launch `clipper.exe`. The tray icon appears; **no window opens** (by design).
2. Clipper auto-detects your Obsidian vault, or creates `~/Documents/ClipperVault`.
   `~/.clipper/config.toml` is written on this first run and then left alone.
3. Tray → *Show dashboard* opens the dashboard window. The same page is served at
   <http://127.0.0.1:7137/> if you prefer a browser.

---

## Usage

### Hotkeys (system-wide — they work while any app is focused)

| Hotkey | Does |
|---|---|
| `Ctrl+Shift+C` | queue the URL currently on the clipboard |
| `Ctrl+Shift+X` | queue the image currently on the clipboard (`Win+Shift+S` first) |

### Tray menu

- **Show dashboard** — opens the dashboard window (Today / Inbox / Library)
- **Settings** — placeholder, planned
- **Show QR** — opens the phone-pairing QR window
- **Quit**

### Phone pair

Tray → *Show QR*. Scan the code with your phone; the browser opens the Clipper PWA.
Type a URL, press **Save**. It takes exactly the same path as a desktop capture — same
queue, same drainer, same vault. The PWA is plain HTML/JS with no build step, and keeps
working if the CDN is unreachable (the form falls back to plain DOM).

The QR encodes `http://<lan-ip>:<port>/pair?token=<session-token>`. The token is 32
random characters and rotates every 24 hours; `POST /save` rejects anything without it.

### Vault layout

```
<vault>/
└── Inbox/
    └── 2026-10-15_1225_clipboard_<uuid>.md    # one structured note per capture
```

Each note opens with YAML frontmatter:

```yaml
---
type: article        # article | video | repo | book | product | note | other | image
source: example.com
url: https://example.com/post
saved_at: 2026-10-15T12:25:00Z
title: How sockets work
summary: One sentence, at most 25 words.
tags: [networking, tcp]
---
```

Editing a note is safe: the index is rebuilt from the vault, keyed by the id in the
filename, and re-indexing never overwrites a status you set in the dashboard.

---

## Config

`~/.clipper/config.toml` is written on first run and left alone afterwards. Every path
accepts a leading `~`.

| Key | Default | Meaning |
|---|---|---|
| `vault.path` | auto-detected | your Obsidian vault |
| `vault.inbox_subdir` | `Inbox` | subfolder notes are filed into |
| `provider.claude.command` / `.args` | `claude` / `["-p"]` | the extractor to spawn |
| `provider.claude.stdin` | `prompt-file` | `prompt-file` pipes the prompt into the child's stdin |
| `provider.image.backend` / `.prompt_hint` | `claude-via-mcp` / MiniMax hint | how images are described |
| `queue.dir` | `~/.clipper/queue` | `inbox.jsonl`, `archive.jsonl`, `deadletter.jsonl` |
| `queue.staging_dir` | `~/.clipper/staging` | batch prompts, generated notes, screenshots |
| `queue.batch_size` | `5` | items per provider run |
| `queue.poll_interval_ms` | `5000` | drainer poll interval (it also wakes on file change) |
| `queue.timeout_s` | `120` | per-batch provider timeout |
| `server.bind` / `.port` | `127.0.0.1` / `7137` | dashboard + API |
| `prompts.extract` | `~/.clipper/prompts/extract.md` | the extraction prompt |

### Memory files

Three markdown files are read at startup and sent ahead of **every** Claude prompt, so
the persona and your context come before the task:

```
~/.clipper/memory/
├── soul.md      # persona and tone
├── user.md      # your expertise, style, dislikes
└── memory.md    # active work, recent focus, today's plan
```

They are created with defaults on first run (seeded from `src-tauri/seed/`). Edit them
and restart Clipper for the change to take effect; a live editor is planned.

Each prompt is assembled as `<soul><user><memory>`, then a `---` fenced copy of
`extract.md`, then the batch instructions.

Setting **`CLIPPER_HOME`** replaces `~/.clipper` entirely — that is how
`scripts/smoke.ps1` runs a whole session without touching your real state, and it also
makes a portable install possible.

---

## API

Bound to `[server] bind`/`port` (`127.0.0.1:7137`). `GET /` serves the dashboard.

| Route | Returns |
|---|---|
| `GET /healthz` | `200 ok` |
| `GET /api/items?limit=50&offset=0&type=&tag=&q=` | `{items, total}`; `q` is an FTS5 phrase match, `type`/`tag` are exact |
| `GET /api/items/{id}` | one item, or `404` |
| `POST /api/items/{id}/status` `{status}` | `{ok:true}`; status must be `inbox`/`reading`/`done`/`archived`/`deleted` |
| `POST /api/items/{id}/tags` `{tags:[…]}` | `{ok:true, tags}` |
| `POST /api/clips` `{url, type?}` | `{id, status:"queued"}`; appends to `inbox.jsonl` |
| `GET /api/today` | `{items}`, newest 3 with status `inbox` |
| `GET /api/stats` | `{total, by_type, by_status, stale_30d}` |
| `GET /pair` | the phone PWA page |
| `GET /pair.png` | QR code for the pairing URL, as `image/png` |
| `GET /qr` | the small page shown by the tray's *Show QR* item |
| `GET /pwa/app.js`, `GET /pwa/manifest.json` | PWA assets |
| `POST /save` `{url}` | same as `/api/clips`; **requires** the pairing token in `?token=` or `X-Clipper-Token`, else `401` |
| `GET /api/skills` | `{skills, total}` from `~/.clipper/skills/` |
| `GET /api/skills/{name}` | one skill plus `html` (markdown rendered, embedded raw HTML stripped) |
| `PUT /api/skills/{name}` `{body_md, source?}` | writes the file and re-indexes it |
| `DELETE /api/skills/{name}` | removes the file and its row |
| `POST /api/skills/rescan` | rebuilds the skill index from the directory |

`items.saved_at` is Unix **seconds** as a JSON *number* — the dashboard multiplies by 1000
before formatting.

Skill names are slugs (`[A-Za-z0-9._-]`, no `..`), so a name can never escape the skills
directory. Skill markdown is rendered with `pulldown-cmark` and the embedded raw-HTML
events are dropped server-side; the client runs DOMPurify as well.

CORS: only `http://127.0.0.1:7137`, `http://localhost:7137` and the app's own webview
origins get CORS headers; everything else is default-deny.

If the port is already in use Clipper logs the address it could not bind and exits
non-zero — it never silently picks another port.

## Architecture

One Tauri 2 binary: Rust backend, vanilla HTML/JS frontend, no framework and no bundler.

A capture appends a line to a JSONL queue. A background drainer batches those lines,
writes a prompt file, runs the Claude Code CLI with a timeout, and files each returned
markdown document into the Obsidian vault — which stays the source of truth. A SQLite
index with an FTS5 virtual table mirrors the vault for search, kept in step by a file
watcher. An axum server on `127.0.0.1:7137` serves the dashboard, the REST API, and the
pairing endpoints; the dashboard is embedded in the binary and served from `/`, so its
relative `/api/...` fetches stay same-origin.

Failures are loud and bounded: a bad provider exit, a timeout, or unparseable stdout
moves the batch to `deadletter.jsonl` with a reason instead of retrying forever, and if
the port is taken Clipper reports it and exits rather than silently relocating.

See [`SPEC.md`](./SPEC.md) for the data model and the build slices.

---

## Development

### Smoke test

```powershell
pwsh -File scripts/smoke.ps1        # or: powershell -File scripts\smoke.ps1
```

Builds the release binary if needed, then runs it fully isolated under a temporary
`CLIPPER_HOME` with a faked provider and asserts the whole pipeline: config + vault
auto-detect, `/healthz`, the items/today API, `/pair`, `/pair.png`, `/save` returning
`401` without a token, then queue → drainer → vault → index for three URLs. It ends with
`SMOKE OK (19 checks)` and leaves the temp vault behind for inspection.

### Per-slice self-checks

The release binary also carries flag-driven self-checks, one per slice:

```powershell
clipper.exe --hag3-smoke   # vault, queue, drainer, writer
clipper.exe --hag4-smoke   # SQLite, FTS5, every REST route over real HTTP
clipper.exe --hag6-smoke   # image capture through the drainer
clipper.exe --hag7-smoke   # pairing: /pair, /pair.png, token-gated /save
clipper.exe --hag10-smoke  # soul/user/memory seeding + injection into the Claude prompt
clipper.exe --integration-smoke  # cross-cutting API contract gates (also tests/integration_api.rs)
```

> These live behind flags rather than in `tests/` because a Rust test-harness binary
> linking the Tauri stack cannot be loaded on the windows-gnu toolchain
> (`STATUS_ENTRYPOINT_NOT_FOUND`), while `clipper.exe` itself loads fine. This way the
> checks exercise the code that actually ships.

---

## Troubleshooting

| Symptom | Fix |
|---|---|
| `claude: command not found` | Install the Claude Code CLI and make sure it is on `PATH`. |
| `Clipper: cannot bind 127.0.0.1:7137 …` | Something else owns 7137. Clipper exits rather than quietly moving to another port — kill the other process, or change `[server] port`. |
| `no .obsidian folder found` | Expected if you have no vault yet: Clipper creates `~/Documents/ClipperVault`. Move it wherever you like and update `vault.path`. |
| `warning: .rsrc merge failure: multiple non-default manifests` during the build | Known and harmless — MinGW `ld` and `tauri-winres` both supply a manifest. Linking succeeds. |
| A hotkey does nothing | Another app may already own `Ctrl+Shift+C` / `Ctrl+Shift+X`. Both are constants at the top of `src-tauri/src/lib.rs`. |
| Notes never appear in the vault | Check `~/.clipper/queue/deadletter.jsonl` — a note there carries the reason (`bad-stdout`, `timeout`, `exit-<code>`). |
| The dashboard shows sample clips that are not yours (e.g. "Tauri 2.0 Architecture…") | That is the dashboard's built-in offline fallback: it could not reach the API, so it rendered mock fixtures. Make sure Clipper is running, and open the dashboard from the tray. |

---

## License

MIT — see [`LICENSE`](./LICENSE).
