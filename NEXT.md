# Next task

**M3.3 — Broken-source fixtures + `insta` snapshots.** Per `docs/plan.md`: "~8 hand-written broken
files" with `insta` snapshot tests over the rendered diagnostic output. M3.1 (`Diagnostic` walk)
and M3.2 (`xoft check`/`xoft transpile` + `codespan-reporting`) are both done.

## What's confirmed (do not re-derive, just verify before coding)

- `xoft-core::diagnostic::diagnostics(tree) -> Vec<Diagnostic>` (M3.1) walks `ERROR`/`MISSING`
  nodes; its `error_message` lookup table has exactly one grounded entry today (`parent.kind() ==
  "assignment"`), everything else falls back to a generic `"unexpected syntax"` message
  (`crates/xoft-core/src/diagnostic.rs`). `docs/insights.md` round 28 flagged this table as
  deliberately small, expecting M3.3's fixtures to surface real contexts worth adding — that's
  likely this milestone's actual work, not just writing snapshot tests over the existing table.
- `xoft-cli::check::check_source(filename, text) -> CheckResult { diagnostics, rendered }` (M3.2,
  `crates/xoft-cli/src/check.rs`) is the thing to snapshot: `rendered` is the codespan-reporting
  text, `diagnostics` is the structured list. `docs/insights.md` round 30 explains why both are
  kept — snapshot the rendered text, assert facts (message content, count) against the structured
  list, don't derive facts by parsing the snapshot string.
- `insta` is not yet a dependency anywhere in the workspace — this milestone adds it (dev-only,
  likely to `xoft-cli` since that's where `check_source`/rendering live). Check `cargo add
  --dev insta` picks up a version compatible with the crates already pinned (`tree-sitter = 0.26`
  etc. shouldn't matter, `insta` has no cross-deps with them).
- "~8 hand-written broken files" — plan doesn't say which 8 breakages. Reasonable candidates,
  cross-checked against what M3.1's tests already cover (don't just duplicate `diagnostic.rs`'s
  own 4 test fixtures as snapshots): missing `;` (existing "assignment" table entry, but a fresh
  file, not the same one path as `crates/xoft-core/tests/diagnostic.rs`), unbalanced `(`/`)`,
  unbalanced `BEGIN`/`END`, a bad `CASE` label, an `IF` with no matching `END`, a malformed
  `PROCEDURE` heading, a stray token where a declaration was expected, two diagnostics in one file
  (to check ordering/multiple-label rendering, not yet exercised by anything so far). Sample the
  real grammar for each (small Rust probe or `tree-sitter parse`, per the checklist's "hand-wrote
  an expected tree from memory" mitigation) before writing the fixture+snapshot, don't guess the
  shape.
- Where do the 8 fixture files live? Not decided yet — `crates/xoft-cli/tests/fixtures/broken/` or
  similar seems natural (parallel to how `manifest.rs`'s tests build files into a `tempdir`, but
  these are meant to be committed and read by name, not generated), but this wasn't settled in any
  prior round. **Ask the user before coding** if it's not obvious once you look at how `insta`
  conventionally wants its inputs laid out (it has opinions about snapshot file placement via
  `insta::assert_snapshot!` + `.snap` files next to the test).

## Definition of done

- A failing-then-passing test per file (or one parametrized test iterating all ~8 fixtures),
  TDD per `CLAUDE.md`. `insta` snapshots get reviewed/accepted (`cargo insta review` or
  `INSTA_UPDATE=always`) as part of turning them green, not hand-typed.
- If sampling the fixtures surfaces new `ERROR`-node parent kinds worth a table entry, add them to
  `error_message` in `crates/xoft-core/src/diagnostic.rs` (that's `xoft-core`, not `xoft-cli` —
  the fixture/snapshot harness itself is CLI-side per the no-I/O rule, but the table it's
  exercising lives in core).
- Update `docs/progress/m3-diagnostics-cli.md`'s M3.3 section (currently "not started"); M3 as a
  whole can likely be declared done once this lands, unless something in it surfaces new scope.
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` only if something genuinely mistake-worthy came up.

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments,diagnostic,rule}.rs` + `build.rs`:
  all green, unchanged this round. M2 done (round 29); M3.1 done (round 28).
- `crates/xoft-cli/src/{manifest,check,transpile,main,lib}.rs`: M3.2 done this round (round 30).
  `xoft check <file>` and `xoft transpile <file> [--out path]` both work end-to-end (manually
  verified against a clean and a broken fixture, not just via `cargo test`) — `check` exits 1 on
  any diagnostics and prints a `<file>: OK` line when clean; `transpile` round-trips
  byte-identically via M2's serializer and also exits 1 on diagnostics, writing to `--out` or
  stdout (raw bytes, not `println!`, since output can be non-UTF-8).
- `codespan-reporting = "0.12"` added to `xoft-cli/Cargo.toml` this round — first use anywhere in
  the workspace, confined to the CLI crate. `insta` is the next new dependency to add, for M3.3.
- `grammar.js`/`src/scanner.c`: unchanged since round 24, M1 is frozen unless a new corpus gap
  surfaces.
- `cargo test --workspace`: green, 31 tests in `xoft-core` (unchanged) + 8 in `xoft-cli` (4 → 8
  this round: 2 new in `tests/check.rs`, 2 new in `tests/transpile.rs`, plus the existing 4 in
  `tests/manifest.rs`).
