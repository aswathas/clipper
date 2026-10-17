# The master index

A single markdown file at `<vault>/Zettelkasten.md` lists every
note with its id, title, and tags. One line per note.

Rebuild it from the API:

```
POST /api/zettelkasten/rebuild
```

The index is the entry point. Open it, find what you forgot, follow
the link.
