# Clipper

A clipboard-capture desktop app built with [Tauri 2](https://tauri.app) (Rust backend,
vanilla HTML + JS frontend — no framework, no bundler).

**This slice (HAG-2) is the scaffold:** the app starts hidden, lives in the system tray,
and a global hotkey reads the clipboard. The capture queue, vault writer and Claude
dispatch land in later slices. See [`SPEC.md`](./SPEC.md) for the full design.

## What works today

- **System tray** with three menu items: *Show dashboard* (placeholder), *Settings*
  (placeholder), *Quit*.
- **Global hotkey `Ctrl+Shift+C`** — anywhere in Windows, reads the clipboard. If the
  text looks like a URL (`http://` / `https://`), it prints to the console:
  `clip captured: <url>`.
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
4. Press **`Ctrl+Shift+C`**. The terminal running the app prints `clip captured: <url>`.
5. Right-click the tray icon to see *Show dashboard*, *Settings*, *Quit*.

## Layout

```
src/                 vanilla HTML + CSS + JS frontend (no build step)
src-tauri/
  src/main.rs        thin binary entry point
  src/lib.rs         tray, global hotkey, clipboard read
  tauri.conf.json    product name, identifier, hidden window
  Cargo.toml         Rust dependencies
```
