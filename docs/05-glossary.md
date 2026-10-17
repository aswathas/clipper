# Glossary

- **Vault** — the root folder where every saved item is written as a
  markdown file. The user picks it on first run.
- **Clip** — a single captured item (URL, image, or text).
- **Queue** — the in-memory list of clips awaiting the drainer.
- **Drainer** — the async task that turns a queue entry into a
  markdown file plus a SQLite row.
- **Muse** — the passive watcher that observes foreground windows
  and writes a one-line activity log.
- **Stapler** — the floating mascot that anchors Muse to the user's
  screen.
- **Skill** — a folder under `<vault>/Skills/` that bundles a system
  prompt, a tool schema, and an optional script.
- **Soul** — a single markdown file that holds the user-facing
  voice and the agent-facing instructions.
- **Memory** — a single markdown file that holds the user's
  long-term context (people, projects, preferences).
- **HAG-NN** — internal ticket id used on the kanban.
