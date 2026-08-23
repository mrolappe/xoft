//! M6.1 -- integration tests for the Tauri command bodies (`xoft_testbed_lib::commands`).
//! No Tauri runtime involved: these are plain functions, called directly, exercised against
//! real corpus/golden-file fixtures rather than hand-invented sources (checklist: prefer
//! corpus-copied shapes over invented minimal skeletons).

use std::path::Path;

use xoft_testbed_lib::commands::{self, Direction};

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> Vec<u8> {
    let path = repo_root().join(rel);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn read_to_string(rel: &str) -> String {
    String::from_utf8(read(rel)).expect("fixture is UTF-8")
}

#[test]
fn roundtrip_check_reports_clean_on_a_golden_fixture() {
    let raw = read("corpus/cases/comment_gap.2.mod");
    let result = commands::roundtrip_check("comment_gap.2.mod", &raw);
    assert!(result.parse_ok, "expected clean parse: {:?}", result.diagnostics);
    assert!(result.round_trip_ok);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn roundtrip_check_reports_the_parse_failure_on_a_broken_fixture() {
    let raw = read("crates/xoft-cli/tests/fixtures/broken/missing_semicolon.mod");
    let result = commands::roundtrip_check("missing_semicolon.mod", &raw);
    assert!(!result.parse_ok);
    assert!(!result.diagnostics.is_empty());
}

#[test]
fn transpile_maps_oberon_x_to_oberon2() {
    let x = read_to_string("corpus/cases/unless_body.x.mod");
    let two = read_to_string("corpus/cases/unless_body.2.mod");

    let result = commands::transpile(Direction::OberonXToOberon2, &x);

    assert_eq!(result.output, two);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn transpile_maps_oberon2_to_oberon_x() {
    let x = read_to_string("corpus/cases/unless_body.x.mod");
    let two = read_to_string("corpus/cases/unless_body.2.mod");

    let result = commands::transpile(Direction::Oberon2ToOberonX, &two);

    assert_eq!(result.output, x);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn list_corpus_walks_the_roots_given_in_toml_content() {
    let roots_toml = format!(
        "[[root]]\nalias = \"cases\"\npath = \"{}\"\norigin = \"local\"\nlicense = \"MIT\"\n",
        repo_root().join("corpus/cases").display()
    );

    let manifest = commands::list_corpus(&roots_toml).expect("valid roots.toml content");

    assert_eq!(manifest.roots.len(), 1);
    assert_eq!(manifest.roots[0].alias, "cases");
    assert!(manifest.files.iter().any(|e| e.path == "unless_body.x.mod"));
}
