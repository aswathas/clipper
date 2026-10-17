#!/usr/bin/env python3
"""Write Zettelkasten docs. One file per concept, one commit per file."""
import os

ROOT = r"C:\Users\aswat\Downloads\Projects\SmartClipboard\docs\13-zettelkasten"
os.makedirs(ROOT, exist_ok=True)

def w(name, body):
    with open(os.path.join(ROOT, name), "w", encoding="utf-8") as f:
        f.write(body)

H = lambda t: f"# {t}\n\n"

w("00-index.md", H("Zettelkasten") + """The Slip-Box Method, as Niklas Luhmann used it, applied to Clipper.

## Reading order
1. 01-method — what the method is
2. 02-atomic — one idea per note
3. 03-links — how notes link
4. 04-backlinks — how Obsidian renders them
5. 05-index — the master index
6. 06-followup — what we did not borrow
7. 07-implementation — what we shipped
8. 08-future — branching, sequences, digraph

Each file is one idea.
""")

w("01-method.md", H("The Zettelkasten method") + """Niklas Luhmann, a German sociologist, kept 90,000 paper notes in a
wooden slip-box. Each note held one idea, had a unique id, and pointed
to other notes. The graph was the system; the box was the storage.

Luhmann credited the slip-box with his prolific output. He said he
did not think; he asked the box.

The method works because linking is cheap, retrieval is free, and the
graph reveals patterns a list never does.
""")

w("02-atomic.md", H("Atomic notes") + """A note holds one idea. Not a topic. Not a chapter. One idea.

A clip is atomic by default: one URL, one capture, one thought. The
frontmatter captures the metadata; the body captures the idea.

Atomic notes are easier to link, easier to recombine, easier to
delete. They are the unit of recall.
""")

w("03-links.md", H("Links") + """A note points to other notes via `[[id]]` wikilinks. Obsidian
renders them as inline links; the dashboard renders them as a
connections panel.

Two notes are linked when one references the other in its body. The
link is typed by context: same tag, same host, same project, same
person.

Links are cheap. Make them generously.
""")

w("04-backlinks.md", H("Backlinks") + """A backlink is a note that points to the current note. Obsidian's
backlinks panel lists them; Clipper exposes them at
`/api/items/{id}/backlinks`.

The indexer computes backlinks live: when a note is saved, the
indexer walks every other note's body and asks "does this mention
my id?" The answer is the backlink graph.
""")

w("05-index.md", H("The master index") + """A single markdown file at `<vault>/Zettelkasten.md` lists every
note with its id, title, and tags. One line per note.

Rebuild it from the API:

```
POST /api/zettelkasten/rebuild
```

The index is the entry point. Open it, find what you forgot, follow
the link.
""")

w("06-followup.md", H("What we did not borrow") + """Luhmann's slip-box used a fixed physical address per note. We use
content-addressed ids (UUIDs) so the address is stable across
renames.

We did not borrow the idea of a "branch" — Luhmann split a note
into multiple notes when it had two ideas. Our drainer writes
atomic notes from the start.

We did not borrow the idea of a "sequence" — Luhmann numbered
notes to chain arguments. The user can do this with `[[id1]] →
[[id2]]` in the body if they want.
""")

w("07-implementation.md", H("What we shipped") + """- `db::related_by_tags(id, limit)` — items sharing at least one
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
""")

w("08-future.md", H("Future Zettelkasten work") + """- **Branching** — detect a note that has grown two ideas and
  suggest a split.
- **Sequences** — chain `[[id1]] -> [[id2]] -> [[id3]]` with a
  dedicated rendering.
- **Digraph view** — Obsidian's graph view, but local. Render the
  backlink graph as inline SVG in the dashboard.
- **Citation graph** — if a clip cites another clip, draw a
  thicker line. The web of references is the user's real second
  brain.
""")

w("09-vs-pkm.md", H("vs traditional PKM") + """Traditional PKM (Notion, Evernote) is a folder tree. Zettelkasten is
a graph.

Folders scale with hierarchy. Graphs scale with ideas. When the
user has 1,000 notes, the folder tree is a nightmare; the graph is
a playground.

Clipper is graph-first. The Library is a grid by recency; the
Connections panel is a graph by relatedness.
""")

w("10-vs-original.md", H("vs Luhmann's slip-box") + """Luhmann's slip-box was paper, hand-numbered, and never searched.
Clipper is digital, content-addressed, and FTS5-indexed.

The trade: digital is faster to write, slower to dwell. The paper
slip-box forced Luhmann to handle every card; the digital vault
lets the user ignore 99% of their notes.

The fix: serendipity. A daily digest that surfaces 5 forgotten
notes. A weekly review that asks "what did you link last week?"
""")

w("11-serendipity.md", H("Serendipity") + """A Zettelkasten earns its keep when the user finds a note they
forgot they wrote, in a context they did not expect.

Clipper surfaces this three ways:
1. The Connections panel on every item.
2. The daily digest picks three forgotten items.
3. The "this week" widget shows the most-touched graph cluster.

The first time the user finds a forgotten note, the method
clicks. The graph earns its keep.
""")

w("12-tags.md", H("Tags vs links") + """Tags are nouns. Links are sentences.

A tag says "this note is about rust." A link says "this note
contradicts the other note about rust."

Use tags for facets (host, topic, type). Use links for arguments
(claims, refutes, supports, follows from).

Clipper auto-creates links from shared tags. The user creates
links from arguments in the body.
""")

w("13-wikilinks.md", H("Wikilink syntax") + """Clipper recognizes two wikilink forms:

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
""")

w("14-no-fork.md", H("Why we are not Obsidian") + """Obsidian is the user's editor. Clipper is the user's inbox.

We do not replace Obsidian. We feed it markdown it already
understands. The user opens Obsidian when they want to write;
they open Clipper when they want to recall.

The slip-box lives in the vault, on disk, in plain text. Obsidian
owns the rendering. We own the capture.
""")

w("15-glossary.md", H("Zettelkasten glossary") + """- **Slip-box** — the physical or digital container of notes.
- **Atomic note** — one idea per note.
- **Folgezettel** — a follow-up note (Luhmann's "F" prefix).
- **Branch** — a split of one note into two.
- **Sequence** — an ordered chain of notes.
- **Backlink** — a note that points to the current note.
- **Index** — the master list of every note.
- **Serendipity** — the moment a forgotten note finds the user.
- **Tag** — a facet label (host, topic, type).
- **Link** — a typed reference between two notes.
""")

print("done")
