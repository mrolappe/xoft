//! M3.2 -- `xoft check`: parse a file and render its diagnostics via codespan-reporting
//! (docs/plan.md line 115). Written before the implementation (TDD).

use std::fs;

use xoft_cli::check::check_file;

#[test]
fn clean_file_has_no_diagnostics_and_empty_rendering() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("Alpha.mod");
    fs::write(&path, b"MODULE Alpha;\nEND Alpha.\n").unwrap();

    let result = check_file(&path).unwrap();
    assert!(result.diagnostics.is_empty());
    assert!(result.rendered.is_empty());
}

#[test]
fn broken_file_reports_a_rendered_diagnostic_with_message_and_span_marker() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("Broken.mod");
    // Missing ";" between two statements -- confirmed (M3.1) to surface as an ERROR node
    // whose message is upgraded via the "assignment" parent-kind table entry.
    fs::write(
        &path,
        b"MODULE Broken;\nVAR x, y: INTEGER;\nBEGIN\n  x := 1\n  y := 2\nEND Broken.\n",
    )
    .unwrap();

    let result = check_file(&path).unwrap();
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("assignment"));
    assert!(
        result.rendered.contains(&result.diagnostics[0].message),
        "rendered output should include the diagnostic message:\n{}",
        result.rendered
    );
    assert!(
        result.rendered.contains("┌─"),
        "rendered output should include a codespan-reporting location marker:\n{}",
        result.rendered
    );
    assert!(
        result.rendered.contains(&path.display().to_string()) || result.rendered.contains("Broken.mod"),
        "rendered output should reference the source file:\n{}",
        result.rendered
    );
}
