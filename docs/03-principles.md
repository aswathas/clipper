# Principles

1. **Local first.** The vault is a folder of plain markdown. The
   index is a single SQLite file. Network is a feature, not a
   requirement.
2. **Boring tech.** Tauri, SQLite, axum, vanilla JS. The
   "framework of the month" costs more than it buys at this scale.
3. **One concept, one commit.** Every change in `git log` should be
   readable in a single sentence.
4. **Mocks behind `?mock=1`.** When the backend is slow or missing,
   the UI demos with realistic data instead of breaking.
5. **No silent failures.** Every error path prints and writes a log
   line. The user is never stuck wondering.
6. **The keyboard wins.** Every action has a hotkey. The mouse is
   for navigation, not for getting work done.
