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
