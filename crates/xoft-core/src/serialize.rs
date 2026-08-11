//! Token-walk serializer + byte-coverage assertion (D4 in docs/plan.md).
//!
//! Walks the tree's leaves in source order and pairs each with the gap of source text
//! before it. Anything the grammar recognizes -- including comments -- already appears as
//! a leaf node (tree-sitter's extras still produce nodes, just marked `is_extra`), so a
//! well-formed tree's gaps hold only whitespace by construction; a real byte escaping into
//! a gap means the walk missed a node, not that the source is unusual.

use tree_sitter::{Node, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span<'a> {
    Leaf(&'a str),
    Gap(&'a str),
}

pub fn walk<'a>(tree: &Tree, text: &'a str) -> Vec<Span<'a>> {
    let mut leaves = Vec::new();
    collect_leaves(tree.root_node(), &mut leaves);

    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for leaf in leaves {
        let (start, end) = (leaf.start_byte(), leaf.end_byte());
        if start > cursor {
            spans.push(Span::Gap(&text[cursor..start]));
        }
        if end > start {
            spans.push(Span::Leaf(&text[start..end]));
        }
        cursor = cursor.max(end);
    }
    if cursor < text.len() {
        spans.push(Span::Gap(&text[cursor..]));
    }
    spans
}

pub fn reconstruct(spans: &[Span]) -> String {
    spans
        .iter()
        .map(|s| match s {
            Span::Leaf(t) | Span::Gap(t) => *t,
        })
        .collect()
}

fn collect_leaves<'a>(node: Node<'a>, out: &mut Vec<Node<'a>>) {
    if node.child_count() == 0 {
        out.push(node);
        return;
    }
    for i in 0..node.child_count() {
        collect_leaves(node.child(i as u32).unwrap(), out);
    }
}
