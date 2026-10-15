//! Clipper configuration: `~/.clipper/config.toml`.
//!
//! Written once on first run (with the resolved vault path) and left alone
//! afterwards. Every path in the schema may use a leading `~`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// The user's home directory.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Root of Clipper's own state: `~/.clipper`.
pub fn state_dir() -> PathBuf {
    home_dir().join(".clipper")
}

/// `~/.clipper/config.toml`.
pub fn config_path() -> PathBuf {
    state_dir().join("config.toml")
}

/// Expand a leading `~` / `~/` into the home directory.
pub fn expand_tilde(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw == "~" {
        return home_dir();
    }
    for prefix in ["~/", "~\\"] {
        if let Some(rest) = raw.strip_prefix(prefix) {
            return home_dir().join(rest);
        }
    }
    PathBuf::from(raw)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub vault: VaultCfg,
    pub provider: ProviderCfg,
    pub queue: QueueCfg,
    pub server: ServerCfg,
    pub prompts: PromptsCfg,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault: VaultCfg::default(),
            provider: ProviderCfg::default(),
            queue: QueueCfg::default(),
            server: ServerCfg::default(),
            prompts: PromptsCfg::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VaultCfg {
    /// Resolved at first run by [`crate::vault::detect`].
    pub path: String,
    pub inbox_subdir: String,
}

impl Default for VaultCfg {
    fn default() -> Self {
        Self {
            path: String::new(),
            inbox_subdir: "Inbox".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderCfg {
    pub claude: ClaudeCfg,
    pub image: ImageCfg,
}

impl Default for ProviderCfg {
    fn default() -> Self {
        Self {
            claude: ClaudeCfg::default(),
            image: ImageCfg::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeCfg {
    pub command: String,
    pub args: Vec<String>,
    /// `prompt-file` pipes the batch prompt into the child's stdin.
    pub stdin: String,
}

impl Default for ClaudeCfg {
    fn default() -> Self {
        Self {
            command: "claude".into(),
            args: vec!["-p".into()],
            stdin: "prompt-file".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageCfg {
    pub backend: String,
    pub prompt_hint: String,
}

impl Default for ImageCfg {
    fn default() -> Self {
        Self {
            backend: "claude-via-mcp".into(),
            prompt_hint: "Use mcp__MiniMax__understand_image to caption. Return same markdown schema."
                .into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueCfg {
    pub dir: String,
    pub staging_dir: String,
    pub batch_size: usize,
    pub poll_interval_ms: u64,
    pub timeout_s: u64,
}

impl Default for QueueCfg {
    fn default() -> Self {
        Self {
            dir: "~/.clipper/queue".into(),
            staging_dir: "~/.clipper/staging".into(),
            batch_size: 5,
            poll_interval_ms: 5000,
            timeout_s: 120,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerCfg {
    pub bind: String,
    pub port: u16,
}

impl Default for ServerCfg {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".into(),
            port: 7137,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PromptsCfg {
    pub extract: String,
}

impl Default for PromptsCfg {
    fn default() -> Self {
        Self {
            extract: "~/.clipper/prompts/extract.md".into(),
        }
    }
}

impl Config {
    /// Defaults with the vault path already resolved.
    pub fn for_vault(vault_path: &Path) -> Self {
        let mut cfg = Self::default();
        cfg.vault.path = vault_path.to_string_lossy().into_owned();
        cfg
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let text = fs::read_to_string(path)?;
        toml::from_str(&text).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{}: {e}", path.display()),
            )
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, text)
    }

    pub fn vault_path(&self) -> PathBuf {
        expand_tilde(&self.vault.path)
    }

    pub fn queue_dir(&self) -> PathBuf {
        expand_tilde(&self.queue.dir)
    }

    pub fn staging_dir(&self) -> PathBuf {
        expand_tilde(&self.queue.staging_dir)
    }

    pub fn extract_prompt_path(&self) -> PathBuf {
        expand_tilde(&self.prompts.extract)
    }

    pub fn inbox_jsonl(&self) -> PathBuf {
        self.queue_dir().join("inbox.jsonl")
    }

    pub fn processing_jsonl(&self) -> PathBuf {
        self.queue_dir().join("inbox.processing")
    }

    pub fn archive_jsonl(&self) -> PathBuf {
        self.queue_dir().join("archive.jsonl")
    }

    pub fn deadletter_jsonl(&self) -> PathBuf {
        self.queue_dir().join("deadletter.jsonl")
    }

    /// Where finished notes land inside the vault.
    pub fn vault_inbox(&self) -> PathBuf {
        self.vault_path().join(&self.vault.inbox_subdir)
    }

    /// Create every directory Clipper owns. Safe to call repeatedly.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for dir in [
            self.queue_dir(),
            self.staging_dir(),
            self.vault_inbox(),
            self.extract_prompt_path()
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| state_dir().join("prompts")),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }
}
