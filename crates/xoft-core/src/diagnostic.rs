//! M3.1 — `Diagnostic`: ERROR/MISSING walk over a parsed tree, byte spans throughout
//! (docs/plan.md). A MISSING node's own kind already names the expected token, so it *is*
//! the message, no synthesis needed. An ERROR node's message is upgraded from a small table
//! keyed on the immediate parent's kind, falling back to a generic message when the parent
//! isn't in the table; the table starts small and is expected to grow once M3.3's
//! broken-source fixtures surface more real ERROR contexts.

use tree_sitter::{Node, Tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub start_byte: usize,
    pub end_byte: usize,
    pub message: String,
}

fn error_message(node: Node) -> String {
    match node.parent().map(|p| p.kind()) {
        // Observed case: a missing statement separator makes the next statement's value
        // misparse as an ERROR node directly inside the prior "assignment".
        Some("assignment") => "unexpected token in assignment (missing ';'?)".to_string(),
        // Observed cases (M3.3 fixtures): a stray token at module scope -- either a keyword
        // where a declaration/statement was expected, or a spurious extra "END" before the
        // module's own -- surfaces as an ERROR node directly inside "module".
        Some("module") => "unexpected token in module body".to_string(),
        _ => "unexpected syntax".to_string(),
    }
}

pub fn diagnostics(tree: &Tree) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    walk(tree.root_node(), &mut out);
    out
}

fn walk(node: Node, out: &mut Vec<Diagnostic>) {
    if node.is_missing() {
        out.push(Diagnostic {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            message: node.kind().to_string(),
        });
        return;
    }
    if node.is_error() {
        out.push(Diagnostic {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            message: error_message(node),
        });
        // The erroneous span is already covered by this one diagnostic -- don't also
        // report whatever partial nodes the recovery managed to parse underneath it.
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, out);
    }
}
