# M4 — Corpus runner

## M4.1 — `xoft corpus run` → `reports/corpus-report.json`, honoring the allowlist ✅ (round 32, 2026-08-11)

`crates/xoft-cli/src/corpus_run.rs`, new module. Two-layer split, pure aggregation separated
from I/O so the counting/bucketing rules are unit-testable without a real corpus:

- `aggregate(allowlist: &Allowlist, outcomes: Vec<FileOutcome>) -> CorpusReport` — no I/O.
  Buckets each non-allowlisted outcome by which metric(s) failed (`"parse"`, `"round-trip"`,
  or `"parse+round-trip"`), builds a per-root `BTreeMap<String, RootBreakdown>` and a
  `BTreeMap<String, usize>` failure histogram (both sorted by construction), and computes
  `parse_pct`/`round_trip_pct` over `counted_files` (= `total_files - allowlisted_files`,
  D8). `failures` is sorted `(root, path)` before returning, so aggregation is deterministic
  regardless of input order.
- `run(roots: &[Root], allowlist: &Allowlist) -> Result<CorpusReport>` — the I/O layer. Reuses
  `manifest::build`'s file list (not a second walk) and `transpile::transpile_source`'s
  parse+round-trip logic (not a third implementation) per `NEXT.md`'s explicit steer. Compares
  `transpile_source`'s `output_bytes` against the raw bytes read for the manifest walk, so
  round-trip is checked against the real file, not a re-derived value.

`transpile::transpile_file` was split into `transpile_source(filename, text) -> TranspileResult`
(pure, already-decoded text) plus a thin `transpile_file(path)` wrapper that reads and decodes
before delegating — pulled out so `corpus_run::run` doesn't re-read a file `manifest::build`
already read, without duplicating the parse/round-trip body. Existing `transpile.rs` tests
covered the refactor for free (same public `transpile_file` signature, unchanged).

`Allowlist { entry: Vec<AllowlistEntry> }` (`AllowlistEntry { root, path, reason }`,
`#[derive(Deserialize)]`), same `[[entry]]`-array-of-tables TOML shape as `RootsConfig`'s
`[[root]]`. `Allowlist::contains` matches on exact `(root, path)`.

`main.rs` gained `Corpus Run { roots, allowlist, out }`, sibling of `Corpus Manifest`, same
`--roots`/`--out` flag shape plus `--allowlist` (default `corpus/allowlist.toml`). Exits 1
unless every counted file is both parse-ok and round-trip-ok.

TDD: `crates/xoft-cli/tests/corpus_run.rs`, 5 tests, all red (missing `corpus_run` module)
before implementation, all green after — no failed assumption mid-implementation. Four test
`aggregate` directly with synthetic `FileOutcome`s (clean/broken counts and percentages,
allowlist exclusion from both the numerator and denominator, all three failure-histogram
buckets including a synthetic round-trip-only failure the real corpus doesn't happen to
contain, byte-stable JSON across two calls); one drives `run` end-to-end through a tempdir
fixture (mirrors `manifest.rs`'s fixture style) to prove the manifest-walk/allowlist-matching/
absolute-path-leak-freedom/determinism chain works together, not just each piece in isolation.

### The real corpus run — 26/792 (3.28%) allowlisted, 100%/100% on the rest

Ran `xoft corpus run` against the actual `corpus/roots.toml` paths (all four roots resolve on
this machine) with an empty allowlist first, to see real output before writing any allowlist
entries — per `NEXT.md`'s explicit instruction not to port M1's backlog blind. Result: **766/792
parse clean, 792/792 round-trip clean** — every failure is parse-only, confirming M2's ad hoc
240-file round-26 sample (round-trip correctness is orthogonal to parse coverage) at full
corpus scale for the first time. 26 failures matches M1's round-26 residual count exactly
(792 − 766 = 26), but this run independently re-derived *which* 26 and *why*, rather than
trusting that number's staleness.

Every failure re-confirmed against its actual bytes/diagnostic this round (not copied from
M1's notes): read raw bytes for size/tail-byte checks, ran `xoft check` for the real diagnostic
message + span, and for one file, bisected by truncating the source at increasing line counts
(`head -n N` + a synthetic `END X.` to keep it parseable) until the failure disappeared, to
localize a single-token cause inside a 421-line file whose reported `ERROR` span was the whole
file. Full reasons in `corpus/allowlist.toml`; categories:

- **7 files** — Amiga Oberon's real conditional-compilation preprocessor (comment-embedded
  `$IF`/`$ELSE`/`$END` or bracket-pragma `<*IF*>`/`<*ELSE*>`/`<*END*>`, both branches' full code
  present unconditionally, no parseable separator between them) — already known from M1 rounds
  20/23/25, now with all 7 instances (`Break.mod`, `NoGuru.mod`, `amiga-oberon-31`'s
  `OberonLib.mod`, `IntuiPointerDemo.mod`, `Kernel.mod`, `oberon-a`'s `OberonLib.mod`,
  `amiga/Utility.mod`) confirmed via their actual `$IF`/bracket-pragma text.
- **8 files** — 87/68-byte "moved to..." stub files, confirmed via `stat` size matching M1
  round-25's count exactly.
- **4 files** — a single stray non-source byte unique to that one file corpus-wide (two NUL at
  EOF, one 0xFE, one 0x08 backspace) — confirmed via direct byte inspection this round, matching
  M1 round-25's tally.
- **2 files** — `\"` string-escape dialect gap, already flagged (M1 round 21) as directly
  conflicting with `voc`'s own `"\"`-as-complete-string usage; still unresolved, still
  allowlisted rather than picking a reading arbitrarily.
- **1 file** — malformed preamble (`@DATABASE` AmigaGuide directive with no enclosing comment,
  before `MODULE` even appears).
- **3 files** — `voc`'s trailing-content files: two have Oberon System tool-command text
  appended after `END` (`MultiArrayRiders.Mod`, `MultiArrays.Mod` — e.g. `System.Free
  MultiArrays~`, a "select and execute" convention from the interactive Oberon System, not part
  of the module), one (`ethUnicode.Mod`) has a binary font-resource blob (`Oberon10.Scn.Fnt`)
  concatenated after `END` — confirmed by reading past each file's `END <Module>.` line.
- **1 file, newly characterized** — `voc/ulm/ulmRandomGenerators.Mod` uses the literal `1.`
  (no digit after the decimal point) inside an expression (`(1. - real - real)`).
  `grammar.js`'s `real` rule already documents *why* it requires ≥1 trailing digit — to avoid
  swallowing the first `.` of the `..` range operator (`2..4`) via maximal munch — with the
  comment "no real-world corpus code relies on [the bare-decimal form]." This file is a live
  counter-example to that claim, surfaced for the first time by this round's full-corpus,
  round-trip-inclusive sweep (M1's own sweep only checked parse success, and evidently never
  hit this exact file/expression in prior sampling). **Asked the user** (`AskUserQuestion`,
  since M1 is frozen and reopening it mid-M4.1 is a real scope question, not a routine
  allowlist entry): allowlist it and keep M1 frozen, vs. reopen M1 now to widen `real` and
  re-derive the `..` disambiguation. User chose to allowlist — one file, well under the 5% cap,
  Phase 2 backlog like the rest.

`corpus/allowlist.toml` created: 26 entries, each with a one-line reason grounded in this
round's own verification (not copy-pasted from M1's docs), grouped by category with a short
comment block per group. 26/792 = 3.28%, under the 5% (~40 file) D8 cap.

Confirmed byte-stability: ran `corpus run` twice, `diff`'d `reports/corpus-report.json`,
identical both times.

`reports/corpus-report.json` (checked in): `total_files: 792`, `allowlisted_files: 26`,
`counted_files: 766`, `parse_ok: 766`, `round_trip_ok: 766`, `parse_pct: 100.0`,
`round_trip_pct: 100.0`, empty `failure_histogram`/`failures` — **D8's exit criterion is met**:
byte-identical round-trip + zero `ERROR`/`MISSING` on 100% of the non-allowlisted corpus.

`cargo test --workspace` green: `xoft-cli` 9 → 14 (5 new `corpus_run` tests), `xoft-core`
unchanged (28).

M4.2 (CI: fail on undiffed report change) is the only M4 item left.
