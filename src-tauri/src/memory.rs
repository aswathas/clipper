//! The second brain: `soul.md`, `user.md`, `memory.md`.
//!
//! The three files live in `<state dir>/memory/`, are seeded from
//! `src-tauri/seed/` on first run, and are read once at startup. Their contents
//! are prepended to **every** Claude prompt, so the persona and the user's
//! context always come before the task.

use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use crate::config;

/// File stem for the persona.
pub const SOUL: &str = "soul";
/// File stem for the user's own context.
pub const USER: &str = "user";
/// File stem for the working memory.
pub const MEMORY: &str = "memory";

const DEFAULT_SOUL: &str = include_str!("../seed/soul.md");
const DEFAULT_USER: &str = include_str!("../seed/user.md");
const DEFAULT_MEMORY: &str = include_str!("../seed/memory.md");

/// The three parts, as read from disk.
#[derive(Debug, Clone)]
pub struct Memory {
    pub soul: String,
    pub user: String,
    pub memory: String,
}

pub fn default_soul() -> &'static str {
    DEFAULT_SOUL
}

pub fn default_user() -> &'static str {
    DEFAULT_USER
}

pub fn default_memory() -> &'static str {
    DEFAULT_MEMORY
}

/// `<state dir>/memory`.
pub fn memory_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("memory")
}

fn part_path(state_dir: &Path, name: &str) -> PathBuf {
    memory_dir(state_dir).join(format!("{name}.md"))
}

/// Write any missing part from the embedded default. Idempotent.
pub fn seed_defaults(state_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(memory_dir(state_dir))?;
    for (name, default) in [
        (SOUL, DEFAULT_SOUL),
        (USER, DEFAULT_USER),
        (MEMORY, DEFAULT_MEMORY),
    ] {
        let path = part_path(state_dir, name);
        if !path.exists() {
            fs::write(&path, default)?;
        }
    }
    Ok(())
}

/// Seed anything missing, then read all three parts.
pub fn load_now(state_dir: &Path) -> io::Result<Memory> {
    seed_defaults(state_dir)?;
    Ok(Memory {
        soul: fs::read_to_string(part_path(state_dir, SOUL))?,
        user: fs::read_to_string(part_path(state_dir, USER))?,
        memory: fs::read_to_string(part_path(state_dir, MEMORY))?,
    })
}

/// Awaitable form of [`load_now`], for callers already on a runtime.
pub async fn load(state_dir: &Path) -> io::Result<Memory> {
    let dir = state_dir.to_path_buf();
    match tokio::task::spawn_blocking(move || load_now(&dir)).await {
        Ok(result) => result,
        Err(e) => Err(io::Error::other(e)),
    }
}

/// The block prepended to every Claude prompt.
pub fn render_header(memory: &Memory) -> String {
    format!(
        "# Soul\n{}\n\n# User\n{}\n\n# Memory\n{}\n",
        memory.soul.trim_end(),
        memory.user.trim_end(),
        memory.memory.trim_end()
    )
}

impl Memory {
    /// Persona + context, then a `---` fenced copy of the task's own system
    /// prompt. The caller appends the per-batch instructions after this.
    pub fn build_claude_prompt(&self, extra_system: &str) -> String {
        format!(
            "{}\n---\n{}\n---\n",
            render_header(self),
            extra_system.trim()
        )
    }
}

fn write_part(state_dir: &Path, name: &str, content: &str) -> io::Result<()> {
    fs::create_dir_all(memory_dir(state_dir))?;
    let path = part_path(state_dir, name);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &path)
}

/// Write one part back atomically, under the active state directory.
pub fn save_part(name: &str, content: &str) -> io::Result<()> {
    write_part(&config::state_dir(), name, content)
}

/// Same as [`save_part`] but against an explicit state directory.
pub fn save_part_at(state_dir: &Path, name: &str, content: &str) -> io::Result<()> {
    write_part(state_dir, name, content)
}
