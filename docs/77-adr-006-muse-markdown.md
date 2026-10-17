# ADR 006: Muse writes markdown

**Status:** accepted.

**Context.** The Muse watcher emits one entry per focus change.
The storage choice is a SQLite log, a JSON-lines file, or a
markdown log.

**Decision.** One markdown file per day, append-only.

**Consequences.** The user can read the log in any editor. The
dashboard reads the file with a tiny parser. The downside is
the parse cost; negligible at 200 lines per day.
