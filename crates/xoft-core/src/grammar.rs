//! Links the vendored grammars (`grammars/tree-sitter-oberon2` and
//! `grammars/tree-sitter-oberon-x`, both compiled by build.rs) and exposes them as
//! `tree_sitter::Language`s. The C symbol name is `tree_sitter_<grammar.js name field>`,
//! which is also what `src/scanner.c`'s external-scanner symbols must be prefixed with.

use tree_sitter::Language;
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_oberon2() -> *const ();
    fn tree_sitter_oberon_x() -> *const ();
}

/// The base Oberon-2 grammar (plus D1's lexical superset).
pub fn language() -> Language {
    Language::new(unsafe { LanguageFn::from_raw(tree_sitter_oberon2) })
}

/// The Oberon-X toy dialect (D7): `DO` as a `BEGIN` synonym, plus `UNLESS ... DO ... END`.
pub fn language_oberon_x() -> Language {
    Language::new(unsafe { LanguageFn::from_raw(tree_sitter_oberon_x) })
}
