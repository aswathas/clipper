//! Pre-write redaction: scrub secrets, PII, and credentials from clip bodies
//! before they land in the vault.
//!
//! The drainer calls [`redact`] on every body. The default patterns catch the
//! common accidental leaks (API keys, credit cards, JWTs, SSNs, emails).
//! Users can extend the list in `~/.clipper/config.toml` under `[redact]`.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default)]
pub struct RedactCfg {
    pub extra_patterns: Vec<String>,
    pub disabled: bool,
}

struct Compiled {
    patterns: Vec<(&'static str, Regex)>,
}

static COMPILED: OnceLock<Compiled> = OnceLock::new();

fn compiled(cfg: &RedactCfg) -> &'static Compiled {
    COMPILED.get_or_init(|| {
        let mut patterns: Vec<(&'static str, Regex)> = vec![
            ("anthropic-key", Regex::new(r"sk-ant-[A-Za-z0-9_\-]{20,}").unwrap()),
            ("openai-key", Regex::new(r"sk-[A-Za-z0-9_\-]{20,}").unwrap()),
            ("aws-access-key", Regex::new(r"AKIA[0-9A-Z]{16}").unwrap()),
            ("github-token", Regex::new(r"ghp_[A-Za-z0-9]{30,}").unwrap()),
            ("github-fine", Regex::new(r"github_pat_[A-Za-z0-9_]{60,}").unwrap()),
            ("slack-token", Regex::new(r"xox[baprs]-[A-Za-z0-9-]{10,}").unwrap()),
            ("google-api-key", Regex::new(r"AIza[0-9A-Za-z_\-]{35}").unwrap()),
            ("jwt", Regex::new(r"eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}").unwrap()),
            ("private-key", Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]+?-----END [A-Z ]*PRIVATE KEY-----").unwrap()),
            ("credit-card", Regex::new(r"\b(?:\d[ -]?){13,19}\b").unwrap()),
            ("ssn", Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()),
            ("email", Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap()),
            ("phone-intl", Regex::new(r"\+\d{1,3}[\s\-]?\d{6,14}").unwrap()),
        ];
        for raw in &cfg.extra_patterns {
            if let Ok(re) = Regex::new(raw) {
                patterns.push(("user", re));
            }
        }
        Compiled { patterns }
    })
}

/// Run every pattern over `body` and replace matches with `[REDACTED:<label>]`.
/// Returns the cleaned text and a list of labels that fired.
pub fn redact(body: &str, cfg: &RedactCfg) -> (String, Vec<String>) {
    if cfg.disabled {
        return (body.to_string(), Vec::new());
    }
    let compiled = compiled(cfg);
    let mut out = body.to_string();
    let mut fired: Vec<String> = Vec::new();
    for (label, re) in &compiled.patterns {
        if re.is_match(&out) {
            fired.push((*label).to_string());
            out = re.replace_all(&out, format!("[REDACTED:{label}]")).into_owned();
        }
    }
    (out, fired)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_anthropic_key() {
        let (clean, fired) = redact("key is sk-ant-abcdef0123456789ABCDEF", &RedactCfg::default());
        assert!(clean.contains("[REDACTED:anthropic-key]"));
        assert_eq!(fired, vec!["anthropic-key".to_string()]);
    }

    #[test]
    fn strips_email() {
        let (clean, fired) = redact("mail me at a@b.co", &RedactCfg::default());
        assert!(clean.contains("[REDACTED:email]"));
        assert!(fired.contains(&"email".to_string()));
    }

    #[test]
    fn disabled_returns_unchanged() {
        let cfg = RedactCfg { disabled: true, ..Default::default() };
        let (clean, fired) = redact("sk-ant-abcdef0123456789ABCDEF", &cfg);
        assert_eq!(clean, "sk-ant-abcdef0123456789ABCDEF");
        assert!(fired.is_empty());
    }
}
