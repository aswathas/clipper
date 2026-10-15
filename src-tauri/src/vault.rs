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

/// Scan the well-known locations, plus every first-level folder of the home
/// directory, for an Obsidian vault.
///
/// Exactly one -> use it. Several -> use the first (sorted) and log them all.
/// None -> create `~/Documents/ClipperVault` (with `.obsidian`) and use that.
pub fn detect() -> Detected {
    let home = home_dir();
    let bases = [
        home.join("Documents"),
        home.join("ObsidianVaults"),
        home.join("Notes"),
        home.join("repos"),
        home.join("OneDrive").join("Documents"),
    ];

    let mut candidates: Vec<PathBuf> = Vec::new();

    for base in &bases {
        if is_vault(base) {
            push_unique(&mut candidates, base.clone());
        }
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && is_vault(&path) {
                    push_unique(&mut candidates, path);
                }
            }
        }
    }

    // Any first-level folder of ~ that is itself a vault.
    if let Ok(entries) = fs::read_dir(&home) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && is_vault(&path) {
                push_unique(&mut candidates, path);
            }
        }
    }

    candidates.sort();

    match candidates.len() {
        0 => {
            let fallback = home.join("Documents").join("ClipperVault");
            let _ = fs::create_dir_all(fallback.join(".obsidian"));
            Detected {
                path: fallback,
                candidates: Vec::new(),
                created: true,
            }
        }
        1 => Detected {
            path: candidates[0].clone(),
            candidates,
            created: false,
        },
        _ => {
            for candidate in &candidates {
                println!("vault candidate: {}", candidate.display());
            }
            println!(
                "vault: {} candidates found, using {}",
                candidates.len(),
                candidates[0].display()
            );
            Detected {
                path: candidates[0].clone(),
                candidates,
                created: false,
            }
        }
    }
}
