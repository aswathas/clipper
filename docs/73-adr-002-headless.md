# ADR 002: headless daemon

**Status:** accepted.

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
