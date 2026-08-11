# Next task

**M4.1 — `xoft corpus run` → `reports/corpus-report.json`, honoring the allowlist.** Per
`docs/plan.md`: parse + round-trip every corpus file, write a deterministic report, and exclude
`corpus/allowlist.toml` entries (capped at 5% of files / D8) from the pass/fail count. M3 (all of
M3.1/M3.2/M3.3) is done as of this round.

## What's confirmed (do not re-derive, just verify before coding)

- `docs/plan.md`'s layout section names two files that don't exist yet and this milestone creates
  both: `corpus/allowlist.toml` (excluded files + one-line reason each, D8, capped at 5% ≈ 40 of
  792 files) and `reports/corpus-report.json` (the `reports/` directory already exists, empty).
- Metrics required, per the plan's M4.1 row: parse % (zero `ERROR`/`MISSING`), round-trip % (byte-
  identical via M2's `serialize::walk`/`reconstruct`, already exercised end-to-end by `xoft
  transpile`, `crates/xoft-cli/src/transpile.rs`), a failure histogram, and a per-root breakdown
  (the four root aliases in `corpus/roots.toml`: `oberon-a`, `stj`, `amiga-oberon-31`, `voc`).
  "Sorted keys, relative paths, no timestamps" (plan.md) — the report must be byte-stable across
  consecutive runs on an unchanged corpus, that's M4's own exit criterion and M4.2's CI check.
- `crates/xoft-cli/src/manifest.rs` (`build(roots) -> Manifest`, walks `corpus/roots.toml`'s roots
  via `walkdir`, already computes `FileFacts` per file and writes `corpus/manifest.json`) is the
  thing to extend or sit alongside, not duplicate — it already has the root-walking, relative-path,
  and sorted-output machinery this milestone needs. Read it first (`codegraph_explore` for the
  `manifest`/`Entry`/`RootSummary` shapes) before deciding whether `corpus run` reuses `build()`'s
  file list or is a new pass over the same roots.
- `check_source`/`check_file` (`crates/xoft-cli/src/check.rs`, M3.2) already gives parse-diagnostic
  results per file; `transpile_file` (`crates/xoft-cli/src/transpile.rs`) already gives the
  round-trip byte-comparison. `corpus run` is very likely "walk the corpus, call these two per
  file, aggregate" rather than new parsing/round-trip logic — check before writing anything that
  even resembles a third implementation of either.
- The allowlist's *initial contents* are not yet decided in code, only informally identified across
  M1's rounds 23-25: dual pragma-guarded `MODULE` headers (`Break.mod`, `NoGuru.mod` in
  `amiga-oberon-31`), Amiga Oberon's conditional-compilation preprocessor files (`Kernel.mod`,
  `IntuiPointerDemo.mod`, `amiga/Utility.mod`), and a handful of one-off corpus artifacts (stray
  bytes, malformed preambles) called out in round 25's `oberon-a` retriage — see
  `docs/progress/m1-grammar.md` rounds 23-25 for the full list with counts. **Don't just port that
  list blind** — M1's 26 residual grammar failures (766/792, round 26) are a different measurement
  than what M4.1 will actually find, since M4.1 additionally checks the *round-trip*, not just the
  parse — a file can parse clean (M1's metric) and still fail byte-identical round-trip (M2's own
  ad hoc 240-file sample, round 26, found `rt_ok` true even on `ERROR`-containing files, but that
  was a small sample, not the full corpus). Run the tool first, look at what it actually reports,
  build the allowlist from real M4 output, not from the M1 backlog.
- If the real run's allowlist would exceed the 5% cap (~40 files), that's a D8 exit-criterion
  question worth flagging to the user rather than silently allowlisting past the cap or silently
  declaring M4 not-done — ask before deciding which way to resolve it.

## Definition of done

- `xoft corpus run` subcommand in `xoft-cli` (sibling of `Corpus manifest`, `Check`, `Transpile` in
  `main.rs`), writes `reports/corpus-report.json`.
- `corpus/allowlist.toml` created, each entry with a one-line reason, total ≤5% of 792 files.
- TDD per `CLAUDE.md`: failing test first. Likely needs a small fixture corpus (tempdir-based, like
  `manifest.rs`'s tests) for the unit-level test, since running against the real out-of-repo corpus
  isn't reproducible in CI the same way — decide this shape and, if genuinely unclear from how
  `manifest.rs`'s existing tests already solved the same problem, ask before coding.
- Confirm the "byte-stable across consecutive runs" exit criterion by literally running it twice
  and diffing (`git diff --exit-code reports/` is M4.2's own check, but M4.1 should self-verify
  this before calling itself done).
- Update `docs/progress/` with a new `m4-corpus-runner.md` (M3's file is
  `docs/progress/m3-diagnostics-cli.md` — follow its structure) and `PROGRESS.md`'s table row.
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` only if something genuinely mistake-worthy came up.

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments,diagnostic,rule}.rs` + `build.rs`:
  all green, unchanged this round. M1/M2 done (rounds 26/29).
- `crates/xoft-core/src/diagnostic.rs`: `error_message` table now has two grounded entries —
  `"assignment"` (round 28) and `"module"` (this round, round 31) — both found by probing real
  parser output, not guessed. Still deliberately small; add entries only from evidence.
- `crates/xoft-cli/src/{manifest,check,transpile,main,lib}.rs`: unchanged this round. `xoft check`
  and `xoft transpile` both work end-to-end (M3.2, round 30).
- **New this round (M3.3, round 31):** `crates/xoft-cli/tests/fixtures/broken/*.mod` (8 hand-
  written broken files) + `crates/xoft-cli/tests/broken_fixtures.rs` (one parametrized test,
  `insta` snapshots over `check_source`'s rendered output plus structural assertions against
  `CheckResult::diagnostics`) + `crates/xoft-cli/tests/snapshots/*.snap` (8 accepted snapshots).
  `insta = "1.48.0"` added dev-only to `xoft-cli`. **M3 is fully done** (M3.1 + M3.2 + M3.3).
- `grammar.js`/`src/scanner.c`: unchanged since round 24, M1 is frozen unless a new corpus gap
  surfaces (M4.1 running the real corpus at full round-trip scale for the first time might surface
  one — if so, that's a scoping question for the user per M1's precedent, not an automatic reopen).
- `cargo test --workspace`: green, 28 tests in `xoft-core` (unchanged) + 9 in `xoft-cli` (8 → 9
  this round: the one new parametrized `broken_fixtures` test, covering all 8 fixtures).
