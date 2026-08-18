//! M5.2 — the two Oberon-X <-> Oberon-2 mapping rules and the splicing emit path.
//! Written before the implementation (TDD).
//!
//! Rule A  `DO` as a block opener  ->  `BEGIN`      (X->2 only; see the invertibility note)
//! Rule B  `UNLESS E DO S END`     <-> `IF ~(E) THEN S END`

use xoft_core::grammar;
use xoft_core::mapping::{to_oberon2, to_oberon_x};

fn parse(source: &str, language: &tree_sitter::Language) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error(), "fixture failed to parse: {source:?}");
    tree
}

fn x_to_2(source: &str) -> String {
    to_oberon2(&parse(source, &grammar::language_oberon_x()), source)
}

fn two_to_x(source: &str) -> String {
    to_oberon_x(&parse(source, &grammar::language()), source)
}

// Rule A, both sites `grammar.js` spells `choice($.kBegin, $.kDo)`.
const X_DO_PROC: &str = "MODULE M;\n  PROCEDURE P;\n  DO\n    Out.Ln;\n  END P;\nEND M.\n";
const O2_DO_PROC: &str = "MODULE M;\n  PROCEDURE P;\n  BEGIN\n    Out.Ln;\n  END P;\nEND M.\n";

const X_DO_MODULE: &str = "MODULE M;\nDO\n  Out.Ln;\nEND M.\n";
const O2_DO_MODULE: &str = "MODULE M;\nBEGIN\n  Out.Ln;\nEND M.\n";

// Rule B, with a body and with an empty one.
const X_UNLESS: &str =
    "MODULE M;\n  PROCEDURE P;\n  BEGIN\n    UNLESS x = 0 DO\n      Out.Ln;\n    END\n  END P;\nEND M.\n";
const O2_UNLESS: &str =
    "MODULE M;\n  PROCEDURE P;\n  BEGIN\n    IF ~(x = 0) THEN\n      Out.Ln;\n    END\n  END P;\nEND M.\n";

const X_UNLESS_EMPTY: &str =
    "MODULE M;\n  PROCEDURE P;\n  BEGIN\n    UNLESS x = 0 DO\n    END\n  END P;\nEND M.\n";
const O2_UNLESS_EMPTY: &str =
    "MODULE M;\n  PROCEDURE P;\n  BEGIN\n    IF ~(x = 0) THEN\n    END\n  END P;\nEND M.\n";

// Rule B's expression is a single leaf here -- the prefix and the suffix land on the
// same leaf, which is the one splice case that can silently drop an insertion.
const X_UNLESS_ATOM: &str = "MODULE M;\nBEGIN\n  UNLESS ok DO\n    Out.Ln;\n  END\nEND M.\n";
const O2_UNLESS_ATOM: &str = "MODULE M;\nBEGIN\n  IF ~(ok) THEN\n    Out.Ln;\n  END\nEND M.\n";

const PAIRS: &[(&str, &str)] = &[
    (X_UNLESS, O2_UNLESS),
    (X_UNLESS_EMPTY, O2_UNLESS_EMPTY),
    (X_UNLESS_ATOM, O2_UNLESS_ATOM),
];

#[test]
fn rule_a_rewrites_do_block_openers_to_begin() {
    assert_eq!(x_to_2(X_DO_PROC), O2_DO_PROC);
    assert_eq!(x_to_2(X_DO_MODULE), O2_DO_MODULE);
}

#[test]
fn rule_a_leaves_non_block_opener_do_alone() {
    // WHILE/FOR/WITH all spell `DO` too; only the two block-opener sites are Rule A's.
    let src = "MODULE M;\nDO\n  WHILE x < 1 DO\n    Out.Ln;\n  END;\nEND M.\n";
    let want = "MODULE M;\nBEGIN\n  WHILE x < 1 DO\n    Out.Ln;\n  END;\nEND M.\n";
    assert_eq!(x_to_2(src), want);
}

#[test]
fn rule_a_is_not_applied_in_reverse() {
    // Deliberate: Oberon-2 source is already valid Oberon-X, so 2->X leaves BEGIN alone.
    assert_eq!(two_to_x(O2_DO_PROC), O2_DO_PROC);
    assert_eq!(two_to_x(O2_DO_MODULE), O2_DO_MODULE);
}

#[test]
fn rule_b_maps_unless_down_to_negated_if() {
    for (x, o2) in PAIRS {
        assert_eq!(&x_to_2(x), o2);
    }
}

#[test]
fn rule_b_maps_negated_if_back_up_to_unless() {
    for (x, o2) in PAIRS {
        assert_eq!(&two_to_x(o2), x);
    }
}

#[test]
fn rule_b_does_not_match_an_if_it_did_not_produce() {
    // No `~`, an ELSE branch, or a `~` without the parens: none of these are in the
    // image of X->2, so lifting them would not round-trip.
    for src in [
        "MODULE M;\nBEGIN\n  IF x = 0 THEN\n    Out.Ln;\n  END\nEND M.\n",
        "MODULE M;\nBEGIN\n  IF ~(x = 0) THEN\n    Out.Ln;\n  ELSE\n    Out.Ln;\n  END\nEND M.\n",
        "MODULE M;\nBEGIN\n  IF ~ok THEN\n    Out.Ln;\n  END\nEND M.\n",
        "MODULE M;\nBEGIN\n  IF ~(x = 0) & ok THEN\n    Out.Ln;\n  END\nEND M.\n",
    ] {
        assert_eq!(two_to_x(src), src, "should have been left untouched");
    }
}

#[test]
fn gaps_are_inherited_verbatim_including_comments_and_odd_indentation() {
    // "Template splicing with inherited indentation": nothing recomputes layout, so a
    // comment sitting between UNLESS and its expression, and a ragged indent, survive.
    let x = "MODULE M;\nBEGIN\n\tUNLESS (* why *) x = 0 DO\n          Out.Ln;\n\tEND\nEND M.\n";
    let o2 = "MODULE M;\nBEGIN\n\tIF (* why *) ~(x = 0) THEN\n          Out.Ln;\n\tEND\nEND M.\n";
    assert_eq!(x_to_2(x), o2);
    assert_eq!(two_to_x(o2), x);
}

#[test]
fn round_trip_2_x_2_is_byte_identical() {
    for (_, o2) in PAIRS {
        let up = two_to_x(o2);
        let down = to_oberon2(&parse(&up, &grammar::language_oberon_x()), &up);
        assert_eq!(&down, o2);
    }
    for o2 in [O2_DO_PROC, O2_DO_MODULE] {
        let up = two_to_x(o2);
        let down = to_oberon2(&parse(&up, &grammar::language_oberon_x()), &up);
        assert_eq!(down, o2);
    }
}

#[test]
fn round_trip_x_2_x_is_byte_identical_for_rule_b() {
    for (x, _) in PAIRS {
        let down = x_to_2(x);
        let up = to_oberon_x(&parse(&down, &grammar::language()), &down);
        assert_eq!(&up, x);
    }
}

#[test]
fn round_trip_x_2_x_normalizes_do_openers_to_begin() {
    // Rule A is many-to-one by construction: `BEGIN` and `DO` are synonyms in Oberon-X and
    // both map to the single Oberon-2 spelling `BEGIN`, so no reverse rule can recover
    // which one was written. X->2->X is byte-identical up to this normalization; it is a
    // measured property of a synonym-style dialect feature, not a defect in the emit path.
    let down = x_to_2(X_DO_PROC);
    let up = to_oberon_x(&parse(&down, &grammar::language()), &down);
    assert_eq!(up, O2_DO_PROC);
}
