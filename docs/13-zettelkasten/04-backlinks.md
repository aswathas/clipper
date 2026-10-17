# Backlinks

A backlink is a note that points to the current note. Obsidian's
backlinks panel lists them; Clipper exposes them at
`/api/items/{id}/backlinks`.

The indexer computes backlinks live: when a note is saved, the
indexer walks every other note's body and asks "does this mention
my id?" The answer is the backlink graph.
