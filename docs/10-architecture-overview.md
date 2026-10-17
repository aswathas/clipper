# Architecture overview

Clipper is two things in one binary:

1. A **headless daemon** written in Rust. It owns the tray icon, the
   global hotkeys, the clipboard listener, the foreground watcher,
   and the SQLite index. It does not own a window.
2. A **web app** served on `127.0.0.1:7137` from the same binary.
   Every screen the user sees — Today, Library, Inbox, Ask, Muse
   settings — is a static page that talks to the daemon over the
   REST API.

The split is deliberate. The desktop is for capture; the browser is
for review. Capture must never be blocked by a hung tab.
