# Architecture: musewatch

```text
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
