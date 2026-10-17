# Compatibility

- **OS**: Windows 11 23H2 or later. Windows 10 22H2 may
  work but is not tested.
- **WebView2**: bundled with Windows 11. On Windows 10 it is a
  one-time install.
- **Claude CLI**: optional. Without it, the summarizer is a
  no-op and the watcher still writes titles.
- **Microphone**: required only for `record_calls`.
- **Disk**: 80 MB for the binary, 200 MB for the cache
  (screenshots + Whisper weights).
