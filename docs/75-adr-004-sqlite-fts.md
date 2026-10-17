# ADR 004: SQLite + FTS5

**Status:** accepted.

**Context.** We need a search index over a personal vault. The
candidates are SQLite FTS5, Tantivy (Rust), and a hosted
search.

**Decision.** SQLite + FTS5 + the bundled `rusqlite` crate.

**Consequences.** One file, no service, fast. The downside is
the lack of a vector index; deferred to v0.9 with `sqlite-vec`.

**Alternatives.** Tantivy (great, but a separate process and a
new build dep), hosted (privacy, latency).
