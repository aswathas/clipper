//! Minimal YAML frontmatter for Clipper notes.
//!
//! Deliberately small: flat `key: value` pairs plus the `tags: [a, b]` inline
//! list the extraction prompt produces. Not a general YAML parser.

use std::collections::BTreeMap;

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    fields: BTreeMap<String, String>,
}

impl Frontmatter {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    pub fn fields(&self) -> &BTreeMap<String, String> {
        &self.fields
    }

    pub fn tags(&self) -> Vec<String> {
        self.get("tags").map(parse_inline_list).unwrap_or_default()
    }
}

/// Split a note into `(raw_frontmatter, body)` if a `---` block is at the top.
pub fn split(text: &str) -> (Option<&str>, &str) {
    let Some(rest) = text.strip_prefix("---") else {
        return (None, text);
    };
    let Some(rest) = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
    else {
        return (None, text);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, text);
    };
    let block = &rest[..end];
    let after = &rest[end + 4..];
    let body = after
        .strip_prefix("\r\n")
        .or_else(|| after.strip_prefix('\n'))
        .unwrap_or(after);
    (Some(block), body)
}

/// Parse the `---` delimited block at the top of a note, if there is one.
pub fn parse(text: &str) -> Option<Frontmatter> {
    let (block, _) = split(text);
    let block = block?;

    let mut fields = BTreeMap::new();
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            fields.insert(
                key.trim().to_ascii_lowercase(),
                value.trim().trim_matches('"').trim_matches('\'').to_string(),
            );
        }
    }

    Some(Frontmatter { fields })
}

/// `[a, b, "c"]` -> `["a", "b", "c"]`.
pub fn parse_inline_list(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// ISO-8601 -> unix seconds.
pub fn parse_iso8601(raw: &str) -> Option<i64> {
    OffsetDateTime::parse(raw.trim(), &Rfc3339)
        .ok()
        .map(|when| when.unix_timestamp())
}

/// Recover `(id, source)` from a note filename.
///
/// The writer names notes `<YYYY-MM-DD_HHMM>_<source>_<id>.md`, so the last two
/// underscore-separated fields are the source and the id. This is what makes
/// re-indexing idempotent: the id is stable across runs.
pub fn parse_note_filename(file_name: &str) -> Option<(String, String)> {
    let stem = file_name.strip_suffix(".md")?;
    let mut parts = stem.rsplitn(3, '_');
    let id = parts.next().unwrap_or_default().to_string();
    let source = parts.next().unwrap_or_default().to_string();
    if id.is_empty() || source.is_empty() {
        return None;
    }
    Some((id, source))
}
