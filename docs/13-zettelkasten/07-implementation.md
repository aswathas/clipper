# What we shipped

- `db::related_by_tags(id, limit)` — items sharing at least one
  tag with `id`, ranked by overlap.
- `db::backlinks(id, limit)` — items whose body contains
  `[[<id>]]`.
- `db::extract_wikilinks(body)` — `[[id]]` references in any
  body. Used by future features.
- `links::backfill_related_section(id, path)` — appends a
  `## Related` section to a note with `[[id]]` links.
- `links::rebuild_index(vault)` — writes the master
  `Zettelkasten.md`.

The indexer calls `backfill_related_section` after every save.
The drainer does not (the note may not be in the index yet).
