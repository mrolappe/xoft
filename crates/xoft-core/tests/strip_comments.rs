//! M2.3 — `strip_comments` (docs/plan.md: "pragma comments are kept -- they are semantics.
//! Output must re-parse"). Written before the implementation (TDD).

use xoft_core::strip_comments::strip_comments;

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&xoft_core::grammar::language()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn removes_an_ordinary_comment() {
    let source = "MODULE M;\n  (* a comment *)\n  CONST x = 1;\nEND M.\n";
    let tree = parse(source);
    let stripped = strip_comments(&tree, source);
    assert!(!stripped.contains("a comment"));
}

#[test]
fn keeps_a_pragma_comment() {
    let source = "MODULE M;\n  (*$R+*)\n  CONST x = 1;\nEND M.\n";
    let tree = parse(source);
    let stripped = strip_comments(&tree, source);
    assert!(stripped.contains("(*$R+*)"));
}

#[test]
fn keeps_a_bracket_pragma() {
    let source = "MODULE M;\n  <* STANDARD- *>\n  CONST x = 1;\nEND M.\n";
    let tree = parse(source);
    let stripped = strip_comments(&tree, source);
    assert!(stripped.contains("<* STANDARD- *>"));
}

#[test]
fn output_still_parses_with_zero_errors() {
    let source = "MODULE M;\n  (* a comment *)\n  CONST x = 1;\nEND M.\n";
    let tree = parse(source);
    let stripped = strip_comments(&tree, source);
    let reparsed = parse(&stripped);
    assert!(!reparsed.root_node().has_error());
}

#[test]
fn does_not_merge_two_tokens_when_the_comment_was_their_only_separator() {
    // "THEN(*c*)y" has no whitespace either side of the comment -- deleting its bytes
    // outright would glue the keyword and the identifier into one "THENy" token.
    let source = "MODULE M;\nVAR x, y: INTEGER;\nBEGIN\n  IF x = 0 THEN(*c*)y := 1 END\nEND M.\n";
    let tree = parse(source);
    let stripped = strip_comments(&tree, source);
    assert!(!stripped.contains("THENy"));
    let reparsed = parse(&stripped);
    assert!(!reparsed.root_node().has_error());
}
