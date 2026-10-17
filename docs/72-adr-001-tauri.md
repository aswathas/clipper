# ADR 001: Tauri 2

**Status:** accepted.

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
