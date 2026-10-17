# Feature: clipboard capture

Press `Ctrl+Shift+C` from any focus. Clipper reads the current
clipboard, dedupes by URL host + canonical path, and writes a new
markdown file with frontmatter:

```markdown
---
id: 9b0e3a...
url: https://example.com/article
title: An interesting read
host: example.com
saved_at: 1760657412
tags: [tech, reading]
---

# An interesting read

[clipped text from the page]
```

If the URL is already in the index, the tray tooltip flashes
"Already saved: example.com" and nothing else happens.
