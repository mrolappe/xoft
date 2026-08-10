# Next task

**M1.2b — statements: `WITH`, `LOOP`, `EXIT`, `RETURN` as a statement, `CASE` label ranges,
empty statements.**

Everything a fresh session needs to start cold is below.

## What to do

Working tree: `grammars/tree-sitter-oberon2/grammar.js`. Run `tree-sitter generate &&
tree-sitter test` from that directory after every change (`src/` is gitignored — generate it
locally, it's not there after a fresh checkout).

1. **Empty statements**: the EBNF's `Statement` production is `[ ... ]` — the whole thing is
   optional (see `docs/language-baseline.md` line 57). `statement_seq` currently is
   `$.statement, repeat(seq(';', $.statement))` — every element mandatory. Real files rely on
   this (`Out.Ln;` immediately before `END`, or `BEGIN ; END`) — already confirmed by hand
   (`docs/progress/m1-grammar.md` M1.5 entry). Fix: make each element of the sequence
   `optional($.statement)` rather than widening `statement` itself (an empty statement isn't a
   *kind* of statement, it's the absence of one — don't invent a node for it).
2. **`LOOP StatementSeq END`** and **`EXIT`**: two new statement forms, straightforward, add to
   the `statement` choice.
3. **`RETURN [Expr]`** as a statement: NOTE `procedure_body` already has its own
   `optional(seq($.kReturn, $.expression))` hardcoded as the last thing before `END` — that's
   modeling the old (Oberon-2 report's actual) restriction that `RETURN` only appears once, at
   the end. But the EBNF's `Statement` alternative list also includes `RETURN [Expr]` directly
   (line 69) as an ordinary statement, and `docs/insights.md` round 1 lists `RETURN` as missing
   from the `statement` choice. Decide whether to keep `procedure_body`'s existing
   `kReturn`-at-the-end handling AND add a general `RETURN` statement, or replace the former
   with the latter — check whether any corpus file has `RETURN` control flow in the middle of a
   procedure body (early return) before assuming the report's "return is last" restriction
   holds for the dialects in this corpus. Grep before deciding, per this project's repeated
   lesson (see `docs/insights.md` round 3, "a label on a task brief is a hypothesis").
4. **`WITH Guard DO StatementSeq {"|" Guard DO StatementSeq} [ELSE StatementSeq] END`**: new
   `with_statement` rule, `guard = qualident ":" qualident`.
5. **`CASE` label ranges**: `case_label_list`/`label_range`/`label` already exist in
   `grammar.js` (lines ~400-416) and already implement `CaseLabels = ConstExpr [".."
   ConstExpr]` shape via `label_range`. Cross-check against the EBNF fragment above — this item
   in the milestone name may already be done; confirm with a corpus example using a label range
   (e.g. `1..5:`) before writing new grammar for it. Don't re-derive what's already there.

## Definition of done

- `tree-sitter test` still green (20 tests before this round; add corpus cases per construct —
  `test/corpus/statements.txt` or split further if it gets unwieldy).
- The already-confirmed empty-statement gap (`docs/progress/m1-grammar.md` M1.5 entry,
  `Out.Ln;` before `END`) parses with zero `ERROR` nodes.
- Spot-check `LOOP`/`EXIT`/`WITH` against real corpus occurrences (grep the STJ/Oberon-A/
  AmigaOberon roots — see `corpus/roots.toml` for paths) before declaring done, same discipline
  as M1.2a: a construct's EBNF shape and its real-world usage can diverge.
- No changes outside `grammars/tree-sitter-oberon2/` — this task doesn't touch `crates/`.

## Context a fresh session needs

- `docs/plan.md` line 86 — model Sonnet, receives the statement EBNF fragment only.
- `docs/language-baseline.md` lines 55-84 — `FieldList` through `MulOp`, the whole
  statement/expression EBNF, already extracted.
- `docs/insights.md` round 1 ("Empty statements are legal") and round 3 (the `FieldList`
  trailing-separator sibling gap, and the "verify hypotheses against the real corpus before
  coding" lesson from M1.2a — apply it again here).
- `docs/progress/m1-grammar.md` — what M1.1/M1.2a/M1.5 already did, including the exact
  grammar-conflict outcome for receivers (there wasn't one) so this round doesn't re-litigate
  it.
- `CLAUDE.md` — test-first rule and the end-of-round ritual.

## State of the tree

- `grammars/tree-sitter-oberon2/grammar.js`: M1.1 base + M1.2a (`receiver`, `forward_decl`,
  `definition_header`/`definition_proc_decl`, the `module` rule is now a `choice` of the
  `MODULE` and `DEFINITION` forms). 20/20 `tree-sitter test` green.
- Known, not-yet-fixed gap found *during* M1.2a but out of scope for it: `field_list_seq` (a
  `RECORD`'s fields) rejects a trailing `";"` before `END`, same shape as the statement-seq gap
  this task (M1.2b) is about to fix for statements — worth fixing both in the same style if
  picked up, though `FieldList` itself isn't in this milestone's scope per `docs/plan.md`;
  flag it rather than silently expanding scope.
- `queries/highlights.scm` (M1.5) done, not expected to need changes for M1.2b's new statement
  node kinds unless they introduce new keyword tokens not yet captured — check after.
- Rust workspace untouched since M0 — this task doesn't touch it.

## After M1.2b

M1.2c (expressions/types — `IS`, `SET` literals with ranges, open arrays, procedure types), then
M1.3 (external C scanner for nested comments — highest-risk item in M1, don't leave it last if
time is short).
