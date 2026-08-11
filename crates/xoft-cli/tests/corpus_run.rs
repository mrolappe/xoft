//! M4.1 -- `xoft corpus run`: parse + round-trip every corpus file into a deterministic
//! report, honoring the allowlist (D8). Written before the implementation (TDD).

use std::fs;

use xoft_cli::corpus_run::{aggregate, run, Allowlist, AllowlistEntry, FileOutcome};
use xoft_cli::manifest::Root;

fn outcome(root: &str, path: &str, parse_ok: bool, round_trip_ok: bool) -> FileOutcome {
    FileOutcome {
        root: root.into(),
        path: path.into(),
        parse_ok,
        round_trip_ok,
    }
}

#[test]
fn aggregate_counts_clean_and_broken_files() {
    let outcomes = vec![
        outcome("a", "Clean.mod", true, true),
        outcome("a", "Broken.mod", false, true),
    ];
    let report = aggregate(&Allowlist::default(), outcomes);

    assert_eq!(report.total_files, 2);
    assert_eq!(report.allowlisted_files, 0);
    assert_eq!(report.counted_files, 2);
    assert_eq!(report.parse_ok, 1);
    assert_eq!(report.round_trip_ok, 2);
    assert_eq!(report.parse_pct, 50.0);
    assert_eq!(report.round_trip_pct, 100.0);

    let a = &report.roots["a"];
    assert_eq!((a.files, a.parse_ok, a.round_trip_ok, a.allowlisted), (2, 1, 2, 0));

    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].path, "Broken.mod");
    assert_eq!(report.failure_histogram.get("parse"), Some(&1));
}

#[test]
fn aggregate_excludes_allowlisted_files_from_percentages() {
    let allowlist = Allowlist {
        entry: vec![AllowlistEntry {
            root: "a".into(),
            path: "Broken.mod".into(),
            reason: "known gap".into(),
        }],
    };
    let outcomes = vec![
        outcome("a", "Clean.mod", true, true),
        outcome("a", "Broken.mod", false, true),
    ];
    let report = aggregate(&allowlist, outcomes);

    assert_eq!(report.total_files, 2);
    assert_eq!(report.allowlisted_files, 1);
    assert_eq!(report.counted_files, 1);
    assert_eq!(report.parse_ok, 1);
    assert_eq!(report.parse_pct, 100.0);
    assert!(report.failures.is_empty(), "allowlisted failure must not count as a failure");
    assert!(report.failure_histogram.is_empty());
    assert_eq!(report.roots["a"].allowlisted, 1);
}

#[test]
fn aggregate_buckets_failures_by_which_metric_failed() {
    let outcomes = vec![
        outcome("a", "ParseOnly.mod", false, true),
        outcome("a", "RoundTripOnly.mod", true, false),
        outcome("a", "Both.mod", false, false),
    ];
    let report = aggregate(&Allowlist::default(), outcomes);

    assert_eq!(report.failure_histogram.get("parse"), Some(&1));
    assert_eq!(report.failure_histogram.get("round-trip"), Some(&1));
    assert_eq!(report.failure_histogram.get("parse+round-trip"), Some(&1));
}

#[test]
fn report_json_is_byte_stable_across_repeated_aggregation() {
    let outcomes = vec![outcome("b", "X.mod", true, true), outcome("a", "Y.mod", false, false)];
    let a = serde_json::to_string_pretty(&aggregate(&Allowlist::default(), outcomes.clone())).unwrap();
    let b = serde_json::to_string_pretty(&aggregate(&Allowlist::default(), outcomes)).unwrap();
    assert_eq!(a, b);
}

fn roots(d: &tempfile::TempDir) -> Vec<Root> {
    vec![Root {
        alias: "test".into(),
        path: d.path().to_path_buf(),
        origin: "fixture".into(),
        license: "n/a".into(),
    }]
}

#[test]
fn run_walks_the_corpus_honors_the_allowlist_and_is_deterministic() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("Clean.mod"), b"MODULE Clean; (* ok *)\nEND Clean.\n").unwrap();
    // Missing a statement separator -- one diagnostic, still round-trips (M3.2 precedent:
    // ERROR nodes are still leaves the walker covers, see transpile.rs's own test).
    fs::write(
        d.path().join("Broken.mod"),
        b"MODULE Broken;\nVAR x, y: INTEGER;\nBEGIN\n  x := 1\n  y := 2\nEND Broken.\n",
    )
    .unwrap();

    let allowlist = Allowlist {
        entry: vec![AllowlistEntry {
            root: "test".into(),
            path: "Broken.mod".into(),
            reason: "TDD fixture".into(),
        }],
    };

    let report = run(&roots(&d), &allowlist).unwrap();

    assert_eq!(report.total_files, 2);
    assert_eq!(report.allowlisted_files, 1);
    assert_eq!(report.counted_files, 1);
    assert_eq!(report.parse_ok, 1);
    assert_eq!(report.parse_pct, 100.0);
    assert!(report.failures.is_empty());

    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(
        !json.contains(d.path().to_str().unwrap()),
        "report leaked the absolute root path"
    );

    let again = serde_json::to_string_pretty(&run(&roots(&d), &allowlist).unwrap()).unwrap();
    assert_eq!(json, again, "report must be byte-stable across runs");
}
