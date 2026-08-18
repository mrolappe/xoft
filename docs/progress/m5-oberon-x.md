# M5 — Toy dialect Oberon-X

## M5.1 — `grammars/tree-sitter-oberon-x/grammar.js` extending the base ✅ (round 35, 2026-08-18)

Scoping questions (from `NEXT.md`) resolved with the user before coding:

- **Bootstrap:** fork `tree-sitter-oberon2/` wholesale (`rsync -a --exclude 'gen-src/'`), not a
  fresh `tree-sitter-cli init` from the EBNF again.
- **`xoft-core` wiring:** none this round. M5.1 stays confined to `grammars/tree-sitter-oberon-x/`
  and `tree-sitter test`; a second `tree_sitter::Language` waits for M5.2, which is where
  Oberon-X's parsed tree first gets consumed.
- **`BEGIN` → `DO`:** synonym, not a breaking rename. Any base Oberon-2 corpus file stays valid
  Oberon-X input.
- **Test fixtures:** native tree-sitter convention, `grammars/tree-sitter-oberon-x/test/corpus/`
  (not a new top-level `corpus/cases/`) — this dialect has no real-world source to sweep, only
  hand-written cases.

### What changed

`grammars/tree-sitter-oberon-x/` forked from `tree-sitter-oberon2/` (`grammar.js`, `package.json`,
`LICENSE`, `NOTICE`, `queries/highlights.scm`, `test/corpus/*`, `src/scanner.c`). `sweep_corpus.py`
dropped from the fork — it's a real-corpus tool and Oberon-X has no real corpus. `NOTICE` updated
to record the second-order provenance (forked from this repo's own `tree-sitter-oberon2`, which
in turn was forked from upstream `viegasfh/tree-sitter-oberon-2`).

Two grammar changes, both additive to the copied `grammar.js`:

- `kUnless: $ => 'UNLESS'` (new keyword token) and `unless_statement: $ => seq($.kUnless,
  $.expression, $.kDo, optional($.statement_seq), $.kEnd)` — reuses the existing `kDo`/`kEnd`
  tokens `while_statement` already uses, added as a new arm of `statement`'s `choice(...)`.
- `choice($.kBegin, $.kDo)` at both sites where `kBegin` previously appeared alone —
  `module`'s optional `BEGIN`/`CLOSE` section and `procedure_body`'s optional body — making `DO`
  a lexical synonym for `BEGIN` everywhere the latter introduces a `statement_seq`.

`grammar.js`'s `name` changed `'oberon2'` → `'oberon_x'`; `src/scanner.c`'s external-scanner
symbols renamed `tree_sitter_oberon2_external_scanner_*` → `tree_sitter_oberon_x_external_scanner_*`
to match (see `docs/errors.md` round 35 — this is a linker-time requirement, not cosmetic).

TDD: `test/corpus/oberon_x.txt`, 4 new cases (`DO` as `BEGIN` synonym at both procedure- and
module-level, `UNLESS` with a body, `UNLESS` with an empty body). Confirmed red first — ran
`tree-sitter test --file-name oberon_x.txt` against the still-unmodified forked grammar and saw
all 4 fail with `ERROR` nodes around the unrecognized `DO`/`UNLESS` tokens — before writing the
grammar changes. Expected S-expressions generated via `tree-sitter test --update` (not
hand-written) and read before accepting, per the standing checklist item. `tree-sitter generate`
run immediately after the rule-shape change, before touching tests further; produced only the
grammar's pre-existing `unnecessary conflicts: selector, actual_params` warning (present in the
unmodified base fork too, not a regression from this round's changes). Full suite:
**89/89 (100%)** — the 85 inherited base-grammar cases plus the 4 new ones.

### Model note

This task ran on Sonnet, per `docs/plan.md`'s row-level tagging (M5.1 Sonnet / M5.2 **Opus** /
M5.3 Haiku) — flagged to the user up front since a stale note in the M4-round `NEXT.md` had
mis-stated M5 as uniformly Opus-tagged.

M5.2 (mapping rules + emit path) and M5.3 (round-trip tests) not started.

## M5.2 — Two mapping rules + emit path ✅ (round 36, 2026-08-18)

Design decisions (settled with the user before coding):

- **One mapping rule per M5.1 feature.** Rule A: `DO` as a block opener → `BEGIN`. Rule B:
  `UNLESS E DO S END` ⟷ `IF ~(E) THEN S END`. Negation is `~`, not `NOT` —
  `docs/language-baseline.md:112` gives Oberon-2 negation as `"~" Factor`. (`grammar.js` also
  accepts `NOT` as an STJ-Oberon lexical synonym, but `~` is the normative spelling.)
- **Not a `rule::Rule`.** That trait is `check(&Tree, &str) -> Vec<Diagnostic>` — diagnostic-
  producing, with no way to return replacement text. D5 specifies a different shape. New module
  `crates/xoft-core/src/mapping.rs`; `rule.rs` stays reserved for diagnostics.
- **Emit = M2's `Span` machinery, extended.** `serialize::walk` was generalized into
  `walk_with(tree, text, emit)`, where `emit` sees each *leaf* and may replace it with any number
  of spans. Gaps are never offered to `emit`. That is "template splicing with inherited
  indentation" literally: indentation is never computed, it is the original bytes carried forward.
  `reconstruct` reused unchanged. No pretty-printer.

### What changed

- `crates/xoft-core/src/mapping.rs` (new): `to_oberon2(&Tree, &str) -> String` and
  `to_oberon_x(&Tree, &str) -> String`. Both build a `HashMap<leaf start byte, Edit>` from a
  tree walk, then splice via `walk_with`. `Edit { before, text: Option<&str>, after }` —
  `text: Some("")` deletes a leaf, `None` keeps it verbatim.
- `crates/xoft-core/src/serialize.rs`: `walk_with` extracted; `walk` is now a one-line
  delegation, so the gap logic lives in exactly one place.
- `crates/xoft-core/src/grammar.rs`: second `Language`, `language_oberon_x()` (C symbol
  `tree_sitter_oberon_x`). `language()` unchanged, still Oberon-2.
- `crates/xoft-core/build.rs`: loops over `["oberon2", "oberon-x"]` instead of hard-coding one
  grammar dir; same 8 lines of `cc::Build` per grammar.
- `.github/workflows/ci.yml`: matching `tree-sitter generate ... -o gen-src` step for
  `tree-sitter-oberon-x`, since `xoft-core` now links it too. Verified by deleting *both*
  `gen-src/` dirs locally, re-running the exact CI commands, and re-running `cargo test`.

### The invertibility finding (matters for M5.3 and for M5's exit criterion)

`X→2→X` **cannot** be byte-identical for Rule A, and no emit path can fix that. `BEGIN` and `DO`
are *synonyms* in Oberon-X: two X spellings collapse onto the single Oberon-2 spelling `BEGIN`,
so the mapping is many-to-one and the reverse direction has no information to recover which was
written. Making 2→X rewrite `BEGIN`→`DO` does not help — it merely moves the loss onto Oberon-X
sources that spell it `BEGIN`. So Rule A stays one-way (2→X leaves `BEGIN` alone, as decided;
base Oberon-2 is already valid Oberon-X by M5.1's synonym decision), and the honest invariants
are:

| Round trip | Guarantee |
|---|---|
| `2→X→2` | byte-identical, unconditionally |
| `X→2→X`, Rule B only | byte-identical (Rule B *is* a bijection on the shape it emits) |
| `X→2→X`, Rule A | byte-identical up to `DO` block openers normalizing to `BEGIN` |

Rule B stays a bijection because `to_oberon_x` matches *exactly* the shape `to_oberon2` emits —
`IF ~ ( E ) THEN [S] END`, no `ELSIF`, no `ELSE`, the `~(…)` spanning the whole condition. An
`IF` that Rule B did not produce (`IF x = 0 THEN`, `IF ~ok THEN`, `IF ~(x=0) & ok THEN`, anything
with an `ELSE`) is left untouched, which is what keeps `2→X→2` byte-identical.

This is a measured cost of a synonym-style dialect feature, not a defect — worth carrying into
M5's exit write-up: an *additive* construct (Rule B) round-trips losslessly; a *synonym* (Rule A)
never can.

### Tests

`crates/xoft-core/tests/mapping.rs`, 10 tests, confirmed red first (the whole file failed to
compile against a nonexistent `mapping` module and `grammar::language_oberon_x`). Fixtures mirror
`grammars/tree-sitter-oberon-x/test/corpus/oberon_x.txt`, plus three cases that file has no reason
to carry: a single-leaf `UNLESS` condition (`UNLESS ok DO` — the one splice case where prefix and
suffix land on the *same* leaf), a `WHILE ... DO` inside a `DO` block (proving Rule A does not
touch non-block-opener `DO`), and a comment plus ragged indentation straddling the rewrite
(proving gaps are inherited verbatim). Each round-trip direction has its own test, including the
Rule A normalization asserted explicitly rather than left as a known gap.

`cargo test --workspace` green: `xoft-core` 28 → 38, `xoft-cli` 14 unchanged.
`tree-sitter test` in `grammars/tree-sitter-oberon-x/` still 89/89 — M5.2 consumes the grammar,
it does not touch it.

M5.3 (bidirectional round-trip tests against golden files, Haiku) not started.
