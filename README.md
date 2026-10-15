# Clipper

A clipboard-capture desktop app built with [Tauri 2](https://tauri.app) (Rust backend,
vanilla HTML + JS frontend — no framework, no bundler).

Capture a URL anywhere in Windows with one keystroke; Clipper queues it, hands it to
Claude Code to summarise, and files the result as a Markdown note in your Obsidian vault.
See [`SPEC.md`](./SPEC.md) for the full design.

## What works today

- **Vault auto-detect** — on first run Clipper scans `~/Documents`, `~/ObsidianVaults`,
  `~/Notes`, `~/repos`, `~/OneDrive/Documents` and every first-level folder of `~` for a
  `.obsidian` directory. Exactly one vault → use it. Several → use the first and log them
  all. None → create `~/Documents/ClipperVault`. The choice is persisted in
  `~/.clipper/config.toml`, so later runs never re-scan.
- **Global hotkey `Ctrl+Shift+C`** — anywhere in Windows, reads the clipboard. A URL is
  deduped (SHA-256 of the URL, first 16 hex chars, checked against `archive.jsonl`) and
  appended to `~/.clipper/queue/inbox.jsonl`. An already-saved URL reports
  `Already saved: <host>` and is skipped; non-URL text is ignored.
- **Queue drainer** — a background task that polls every 5 s *and* wakes on file change
  (200 ms debounce). It takes up to `batch_size` items, rotates them into
  `inbox.processing`, builds a batch prompt from `~/.clipper/prompts/extract.md`, runs the
  configured provider, and files each returned `.md` into
  `<vault>/Inbox/YYYY-MM-DD_HHMM_clipboard_<id>.md`.
- **Failure handling** — a non-zero provider exit, a timeout, or stdout that is not one
  existing staging path per input moves the batch to `deadletter.jsonl` with a reason
  (`bad-stdout`, `timeout`, `exit-<code>`, `bad-json`) rather than retrying forever.
- **SQLite index** — `~/.clipper/index.db` (bundled SQLite, WAL) with an FTS5
  external-content table kept in sync by triggers. A watcher on `<vault>/Inbox/`
  re-indexes on create/modify with a 500 ms debounce, keyed by the id embedded in the
  note's filename, so re-indexing is idempotent and a dashboard-set status survives it.
- **REST API** on `127.0.0.1:7137` (see below), which also serves the dashboard from `/`
  so its relative `/api/...` fetches stay same-origin.
- **System tray** with three menu items: *Show dashboard* (placeholder), *Settings*
  (placeholder), *Quit*.
- **Single instance** — launching again focuses the existing window instead of starting
  a second copy.
- The main window is **hidden by default**; the app is driven from the tray.

## Prerequisites (Windows)

| Tool | Why | Check |
|---|---|---|
| **Rust** (stable) + **MinGW-w64** | Tauri compiles a native binary. This machine uses the GNU toolchain — see below. | `rustc --version`, `gcc --version` |
| **Node.js** ≥ 18 | Runs the Tauri CLI (`@tauri-apps/cli`) | `node --version` |
| **WebView2 runtime** | Renders the UI (pre-installed on Windows 11) | — |

### Toolchain note: GNU, not MSVC

This machine has **no Visual Studio C++ build tools**, so the app is built with the
**`x86_64-pc-windows-gnu`** toolchain instead of the default MSVC one. Nothing here
needs an elevated shell.

```powershell
winget install --id Rustlang.Rustup -e                                   # Rust
winget install --id BrechtSanders.WinLibs.POSIX.UCRT -e                  # MinGW-w64 (gcc/ld/ar)
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

`~/.cargo/config.toml` points the GNU target at the MinGW linker:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc.exe"
ar = "ar.exe"
```

If you install the MSVC build tools later, `rustup default stable-msvc` and this
override becomes inert.

Known benign warning: every build prints
`warning: .rsrc merge failure: multiple non-default manifests` — MinGW `ld` and
`tauri-winres` both supply a manifest. The binary builds, links and runs.

## Build & run

```powershell
npm install          # fetches the Tauri CLI
npx tauri dev        # dev build, hot reload
```

Equivalent with the Rust CLI (if you have it: `cargo install tauri-cli --locked`):

```powershell
cargo tauri dev
cargo tauri build
```

A production build, without bundling installers:

```powershell
npx tauri build --no-bundle
```

The binary lands in `src-tauri/target/release/clipper.exe`. Bundled installers
(`.msi` / NSIS `.exe`) come from a plain `npx tauri build` and need WiX/NSIS
downloaded on first run.

### Trying it

1. Run `npx tauri dev` (or launch the built `clipper.exe`).
2. The window stays hidden — look for the **Clipper** tray icon.
3. Copy any link (e.g. from a browser address bar).
4. Press **`Ctrl+Shift+C`**. The terminal prints `clip captured: <url> (queued <id>)`,
   then the drainer prints `drainer: picked=1 written=1 ...`.
5. The note appears in `<vault>/Inbox/`. Right-click the tray icon for the menu.

## Configuration

`~/.clipper/config.toml` is written on first run and then left alone. Every path may use
a leading `~`.

| Key | Default | Meaning |
|---|---|---|
| `vault.path` | auto-detected | Obsidian vault |
| `vault.inbox_subdir` | `Inbox` | subfolder notes are filed into |
| `provider.claude.command` / `.args` | `claude` / `["-p"]` | the extractor to spawn |
| `provider.claude.stdin` | `prompt-file` | `prompt-file` pipes the prompt into the child's stdin |
| `queue.dir` | `~/.clipper/queue` | `inbox.jsonl`, `archive.jsonl`, `deadletter.jsonl` |
| `queue.staging_dir` | `~/.clipper/staging` | batch prompts and generated notes |
| `queue.batch_size` | `5` | items per provider run |
| `queue.poll_interval_ms` | `5000` | drainer poll interval |
| `queue.timeout_s` | `120` | per-batch provider timeout |
| `server.bind` / `.port` | `127.0.0.1` / `7137` | reserved for the dashboard |
| `prompts.extract` | `~/.clipper/prompts/extract.md` | the extraction prompt |

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

CORS: only `http://127.0.0.1:7137`, `http://localhost:7137` and the app's own webview
origins get CORS headers; everything else is default-deny.

If the port is already in use Clipper logs the address it could not bind and exits
non-zero — it never silently picks another port.

## Self-check

```powershell
.\src-tauri\target\debug\clipper.exe --hag3-smoke
.\src-tauri\target\debug\clipper.exe --hag4-smoke
```

`--hag3-smoke` runs four assertions against an isolated temp config with a faked provider
(no network, no `claude`): three URLs become three frontmatter-valid notes, an already
archived URL is skipped, non-URL text is ignored, and garbage provider stdout deadletters
the batch.

`--hag4-smoke` starts the real server on a random port with a temp vault, writes three
notes *after* the watcher is live, and drives every route over real HTTP — listing, FTS
search, `today`, status (valid and invalid), tags, clips, stats, `healthz`, and the
dashboard. Exit code `0` means every check passed.

> The self-check is a flag on the real binary rather than a `tests/` target because a Rust
> test-harness binary linking the Tauri stack cannot be loaded on this windows-gnu
> toolchain (`STATUS_ENTRYPOINT_NOT_FOUND`), while `clipper.exe` itself loads fine. This
> way the checks run against the code that actually ships.

## Layout

```
src/                    vanilla HTML + CSS + JS frontend (no build step)
src/ui/                 the dashboard (embedded in the binary and served at /)
src-tauri/src/
  main.rs               entry point; `--hag<N>-smoke` runs a self-check
  lib.rs                Tauri wiring: boot, tray, hotkey, drainer, indexer, server
  config.rs             ~/.clipper/config.toml schema, paths, load/save
  vault.rs              Obsidian vault auto-detect
  queue.rs              JSONL append/rotate, SHA-256 dedupe index
  capture.rs            clipboard text -> deduped queue append
  drainer.rs            batching, prompt build, provider spawn, vault write
  frontmatter.rs        note frontmatter parse + filename id recovery
  db.rs                 SQLite schema, FTS5, upsert and queries
  vaultwatch.rs         watches the vault, keeps the index in step
  server.rs             axum REST API + dashboard
  prompts.rs            the default extract prompt
  selfcheck.rs          the per-slice self-checks
  clock.rs              UTC timestamps
```
