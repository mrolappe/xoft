//! M3.3 -- broken-source fixtures + insta snapshots over `xoft check`'s rendered output
//! (docs/plan.md, "~8 hand-written broken files"). Written before the table/snapshot updates
//! (TDD): each fixture's shape was probed against the real parser first (throwaway
//! `_scratch_probe.rs` in xoft-core, deleted before commit), not hand-derived, per
//! `docs/checklist.md`. Snapshots assert the rendered text; per docs/insights.md round 30,
//! facts about *what* was found (message content, diagnostic count) are asserted against the
//! structured `CheckResult::diagnostics` list instead of parsed out of the snapshot string.

use std::path::Path;

use xoft_cli::check::check_source;

// `check_source` (not `check_file`) so the rendered snapshot names the fixture by its stable
// relative filename, not the checkout's absolute path -- otherwise the snapshot would differ
// on every machine/CI run.
fn read_fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/broken").join(name);
    std::fs::read_to_string(path).unwrap()
}

struct Case {
    file: &'static str,
    snapshot_name: &'static str,
    diagnostic_count: usize,
    message_contains: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        file: "unbalanced_parens.mod",
        snapshot_name: "unbalanced_parens",
        diagnostic_count: 1,
        message_contains: &[")"],
    },
    Case {
        file: "unbalanced_begin_end.mod",
        snapshot_name: "unbalanced_begin_end",
        diagnostic_count: 1,
        message_contains: &["module body"],
    },
    Case {
        file: "bad_case_label.mod",
        snapshot_name: "bad_case_label",
        diagnostic_count: 1,
        message_contains: &["ident"],
    },
    Case {
        file: "if_no_matching_end.mod",
        snapshot_name: "if_no_matching_end",
        diagnostic_count: 1,
        message_contains: &["unexpected syntax"],
    },
    Case {
        file: "malformed_procedure_heading.mod",
        snapshot_name: "malformed_procedure_heading",
        diagnostic_count: 1,
        message_contains: &["ident"],
    },
    Case {
        file: "stray_token_in_declaration.mod",
        snapshot_name: "stray_token_in_declaration",
        diagnostic_count: 1,
        message_contains: &["module body"],
    },
    Case {
        file: "missing_semicolon.mod",
        snapshot_name: "missing_semicolon",
        diagnostic_count: 1,
        message_contains: &["assignment"],
    },
    Case {
        file: "two_diagnostics.mod",
        snapshot_name: "two_diagnostics",
        diagnostic_count: 2,
        message_contains: &["ident"],
    },
];

#[test]
fn broken_fixtures_render_expected_diagnostics() {
    for case in CASES {
        let text = read_fixture(case.file);
        let result = check_source(case.file, &text);
        assert_eq!(
            result.diagnostics.len(),
            case.diagnostic_count,
            "{}: unexpected diagnostic count",
            case.file
        );
        for want in case.message_contains {
            assert!(
                result.diagnostics.iter().any(|d| d.message.contains(want)),
                "{}: expected some diagnostic message to contain {:?}, got {:?}",
                case.file,
                want,
                result.diagnostics
            );
        }
        insta::assert_snapshot!(case.snapshot_name, result.rendered);
    }
}
