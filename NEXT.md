# Next task

**M5.3 — Bidirectional round-trip tests `X→2→X`, `2→X→2` against golden files.** Tagged **Haiku**
in `docs/plan.md` line 133 ("golden files in `corpus/cases/`"). M5.2 (round 36) built and unit-
tested the mapping rules; M5.3 promotes that to a file-driven, integration-level suite. This is
the last M5 item — after it, M5's exit criterion ("a measured answer to *what does one dialect
experiment cost?*") gets written up.

## What already exists (do not rebuild)

`crates/xoft-core/src/mapping.rs`, public API:

```rust
pub fn to_oberon2(tree: &Tree, text: &str) -> String;   // tree from grammar::language_oberon_x()
pub fn to_oberon_x(tree: &Tree, text: &str) -> String;  // tree from grammar::language()
```

`crates/xoft-core/src/grammar.rs` exposes both languages: `language()` (Oberon-2, unchanged name)
and `language_oberon_x()`. `crates/xoft-core/build.rs` compiles both grammar dirs;
`.github/workflows/ci.yml` runs `tree-sitter generate -o gen-src` for both.

`crates/xoft-core/tests/mapping.rs` is the unit-level version of M5.3's suite: 10 tests, in-code
fixtures, both directions plus both round trips. **Read it before writing M5.3** — the fixtures
there are the shapes the golden files should cover, and its assertions are the ones to lift.

## The two mapping rules

| | Oberon-X | Oberon-2 |
|---|---|---|
| A | `DO` opening a module or procedure body | `BEGIN` |
| B | `UNLESS E DO S END` | `IF ~(E) THEN S END` |

Negation is `~`, never `NOT` (`docs/language-baseline.md:112`). Rule B's reverse matches *only*
the exact shape Rule B emits — `IF ~ ( E ) THEN [S] END`, no `ELSIF`, no `ELSE`, the `~(…)`
covering the whole condition. Any other `IF` is left untouched, deliberately.

## The invariant M5.3 must assert (read this before writing a single golden file)

Round 36 found that "byte-identical in both directions" is **not** achievable for Rule A, and no
amount of test or emit work changes that. `BEGIN` and `DO` are synonyms in Oberon-X, so X→2 is
many-to-one and has no inverse. What actually holds:

| Round trip | Guarantee |
|---|---|
| `2→X→2` | byte-identical, unconditionally |
| `X→2→X` where only Rule B applies | byte-identical |
| `X→2→X` where Rule A applies | byte-identical **up to `DO` block openers becoming `BEGIN`** |

So the golden-file suite needs **three** groups, not two, and the Rule A group's expected `X` side
after a round trip is the `BEGIN`-normalized text, asserted on purpose — see
`round_trip_x_2_x_normalizes_do_openers_to_begin` in `tests/mapping.rs`. Do not write a
`DO`-spelled fixture into the "byte-identical" group and then weaken the assertion to make it
pass. Also make sure at least one Oberon-X fixture spells its block opener `BEGIN` (legal, and the
case that catches anyone "fixing" Rule A by making it symmetric).

## Suggested shape

- Golden files in `corpus/cases/` (the plan's stated home; the directory does not exist yet —
  `corpus/` currently holds `roots.toml`, `manifest.json`, `allowlist.toml`, `fixtures/`).
  Pairs, e.g. `corpus/cases/unless_body.x.mod` + `corpus/cases/unless_body.2.mod`.
- One integration test in `crates/xoft-core/tests/` (or `crates/xoft-cli/tests/` if it reads
  files — remember `xoft-core` does no I/O, so a file-reading harness belongs in `xoft-cli`; that
  is a real decision to make, not a formality). Table-driven over the directory, one assertion
  block per direction.
- The 6 unit fixtures worth promoting: `DO` at procedure level, `DO` at module level, `UNLESS`
  with a body, `UNLESS` with an empty body, `UNLESS` with a single-leaf condition (`UNLESS ok DO`
  — prefix and suffix land on the same leaf, the one splice case that can silently drop an
  insertion), and one with a comment plus ragged indentation straddling the rewrite.
- Every fixture must parse with zero `ERROR`/`MISSING` before anything else is asserted; the unit
  tests' `parse` helper already does this and is worth copying.

## Not in scope

Wiring `mapping` into `xoft-cli` (`xoft transpile --to oberon2` or similar) — no milestone asks
for it, and `transpile` currently means "check + lossless round trip" per round 30. Raise it as a
Phase 2 / M7 item rather than doing it here. Likewise the real 792-file corpus: Oberon-X has no
real-world source, only hand-written cases.

## State of the tree

- `cargo test --workspace` green: `xoft-core` 38, `xoft-cli` 14.
- `tree-sitter test` in both `grammars/tree-sitter-oberon2/` and `grammars/tree-sitter-oberon-x/`
  green (85 and 89). M5.2 consumed the Oberon-X grammar without touching it.
- `crates/xoft-core/src/serialize.rs` gained `walk_with(tree, text, emit)`; `walk` is now a
  one-line delegation to it. `emit` sees each *leaf* and returns any number of spans; gaps are
  never offered to it, which is what makes indentation "inherited" — there is no code that
  computes layout anywhere in the crate.
- `crates/xoft-core/src/rule.rs` is untouched and still empty of real rules. Its doc comment says
  "M5 is what actually registers rules" — that is now known to be misleading: the `Rule` trait is
  diagnostic-shaped (`check(&Tree, &str) -> Vec<Diagnostic>`) and the mapping rules deliberately
  do not go through it. Worth correcting that comment if M5.3 touches the file, but not worth a
  commit on its own.
- Both `gen-src/` dirs are gitignored and generated locally. A fresh checkout needs
  `tree-sitter generate grammars/tree-sitter-oberon{2,-x}/grammar.js -o grammars/tree-sitter-oberon{2,-x}/gen-src`
  before `cargo build` will link.
