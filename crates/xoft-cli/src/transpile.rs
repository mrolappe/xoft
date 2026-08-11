//! `xoft transpile` -- Phase 1 scope (docs/plan.md line 115): `check` plus a lossless
//! round-trip through the M2 serializer. No dialect-mapping rules exist until M5, so for now
//! this just exercises the codec/serializer end-to-end from the CLI (user-confirmed scope,
//! see NEXT.md history).

use std::path::Path;

use anyhow::{Context, Result};
use xoft_core::codec::Document;
use xoft_core::grammar;
use xoft_core::serialize;

use crate::check::{check_source, CheckResult};

pub struct TranspileResult {
    pub check: CheckResult,
    pub output_bytes: Vec<u8>,
}

pub fn transpile_file(path: &Path) -> Result<TranspileResult> {
    let raw = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let doc = Document::from_bytes(&raw);
    let filename = path.display().to_string();

    let check = check_source(&filename, &doc.text);

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar::language()).expect("grammar loads");
    let tree = parser
        .parse(&doc.text, None)
        .expect("parse always returns a tree");
    let rebuilt = serialize::reconstruct(&serialize::walk(&tree, &doc.text));

    let output_bytes = Document { text: rebuilt }.to_bytes();
    Ok(TranspileResult { check, output_bytes })
}
