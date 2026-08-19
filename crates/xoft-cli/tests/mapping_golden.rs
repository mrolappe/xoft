//! M5.3 -- file-driven golden-file suite for `xoft_core::mapping`, promoting the fixtures
//! in `xoft-core/tests/mapping.rs` to `corpus/cases/`. Lives here, not in xoft-core, because
//! reading the fixture files from disk is I/O and xoft-core performs none (see CLAUDE.md).
//!
//! `lossy` cases are Rule A (`DO` block opener -> `BEGIN`): many-to-one, so X->2->X only
//! reaches the `.2.mod` file (BEGIN-normalized), never byte-identical to the `.x.mod` source.
//! Non-lossy cases are Rule B (`UNLESS`), a bijection on the shape it produces: both round
//! trips are byte-identical. See docs/errors.md's injectivity note before changing this.

use std::path::Path;

use xoft_core::grammar;
use xoft_core::mapping::{to_oberon2, to_oberon_x};

fn cases_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/cases")
}

fn read_case(name: &str, side: &str) -> String {
    let path = cases_dir().join(format!("{name}.{side}.mod"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

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

struct Case {
    name: &'static str,
    /// Rule A: `DO` block opener, many-to-one, no exact reverse.
    lossy: bool,
}

const CASES: &[Case] = &[
    Case { name: "do_proc", lossy: true },
    Case { name: "do_module", lossy: true },
    Case { name: "unless_body", lossy: false },
    Case { name: "unless_empty", lossy: false },
    Case { name: "unless_atom", lossy: false },
    Case { name: "comment_gap", lossy: false },
];

#[test]
fn golden_files_map_in_both_directions() {
    for case in CASES {
        let x = read_case(case.name, "x");
        let two = read_case(case.name, "2");

        assert_eq!(x_to_2(&x), two, "{}: X -> 2", case.name);

        // 2 -> X reaches `x` only when Rule B is the only rule in play; a lossy (Rule A)
        // fixture's `.2.mod` is already valid Oberon-X (BEGIN is legal there too), so 2 -> X
        // is the identity on it.
        let want_up = if case.lossy { &two } else { &x };
        assert_eq!(&two_to_x(&two), want_up, "{}: 2 -> X", case.name);

        // 2 -> X -> 2 is byte-identical unconditionally.
        let up = two_to_x(&two);
        assert_eq!(x_to_2(&up), two, "{}: round trip 2->X->2", case.name);

        // X -> 2 -> X reaches `x` for Rule B, but only the BEGIN-normalized `.2.mod` for
        // Rule A -- the documented, deliberate exception to byte-identical round-tripping.
        let down = x_to_2(&x);
        let back = two_to_x(&down);
        assert_eq!(&back, want_up, "{}: round trip X->2->X", case.name);
    }
}
