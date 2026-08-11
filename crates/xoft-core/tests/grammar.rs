//! M2.2 (start) — links the vendored tree-sitter-oberon2 grammar. Written before the
//! implementation (TDD).

use xoft_core::grammar;

#[test]
fn parses_a_trivial_module_with_no_errors() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar::language()).unwrap();
    let tree = parser.parse("MODULE M;\nEND M.\n", None).unwrap();
    assert!(!tree.root_node().has_error());
}
