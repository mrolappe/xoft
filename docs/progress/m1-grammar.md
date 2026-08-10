# M1 — Grammar

## M1.1 — vendor base grammar ✅ (round 2, 2026-08-10)

`grammars/tree-sitter-oberon2/` now holds `viegasfh/tree-sitter-oberon-2` (MIT, commit
`bb5282d6`) minus the checked-in parser: `grammar.js`, `package.json`, `LICENSE`,
`test/corpus/*` (5 files, upstream). `NOTICE` records both fork origins and commits.

`tree-sitter generate` runs clean under CLI 0.26.11 — no rule changes were needed, only two
warnings (ABI 14 fallback because there is no `tree-sitter.json`; one unnecessary `seq` in the
`comment` rule). `tree-sitter test` is 14/14 green on the upstream corpus.

`src/` (parser.c, grammar.json, node-types.json, tree_sitter/*) is generated, not committed —
gitignored via `grammars/*/src/` in the root `.gitignore`, regenerate with `tree-sitter
generate` before `tree-sitter test` or any parse.

### M1.5 — highlights.scm ✅ (done in the same pass, cheap)

`queries/highlights.scm` is adapted from `geekstakulus/tree-sitter-oberon-07` (MIT, commit
`162c3432`), rewritten against this grammar's actual node names — the two forks share a
skeleton but diverge on field names (this grammar has no `param:`/`paramtype:`/`returntype:`
fields, and no `base_type` wrapper around builtin qualidents outside record extension).
Validated with `tree-sitter query queries/highlights.scm <file>` against the corpus and
`examples/Hello.Mod` from upstream — captures are sane on everything the grammar actually
parses.

Sanity-checked `Hello.Mod` end to end and confirmed by hand a gap already logged in
`docs/insights.md`: `Out.Ln;` followed directly by `END` produces an `ERROR` node, because
`statement_seq` has no empty-statement alternative. This is the documented M1.2b gap, not a new
bug — left alone per the task boundary ("do not start adding missing constructs").

## M1.2a — declarations: receivers, forward decl, DEFINITION header ✅ (round 3, 2026-08-10)

Three additions to `grammar.js`, all in `grammars/tree-sitter-oberon2/`:

- **`receiver`** — `"(" ["VAR"] ident ":" ident ")"`, spliced into `procedure_heading` between
  `kProcedure` and `ident_def`. No grammar conflict materialized: `ident_def` is mandatory and
  cannot start with `"("`, so seeing `"("` right after `kProcedure` already commits the parser
  to `receiver` before `formal_params` (which comes later, after `ident_def`) is even in play.
  The conflict flagged in `NEXT.md` didn't happen — `tree-sitter generate` reported zero
  conflicts on the first attempt.
- **`forward_decl`** — `PROCEDURE "^" [receiver] ident_def [formal_params]`, a new rule, folded
  into `procedure_decls` as `choice($.procedure_decl, $.forward_decl)`. Applies at both module
  level and inside `procedure_body` (both reuse `procedure_decls`), matching the EBNF's
  `DeclSeq` which allows `ForwardDecl` anywhere `ProcDecl` can appear.
- **`DEFINITION` module** — NOT just a second keyword for `module_header`. Grepping the STJ
  corpus (`DEF/*.DEF`, 70/112 files use `DEFINITION`, the rest use `MODULE` for the same
  interface-file role) showed procedure declarations in a `DEFINITION` module have no body at
  all — `PROCEDURE Open;` directly followed by the next declaration, no `END ident`. Modeled as
  a second top-level alternative in `module` (`choice(seq(module_header, ...), seq(
  definition_header, ...))`), each branch unambiguously selected by its first token
  (`kModule` vs `kDefinition`), with a new `definition_proc_decl = procedure_heading ";"` used
  in place of `procedure_decls` in that branch. Kept `module`'s node shape unchanged for the
  `kModule` branch — no wrapper node, existing tests didn't need touching.

New corpus tests: `receivers.txt`, `forward_decl.txt`, `definition_module.txt` (2 cases each).
`tree-sitter test`: 20/20 green (was 14 upstream + 2 from M1.1's own additions... actually 16
before this round, see git history).

Spot-checked against real corpus fragments (full real files often also hit an *unrelated*
pre-existing gap — trailing `";"` before `END` in record `FieldList`, same class of issue as
the already-logged empty-statement gap, just for `FieldList` instead of `StatementSeq`; left
alone, out of scope for this task):
- `DEF/MATHCOM.DEF` (real `DEFINITION` module, `CONST` + multiple headless `PROCEDURE`s) parses
  with **zero** `ERROR` nodes as-is, no minimization needed.
- A receiver method lifted verbatim from `DEF/PROCLIST.DEF` (`PROCEDURE (VAR self: Desc)
  AddProc*(proc: Proc);`) parses clean once isolated from that file's unrelated record-trailing-
  semicolon errors.
- A forward decl lifted verbatim from `LTL.PRJ/CHATIO.MOD` (`PROCEDURE^ KeyPressed*() :
  BOOLEAN;`) parses clean once isolated from that file's M1.2b-scope statement gaps.

## M1.2b — statements: WITH, LOOP, EXIT, RETURN, empty statements ✅ (round 4, 2026-08-10)

Five changes to `grammar.js`, all in `grammars/tree-sitter-oberon2/`:

- **Empty statements** — `statement_seq` was `$.statement, repeat(seq(';', $.statement))`
  (every element mandatory). Widening each element to `optional($.statement)` alone makes the
  whole rule able to match the empty string, which `tree-sitter generate` rejects outright
  ("rule matches the empty string"). Fixed with a two-branch `choice`: `seq($.statement,
  repeat(seq(';', optional($.statement))))` for sequences that start with a real statement
  (covers the trailing-`;`-before-`END` case), or `repeat1(seq(';', optional($.statement)))`
  for sequences that are only semicolons (`BEGIN ; END`). Either branch always consumes at
  least one token, so the empty case is expressed the correct way — by omitting the whole
  `statement_seq` at the call site (`optional($.statement_seq)`, already how every caller
  invokes it), not by the rule matching nothing itself.
- **`LOOP StatementSeq END`**, **`EXIT`** — new `loop_statement` and `exit_statement` rules,
  added to the `statement` choice. Both confirmed against real usage: `EXIT` nested inside an
  inner `LOOP`'s `IF` (Oberon-A `Misc/HexConvert.mod`), which only works once `EXIT` is a
  statement in its own right rather than needing special-casing.
- **`RETURN [Expr]`** — added as a general `return_statement` in the `statement` choice, and
  **removed** the hardcoded `optional(seq($.kReturn, $.expression))` that `procedure_body` had
  tacked on after its `statement_seq`. The task brief flagged this as a genuine choice point
  ("keep both, or replace"); grepping the corpus first settled it — `Oberon-A/source/ol/
  OLPrefsStrings.mod:157-160` has `RETURN` as the last statement of *both* branches of an `IF`,
  i.e. mid-body, not at the textual end of the procedure. The report's "`RETURN` only once, at
  the end" restriction isn't in the EBNF's `Statement` production at all (confirmed against
  `docs/language-baseline.md`'s `ProcDecl`, which is just `DeclSeq [BEGIN StatementSeq] END
  ident` — no separate `RETURN` slot), so the old hardcoded field was modeling a restriction
  the grammar was never asked to enforce. One general statement replaces it cleanly.
- **`WITH Guard DO StatementSeq {"|" Guard DO StatementSeq} [ELSE StatementSeq] END`** — new
  `with_statement` (repeated `with_arm = guard kDo optional(statement_seq)`, joined by `"|"`)
  plus `guard = qualident ":" qualident`. Confirmed against real multi-arm usage in `voc`'s
  `MultiArrays.Mod` (`WITH A: SIntArray DO ... ELSE HALT(100) END`, nested two deep in
  `AllSInt2`) and Oberon-A's `Viewers.Mod`.
- **`CASE` label ranges** — already implemented before this round (`label_range = label [".."
  label]`, in place since M1.1). Confirmed still correct against real usage (`voc`'s
  `v4/Printer.Mod`, `| 65..90: Ch(fontR, CHR(m))`) rather than re-derived; no grammar change
  needed, just a corpus test added.

New corpus tests: `statements.txt` (7 cases, generated via `tree-sitter test --update` from
real-shaped snippets, then read back to confirm no `ERROR` nodes and sane tree shapes before
trusting the auto-fill). `tree-sitter test`: 27/27 green (20 before this round + 7 new).

Spot-checked against five real corpus files end to end (`HexConvert.mod`, `Viewers.Mod`,
`MultiArrays.Mod`, `OLPrefsStrings.mod`, `Printer.Mod`) — each still has exactly one `ERROR`
region, same as before this round's changes, and each is confirmed unrelated to statements:
a `<*STANDARD-*>` compiler pragma, a nested `(** ... **)` comment (M1.3 scope), `ARRAY 8 OF
SET` (M1.2c scope, `SET` isn't a type yet), and a type-guard chain inside a larger expression
that predates this round. No regressions, no new errors introduced by the statement work.

## M1.2c / M1.3 — not started

See `NEXT.md` for the current task and `docs/insights.md` for the full list of gaps against the
EBNF baseline these subtasks close.
