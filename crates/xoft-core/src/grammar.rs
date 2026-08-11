//! Links the vendored tree-sitter-oberon2 grammar (grammars/tree-sitter-oberon2, compiled
//! by build.rs) and exposes it as a `tree_sitter::Language`.

use tree_sitter::Language;
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_oberon2() -> *const ();
}

pub fn language() -> Language {
    Language::new(unsafe { LanguageFn::from_raw(tree_sitter_oberon2) })
}
