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

## M1.2c — expressions/types: IS, SET, open arrays, procedure types ✅ (round 5, 2026-08-10)

Confirmed-before-coding discipline (same as M1.2a/b) paid off again: three of the four items in
`NEXT.md` needed zero grammar changes, only a corpus test each —

- **`SET`** — not a grammar keyword at all (no `kSet` token exists), so it already lexes as a
  plain `ident` and is covered by `type = qualident | struct_type`. `ARRAY 8 OF SET` parses
  clean as-is.
- **`IS` in expressions** — `relation` already included `$.kIs`; `x IS INTEGER` inside an `IF`
  parses clean as-is.
- **Open arrays in formal params** — `formal_type`'s `repeat(seq($.kArray, $.kOf))` prefix
  already covers `ARRAY OF CHAR`-shaped parameters; parses clean as-is.

One item needed an actual fix: **procedure types as formal parameters**. `procedure_type` was
already wired into `struct_type` (so `PROCEDURE(...)` works fine as a `RECORD` field or a
standalone `TYPE` declaration), but `formal_type` — the separate, narrower rule used specifically
inside `fp_section` — only ever allowed `{"ARRAY" "OF"} qualident`, with no path to
`struct_type` at all. Real usage confirmed in `voc/src/library/misc/MultiArrays.Mod` (ten
`f: PROCEDURE(s1,s2:SHORTINT): SHORTINT`-shaped parameters). Fixed by widening `formal_type` to
`{"ARRAY" "OF"} (qualident | procedure_type)` — the minimal change for what the corpus actually
uses, not the EBNF's full `Type` recursion into `formal_type` (which would also add `RECORD`/
`POINTER TO` as anonymous parameter types, not observed anywhere in the corpus).

New corpus tests: `types.txt` (5 cases covering all four items, generated via `tree-sitter test
--update` per the established workflow, read back to confirm zero `ERROR`/`MISSING` nodes).

**Incidental pickup, flagged as fair game in `NEXT.md`'s "State of the tree" section:**
`field_list_seq` (a `RECORD`'s fields) rejected a trailing `";"` before `END` — the same shape as
the `statement_seq` gap fixed in M1.2b (`FieldList` is optional in the EBNF, same as
`StatementSeq`). This turned out to be the *actual* cause of the one residual `ERROR` in
`voc/v4/Printer.Mod` that M1.2b's notes attributed to `SET` ("`ARRAY 8 OF SET` (M1.2c scope, SET
isn't a type yet)") — that diagnosis was wrong: isolating the field (`used: ARRAY 8 OF SET;`)
showed `SET` parses fine on its own, and the error only appears when it's a `RECORD` field
followed by a trailing `;` before `END`, which is exactly the field-list gap logged back in
M1.2a's round-3 insights ("a second gap in the same shape as the already-known one"). Fixed with
the identical two-branch `choice` pattern already proven for `statement_seq`. New corpus test:
`records.txt` → "Record Trailing Semicolon".

`tree-sitter test`: 33/33 green (27 before this round + 5 `types.txt` + 1 `records.txt`).

Re-ran the M1.2b spot-check files:
- `Printer.Mod` — the line-15 `ERROR` (the misattributed "SET" one) is gone; five unrelated,
  pre-existing `ERROR` regions remain (a `<*STANDARD-*>`-style pragma, a nested comment, and a
  few likely single-quoted-string / other lexical gaps — none touched by this round, none newly
  introduced: both grammar changes this round are pure widenings via `choice`/`repeat1`, so they
  cannot make previously-accepted input newly fail).
- `MultiArrays.Mod` — still fails extensively on nested comments (M1.3 scope, expected per
  `NEXT.md`); now additionally exercises the `formal_type` fix (10 `PROCEDURE(...)` parameters)
  without contributing new errors of its own.
- `Viewers.Mod` — unchanged from M1.2b (doesn't use any construct touched this round).

## M1.3 — external C scanner for nested comments + pragmas ✅ (round 6, 2026-08-10)

Added `src/scanner.c` (`tree_sitter_oberon2_external_scanner_{create,destroy,serialize,
deserialize,scan}`, ~60 lines): depth-counts `(*`/`*)` pairs, reporting `COMMENT` unless the
character right after the opening `(*` is `$`, in which case it reports `PRAGMA` — per
`docs/plan.md` D1 / M1.3's row, "a distinct node kind, lexically a comment", confirmed correct
scope (the plan table explicitly assigns the pragma node to M1.3, resolving `NEXT.md`'s "check
whether this is in scope" flag). `grammar.js`: `externals: $ => [$.comment, $.pragma]`,
`extras: $ => [$.comment, $.pragma, /\s/]`, old regex `comment: $ => token(seq(...))` rule
removed (a `token()` rule can't express nesting at all). `queries/highlights.scm` gained
`(pragma) @comment` — the plan's "lexically a comment" is exactly a highlighting statement too.

**`.gitignore` fix, not optional:** `grammars/*/src/` blanket-ignored the whole `src/` directory,
which was fine while everything in it was generated (M1.1). `scanner.c` is hand-written source,
not generated, and would have silently vanished on a fresh checkout. Changed to
`grammars/*/src/*` + `!grammars/*/src/scanner.c` so the generated files (`parser.c`,
`grammar.json`, `node-types.json`, `tree_sitter/`) stay ignored but the scanner is tracked.

**Real bug, not a source problem — see `docs/errors.md` round 6:** the first working version of
the scanner failed on *every* comment except one at byte 0 of the file (confirmed by depth-count
simulation in Python against the actual corpus files: comments were always balanced; the parser
still errored). Root cause: tree-sitter calls the external scanner exactly once per token
boundary, before it tries to skip whitespace itself via the internal DFA — so if the scanner
declines because `lookahead` is a space/newline rather than `(`, it never gets a second look
after that whitespace is skipped. Fixed by having the scanner skip its own leading whitespace
(`lexer->advance(lexer, true)`, `skip=true`) before checking for `(`.

New corpus tests in `test/corpus/comments.txt`: "nested comment" (`(* outer (* inner *) still
outer *)`) and "pragma" (`(*$-k *)`, the exact form found in the STJ-Oberon corpus — the only
corpus root that actually uses `(*$…*)`; Oberon-A/AmigaOberon have none. `Printer.Mod`'s
`<*STANDARD-*>`-style bracket pragma is a *different* delimiter, not covered by, or in scope
for, this scanner). `tree-sitter test`: 35/35 green (33 before this round + 2 new).

Spot-checked `ERROR`-region counts (`grep -c "(ERROR"` on `tree-sitter parse` output, excluding
the CLI's own summary line which also contains the string `(ERROR`) against the pre-M1.3
baseline via `git stash`:

| File | Before | After |
|---|---|---|
| `Viewers.Mod` | 5 | 4 |
| `Printer.Mod` | 5 | 5 (unchanged — see below) |
| `MultiArrays.Mod` | 46 | 28 |

`MultiArrays.Mod` drops by 18 regions — nested comments are exactly the dominant cause `NEXT.md`
described (25 corpus files' worth), and the depth-counting scanner now swallows all of them
(including `(** ... **)`-style ones, which are just ordinary Oberon comments with an extra
leading/trailing `*` as content — no special-casing needed, the "does the char after `*)` — or
before it — close the pair" check handles it for free). It doesn't reach zero because of an
unrelated, newly-discovered gap: `POINTER TO ARRAY OF Type` (open array as a pointer's base
type, no explicit `length`) isn't reachable through `array_type` at all — confirmed by isolating
`TYPE P = POINTER TO ARRAY OF INTEGER;` alone, still an `ERROR`. `array_type` requires a
`length` between `ARRAY` and `OF`; only `formal_type` (M1.2c) has the length-less `ARRAY OF`
shorthand, and only for formal parameters. This is a real, separate grammar gap, not touched
here — flagged in `NEXT.md` for a future round rather than picked up incidentally, since it's
unrelated to comments and this round is already at its diff budget.

`Printer.Mod` staying at 5 is a correction, not a regression: M1.2c's notes attributed one of its
five errors to "a nested comment", but `grep -n '(\*.*(\*\|(\*\*'` finds no nested-comment
pattern anywhere in that file — the attribution was never checked, just guessed. The five
remaining errors are the `<*STANDARD-*>`-style bracket pragma (different delimiter, not this
scanner's scope) and other pre-existing, unrelated lexical gaps.

No `binding.gyp` changes needed (item 4 in `NEXT.md`'s task list) — `tree-sitter generate` /
`tree-sitter test` compile `scanner.c` directly via the CLI's own `cc` invocation; this project
still doesn't build a node addon, so there's nothing for `node-gyp` to pick up.

## After M1.3

M1 is feature-complete against `docs/plan.md`'s D1 scope. Remaining before M1 is declared done:
a full-corpus parse sweep (not just the three spot-check files used through M1.1–M1.3) to get a
real `ERROR`-free percentage — see `NEXT.md`.
