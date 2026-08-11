//! M2.3 — comment stripping. Ordinary comments are removed; pragma comments
//! (`(*$...*)`/`<*...*>`, distinct node kinds per D1) are kept because they carry semantics.
//! docs/plan.md: "Output must re-parse" -- callers re-parse the result and check for errors.

use crate::serialize::collect_leaves;
use tree_sitter::Tree;

pub fn strip_comments(tree: &Tree, text: &str) -> String {
    let mut leaves = Vec::new();
    collect_leaves(tree.root_node(), &mut leaves);

    let mut out = String::new();
    let mut cursor = 0usize;
    for leaf in leaves {
        let (start, end) = (leaf.start_byte(), leaf.end_byte());
        if leaf.kind() == "comment" {
            if start > cursor {
                out.push_str(&text[cursor..start]);
            }
            // A single space stands in for the removed bytes so two tokens that had no
            // other whitespace between them (only the comment) don't fuse into one.
            out.push(' ');
            cursor = end;
            continue;
        }
        if start > cursor {
            out.push_str(&text[cursor..start]);
        }
        out.push_str(&text[start..end]);
        cursor = end;
    }
    if cursor < text.len() {
        out.push_str(&text[cursor..]);
    }
    out
}
