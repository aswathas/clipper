# Wikilink syntax

Clipper recognizes two wikilink forms:

```
[[id]]                          # link to id with default label
[[id|Title text]]               # link to id with custom label
```

Obsidian uses the same syntax. Open the same vault in Obsidian and
the links resolve. Clipper's API uses the id directly; the label
is for human reading.

Wikilinks inside a note's body are extracted by
`db::extract_wikilinks`. Wikilinks appended by the indexer live in
the `## Related` section.
