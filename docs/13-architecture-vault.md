# Architecture: vault

```
<vault>/
├── Activity/<YYYY-MM-DD>.md     # Muse watcher output
├── Calls/<YYYY-MM-DD>.md         # call summaries (v0.7+)
├── WhatsApp/<YYYY-MM-DD>.md      # captured WhatsApp messages
├── Inbox/<id>.md                 # pending drainer output
├── Library/<id>.md               # settled, indexed items
├── Memory/user.md                # long-term user context
├── Skills/<slug>/SKILL.md        # installed skills
└── Soul.md                       # voice + persona
```

The drainer writes to `Inbox/` first, then promotes to `Library/`
once the SQLite row is committed. A power loss between the two
leaves the file in `Inbox/` — recoverable, never lost.
