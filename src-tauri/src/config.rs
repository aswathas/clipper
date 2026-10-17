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

/// Root of Clipper's own state: `~/.clipper`, or `$CLIPPER_HOME` when set.
///
/// `CLIPPER_HOME` is what makes the app portable and what lets `scripts/smoke.ps1`
/// run a whole session without touching the user's real state.
pub fn state_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("CLIPPER_HOME").filter(|value| !value.is_empty()) {
        return PathBuf::from(custom);
    }
    home_dir().join(".clipper")
}

/// `~/.clipper/config.toml`.
pub fn config_path() -> PathBuf {
    state_dir().join("config.toml")
}

/// Expand a leading `~` / `~/` into the home directory.
///
/// `~/.clipper` is special-cased to [`state_dir`]: that path *is* Clipper's
/// state directory, so it has to follow `CLIPPER_HOME` rather than the literal
/// home. Without this, the default `queue.dir` / `staging_dir` / `prompts`
/// paths would escape an isolated `CLIPPER_HOME` and touch the user's real
/// state.
pub fn expand_tilde(raw: &str) -> PathBuf {
    let raw = raw.trim();

    for prefix in ["~/.clipper", "~\\.clipper"] {
        if raw == prefix {
            return state_dir();
        }
        for separator in ['/', '\\'] {
            let with_separator = format!("{prefix}{separator}");
            if let Some(rest) = raw.strip_prefix(&with_separator) {
                return state_dir().join(rest);
            }
        }
    }

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
    pub skills: SkillsCfg,
    pub memory: MemoryCfg,
    pub muse: MuseCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MuseCfg {
    /// Process names (case-insensitive, without `.exe`) that the Muse watcher
    /// should skip without summarizing — e.g. `["zoom", "whatsapp", "slack"]`.
    pub skip_processes: Vec<String>,
    /// Whether to group consecutive activity entries for the same `(process, normalized_title)`.
    pub smart_dedup: bool,
    /// How long the same task must stay stable before we record a new entry.
    /// `0` = react on every foreground change; otherwise seconds.
    pub debounce_secs: u32,
    /// Capture a screenshot of the foreground HWND and include it in the
    /// `claude -p` prompt for richer summaries. Disabled by default; needs
    /// `claude` to accept image attachments via stdin.
    pub screenshots: bool,
    /// Auto-record system + mic audio when a Zoom/Teams call is detected.
    /// The recording is sent to a local Whisper-ggml model (set up by the
    /// ML/AI engineer); the transcript appends to today's activity.
    pub record_calls: bool,
    /// Capture WhatsApp text messages via UI automation. The watcher
    /// reads the foreground WhatsApp window, copies the most recent
    /// incoming message via UI Automation, and appends it to today's note.
    pub capture_whatsapp: bool,
    /// Foreground process names (case-insensitive, no `.exe`) that should
    /// trigger audio capture. The audiocapture worker records 30-second
    /// windows, transcribes with the local Whisper model, and appends the
    /// transcript to `<vault>/Calls/<date>.md`.
    pub calls_processes: Vec<String>,
    /// Path to a `whisper-cli` binary. Built from whisper.cpp; not bundled.
    pub transcription_cli: String,
    /// Path to a ggml Whisper model (e.g. `ggml-base.en.bin`).
    pub transcription_model: String,
}

impl Default for MuseCfg {
    fn default() -> Self {
        Self {
            skip_processes: vec![
                "lockapp".into(),
                "screensaver".into(),
                "zoom".into(),
                "whatsapp".into(),
                "slack".into(),
                "telegram".into(),
            ],
            smart_dedup: true,
            debounce_secs: 10,
            screenshots: false,
            record_calls: false,
            capture_whatsapp: false,
            calls_processes: vec![
                "zoom".into(),
                "teams".into(),
                "slack".into(),
                "discord".into(),
                "skype".into(),
                "webexmta".into(),
            ],
            transcription_cli: String::new(),
            transcription_model: String::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            vault: VaultCfg::default(),
            provider: ProviderCfg::default(),
            queue: QueueCfg::default(),
            server: ServerCfg::default(),
            prompts: PromptsCfg::default(),
            skills: SkillsCfg::default(),
            memory: MemoryCfg::default(),
            muse: MuseCfg::default(),
        }
    }
}

/// Where `soul.md`, `user.md` and `memory.md` live.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryCfg {
    pub dir: String,
}

impl Default for MemoryCfg {
    fn default() -> Self {
        Self {
            dir: "~/.clipper/memory".into(),
        }
    }
}

/// Where skill markdown files live.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillsCfg {
    pub dir: String,
}

impl Default for SkillsCfg {
    fn default() -> Self {
        Self {
            dir: "~/.clipper/skills".into(),
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
    /// Optional provider override for image batches. When absent, images are
    /// sent to `[provider.claude]` like everything else.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

impl Default for ImageCfg {
    fn default() -> Self {
        Self {
            backend: "claude-via-mcp".into(),
            prompt_hint: "Use mcp__MiniMax__understand_image to caption. Return same markdown schema."
                .into(),
            command: None,
            args: None,
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

    /// The soul/user/memory directory.
    pub fn memory_dir(&self) -> PathBuf {
        expand_tilde(&self.memory.dir)
    }

    /// `<state dir>`, for callers that need it directly.
    pub fn state_dir(&self) -> PathBuf {
        state_dir()
    }

    pub fn skills_dir(&self) -> PathBuf {
        expand_tilde(&self.skills.dir)
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
            self.skills_dir(),
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
