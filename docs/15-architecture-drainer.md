# Architecture: drainer

Single async task. Pops one item at a time. For each item:
1. Compute SHA-256 over the canonical text.
2. Insert the SQLite row (`INSERT OR IGNORE` on hash).
3. Write the markdown file with frontmatter.
4. If the row was new, fan out a webhook (configurable; off by
   default).

The drainer is the only writer. There is no second path. That
keeps the index honest and the lock surface tiny.
