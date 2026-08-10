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

## M1.4 — corpus sweep + `INLINE` triage ✅ (round 7, 2026-08-10)

Built `grammars/tree-sitter-oberon2/sweep_corpus.py` (throwaway per `NEXT.md`, committed
anyway): reads `corpus/manifest.json`, runs `tree-sitter parse --quiet` per file (transcoding
`encoding: "high-bytes"` entries from Latin-1 to UTF-8 into a temp file first — tree-sitter's
CLI always reads UTF-8, and ~42% of the corpus, 330/792 files, is flagged high-bytes), reports
the `ERROR`-free percentage and the failure list. First real number, not a 3-5-file spot-check:
baseline **15.78%** (125/792) — far below the M1.2b/M1.2c/M1.3 spot-checks' implied health,
because those rounds' "still has one `ERROR` region" framing measured depth on a handful of
files, not breadth across the corpus.

**`INLINE`'s premise was wrong — confirmed before coding, per the task's own instruction.**
Grepping the corpus for real usage (`SYSTEM.INLINE`, not just the string `INLINE`) found it only
in STJ-Oberon (`DISASS.MOD`, `RSRC.MOD`, `VDICONTR.MOD`, `OCSYMBOL.MOD`, `RSRC.DEF` — Oberon-A
and AmigaOberon have none) and it is not block syntax at all: `S.INLINE(02F0EH,0206EH,...)` is an
ordinary call to a pseudo-procedure `SYSTEM.INLINE`, taking machine-code words as normal
`actual_params`. `docs/language-baseline.md`'s dialect table calling it "opaque token, contents
unparsed" was an assumption made without checking the corpus (the doc itself admits this — "not
in normative EBNF"). No grammar rule needed for `INLINE` itself.

What actually blocked it, and three other real bugs, found via the sweep's failure list and
fixed (all one-line, all root-cause, each got a new corpus test before the fix per `tree-sitter
test`'s red→green):

- **`kElseif` matched the literal string `'ELSEIF'`, not `'ELSIF'`** (`grammar.js`, keyword
  table). `docs/language-baseline.md`'s reserved-word list (line 125) and EBNF (line 59) both
  say `ELSIF`. This is Oberon-2's second-most-common control construct after plain `IF`/`ELSE`;
  every `IF...ELSIF...` in the corpus was silently falling to `ERROR`. New test: `statements.txt`
  "If Elsif". Impact: 15.78% → 17.93% (125 → 142 passing).
- **Hex-integer literal token only matched one hex digit** — `integer` had
  `token(seq(hex_digit, 'H'))` where the EBNF comment two lines above it (`digit {hex_digit}
  "H"`) already specified the correct shape. `"3H"` parsed; `"02F0EH"` (any real address/mask/
  opcode constant) didn't — column pointed straight at the second hex digit every time. This is
  what was actually breaking `SYSTEM.INLINE(02F0EH,...)`: not missing block syntax, an unrelated
  lexer bug in a token INLINE happens to use heavily. Fixed to `token(seq(digit,
  repeat(hex_digit), 'H'))`. New test: `statements.txt` "Multi Digit Hex Literal" (the real
  `S.INLINE(...)` shape, doubling as `INLINE`'s only needed corpus coverage). Impact: 17.93% →
  21.21% (142 → 168 passing).
- **`import` had no path for two AmigaOberon-only rename/re-export variants** — confirmed via
  corpus grep, undocumented anywhere in `docs/language-baseline.md`: `IMPORT e * := Exec` (a
  `*` re-export marker after the local alias, always paired with `:=`, never seen with plain
  `:`) and `IMPORT e: Exec` (plain `:` as an alternate rename operator, seen in different
  AmigaOberon files than the `*` marker — looks like two compiler-version dialects, not one).
  Both confirmed absent from Oberon-A and STJ, which only ever use plain `:=`. Widened `import`
  to `ident, optional('*'), optional(seq(choice(':=', ':'), ident))`. New tests: `module.txt`
  "module import with re-export marker", "module import with colon rename". Impact: 21.21% →
  21.97% (168 → 174 passing).

`tree-sitter test`: 39/39 green (35 before this round + "If Elsif" + "Multi Digit Hex Literal" +
2 import-variant cases).

**Triaged, not fixed — each needs its own scoping decision, not a silent fold-in (same boundary
`NEXT.md` set for the bracket-pragma item specifically, extended here to everything else this
round's investigation turned up):**

| Pattern | Corpus files (substring grep) | Notes |
|---|---|---|
| `<* ... *>` bracket pragmas | 212 (27% of corpus) | Different delimiter from the `(*$…*)` pragma M1.3 implemented; not in `docs/language-baseline.md` at all. Largest single remaining cluster — bigger than `INLINE` ever was. |
| `STRUCT` record variant | 43 | C-interop struct-like type (AmigaOberon), e.g. `Point2D = STRUCT x,y: INTEGER; ... END`. Not `RECORD`, not in the baseline EBNF. |
| `PROCEDURE ... *{base,-N}(...)` / `param{N}` brace annotations | 42 | Library-vector-offset metadata attached to AmigaOberon procedure/parameter declarations. Found while investigating a file whose failure looked like an encoding issue (see below) — it wasn't. |
| `ASSEMBLER` blocks | 32 (STJ only) | A `PROCEDURE ... ASSEMBLER ... END`-shaped raw-assembly section, same conceptual family as `INLINE` but this one really does look like block syntax (unconfirmed — not minimized this round). |
| `POINTER TO ARRAY OF Type` | not re-measured this round | Carried over from M1.3, still open. |
| Single-quoted strings | not re-measured this round | Carried over from M1.2c, still unconfirmed as an actual failure cause. |

**Dead end, worth recording so it isn't re-tried:** files flagged `encoding: "high-bytes"` in the
manifest (Latin-1 banner comments, e.g. `©`) were suspected as a parse-collapse cause (`ERROR`
spanning `[0,0]` to end-of-file) before transcoding was added to the sweep script. Transcoding
changed **zero** files from fail to pass — every `[0,0]`-span failure has a real syntax cause
early in the file (e.g. the brace-annotation pattern above), the Latin-1 byte was just
incidentally nearby in a banner comment. Kept the transcoding fix anyway (it's still the
correct, honest way to feed these files to a UTF-8-only parser, and 330/792 files carry the
flag), but the theory that motivated it doesn't hold — see `docs/insights.md`.

## After M1.4

M1 is not at its ≥95% exit criterion (21.97%, corpus-wide, first honest measurement). The
remaining gap is dominated by the bracket-pragma cluster (212 files) plus several newly-found
AmigaOberon-specific extensions (`STRUCT`, brace annotations, `ASSEMBLER`) that together are a
materially bigger scope than "add `INLINE`" ever was, and — per D8's 5%-of-corpus allowlist cap
(≈40 files) — cannot be closed by allowlisting alone; most of this has to become grammar. Each
needs a scoping decision (grammar addition vs. allowlist vs. explicitly out-of-D1-scope) before
the next round picks one to implement — see `NEXT.md`.

## M1.4 continued — bracket pragmas + brace annotations ✅ (round 9, 2026-08-10)

Per `NEXT.md`'s instruction, flagged the scoping decision to the user before implementing
anything (four triaged patterns, very different amounts of work). Decided: `STRUCT` deferred to
Phase 2 (genuine second record-like type, bigger than D1's "lexical superset" scope). Bracket
pragmas and brace annotations picked for this round (both lexical-superset-tier, cheap to
moderate). `ASSEMBLER` deliberately not picked — still triaged only, see below.

**Confirmed real syntax before writing any grammar rule** (same discipline as M1.4's `INLINE`
lesson), by grepping the actual corpus roots per `corpus/roots.toml`:

- **`ASSEMBLER`** (STJ, 32 files) — genuine block syntax, `ASSEMBLER <raw M68K opcodes> END` used
  as a statement inside a procedure body. Content is not Oberon tokens at all (opcodes, register
  names, `(A0,D0.L)`-style addressing, `#imm`) — needs external-scanner treatment (raw-scan to a
  matching `END`), same technique class as the nested-comment scanner, not a plain grammar rule.
  Left triaged, not implemented, this round — see `NEXT.md`.
- **Brace annotations** (AmigaOberon, 42 files) — confirmed simple: `PROCEDURE Name
  *{base,-54}(param{2}: T): T`. A `{ident, "-" integer}` group after the procedure's `ident_def`
  (base name varies: `base`, `cwBase`, ...; offset always negative, decimal or hex), and a bare
  `{integer}` after each formal parameter's `ident`. No nesting, no scanner work.
- **Bracket pragmas `<* ... *>`** (Oberon-A only, 212 files) — confirmed two sub-forms sharing
  one delimiter: bare flags (`<* STANDARD- *>`) and `$`-prefixed sub-pragmas (`<*$LongVars-*>`
  — the Oberon-A compiler's own error message, `"Pragma must start with '<*$'"`, found via grep
  of `ErrorMessages.mod`, confirms `$` as the "real" pragma marker and bare flags as a shorthand
  form). Also holds conditional-compilation directives (`<*IF OberonA THEN*>`, `<*ELSE*>`,
  `<*END*>`) — per D1 these are swallowed opaquely, same as `(*$...*)`, not given real
  conditional-compilation semantics. Confirmed non-nesting across every corpus occurrence
  (`grep -rzoE '<\*[^>]*<\*'` found none).

**Grammar changes**, both test-first (`tree-sitter test` red before, green after):

- `src/scanner.c`: third external token `BRACKET_PRAGMA`, alongside the existing `COMMENT`/
  `PRAGMA`. Same shape as the `(* ... *)` handler but for `<* ... *>` and without depth tracking
  (confirmed non-nesting, so a flat scan to the first `*>` is correct — no need to carry the
  `(*...*)` handler's nesting-depth counter into this one).
- `grammar.js`: `$.bracket_pragma` added to `externals`/`extras` (enum order in `scanner.c` must
  match the `externals` array order — `[$.comment, $.pragma, $.bracket_pragma]` mirrors
  `{COMMENT, PRAGMA, BRACKET_PRAGMA}`). New rules `vector_offset` (spliced into
  `procedure_heading` after `ident_def`, before `formal_params`) and `param_offset` (spliced
  into `fp_section` after each parameter `ident`). `forward_decl` left unchanged — corpus grep
  found no brace-annotated forward declarations.

New corpus tests: `comments.txt` "bracket pragma", "bracket pragma dollar sub-pragma";
`procedures.txt` "Procedure With Vector Offset" (covers both `vector_offset` and `param_offset`
in one case, the real `BinKoeff*{base,-54}(n{2},k{3}: LONGINT)` shape). `tree-sitter test`:
42/42 green (39 before this round + 3 new).

**Impact:** `sweep_corpus.py` 21.97% → 27.15% (174 → 215 of 792 passing), the single largest
jump since the M1.4 hex-literal fix.

**Triage table, updated:**

| Pattern | Corpus files | Status |
|---|---|---|
| `<* ... *>` bracket pragmas | 212 | ✅ done this round |
| Brace annotations (`*{base,-N}` / `param{N}`) | 42 | ✅ done this round |
| `STRUCT` record variant | 43 | Scoped out of M1 — Phase 2 (`corpus/allowlist.toml` or a later milestone) |
| `ASSEMBLER` blocks | 32 (STJ only) | Still triaged only — needs external-scanner work, not picked this round |
| `POINTER TO ARRAY OF Type` | not re-measured | Carried over from M1.3, still open |
| Single-quoted strings | not re-measured | Carried over from M1.2c, still unconfirmed as an actual failure cause |

M1 is still below its ≥95% exit criterion (27.15%). `ASSEMBLER` is now the largest unimplemented
cluster with confirmed real syntax; `STRUCT` is explicitly out of scope. Next round: implement
`ASSEMBLER` (needs scanner work) or re-measure the two carried-over items, per `NEXT.md`.

## M1.4 continued — `ASSEMBLER` blocks ✅ (round 10, 2026-08-10)

Confirmed the round-9 characterization against real files before coding: `HALT.MOD` and
`QSORT.MOD` (STJ) both show `ASSEMBLER <raw M68K> END` used as a statement mid-`BEGIN...END`,
never as a whole-body replacement. Checked the specific open question round 9 left ("could `END`
appear inside operand text?") by reading every `ASSEMBLER` block in the corpus (32 occurrences,
`grep -rn ASSEMBLER` under the `stj` root): the terminator is always a clean word-boundary `END`
(sometimes followed by `(*ASSEMBLER*)` or `;`), and no M68K mnemonic or operand in the corpus
contains `END` as a substring (checked `MATHLIB0.MOD`'s six blocks specifically, the file with
the most occurrences). So no nesting, and a plain word-boundary scan is correct — no need for the
comment scanner's depth-counting.

**Grammar changes**, test-first:

- `src/scanner.c`: fourth external token, `ASSEMBLER_BODY`. Same raw-scan-to-a-delimiter
  technique as `COMMENT`/`PRAGMA`/`BRACKET_PRAGMA`, but scanning for a word-boundary `"END"`
  instead of a fixed close-bracket string, via a small `is_ident_char` check on the byte before
  and after each candidate `E`. Checked first, before the `'<'`/`'('` branches, since when
  `valid_symbols[ASSEMBLER_BODY]` is true the parser is specifically expecting asm content right
  after the `ASSEMBLER` keyword — content that may itself contain `(` (addressing syntax like
  `(A0,D0.L)`) which must not be mistaken for a comment opener.
- `grammar.js`: `$.assembler_body` added to `externals` only (not `extras` — unlike
  comment/pragma/bracket-pragma, this token is never insertable anywhere, only in the one slot
  between `kAssembler` and `kEnd`). New `kAssembler` keyword and `assembler_statement = "ASSEMBLER"
  assembler_body "END"`, added to the `statement` choice. Reuses the existing `kEnd` for the
  terminator — no new closing-keyword rule needed, matching how `loop_statement`/`with_statement`
  already terminate.

New corpus test: `statements.txt` "Assembler Statement", the exact `HALT.MOD` shape
(`MOVEM.L D0-A7,registers`) as one statement of a two-statement body, confirming the parser
resumes normal statement parsing right after the block. `tree-sitter test`: 43/43 green (42
before this round + 1 new).

**Impact:** `sweep_corpus.py` 27.15% → 29.29% (215 → 232 of 792 passing).

**Fast pass on the two carried-over items** (per `NEXT.md`'s suggestion, cheap relative to
`ASSEMBLER`'s scanner work) — both confirmed as real, isolated `ERROR`-causing gaps, not just
"plausible":

- **`POINTER TO ARRAY OF Type`** — isolating `TYPE P = POINTER TO ARRAY OF INTEGER;` alone
  produces a real `ERROR` node (`array_type` requires a `length` between `ARRAY` and `OF`; only
  `formal_type`, M1.2c, has the length-less shorthand, and only for formal parameters). 36 corpus
  files use this shape.
- **Single-quoted character literals** — isolating `x := ORD('4');` alone produces a real `ERROR`
  node. The report's `string` production is `'"' {char} '"' | digit {hexdigit} 'X'`; there's no
  single-quote form at all, so this is a genuine dialect extension (Pascal/Modula-style char
  literals), not a variant spelling of something already supported. 127 corpus files use this
  shape (grep for `'x'`-style substrings; noisy pattern, but the two files checked by hand
  (`Tetriz.mod`, via `ORD('4')`/`ORD(' ')`/`ORD('q')`) are genuine char-literal usage, not
  apostrophes in comments).

Neither implemented this round — both are now confirmed real, still need scoping/implementation,
see `NEXT.md`.

**Triage table, updated:**

| Pattern | Corpus files | Status |
|---|---|---|
| `<* ... *>` bracket pragmas | 212 | ✅ done (round 9) |
| Brace annotations (`*{base,-N}` / `param{N}`) | 42 | ✅ done (round 9) |
| `ASSEMBLER` blocks | 32 (STJ only) | ✅ done this round |
| `STRUCT` record variant | 43 | Scoped out of M1 — Phase 2 (`corpus/allowlist.toml` or a later milestone) |
| `POINTER TO ARRAY OF Type` | 36 | Confirmed real this round, not yet implemented |
| Single-quoted character literals | 127 (noisy grep) | Confirmed real this round, not yet implemented |

M1 is still below its ≥95% exit criterion (29.29%). No more "still open, never re-measured"
items left — everything in the table is now either done, explicitly out of scope, or confirmed
real and sized. Next round: implement `POINTER TO ARRAY OF Type` and/or single-quoted character
literals, per `NEXT.md`.
