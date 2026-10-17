# ADR 005: markdown vault

**Status:** accepted.

**Context.** The vault holds every saved item. The choices are
a single SQLite blob, a folder of JSON, or a folder of markdown
with frontmatter.

**Decision.** Markdown with frontmatter.

**Consequences.** The user can grep, edit, and version the
vault. The drainer writes the file; the indexer reads it. The
downside is the parser; mitigated by `pulldown-cmark`.

**Alternatives.** JSON (lossy, ugly), SQLite blob (opaque to
the user).
