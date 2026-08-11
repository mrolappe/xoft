//! M3.2 -- `xoft transpile`, Phase 1 scope: `check` plus a lossless round-trip through the
//! M2 serializer (docs/plan.md line 115; user confirmed this scope for Phase 1, no dialect
//! rules exist until M5). Written before the implementation (TDD).

use std::fs;

use xoft_cli::transpile::transpile_file;

#[test]
fn clean_file_round_trips_byte_identical_and_has_no_diagnostics() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("Alpha.mod");
    let source = b"MODULE Alpha; (* a comment *)\nEND Alpha.\n";
    fs::write(&path, source).unwrap();

    let result = transpile_file(&path).unwrap();
    assert!(result.check.diagnostics.is_empty());
    assert_eq!(result.output_bytes, source);
}

#[test]
fn broken_file_still_round_trips_and_reports_its_diagnostic() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("Broken.mod");
    let source = b"MODULE Broken;\nVAR x, y: INTEGER;\nBEGIN\n  x := 1\n  y := 2\nEND Broken.\n";
    fs::write(&path, source).unwrap();

    let result = transpile_file(&path).unwrap();
    assert_eq!(result.check.diagnostics.len(), 1);
    assert_eq!(result.output_bytes, source);
}
