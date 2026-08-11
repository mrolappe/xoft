//! M2.2 — token-walk serializer + byte-coverage assertion (D4). Written before the
//! implementation (TDD).

use xoft_core::codec::Document;
use xoft_core::serialize::{reconstruct, walk, Span};

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&xoft_core::grammar::language()).unwrap();
    parser.parse(source, None).unwrap()
}

const SOURCE: &str = "MODULE M;\n  (* a comment *)\n  CONST x = 1;\nEND M.\n";

#[test]
fn reconstructs_the_source_exactly() {
    let tree = parse(SOURCE);
    let spans = walk(&tree, SOURCE);
    assert_eq!(reconstruct(&spans), SOURCE);
}

#[test]
fn every_gap_is_whitespace_only() {
    // Anything the grammar recognizes (including comments, see the `comment` node above)
    // is already a leaf. A non-whitespace byte in a gap means real content silently
    // escaped the leaf walk -- that's a bug in `walk`, not a property of this source.
    let tree = parse(SOURCE);
    let spans = walk(&tree, SOURCE);
    for span in &spans {
        if let Span::Gap(text) = span {
            assert!(
                text.chars().all(char::is_whitespace),
                "non-whitespace gap: {text:?}"
            );
        }
    }
}

#[test]
fn round_trips_through_the_codec_byte_identically() {
    // The actual M2 invariant (D8): original bytes -> Document -> parse -> walk ->
    // reconstruct -> Document -> bytes, byte-identical, including a high byte (0xE9,
    // "e"-acute in Latin-1) inside a comment, which is where D3's codec earns its keep.
    let source = "MODULE M;\n  (* caf\u{e9} *)\nEND M.\n";
    let bytes: Vec<u8> = source.chars().map(|c| c as u8).collect();

    let doc = Document::from_bytes(&bytes);
    let tree = parse(&doc.text);
    assert!(!tree.root_node().has_error());
    let spans = walk(&tree, &doc.text);
    let reconstructed = Document {
        text: reconstruct(&spans),
    };

    assert_eq!(reconstructed.to_bytes(), bytes);
}

#[test]
fn a_syntax_error_is_detected() {
    let tree = parse("MODULE ;\nEND .\n");
    assert!(tree.root_node().has_error());
}
