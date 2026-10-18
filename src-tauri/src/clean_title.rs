//! Title quality: normalize titles written by the drainer and the Muse watcher.
//!
//! The PowerShell foreground capture returns empty strings for some windows
//! (truncated to "?" or a question mark), and `claude -p` sometimes echoes
//! the article URL when the page is paywalled. This module normalizes a
//! raw title into something a human can read at a glance.

const PLACEHOLDER_TOKENS: &[&str] = &[
    "?", "??", "???", "—", "--", "untitled", "new tab", "loading…", "loading",
    "page not found", "404", "blank", "(untitled)", "no title",
    "external ai agents", "claude code",
];

/// Returns true when `title` is missing, a placeholder, or otherwise unusable.
pub fn is_garbled(title: &str) -> bool {
    let t = title.trim();
    if t.is_empty() || t.len() < 3 {
        return true;
    }
    let lower = t.to_lowercase();
    if PLACEHOLDER_TOKENS.iter().any(|p| lower == *p) {
        return true;
    }
    // Mostly punctuation.
    let alnum = t.chars().filter(|c| c.is_alphanumeric()).count();
    if alnum < t.chars().count() / 2 {
        return true;
    }
    // Mostly whitespace or repeated chars.
    if lower.chars().collect::<std::collections::HashSet<_>>().len() < 3 {
        return true;
    }
    false
}

/// Strip a leading `?` and trailing dash/suffix, collapse whitespace.
pub fn polish(title: &str) -> String {
    let mut s = title.trim().to_string();
    // Drop leading `?` characters (PowerShell default for empty window titles).
    while s.starts_with('?') || s.starts_with('-') {
        s.remove(0);
        s = s.trim_start().to_string();
    }
    // Drop trailing browser suffixes that add no information.
    for suffix in [
        " - Brave", " - Chrome", " - Edge", " - Firefox", " - YouTube",
        " — Brave", " — Chrome",
    ] {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped.to_string();
        }
    }
    // Collapse runs of whitespace.
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Best-effort title from a body: take the first non-empty heading line,
/// stripping the leading `#` and any surrounding whitespace. Falls back
/// to the first non-empty paragraph if no heading exists.
pub fn from_body(body: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            let title = rest.trim();
            if !is_garbled(title) {
                return Some(polish(title));
            }
        } else if let Some(rest) = t.strip_prefix("## ") {
            let title = rest.trim();
            if !is_garbled(title) {
                return Some(polish(title));
            }
        }
    }
    for line in body.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("```") {
            continue;
        }
        let candidate: String = t.chars().take(80).collect();
        if !is_garbled(&candidate) {
            return Some(polish(&candidate));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_question_mark() {
        assert!(is_garbled("?"));
        assert!(is_garbled("? External AI agents"));
        assert!(is_garbled(""));
        assert!(is_garbled("  "));
    }

    #[test]
    fn detects_placeholders() {
        assert!(is_garbled("New Tab"));
        assert!(is_garbled("Loading…"));
        assert!(is_garbled("404"));
    }

    #[test]
    fn passes_good_titles() {
        assert!(!is_garbled("Cargo.toml — editing deps"));
        assert!(!is_garbled("How to become a smart contract auditor"));
    }

    #[test]
    fn polish_strips_leading_question() {
        assert_eq!(polish("? External AI agents"), "External AI agents");
        assert_eq!(polish("--- foo"), "foo");
    }

    #[test]
    fn polish_strips_browser_suffix() {
        assert_eq!(polish("Foo - Brave"), "Foo");
        assert_eq!(polish("Foo — Chrome"), "Foo");
    }

    #[test]
    fn from_body_picks_first_heading() {
        let body = "---\nid: x\ntitle: old\n---\n\n# Real Title Here\n\nbody";
        assert_eq!(from_body(body), Some("Real Title Here".to_string()));
    }

    #[test]
    fn from_body_falls_back_to_paragraph() {
        let body = "Some opening sentence that is long enough to be a title candidate.";
        let got = from_body(body).unwrap();
        assert!(got.starts_with("Some opening sentence"));
    }
}
