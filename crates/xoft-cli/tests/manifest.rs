//! M0.2 — the corpus manifest must be deterministic and must not leak absolute paths.

use std::fs;
use xoft_cli::manifest::{build, Root};

fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(d.path().join("sub")).unwrap();
    // Deliberately created out of alphabetical order.
    fs::write(d.path().join("sub/Zulu.mod"), b"MODULE Zulu;\r\nEND Zulu.\r\n").unwrap();
    fs::write(d.path().join("Alpha.mod"), b"MODULE Alpha;\nEND Alpha.\n").unwrap();
    fs::write(d.path().join("Iface.def"), b"DEFINITION Iface;\nEND Iface.\n").unwrap();
    // Sidecars and unrelated files must not be picked up.
    fs::write(d.path().join("Alpha.mod.uaem"), b"----rwed").unwrap();
    fs::write(d.path().join("readme.txt"), b"hi").unwrap();
    d
}

fn roots(d: &tempfile::TempDir) -> Vec<Root> {
    vec![Root {
        alias: "test".into(),
        path: d.path().to_path_buf(),
        origin: "fixture".into(),
        license: "n/a".into(),
    }]
}

#[test]
fn collects_only_oberon_sources() {
    let d = fixture();
    let m = build(&roots(&d));
    let paths: Vec<_> = m.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["Alpha.mod", "Iface.def", "sub/Zulu.mod"]);
}

#[test]
fn paths_are_root_relative_and_slash_separated() {
    let d = fixture();
    let m = build(&roots(&d));
    for f in &m.files {
        assert!(!f.path.starts_with('/'), "absolute path leaked: {}", f.path);
        assert!(!f.path.contains('\\'), "backslash in path: {}", f.path);
        assert_eq!(f.root, "test");
    }
}

#[test]
fn facts_are_carried_through() {
    let d = fixture();
    let m = build(&roots(&d));
    let zulu = m.files.iter().find(|f| f.path == "sub/Zulu.mod").unwrap();
    assert_eq!(zulu.facts.line_endings, xoft_core::corpus::LineEndings::Crlf);
    let alpha = m.files.iter().find(|f| f.path == "Alpha.mod").unwrap();
    assert_eq!(alpha.facts.line_endings, xoft_core::corpus::LineEndings::Lf);
}

#[test]
fn serialization_is_byte_stable_across_runs() {
    let d = fixture();
    let a = serde_json::to_string_pretty(&build(&roots(&d))).unwrap();
    let b = serde_json::to_string_pretty(&build(&roots(&d))).unwrap();
    assert_eq!(a, b);
    assert!(
        !a.contains(d.path().to_str().unwrap()),
        "manifest leaked the absolute root path"
    );
}

#[test]
fn continues_past_an_unreadable_root() {
    let d = fixture();
    let missing = tempfile::tempdir().unwrap();
    let missing_path = missing.path().join("does-not-exist");
    let missing_path_str = missing_path.to_str().unwrap().to_string();
    drop(missing); // never created on disk -- simulates an unsynced/missing corpus root

    let mut all_roots = roots(&d);
    all_roots.push(Root {
        alias: "missing".into(),
        path: missing_path,
        origin: "fixture".into(),
        license: "n/a".into(),
    });

    let m = build(&all_roots);

    let paths: Vec<_> = m.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["Alpha.mod", "Iface.def", "sub/Zulu.mod"],
        "good root's files must survive a sibling root's failure"
    );
    assert_eq!(m.failures.len(), 1);
    assert_eq!(m.failures[0].alias, "missing");
    assert!(
        !m.failures[0].error.contains(&missing_path_str),
        "a root failure must not leak its absolute path (D8): {}",
        m.failures[0].error
    );
}
