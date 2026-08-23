//! Tauri backend for the xoft testbed (M6.1, docs/plan.md line 141). Command bodies live in
//! `commands` as plain, Tauri-free functions so they're unit-testable without a Tauri
//! runtime; this module only adds the `#[tauri::command]` IPC wrappers.

pub mod commands;

use commands::{Direction, RoundtripResult, TranspileResult};
use xoft_cli::manifest::Manifest;

#[tauri::command]
fn list_corpus(roots_toml: String) -> Result<Manifest, String> {
    commands::list_corpus(&roots_toml).map_err(|e| e.to_string())
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
        .invoke_handler(tauri::generate_handler![list_corpus, roundtrip_check, transpile])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
