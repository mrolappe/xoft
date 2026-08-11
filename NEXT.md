# Next task

**M5.1 — `grammars/tree-sitter-oberon-x/grammar.js` extending the base.** Per `docs/plan.md`:
a new toy dialect grammar, forked from the M1 base grammar, with two suggested extensions:
`BEGIN` → `DO` (a rename, presumably of a statement-sequence delimiter) and a new
`UNLESS Expr DO StatementSeq END` statement. Tagged **Sonnet** in `docs/plan.md` line 131 — M5
as a whole is not one model; only M5.2 (the mapping-rule/emit path) is tagged **Opus** (line
132), M5.3 (round-trip tests) is tagged **Haiku** (line 133). Flag this breakdown to the user
before starting, since M4's `NEXT.md` (superseded) mis-stated M5 as uniformly Opus-tagged.

## What M5 is for

Per `docs/plan.md` line 135, M5's exit criterion is "a measured answer to 'what does one dialect
experiment cost?'" — this milestone is itself a measurement exercise (how much work does forking
a real dialect variant take, given M1–M4's infrastructure), not a deliverable dialect. Re-read
`docs/plan.md`'s M5 row (lines 127–135) and the "Delegation packets" section (lines 149–159)
before starting — M5.1 is scoped as a **grammar** task, which per that section should receive
only "the relevant EBNF fragment from `docs/language-baseline.md`, the matching section of
`grammar.js`, the `tree-sitter test` corpus format, and 2–3 real corpus snippets" — not the
whole corpus report.

## Open questions worth raising with the user before coding

- **Where does `grammars/tree-sitter-oberon-x/` come from?** M1's grammar lives in
  `grammars/tree-sitter-oberon2/`. Nothing in this repo yet copies or forks it — check whether
  the expectation is a fresh `tree-sitter-cli init` pointed at a copy of `tree-sitter-oberon2`'s
  `grammar.js`/`src/scanner.c`, or some other bootstrap. Given M1's grammar took 26 rounds of
  real corpus-driven work, forking it wholesale (rather than starting from the upstream Oberon-2
  EBNF again) is almost certainly the intent, but confirm rather than assume.
  Also check the sibling crate wiring in `crates/xoft-core/build.rs`/`grammar.rs`: does
  `xoft-core` need a second `Language` (oberon-x alongside oberon2), a runtime choice between
  them, or does M5's grammar work stay confined to `grammars/` and `tree-sitter test` only until
  M5.2 needs to actually invoke it? `docs/plan.md`'s M5.2 note ("mapping rules + emit path")
  implies M5.2 is where oberon-x's parsed tree gets consumed — M5.1 itself may not need any
  `xoft-core` wiring at all. Confirm the boundary before touching `xoft-core`.
- **`BEGIN` → `DO` — rename or coexistence?** "suggested" in `docs/plan.md` leaves open whether
  Oberon-X drops `BEGIN` entirely (a breaking rename) or accepts both as synonyms. A breaking
  rename is more interesting as a "what does a real dialect change cost" measurement (M5's own
  stated purpose) since it forces every construct that uses `BEGIN` to be touched, not just one
  new keyword added — but ask rather than assume, since it also determines whether any base
  Oberon-2 corpus file can still be used as an Oberon-X test input.
- **What corpus/test inputs exist for Oberon-X?** Unlike M1's real corpus, no real-world
  Oberon-X source exists (it's a toy dialect invented for this project) — `docs/plan.md`'s M5.3
  says "golden files in `corpus/cases/`", implying hand-written fixtures, not swept corpus files.
  Confirm `corpus/cases/` is a new directory (doesn't exist yet) before creating it.

## What's confirmed (do not re-derive)

- M4 is done: `xoft corpus run` + `corpus/allowlist.toml` (26/792, 3.28%) + `.github/workflows/
  ci.yml` (fixture-corpus CI check, `corpus/fixtures/` vendored subset of `voc`+`oberon-a`, 12
  files). See `docs/progress/m4-corpus-runner.md` for the full M4.1+M4.2 derivation.
- `docs/language-baseline.md` holds the normative Oberon-2 EBNF M1's grammar (and presumably
  Oberon-X's fork) is built from.
- `crates/xoft-core/build.rs` compiles `gen-src/parser.c` + `src/scanner.c` via `cc` directly
  into `xoft-core`; `grammar.rs` exposes the resulting `tree_sitter::Language`. Whatever M5.1
  needs from `xoft-core` (if anything, per the open question above) will follow this same shape.

## Definition of done

- The scoping questions above resolved with the user before writing `grammar.js`.
- Model-delegation breakdown (M5.1 Sonnet / M5.2 Opus / M5.3 Haiku) flagged to the user; confirm
  whether this round continues on Sonnet for M5.1 specifically or the user wants something else.
- Usual end-of-round ritual: `PROGRESS.md` + `docs/progress/` (new `m5-oberon-x.md`, matching
  the one-file-per-milestone precedent), `docs/insights.md`/`docs/errors.md`/`docs/checklist.md`
  only if something genuinely mistake-worthy came up, `cargo test --workspace` (plus
  `tree-sitter test` in the new grammar dir, per `docs/plan.md`'s verification block).

## State of the tree

- `crates/xoft-core/`, `crates/xoft-cli/`: unchanged this round (M4.2 touched no Rust code at
  all — `corpus_run`'s existing `--roots`/`--allowlist`/`--out` flags were the whole mechanism
  for pointing at a second, CI-scoped corpus).
- **New this round (M4.2, round 33):** `corpus/fixtures/` (`roots.toml`, `allowlist.toml` empty,
  `NOTICE.md` provenance/license notes, `report.json` checked in, 100%/100%), `.github/workflows/
  ci.yml` (first CI config in this repo).
- `grammars/tree-sitter-oberon2/`: unchanged, still the only grammar in the repo. M5.1 is where
  `grammars/tree-sitter-oberon-x/` first appears.
- `cargo test --workspace`: green, unchanged counts (28 `xoft-core`, 14 `xoft-cli`) — M4.2 added
  no tests since it's pure data + CI config, not testable Rust logic.
