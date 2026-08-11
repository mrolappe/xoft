//! `xoft check` -- parse a source file and render its diagnostics (M3.2, docs/plan.md line
//! 115). Rendering is codespan-reporting's job; xoft-core stays text-rendering-free, per
//! CLAUDE.md's no-I/O-in-core rule.

use std::path::Path;

use anyhow::{Context, Result};
use codespan_reporting::diagnostic::{Diagnostic as CDiagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::{self, termcolor::Buffer};
use xoft_core::codec::Document;
use xoft_core::diagnostic::{diagnostics, Diagnostic};
use xoft_core::grammar;
use xoft_core::rule::RuleRegistry;

pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
    pub rendered: String,
}

fn parse(text: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar::language()).expect("grammar loads");
    parser.parse(text, None).expect("parse always returns a tree")
}

pub fn check_source(filename: &str, text: &str) -> CheckResult {
    let tree = parse(text);

    let mut ds = diagnostics(&tree);
    ds.extend(RuleRegistry::new().run(&tree, text));

    let mut files = SimpleFiles::new();
    let file_id = files.add(filename, text);
    let config = term::Config::default();
    let mut buffer = Buffer::no_color();
    for d in &ds {
        let cd = CDiagnostic::error()
            .with_message(&d.message)
            .with_labels(vec![Label::primary(file_id, d.start_byte..d.end_byte)]);
        term::emit(&mut buffer, &config, &files, &cd).expect("render to an in-memory buffer");
    }

    CheckResult {
        diagnostics: ds,
        rendered: String::from_utf8_lossy(buffer.as_slice()).into_owned(),
    }
}

pub fn check_file(path: &Path) -> Result<CheckResult> {
    let raw = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let doc = Document::from_bytes(&raw);
    Ok(check_source(&path.display().to_string(), &doc.text))
}
