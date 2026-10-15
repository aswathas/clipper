//! The extraction prompt handed to the provider.

use std::{fs, io, path::Path};

/// Deployed to `~/.clipper/prompts/extract.md` on first run.
pub const DEFAULT_EXTRACT_PROMPT: &str = r#"You are Clipper's extractor. For each URL in the JSON list, fetch it, then return ONE markdown file path per line, in the same order. The file at each path must contain:

---
type: <article|video|repo|book|product|note|other>
source: <hostname>
url: <original url>
saved_at: <iso8601>
title: <page title or best guess>
summary: <one sentence - max 25 words>
tags: [<up to 5 lowercase single-word tags>]
---

# <title>

<summary>

## Key points
- <bullet>
- <bullet>
- <bullet>

## Notes
<anything else worth saving - quotes, code, links>

Write each .md to ~/.clipper/staging/<id>.md, then return the absolute path of each file on its own line, in input order, with no other output. Do not include code fences around the path list.
"#;

/// Read the extract prompt, writing the default first if it is missing.
pub fn ensure_extract_prompt(path: &Path) -> io::Result<String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    if !path.exists() {
        fs::write(path, DEFAULT_EXTRACT_PROMPT)?;
    }
    fs::read_to_string(path)
}
