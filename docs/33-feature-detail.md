# Feature: detail slide-over

Click any card. The right-side slide-over shows the full markdown
body, the frontmatter, the tags, the source URL, and the back
links (other items that link here).

Edit the body in place. The save is `PUT /api/items/:id` with
`If-Match: <mtime>` for optimistic concurrency. 412 means
someone else (you, in another tab) saved first.
