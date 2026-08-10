//! Corpus inventory.
//!
//! The corpus lives outside the repository (archived third-party sources). The manifest
//! pins exactly which files and which bytes were tested, without ever recording an
//! absolute path — see D8 and the determinism rule in docs/plan.md.

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use xoft_core::corpus::FileFacts;

/// Oberon source extensions. `.def` is an STJ-Oberon definition module.
const SOURCE_EXTENSIONS: [&str; 2] = ["mod", "def"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub alias: String,
    pub path: PathBuf,
    pub origin: String,
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsConfig {
    pub root: Vec<Root>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub root: String,
    pub path: String,
    #[serde(flatten)]
    pub facts: FileFacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootSummary {
    pub alias: String,
    pub origin: String,
    pub license: String,
    pub files: usize,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub roots: Vec<RootSummary>,
    pub files: Vec<Entry>,
}

pub fn build(roots: &[Root]) -> Result<Manifest> {
    let mut summaries = Vec::new();
    let mut files = Vec::new();

    for root in roots {
        let mut count = 0;
        let mut bytes = 0;
        for entry in WalkDir::new(&root.path).follow_links(false) {
            let entry = entry.with_context(|| format!("walking {}", root.path.display()))?;
            if !entry.file_type().is_file() || !is_source(entry.path()) {
                continue;
            }
            let raw = std::fs::read(entry.path())
                .with_context(|| format!("reading {}", entry.path().display()))?;
            let rel = entry
                .path()
                .strip_prefix(&root.path)
                .expect("walkdir yields paths under the root")
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            count += 1;
            bytes += raw.len();
            files.push(Entry {
                root: root.alias.clone(),
                path: rel,
                facts: FileFacts::classify(&raw),
            });
        }
        summaries.push(RootSummary {
            alias: root.alias.clone(),
            origin: root.origin.clone(),
            license: root.license.clone(),
            files: count,
            bytes,
        });
    }

    // Determinism: filesystem order is not stable across machines.
    files.sort_by(|a, b| (&a.root, &a.path).cmp(&(&b.root, &b.path)));
    summaries.sort_by(|a, b| a.alias.cmp(&b.alias));
    Ok(Manifest {
        roots: summaries,
        files,
    })
}

fn is_source(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SOURCE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}
