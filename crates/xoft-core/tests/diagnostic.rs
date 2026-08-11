//! M3.1 — `Diagnostic`: ERROR/MISSING walk, byte spans throughout (docs/plan.md).
//! Written before the implementation (TDD). Fixture shapes (which node ends up MISSING vs.
//! ERROR, and where) were probed against the real parser first, not guessed.

use xoft_core::diagnostic::diagnostics;

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&xoft_core::grammar::language()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn clean_source_has_no_diagnostics() {
    let source = "MODULE M;\nEND M.\n";
    let tree = parse(source);
    assert!(diagnostics(&tree).is_empty());
}

#[test]
fn missing_node_reports_its_own_kind_as_the_message() {
    // Unbalanced "(" -- GLR error recovery inserts a zero-width MISSING ")" right before
    // the newline, rather than an ERROR node.
    let source = "MODULE M;\nVAR x: INTEGER;\nBEGIN\n  x := (1 + 2\nEND M.\n";
    let tree = parse(source);
    let ds = diagnostics(&tree);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].message, ")");
    assert_eq!(ds[0].start_byte, ds[0].end_byte);
    assert!(source[ds[0].start_byte..].starts_with('\n'));
}

#[test]
fn error_node_gets_a_context_upgraded_message_from_its_parent_kind() {
    // A missing ";" between two statements makes the second statement's value misparse as
    // an ERROR node whose immediate parent is "assignment" -- confirmed by probing the real
    // tree, not assumed.
    let source = "MODULE M;\nVAR x, y: INTEGER;\nBEGIN\n  x := 1\n  y := 2\nEND M.\n";
    let tree = parse(source);
    let ds = diagnostics(&tree);
    assert_eq!(ds.len(), 1);
    assert_ne!(ds[0].message, "unexpected syntax", "should be upgraded, not the fallback");
    assert!(ds[0].message.contains("assignment"));
}

#[test]
fn error_node_without_a_table_entry_falls_back_to_a_generic_message() {
    // An IF with no ELSE swallows the module's own "END", pushing the whole tree into one
    // root-level ERROR with no parent -- confirmed by probing the real tree.
    let source = "MODULE M;\nVAR x: INTEGER;\nBEGIN\n  IF x = 1 THEN\n    x := 2\nEND M.\n";
    let tree = parse(source);
    let ds = diagnostics(&tree);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].message, "unexpected syntax");
}
