//! `xoft corpus run` -- M4.1: parse + round-trip every corpus file, aggregate a
//! deterministic report (sorted keys, relative paths, no timestamps), honoring the
//! allowlist (D8). Reuses `manifest::build`'s file list and `transpile::transpile_source`'s
//! parse+round-trip logic rather than re-implementing either.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use xoft_core::codec::Document;

use crate::manifest::{self, Root};
use crate::transpile;

#[derive(Debug, Clone, Deserialize)]
pub struct AllowlistEntry {
    pub root: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Allowlist {
    #[serde(default)]
    pub entry: Vec<AllowlistEntry>,
}

impl Allowlist {
    fn contains(&self, root: &str, path: &str) -> bool {
        self.entry.iter().any(|e| e.root == root && e.path == path)
    }
}

#[derive(Debug, Clone)]
pub struct FileOutcome {
    pub root: String,
    pub path: String,
    pub parse_ok: bool,
    pub round_trip_ok: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RootBreakdown {
    pub files: usize,
    pub parse_ok: usize,
    pub round_trip_ok: usize,
    pub allowlisted: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub root: String,
    pub path: String,
    pub parse_ok: bool,
    pub round_trip_ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorpusReport {
    pub total_files: usize,
    pub allowlisted_files: usize,
    pub counted_files: usize,
    pub parse_ok: usize,
    pub round_trip_ok: usize,
    pub parse_pct: f64,
    pub round_trip_pct: f64,
    pub roots: BTreeMap<String, RootBreakdown>,
    pub failure_histogram: BTreeMap<String, usize>,
    pub failures: Vec<Failure>,
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        100.0
    } else {
        n as f64 * 100.0 / d as f64
    }
}

/// Pure aggregation: no I/O. Split out of `run` so the counting/bucketing rules are
/// testable without a real corpus.
pub fn aggregate(allowlist: &Allowlist, outcomes: Vec<FileOutcome>) -> CorpusReport {
    let total_files = outcomes.len();
    let mut roots: BTreeMap<String, RootBreakdown> = BTreeMap::new();
    let mut failure_histogram: BTreeMap<String, usize> = BTreeMap::new();
    let mut failures = Vec::new();
    let mut allowlisted_files = 0;
    let mut parse_ok = 0;
    let mut round_trip_ok = 0;

    for o in outcomes {
        let rb = roots.entry(o.root.clone()).or_default();
        rb.files += 1;

        if allowlist.contains(&o.root, &o.path) {
            allowlisted_files += 1;
            rb.allowlisted += 1;
            continue;
        }

        if o.parse_ok {
            parse_ok += 1;
            rb.parse_ok += 1;
        }
        if o.round_trip_ok {
            round_trip_ok += 1;
            rb.round_trip_ok += 1;
        }
        if !o.parse_ok || !o.round_trip_ok {
            let bucket = match (o.parse_ok, o.round_trip_ok) {
                (false, false) => "parse+round-trip",
                (false, true) => "parse",
                (true, false) => "round-trip",
                (true, true) => unreachable!(),
            };
            *failure_histogram.entry(bucket.to_string()).or_insert(0) += 1;
            failures.push(Failure {
                root: o.root,
                path: o.path,
                parse_ok: o.parse_ok,
                round_trip_ok: o.round_trip_ok,
            });
        }
    }
    failures.sort_by(|a, b| (&a.root, &a.path).cmp(&(&b.root, &b.path)));

    let counted_files = total_files - allowlisted_files;
    CorpusReport {
        total_files,
        allowlisted_files,
        counted_files,
        parse_ok,
        round_trip_ok,
        parse_pct: pct(parse_ok, counted_files),
        round_trip_pct: pct(round_trip_ok, counted_files),
        roots,
        failure_histogram,
        failures,
    }
}

/// Walks `roots` (via `manifest::build`), parses and round-trips every source file, and
/// aggregates the result. `allowlist` entries are excluded from the pass/fail counts (D8)
/// but still appear in `total_files`/per-root `allowlisted` counts.
pub fn run(roots: &[Root], allowlist: &Allowlist) -> Result<CorpusReport> {
    let m = manifest::build(roots)?;
    let root_paths: BTreeMap<&str, &Path> =
        roots.iter().map(|r| (r.alias.as_str(), r.path.as_path())).collect();

    let mut outcomes = Vec::with_capacity(m.files.len());
    for entry in &m.files {
        let abs_path = entry
            .path
            .split('/')
            .fold(root_paths[entry.root.as_str()].to_path_buf(), |p, c| p.join(c));
        let raw = std::fs::read(&abs_path)
            .with_context(|| format!("reading {}", abs_path.display()))?;
        let doc = Document::from_bytes(&raw);
        let result = transpile::transpile_source(&entry.path, &doc.text);
        outcomes.push(FileOutcome {
            root: entry.root.clone(),
            path: entry.path.clone(),
            parse_ok: result.check.diagnostics.is_empty(),
            round_trip_ok: result.output_bytes == raw,
        });
    }

    Ok(aggregate(allowlist, outcomes))
}
