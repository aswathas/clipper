# Architecture: clipboard

Two capture paths:

- **Hotkey** — `Ctrl+Shift+C` reads the current clipboard via
  `arboard`, hands the text to `capture::capture`, which dedupes
  and enqueues.
- **Listener** — `AddClipboardFormatListener` on a hidden
  message-only window. Posts a wake-up to the drainer whenever
  the clipboard changes while the foreground is `whatsapp.exe`.

The hotkey path is opt-in (the user pressed a key). The listener
path is opt-out (the user toggled `capture_whatsapp` in Muse).
