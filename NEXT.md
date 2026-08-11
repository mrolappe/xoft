# Next task

**M4.2 — CI: fail on undiffed report change.** Per `docs/plan.md`: `cargo test` + `corpus run` +
`git diff --exit-code reports/`. M4.1 (`xoft corpus run` + `corpus/allowlist.toml` +
`reports/corpus-report.json`) is done as of this round (round 32) — D8's exit criterion is met
(766/792 counted, 100% parse, 100% round-trip, 26 allowlisted at 3.28%, under the 5% cap).

## The scoping question to resolve first — don't guess, ask

**The corpus is not in this repository and cannot be.** `corpus/roots.toml` pins four
machine-local absolute paths (`/Users/mrolappe/studio/oberon-a-fs-uae-env/...`,
`.../Nextcloud/retro-comp/...`, `.../atari-retro-dev/...`, `.../git-repos/voc/...`) to archived
third-party sources deliberately not vendored (license reasons, see `corpus/roots.toml`'s own
header comment and `docs/plan.md`). A CI runner (GitHub Actions or otherwise) has none of these
paths — `xoft corpus run` as it exists today would fail outright in CI (`reading <path>: No such
file or directory` from `manifest::build`'s `WalkDir::new(&root.path)`, since the root itself
won't exist), not just produce a diff.

`docs/plan.md`'s M4.2 row ("`cargo test` + `corpus run` + `git diff --exit-code reports/`") reads
as if the runner *has* the corpus. It may be understating the real setup this needs (a corpus
cache/mirror step, a self-hosted runner with the paths mounted, a subset-corpus fixture checked
into the repo for CI's purposes while the full corpus stays a local-only "the human runs this
before pushing" check, or something else not yet decided) — this is exactly the kind of
ambiguous-scope case `CLAUDE.md` says to stop and ask about rather than build a guess. **Ask the
user how CI is supposed to get corpus access before writing any workflow file.** Options worth
presenting: (a) CI only runs `cargo test --workspace` and the report-freshness check is a
local/pre-push discipline, not automated; (b) a small fixture corpus gets vendored into the repo
specifically for CI (separate from the real `corpus/roots.toml` machine-local one); (c) CI has
no access at all and M4.2 is descoped/redefined; (d) something else the user has in mind for
where this actually runs.

## What's confirmed (do not re-derive, just verify before coding)

- `xoft corpus run` (`crates/xoft-cli/src/corpus_run.rs`, `main.rs`'s `Corpus Run` subcommand)
  exists, is TDD-tested (`crates/xoft-cli/tests/corpus_run.rs`, 5 tests), and produces a
  byte-stable `reports/corpus-report.json` when the corpus is reachable — confirmed by running it
  twice locally and diffing. `git diff --exit-code reports/` itself is a one-line addition once
  the corpus-access question above is answered; the report generation side is not blocked.
  Read `docs/progress/m4-corpus-runner.md` for the full derivation.
- `corpus/allowlist.toml` (26 entries, 3.28%) and `reports/corpus-report.json` are both checked
  in already. Re-running `corpus run` on an unchanged corpus should reproduce the checked-in
  report byte-for-byte — that reproducibility *is* what M4.2's CI check verifies, so a good first
  local sanity step for M4.2 is confirming that (`cargo run -p xoft-cli -- corpus run && git diff
  --exit-code reports/`) still passes before touching CI config, since corpus content on disk
  could in principle have drifted since this round.
- No CI config exists anywhere in this repo yet (`.github/workflows/` doesn't exist) — M4.2 is
  the first CI setup of any kind for this project, not an addition to an existing pipeline.

## Definition of done

- The corpus-access scoping question above resolved with the user before any workflow file is
  written.
- Whatever CI shape is agreed on, wired up and demonstrated working (or explicitly explained why
  it can't be demonstrated in this environment, e.g. no ability to push and watch Actions run —
  ask the user how they want to verify it if so).
- Usual end-of-round ritual: `PROGRESS.md` round table + `docs/progress/` file (new
  `m4-corpus-runner.md` continuation or a dedicated M4.2 section, match M3's per-file-covers-
  whole-milestone precedent), `docs/insights.md`/`docs/errors.md`/`docs/checklist.md` only if
  something genuinely mistake-worthy came up, `cargo test --workspace`.
- If M4.2 lands cleanly, **M4 is done** — next milestone per `docs/plan.md` is M5 (toy dialect
  Oberon-X), which `docs/plan.md` line 147 notes is "written from the corpus report, the
  allowlist and the measured Oberon-X cost" and tagged **Opus** — worth flagging to the user at
  that point rather than silently starting M5 on whatever model is running this round.

## State of the tree

- `crates/xoft-core/`: unchanged this round (M1/M2/M3 core untouched). 28 tests, all green.
- `crates/xoft-cli/src/{check,manifest,main,lib}.rs`: unchanged in shape this round except
  `main.rs`'s new `Corpus Run` arm and `lib.rs`'s new `pub mod corpus_run;` line.
- `crates/xoft-cli/src/transpile.rs`: `transpile_file` split into `transpile_source(filename,
  text) -> TranspileResult` (pure) + a thin `transpile_file(path)` wrapper, so `corpus_run::run`
  can reuse the parse+round-trip logic on bytes it already read via `manifest::build` without a
  second file read. Existing `transpile.rs` tests (2, unchanged) covered this refactor for free —
  same public signature.
- **New this round (M4.1, round 32):** `crates/xoft-cli/src/corpus_run.rs` (`Allowlist`/
  `AllowlistEntry`, `FileOutcome`, `CorpusReport`/`RootBreakdown`/`Failure`, pure `aggregate()` +
  I/O `run()`), `crates/xoft-cli/tests/corpus_run.rs` (5 tests), `corpus/allowlist.toml` (26
  entries), `reports/corpus-report.json` (checked in, byte-stable). `main.rs` gained `Corpus Run
  { roots, allowlist, out }`.
- `cargo test --workspace`: green, 28 tests in `xoft-core` (unchanged) + 14 in `xoft-cli` (9 → 14:
  the 5 new `corpus_run` tests).
- `grammar.js`/`src/scanner.c`: unchanged. M1 stays frozen — the one new corpus gap M4.1's full
  sweep surfaced (`voc/ulm/ulmRandomGenerators.Mod`'s bare-decimal real literal `1.`) was, per
  user decision this round, allowlisted rather than used to reopen M1.
