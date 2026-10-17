# Data flow

**Capture**
```
hotkey -> arboard -> capture::capture -> queue.append
clipboard listener (WhatsApp) -> append to <vault>/WhatsApp
```

**Watch**
```
SetWinEventHook -> mpsc -> musewatch -> claude -p -> <vault>/Activity
```

**Settle**
```
queue.pop -> drainer.write -> SQLite row + <vault>/Library/<id>.md
```

**Review**
```
browser -> GET /api/library -> SQLite query -> JSON
browser -> GET /api/items/:id -> SQLite row + <vault>/Library/<id>.md body
```

The boundaries are file paths and HTTP. Nothing in the daemon
talks to a cloud service. The summarizer talks to `claude -p`,
which is a local install of the Claude CLI.
