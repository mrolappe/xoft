//! M5.2 — the Oberon-X <-> Oberon-2 mapping rules and their emit path (docs/plan.md line
//! 132, decision D5: untyped CST, transform by text splicing).
//!
//! Two rules, one per Oberon-X feature added in M5.1:
//!
//! | | Oberon-X | Oberon-2 |
//! |---|---|---|
//! | A | `DO` opening a module or procedure body | `BEGIN` |
//! | B | `UNLESS E DO S END` | `IF ~(E) THEN S END` |
//!
//! Rule A is deliberately one-way. `BEGIN` and `DO` are *synonyms* in Oberon-X, so the
//! mapping is many-to-one and no reverse rule can recover which spelling was written;
//! 2->X leaves `BEGIN` alone (base Oberon-2 is already valid Oberon-X input by design).
//! Consequently `X->2->X` is byte-identical only up to `DO` openers normalizing to
//! `BEGIN` -- see `tests/mapping.rs`. Rule B *is* a bijection on the shape it produces,
//! so it round-trips byte-identically in both directions.
//!
//! Emit path: every rewrite is expressed as an edit on a single leaf (replace / prefix /
//! suffix), spliced in by `serialize::walk_with`. Gaps -- whitespace, indentation,
//! comments -- are never touched, so "inherited indentation" is literally the original
//! bytes carried forward rather than a layout computed from scratch. Deliberately not a
//! Wadler/Oppen printer.
//!
//! This is *not* a `rule::Rule`: that trait produces `Diagnostic`s and has no way to
//! return replacement text.

use crate::serialize::{collect_leaves, reconstruct, walk_with, Span};
use std::collections::HashMap;
use tree_sitter::{Node, Tree};

/// What happens to one leaf. `text: Some("")` deletes it, `None` keeps it verbatim.
#[derive(Default, Clone)]
struct Edit {
    before: &'static str,
    text: Option<&'static str>,
    after: &'static str,
}

/// Keyed by leaf start byte, which is unique among leaves.
type Edits = HashMap<usize, Edit>;

/// Oberon-X source -> Oberon-2 source. `tree` must come from `grammar::language_oberon_x`.
pub fn to_oberon2(tree: &Tree, text: &str) -> String {
    let mut edits = Edits::new();
    visit(tree.root_node(), &mut |node| match node.kind() {
        // Rule A.
        "kDo" if is_block_opener(node) => set(&mut edits, node).text = Some("BEGIN"),
        // Rule B.
        "unless_statement" => {
            for child in children(node) {
                match child.kind() {
                    "kUnless" => set(&mut edits, child).text = Some("IF"),
                    "kDo" => set(&mut edits, child).text = Some("THEN"),
                    "expression" => {
                        let mut leaves = Vec::new();
                        collect_leaves(child, &mut leaves);
                        if let (Some(first), Some(last)) = (leaves.first(), leaves.last()) {
                            set(&mut edits, *first).before = "~(";
                            set(&mut edits, *last).after = ")";
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
    splice(tree, text, &edits)
}

/// Oberon-2 source -> Oberon-X source. `tree` must come from `grammar::language`.
/// Rule A has no reverse (see the module docs); only Rule B lifts.
pub fn to_oberon_x(tree: &Tree, text: &str) -> String {
    let mut edits = Edits::new();
    visit(tree.root_node(), &mut |node| {
        if node.kind() == "if_statement" {
            if let Some(m) = match_negated_if(node) {
                set(&mut edits, m.kif).text = Some("UNLESS");
                set(&mut edits, m.kthen).text = Some("DO");
                for dropped in [m.tilde, m.lparen, m.rparen] {
                    set(&mut edits, dropped).text = Some("");
                }
            }
        }
    });
    splice(tree, text, &edits)
}

fn splice(tree: &Tree, text: &str, edits: &Edits) -> String {
    let spans = walk_with(tree, text, |node, leaf| match edits.get(&node.start_byte()) {
        None => vec![Span::Leaf(leaf)],
        Some(edit) => [edit.before, edit.text.unwrap_or(leaf), edit.after]
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(Span::Leaf)
            .collect(),
    });
    reconstruct(&spans)
}

fn set<'e>(edits: &'e mut Edits, node: Node<'_>) -> &'e mut Edit {
    edits.entry(node.start_byte()).or_default()
}

fn visit(node: Node<'_>, f: &mut impl FnMut(Node<'_>)) {
    f(node);
    for child in children(node) {
        visit(child, f);
    }
}

fn children<'a>(node: Node<'a>) -> impl Iterator<Item = Node<'a>> {
    (0..node.child_count()).filter_map(move |i| node.child(i as u32))
}

/// The two sites `grammar.js` spells `choice($.kBegin, $.kDo)`. `WHILE`/`FOR`/`WITH` also
/// carry a `kDo`, but under their own parent, so they are excluded by construction.
fn is_block_opener(node: Node<'_>) -> bool {
    matches!(
        node.parent().map(|p| p.kind()),
        Some("module") | Some("procedure_body")
    )
}

struct NegatedIf<'a> {
    kif: Node<'a>,
    tilde: Node<'a>,
    lparen: Node<'a>,
    rparen: Node<'a>,
    kthen: Node<'a>,
}

/// Matches exactly the shape Rule B emits: `IF ~ ( E ) THEN [S] END`, no ELSIF, no ELSE.
/// Anything else -- an unparenthesized `~`, a negation that is only part of a larger
/// expression, an `ELSE` branch `UNLESS` cannot express -- is left alone, which is what
/// keeps 2->X->2 byte-identical.
fn match_negated_if<'a>(node: Node<'a>) -> Option<NegatedIf<'a>> {
    let (mut kif, mut kthen, mut expr) = (None, None, None);
    for child in children(node) {
        match child.kind() {
            "kElseif" | "kElse" => return None,
            "kIf" => kif = Some(child),
            "kThen" => kthen = Some(child),
            "expression" => expr = Some(child),
            _ => {}
        }
    }
    let factor = sole_child(sole_child(sole_child(expr?, "simple_expression")?, "term")?, "factor")?;

    // factor = "~" factor, whose inner factor = "(" expression ")".
    let mut outer = children(factor);
    let tilde = outer.next().filter(|n| n.kind() == "~")?;
    let inner = outer.next().filter(|n| n.kind() == "factor")?;
    if outer.next().is_some() {
        return None;
    }
    let mut parts = children(inner);
    let lparen = parts.next().filter(|n| n.kind() == "(")?;
    parts.next().filter(|n| n.kind() == "expression")?;
    let rparen = parts.next().filter(|n| n.kind() == ")")?;
    if parts.next().is_some() {
        return None;
    }

    Some(NegatedIf { kif: kif?, tilde, lparen, rparen, kthen: kthen? })
}

/// The node's only child, if it has exactly one and it is of `kind`. Strict on purpose: an
/// interleaved comment shows up as an extra child, and declining to match is the safe answer.
fn sole_child<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut it = children(node);
    let only = it.next().filter(|n| n.kind() == kind)?;
    it.next().is_none().then_some(only)
}
