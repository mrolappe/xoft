//! Command bodies for the testbed's three commands (M6.1, docs/plan.md line 141): plain,
//! `tauri`-free functions -- unit-testable directly, no Tauri runtime needed -- wrapped as
//! `#[tauri::command]`s in `lib.rs`.

use serde::{Deserialize, Serialize};
use tree_sitter::{Language, Tree};
use xoft_cli::manifest::{self, Manifest, RootsConfig};
use xoft_cli::transpile::transpile_source;
use xoft_core::codec::Document;
use xoft_core::diagnostic::{self, Diagnostic};
use xoft_core::grammar;
use xoft_core::mapping;

/// Which way `transpile` maps. The testbed's `transpile` means the Oberon-X<->Oberon-2
/// dialect mapping (`xoft_core::mapping`), not the CLI's check+round-trip `transpile` --
/// resolved with the user before this round, see NEXT.md history.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    OberonXToOberon2,
    Oberon2ToOberonX,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranspileResult {
    pub output: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoundtripResult {
    pub parse_ok: bool,
    pub round_trip_ok: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses `roots_toml`'s *content* (not a path -- the `#[tauri::command]` wrapper owns the
/// one `std::fs::read_to_string` call) and walks it into a `Manifest`, reusing
/// `xoft_cli::manifest::build` rather than re-deriving the corpus walk.
pub fn list_corpus(roots_toml: &str) -> anyhow::Result<Manifest> {
    let config: RootsConfig = toml::from_str(roots_toml)?;
    manifest::build(&config.root)
}

/// Mirrors `xoft-cli corpus run`'s outcome computation (`corpus_run.rs`): `parse_ok` is zero
/// diagnostics, `round_trip_ok` is the serializer's output matching the input bytes exactly,
/// both derived from `transpile_source` rather than reimplemented here.
pub fn roundtrip_check(filename: &str, raw: &[u8]) -> RoundtripResult {
    let doc = Document::from_bytes(raw);
    let result = transpile_source(filename, &doc.text);
    RoundtripResult {
        parse_ok: result.check.diagnostics.is_empty(),
        round_trip_ok: result.output_bytes == raw,
        diagnostics: result.check.diagnostics,
    }
}

/// Oberon-X <-> Oberon-2 dialect mapping. `to_oberon2` needs a tree parsed with the
/// Oberon-X grammar, `to_oberon_x` one parsed with plain Oberon-2 (see `mapping.rs`'s own
/// docs) -- `direction` picks the matching grammar and mapping function together so the two
/// can't drift out of sync.
pub fn transpile(direction: Direction, text: &str) -> TranspileResult {
    let (language, map): (Language, fn(&Tree, &str) -> String) = match direction {
        Direction::OberonXToOberon2 => (grammar::language_oberon_x(), mapping::to_oberon2),
        Direction::Oberon2ToOberonX => (grammar::language(), mapping::to_oberon_x),
    };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).expect("grammar loads");
    let tree = parser.parse(text, None).expect("parse always returns a tree");
    TranspileResult {
        diagnostics: diagnostic::diagnostics(&tree),
        output: map(&tree, text),
    }
}
