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
pub struct RootFailure {
    pub alias: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub roots: Vec<RootSummary>,
    pub files: Vec<Entry>,
    pub failures: Vec<RootFailure>,
}

pub fn build(roots: &[Root]) -> Manifest {
    let mut summaries = Vec::new();
    let mut files = Vec::new();
    let mut failures = Vec::new();

    for root in roots {
        match walk_root(root) {
            Ok((summary, mut entries)) => {
                summaries.push(summary);
                files.append(&mut entries);
            }
            Err(e) => failures.push(RootFailure {
                alias: root.alias.clone(),
                error: e.to_string(),
            }),
        }
    }

    // Determinism: filesystem order is not stable across machines.
    files.sort_by(|a, b| (&a.root, &a.path).cmp(&(&b.root, &b.path)));
    summaries.sort_by(|a, b| a.alias.cmp(&b.alias));
    failures.sort_by(|a, b| a.alias.cmp(&b.alias));
    Manifest {
        roots: summaries,
        files,
        failures,
    }
}

fn walk_root(root: &Root) -> Result<(RootSummary, Vec<Entry>)> {
    let mut count = 0;
    let mut bytes = 0;
    let mut files = Vec::new();

    for entry in WalkDir::new(&root.path).follow_links(false) {
        // `walkdir::Error`'s own `Display` embeds the absolute path it failed on; discard it
        // rather than propagate it into a `RootFailure`, which the CLI/testbed both surface
        // (report file, stderr, IPC) -- the manifest must never record an absolute path (D8).
        let entry = entry.map_err(|_| anyhow::anyhow!("could not walk this root"))?;
        if !entry.file_type().is_file() || !is_source(entry.path()) {
            continue;
        }
        // `std::fs::read`'s bare `io::Error` (unlike `.with_context`) does not embed the path.
        let raw = std::fs::read(entry.path()).context("could not read a file in this root")?;
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

    Ok((
        RootSummary {
            alias: root.alias.clone(),
            origin: root.origin.clone(),
            license: root.license.clone(),
            files: count,
            bytes,
        },
        files,
    ))
}

fn is_source(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SOURCE_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}
