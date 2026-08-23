//! Tauri backend for the xoft testbed (M6.1, docs/plan.md line 141). Command bodies live in
//! `commands` as plain, Tauri-free functions so they're unit-testable without a Tauri
//! runtime; this module only adds the `#[tauri::command]` IPC wrappers.

pub mod commands;

use std::path::Path;

use commands::{Direction, RoundtripResult, TranspileResult};
use xoft_cli::manifest::Manifest;

/// `list_corpus`/`read_corpus_file` used to take `roots_toml` as an IPC argument -- the same
/// shape as M6.1's acknowledged finding (a webview-supplied string picking which filesystem
/// paths get walked/read). Closed at the actual trust boundary instead of just moving the
/// UI's happy path: the IPC wrappers now read this repo's own `corpus/roots.toml` themselves,
/// so the webview can no longer supply a `roots_toml` value at all. `commands::list_corpus`/
/// `read_corpus_file` still take it as a parameter -- that's the tested, Tauri-free layer,
/// unit-tested against synthetic fixtures; only the IPC-facing boundary changes.
fn read_roots_toml() -> Result<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/roots.toml");
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

#[tauri::command]
fn list_corpus() -> Result<Manifest, String> {
    let roots_toml = read_roots_toml()?;
    commands::list_corpus(&roots_toml).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_corpus_file(root: String, path: String) -> Result<Vec<u8>, String> {
    let roots_toml = read_roots_toml()?;
    commands::read_corpus_file(&roots_toml, &root, &path)
}

#[tauri::command]
fn roundtrip_check(filename: String, raw: Vec<u8>) -> RoundtripResult {
    commands::roundtrip_check(&filename, &raw)
}

#[tauri::command]
fn transpile(direction: Direction, text: String) -> TranspileResult {
    commands::transpile(direction, &text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_corpus,
            read_corpus_file,
            roundtrip_check,
            transpile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
