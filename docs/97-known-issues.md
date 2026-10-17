# Known issues

- **The watcher misses UWP apps** that do not fire Win32
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
