//! Obsidian vault auto-detection, run once at startup before the tray appears.

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::home_dir;

/// Outcome of a vault scan.
#[derive(Debug, Clone)]
pub struct Detected {
    /// The vault Clipper will use.
    pub path: PathBuf,
    /// Every vault found. More than one means the choice was arbitrary.
    pub candidates: Vec<PathBuf>,
    /// True when no vault existed and Clipper created the fallback.
    pub created: bool,
}

/// An Obsidian vault is any directory holding a `.obsidian` folder.
fn is_vault(dir: &Path) -> bool {
    dir.join(".obsidian").is_dir()
}

fn push_unique(out: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !out.contains(&candidate) {
        out.push(candidate);
    }
}

/// `CLIPPER_HOME`, when set, is the only place Clipper looks for or creates a
/// vault. That keeps `scripts/smoke.ps1` (and portable installs) away from any
/// real vault on the machine.
fn scoped_root() -> Option<PathBuf> {
    std::env::var_os("CLIPPER_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// `dir` itself, plus each of its immediate subdirectories.
fn scan(dir: &Path, out: &mut Vec<PathBuf>) {
    if is_vault(dir) {
        push_unique(out, dir.to_path_buf());
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && is_vault(&path) {
                push_unique(out, path);
            }
        }
    }
}

/// Scan for an Obsidian vault.
///
/// Exactly one -> use it. Several -> use the first (sorted) and log them all.
/// None -> create `<fallback root>/ClipperVault` (with `.obsidian`) and use it.
pub fn detect() -> Detected {
    let mut candidates: Vec<PathBuf> = Vec::new();

    let fallback_root = match scoped_root() {
        Some(root) => {
            scan(&root, &mut candidates);
            root
        }
        None => {
            let home = home_dir();
            for base in [
                home.join("Documents"),
                home.join("ObsidianVaults"),
                home.join("Notes"),
                home.join("repos"),
                home.join("OneDrive").join("Documents"),
                home.clone(),
            ] {
                scan(&base, &mut candidates);
            }
            home.join("Documents")
        }
    };

    candidates.sort();

    if candidates.is_empty() {
        let fallback = fallback_root.join("ClipperVault");
        let _ = fs::create_dir_all(fallback.join(".obsidian"));
        return Detected {
            path: fallback,
            candidates: Vec::new(),
            created: true,
        };
    }

    if candidates.len() > 1 {
        for candidate in &candidates {
            println!("vault candidate: {}", candidate.display());
        }
        println!(
            "vault: {} candidates found, using {}",
            candidates.len(),
            candidates[0].display()
        );
    }

    Detected {
        path: candidates[0].clone(),
        candidates,
        created: false,
    }
}
