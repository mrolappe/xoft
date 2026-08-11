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

## M1.4 continued — `POINTER TO ARRAY OF Type` + single-quoted strings ✅ (round 11, 2026-08-10)

Both items from the triage table implemented. Re-checked round 10's corpus counts and one of its
claims before coding, per `NEXT.md`'s instruction not to trust the noisy grep blindly:

- **`POINTER TO ARRAY OF Type`**: re-grepped, 39 files (not 36 — count drift, not a shape
  change). Sampled 15 occurrences across `oberon-a` and `amiga-oberon-31`: always the same plain
  shape, `POINTER TO ARRAY OF <qualident-or-basic-type>`, no length, no multi-dimension case. Confirms
  round 10's read.
- **Single-quoted strings**: round 10's claim that "the report's `string` production has no
  single-quote form" is **wrong** — `docs/language-baseline.md` line 140 has always read
  `string = '"' {char} '"' | "'" {char} "'".` (present since the file's first commit, checked via
  `git log -p`). Round 10 apparently never checked the baseline doc against this specific claim.
  Corpus evidence agrees with the baseline, not with round 10: stripping `(* ... *)` comments and
  `"..."` strings first (to kill false positives from English contractions like "don't"/"it's" in
  comment prose, which dominated a naive `'.'`-pairing grep), the real single-quoted literals in
  code include multi-character strings — AmigaOberon FourCC tags (`'KICK'`, `'PREF'`, `'FONT'`,
  ...) and format strings (`'%%%dld'`), not just single characters. So this is the *same* string
  literal, alternate delimiter — not a separate `CHAR` literal type. Widened `string_literal`
  in place rather than adding a new node.

**Grammar changes**, test-first:

- `grammar.js` `array_type`: `length` list wrapped in `optional(...)`, matching `formal_type`'s
  existing length-less shorthand. No new external token — plain `choice`/`optional` widening.
- `grammar.js` `string_literal`: added `/'[^'\n]*'/` as a third choice alternative, symmetric with
  the existing `/"[^"\n]*"/` (any run of non-newline, non-delimiter chars). Both quote styles
  still forbid embedding their own delimiter (matches the baseline's "opening quote must match
  closing quote" note — no escaping, use the other quote char to embed one).

New corpus tests: `types.txt` "Pointer To Length-Less Array Type" (`POINTER TO ARRAY OF CHAR`,
from `ODT.mod`'s `Symbol` type) and `declarations.txt` "Single-Quoted String Constant"
(`idKick* = 'KICK';`, from `BootBlock.mod`). Both filled via `tree-sitter test --update`, read
back to confirm no `ERROR`/`MISSING` nodes. `tree-sitter test`: 45/45 green (43 before this
round + 2 new).

**Impact:** `sweep_corpus.py` 29.29% → 30.68% (232 → 243 of 792 passing), +11 files.

**Triage table, updated:**

| Pattern | Corpus files | Status |
|---|---|---|
| `<* ... *>` bracket pragmas | 212 | ✅ done (round 9) |
| Brace annotations (`*{base,-N}` / `param{N}`) | 42 | ✅ done (round 9) |
| `ASSEMBLER` blocks | 32 (STJ only) | ✅ done (round 10) |
| `POINTER TO ARRAY OF Type` | 39 | ✅ done this round |
| Single-quoted strings | not separately counted (subsumed into `string_literal`) | ✅ done this round |
| `STRUCT` record variant | 43 | Scoped out of M1 — Phase 2 (`corpus/allowlist.toml` or a later milestone) |

M1 is still below its ≥95% exit criterion (30.68%). The triage table from round 9/10 is now
fully resolved — everything is done or explicitly out of scope. Next round needs a fresh sweep of
`sweep_corpus.py`'s remaining 549 failures to find the next cluster (no candidates queued in
`NEXT.md` beyond this point).

## M1.4 continued — AmigaOberon bodiless procedure heading ✅ (round 12, 2026-08-10)

Confirmed `NEXT.md`'s shape claim against `Interfaces/Cia.mod`: a `PROCEDURE ... ;` heading with
no `BEGIN...END Name` body at all, structurally identical to `definition_proc_decl` but appearing
inside a plain `MODULE`. Reused `definition_proc_decl` as a third alternative rather than adding a
new node, per `NEXT.md`'s suggestion — checked and found no semantic reason to keep the two node
types visually distinct (both are exactly `procedure_heading ';'`, differing only in which
enclosing construct permits them).

**Grammar change:** `procedure_decls` changed from `seq(choice(procedure_decl, forward_decl),
';')` to `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` —
`definition_proc_decl` already bakes in its own trailing `';'`, so it couldn't be dropped into the
old shared-`';'` wrapper without doubling the semicolon; moving the `';'` into the first two
branches keeps the tree shape unchanged for those two (the `';'` is anonymous, invisible in the
parse tree either way).

**Mistake avoided, logged for the next round:** this reuse creates a genuine grammar ambiguity —
after `procedure_heading ';'`, the parser can't know with bounded lookahead whether a
`procedure_body` follows (making it a `procedure_decl`) or the declaration is already complete
(making it a `definition_proc_decl`). `tree-sitter generate` caught this immediately with an
"Unresolved conflict" error naming both rules and prescribing exactly the fix: add
`conflicts: $ => [[$.procedure_decl, $.definition_proc_decl]]` at the grammar's top level (new
field, placed next to `externals`/`extras`). This lets GLR explore both interpretations and keep
whichever completes. No corpus test changes were needed for the two pre-existing branches.

New corpus test: `procedures.txt` "Bodiless Procedure Heading", `Cia.mod`'s exact two-procedure
shape (`AddICRVector`/`RemICRVector`, brace vector-offset and param-offset annotations included).
Filled via `tree-sitter test --update`, read back to confirm no `ERROR`/`MISSING` nodes.
`tree-sitter test`: 46/46 green (45 before this round + 1 new).

**Impact:** `sweep_corpus.py` 30.68% → 36.36% (243 → 288 of 792 passing), +45 files — well above
`NEXT.md`'s 125-file *upper-bound-on-whole-file* heuristic, confirming its own caveat that mixed
bodied/bodiless files and files with unrelated `STRUCT` usage elsewhere were undercounted.

M1 is still below its ≥95% exit criterion (36.36%). No candidates queued yet — next round should
re-sample `sweep_corpus.py`'s remaining 504 failures fresh, same method as this round and round
11.

## M1.4 continued — `CASE ... ELSE ... END` ✅ (round 13, 2026-08-10)

Sampled `sweep_corpus.py -v`'s whole-file `ERROR [0, 0]` failures across corpus roots, per
`NEXT.md`'s method. `amiga-oberon-31/Interfaces/*.mod` samples all failed on `STRUCT`/`UNTRACED
POINTER`, already scoped out of M1 (round 9) — skipped rather than rediscovered. A `voc` sample
(`ulmSysIO.Mod`) failed on a different, narrower construct: `CASE whence OF |fromPos: ... |fromEnd:
... ELSE relativity := Platform.SeekSet END`. The leading `|` before the first case and the
label-range cases already parsed fine (round 4's work); the failure was specifically the `ELSE`
arm, which `case_statement` in `grammar.js` never accepted.

Checked `docs/language-baseline.md` line 94 before implementing, per `NEXT.md`'s "cross-check
against the baseline before assuming in-scope" instruction — `Case Statement = CASE Expr OF Case
{"|" Case} [ELSE StatementSeq] END` already includes `[ELSE StatementSeq]` in the *normative*
Oberon-2 EBNF. This is not an AmigaOberon/dialect extension the way `STRUCT` or `ASSEMBLER` are;
`case_statement`'s missing `ELSE` arm was a plain grammar gap against baseline Oberon-2, not a
scoping question — no need to flag it to the user the way round 9 did.

**Grammar change:** `case_statement` gained `optional(seq($.kElse, optional($.statement_seq)))`
before the closing `$.kEnd`, mirroring `if_statement`'s existing `ELSE` handling immediately
above it in `grammar.js`. No new external token, no conflicts.

New corpus test: `statements.txt` "Case Statement With Else", modeled on the existing "Case Label
Range" test's tree shape (hand-written by analogy rather than `--update`-generated, since the
shape — `case_clause`, `case_clause`, `kElse`, `statement_seq`, `kEnd` — was already visible
verbatim in the neighboring `if_statement` pattern and the preceding case test; round 8's
insight about hand-writing trees from guessed rule shapes doesn't apply when a structurally
identical case is right there to copy). Confirmed red before the grammar change, green after.
`tree-sitter test`: 47/47 green (46 before this round + 1 new).

**Impact:** `sweep_corpus.py` 36.36% → 39.39% (288 → 312 of 792 passing), +24 files.

M1 is still below its ≥95% exit criterion (39.39%). No candidates queued — next round should
re-sample `sweep_corpus.py`'s remaining 480 failures fresh, same method as this round.

## M1.4 continued — Oberon-A "system flags" + square-bracket library calls ✅ (round 14, 2026-08-10)

Sampled `sweep_corpus.py -v`'s whole-file `ERROR [0, 0]` failures, filtering out the already-
triaged `amiga-oberon-31/Interfaces/*` (`STRUCT`/`UNTRACED POINTER`, out of scope). The
`oberon-a/source/amiga/*.mod` files stood out as a large, previously-unsampled block — nearly
every file in that directory. `BattClock.mod` (small, representative) failed at `MODULE [2]
BattClock;` — right after the `MODULE` keyword, before the module's own `ident`.

That `[2]` is not the curly-brace AmigaOberon `vector_offset`/`param_offset` from round 12
(different corpus root, different delimiter) — it's a distinct Oberon-A-only construct. Found
the normative description in the Oberon-A compiler's own docs (`Oberon-A/docs/OC.doc`, nodes
`SysFlags`, `LibCalls`, `RegPars`, `ExternalCode`), not the corpus itself:

- **System flags** — `"[" integer "]"` placed directly after `MODULE`, `POINTER`, `PROCEDURE` or
  `RECORD`, marking the following declaration as following a foreign calling/layout convention
  (1=Modula-2, 2=C, 3=BCPL, 4=Assembly). Confirmed via corpus grep: 76 files use it on `MODULE`,
  24 on `PROCEDURE`, 22 on `POINTER`, 14 on `RECORD` (grep needed `-a`, not just `-r` — Latin-1
  files with high-bit bytes are silently skipped as "binary" by default, a repeat of round 11's
  encoding lesson but on the *input* side this time, not string-pairing).
- **Square-bracket library-call heading** — `PROCEDURE identdef "[" ident "," ["-"] integer "]"
  [formal_params]`, e.g. `PROCEDURE OpenLibrary* [base,-552] (...)`. Same concept as the
  curly-brace `vector_offset` (round 12) — a library-base variable and a negative vector
  offset — but a different dialect's delimiter and grammar (leading `"-"` is optional here, not
  mandatory). Modeled as a sibling rule, `square_vector_offset`, not a reuse of `vector_offset`,
  since the two corpora never mix delimiters and forcing one rule to accept both would blur a
  real dialect distinction. 49 files.
- **Register parameters** — `RegSpec = "[" integer "]" [".."]` after a formal parameter's
  `ident`, alongside `param_offset`'s curly-brace form in `fp_section` (`choice($.param_offset,
  $.reg_spec)`). The integer is a CPU register number (0-15: D0-D7, A0-A7); a trailing `".."`
  marks the (always-last) parameter as a variable-length argument list — confirmed structurally
  identical to `param_offset`'s slot, no new rule needed in `fp_section` itself. 89 files use the
  bracket form, 25 of those also use the `".."` vararg marker.
- **External code names** — `"[" string {"," string} "]"` in the same post-`ident_def` slot as
  `vector_offset`/`square_vector_offset`, e.g. `PROCEDURE Foo* ["_Foo"](...)` — the linker
  symbol name(s) of an externally-compiled (non-Oberon) procedure, used instead of a library
  vector offset when the procedure isn't a library call. Distinguishable from
  `square_vector_offset` by first token (`ident` vs. `string`), so `choice($.vector_offset,
  $.square_vector_offset, $.external_code_names)` in `procedure_heading` needed no precedence or
  conflict declaration. 9 files.

**Grammar changes**, test-first (`tree-sitter test` red before, green after):

- `sysflag: $ => seq('[', $.integer, ']')`, spliced into `module_header` (after `kModule`),
  `pointer_type` (after `kPointer`), `record_type` (after `kRecord`) and `procedure_heading`
  (after `kProcedure`, before `receiver`/`ident_def`) — all four are `optional($.sysflag)` at
  the same "right after the keyword" position per `OC.doc`.
- `square_vector_offset` and `external_code_names`, both new rules, added as `choice` siblings
  of the existing `vector_offset` in `procedure_heading`'s post-`ident_def` slot.
- `reg_spec: $ => seq('[', $.integer, ']', optional('..'))`, added as a `choice` sibling of
  `param_offset` in `fp_section`.
- No scanner changes — every addition is a plain context-free rule, no new external token.
  `tree-sitter generate` reported zero conflicts.

New corpus tests: `module.txt` "module system flag"; `types.txt` "Pointer System Flag", "Record
System Flag"; `procedures.txt` "Procedure With System Flag", "Procedure With Square Vector
Offset And Register Parameters", "Procedure With External Code Names", "Procedure With VarArg
Register Parameter" (7 new cases, real shapes copied from `OC.doc`'s own examples —
`OpenLibrary`, `CoerceMethodA` — not guessed). `tree-sitter test`: 54/54 green (47 before this
round + 7 new).

**Impact:** `sweep_corpus.py` 39.39% → 41.41% (312 → 328 of 792 passing), +16 files.

M1 is still below its ≥95% exit criterion (41.41%). Not yet implemented from the same `OC.doc`
family: the external-code declaration's own heading shape when it has neither a library base
nor a vector offset (rare — only the string-list form was confirmed in the corpus), and no
attempt yet to re-check whether `oberon-a/source/amiga/*.mod` files now pass in bulk or hit a
*second* blocking construct further down (`sweep_corpus.py` was only re-run in aggregate this
round, not re-sampled per-file — that's the natural first step next round: check how many
`oberon-a/source/amiga/*.mod` files still fail and on what, before sampling fresh territory).

## M1.4 continued — repeated/interleaved declaration sections (round 15, 2026-08-10)

Followed up on round 14's concrete lead: sampled fresh `oberon-a/source/amiga/*.mod` failures
(`CDDevice.mod`, `Config.mod`, `ClipBoard.mod`, `Disk.mod`, `Graphics.mod`) and found every one
failing on a bare `CONST` or `TYPE` keyword mid-file, with narrow `(ERROR [n,0]-[n,5])` spans —
not a whole-file failure, and not a new dialect construct. These files group logically-related
constants/types under multiple separate `CONST`/`TYPE` blocks within a single module (e.g.
`CDDevice.mod` has three separate `CONST` sections at lines 88, 130, 168, each preceded by a
comment banner).

Checked `docs/language-baseline.md` first per round 13's insight ("not every gap is a scoping
question"): the normative EBNF's `DeclSeq` is `{ CONST {ConstDecl ";"} | TYPE {TypeDecl ";"} |
VAR {VarDecl ";"}} {ProcDecl ";" | ForwardDecl ";"}` — the *outer* `{}` means the whole
CONST/TYPE/VAR alternation repeats zero or more times, not just the inner per-declaration lists.
`grammar.js` had this wrong at all three declaration-sequence sites (plain `MODULE`, `DEFINITION`
module, `procedure_body`): each used `optional($.const_decls), optional($.type_decls),
optional($.variable_decls)` — one section of each kind, fixed order. This was a plain grammar
bug against the normative baseline, not a dialect extension; nothing to flag to the user.

**Grammar change**, test-first (`tree-sitter test` red before, green after): all three sites'
three `optional(...)` lines replaced with one `repeat(choice($.const_decls, $.type_decls,
$.variable_decls))`. No new rules, no scanner changes, `tree-sitter generate` reported zero
conflicts (the three section rules already start with distinct keywords, so GLR needs no help
disambiguating).

New corpus test: `declarations.txt` "Repeated and interleaved decl sections" (`CONST`/`TYPE`/
`CONST`, three sections in one module, shapes copied verbatim from existing single-section
tests). `tree-sitter test`: 55/55 green (54 before + 1 new).

**Impact:** `sweep_corpus.py` 41.41% → 54.42% (328 → 431 of 792 passing), **+103 files** — by far
the largest single-round gain since the sweep tool existed, confirming this is a widespread
pattern across the corpus, not an Oberon-A-only quirk (`grep`-plausible: any module organizing
declarations by logical grouping rather than one big `CONST`/`TYPE`/`VAR` block hits this).

M1 is still below its ≥95% exit criterion (54.42%). Next round should re-sample fresh failures
across corpus roots (not just `oberon-a/source/amiga/`) since this fix likely cleared out a
different mix of files than round 14 anticipated — no re-sampling done yet this round beyond the
aggregate number.

## M1.4 continued — Oberon-A "assignable procedure" mark (round 16, 2026-08-10)

Checked round 15's carried-over lead first: `oberon-a/source/amiga/*.mod` was **not** fully
cleared by round 15's fix (31/121 files in that directory still failed), so sampled there before
moving to fresh territory. `Bullet.mod`'s failure was a 10-line `ERROR` span starting right at a
`PROCEDURE* [0] CloseLib (VAR rc : LONGINT);` heading — a `*` immediately after the `PROCEDURE`
keyword, before the system flag and identifier (not the usual export mark, which comes after the
identifier).

`docs/OC.doc`'s "AssignableProcs" node names this directly: "Procedures that are to be assigned
to procedure variables must be marked with a `*` character, unless they are marked as exported,"
with the example `PROCEDURE * Assignable;`. Grepping both Oberon-A roots confirmed this is
widespread and independent of round 14's square-bracket family: 78 files in `oberon-a`, 11 in
`amiga-oberon-31` (which predates Oberon-2 and has no `OC.doc`, but uses the identical mark) —
89 files total, no case found combining the mark with the identifier's own export mark (the doc's
"unless exported" phrasing explains why: they're alternatives, not both used together).

**Grammar change**, test-first (`tree-sitter test` red before, green after): `procedure_heading`
gained `optional($.kStar)` between `$.kProcedure` and `optional($.sysflag)`. Reused the existing
`kStar` token (already used inside `ident_def` for the export mark) — no new token, no scanner
change, `tree-sitter generate` reported zero conflicts.

New corpus test: `procedures.txt` "Procedure With Assignable Mark" (`Bullet.mod`'s own
`PROCEDURE* [0] CloseLib` shape, copied verbatim). `tree-sitter test`: 56/56 green (55 before +
1 new).

**Impact:** `sweep_corpus.py` 54.42% → 60.61% (431 → 480 of 792 passing), +49 files.

M1 is still below its ≥95% exit criterion (60.61%). Post-fix root breakdown of remaining
failures: `stj` 105, `amiga-oberon-31` 92, `oberon-a` 72, `voc` 43 — `oberon-a` dropped the most
(was 118) but `stj` and `voc` are now the largest untouched territory relative to their size and
haven't been sampled at all yet this round; worth checking those first next round.

## M1.4 continued — STJ-Oberon `AND`/`NOT` keyword operators (round 17, 2026-08-10)

First-ever sampling pass over `stj` (105 failures, largest untouched root). Bisected a minimal
repro (`IF (byte >= 0) AND (byte < 20) THEN ... END`) down from a real failure in
`DEBUGGER.PRJ/HEXDUMP.MOD`: the parser reported a confusing `MISSING "*"` at an unrelated column
rather than a clean `ERROR` on `AND` itself — worth remembering as a signature: a `MISSING`
node for an unrelated token, landing mid-expression, can mean the real problem is an operator
the grammar has no rule for at all (GLR error recovery guesses a continuation using whatever
token *is* known, producing a misleading location).

Confirmed via corpus grep that STJ-Oberon's `.MOD` sources use `AND` and `NOT` as textual
synonyms for `&` and `~` — not a replacement, since `&` (70 files) and `~` both still coexist
with `AND` (55 files) in the same corpus. Cross-checked `docs/language-baseline.md`: neither
`AND` nor `NOT` appears anywhere in the normative EBNF, so this is a genuine STJ dialect
extension, not a missed baseline rule (round 13's distinction). Unlike `STRUCT`/`ASSEMBLER`
(round 9), this isn't a structural extension needing a scoping conversation — it's a lexical
keyword synonym for an operator the grammar already has, squarely inside D1's "lexical
superset" scope, so implemented directly. Corroborating evidence: two `.OBJ` files in the corpus
are STJ's own compiler binaries and happen to embed a plaintext keyword table (visible via
`grep -a`) that lists `AND` and `NOT` alongside `DIV`, `MOD`, `NOT` as reserved words — confirms
this is the compiler's own vocabulary, not a corpus-author idiosyncrasy.

**Grammar change**, test-first (`tree-sitter test` red before, green after): `mul_operator`
gained `$.kAnd` as a `choice` sibling of `'&'`; `factor` gained `seq($.kNot, $.factor)` as a
sibling of the existing `seq('~', $.factor)`. Two new keyword tokens (`kAnd => 'AND'`,
`kNot => 'NOT'`), no scanner changes. `tree-sitter generate` reported zero conflicts — tree-sitter's
keyword-vs-identifier precedence handles `AND`/`NOT` automatically, no reserved-word list to
maintain separately.

New corpus test: `statements.txt` "AND/NOT Keyword Operators" (`IF (x >= 0) AND NOT (x < 1)
THEN...`, exercises both new tokens together since the corpus commonly combines them). Filled
via `tree-sitter test --update`, read back to confirm no `ERROR`/`MISSING` nodes.
`tree-sitter test`: 57/57 green (56 before + 1 new).

**Impact:** `sweep_corpus.py` 60.61% → 66.41% (480 → 526 of 792 passing), **+46 files**, all in
`stj` (105 → 59 failing there) — confirms the fix was isolated to this one root, as expected for
a dialect-specific keyword synonym.

M1 is still below its ≥95% exit criterion (66.41%). Post-fix root breakdown: `amiga-oberon-31`
92, `oberon-a` 72, `stj` 59, `voc` 43. `stj` still has the largest raw count of any root but is
no longer the most *disproportionately* unsampled — `amiga-oberon-31` (92 failures, last
dedicated round was round 12) and `voc` (43, never sampled) are the natural next candidates.
`stj` itself likely has more clusters left (59 files) and could also be resampled fresh.

## M1.4 continued — AmigaOberon 3.1 cluster: CLOSE, typed sets, real/range lexer bug, U-hex, curly names, param varargs (round 18, 2026-08-10)

First dedicated sampling pass over `amiga-oberon-31` since round 12 (92 failures, per round 17's
note the natural next candidate over `stj`/`voc`). Tallied failures by root first to confirm the
picture still held, then filtered out files containing `STRUCT`/`UNTRACED` (60 of 92 — still
scoped out to Phase 2 per round 9) to find the actual next cluster among the remaining 32.

Six grammar changes this round, all test-first (`tree-sitter test` red before, green after each):

1. **Module-level `CLOSE` section.** `BootBlock.mod`-adjacent files showed a `CLOSE` keyword
   between a module's `BEGIN` statement sequence and its `END` — a finalizer section run on
   module unload, confirmed via corpus grep (45 files, always paired with a preceding `BEGIN` in
   the same file, never standalone). Not in `docs/language-baseline.md` — genuine AmigaOberon
   dialect extension. `module`'s `BEGIN` arm gained `optional(seq($.kClose,
   optional($.statement_seq)))` after the existing `optional($.statement_seq)`; new keyword
   token `kClose => 'CLOSE'`. No conflicts.
2. **Typed set constructor** `LONGSET{...}` / `SHORTSET{...}` — AmigaOberon's fixed-width SET
   types (16-bit `SHORTSET`, 32-bit `LONGSET`) used both as ordinary type names (formal params,
   return types — already handled generically via `qualident`) and as a constructor prefix
   directly before a `{...}` set literal. New `typed_set: $ => seq($.qualident, $.set)` rule,
   added as a `factor` sibling of the existing bare `$.set`. No conflicts — `{` is never a valid
   `selector` continuation after a designator, so no ambiguity with the `seq($.designator,
   optional($.actual_params))` factor branch.
3. **Real-number/range lexer bug** (affects all four corpus roots, not just AmigaOberon): writing
   the typed-set test with a range element (`LONGSET{1, 2..4}`) exposed a pre-existing bug —
   `real`'s grammar (`digit {digit} "." {digit} [ScaleFactor]`) allows zero digits after the
   `.`, so the lexer's maximal-munch greedily matches `2.` as a bare real literal, leaving only
   one `.` where `element`'s `".." expression` needs two. `label_range` (CASE label ranges)
   never hit this because `label` uses `$.integer` directly, not `$.number`/`real` — only
   `element` (used by `set`) goes through `number`. Grepped all four corpus roots for genuine
   bare-`N.`-real usage (found none — matches were all false positives from identifiers
   containing digits, e.g. `VT100.ED`); tree-sitter has no lookahead/lookbehind support (Rust's
   `regex` crate excludes it by design) so an external-scanner fix wasn't attempted for what
   turned out to be unnecessary. Fix: require at least one digit after the `.` (`real`'s
   fractional part changed from `repeat(digit)` to `digit, repeat(digit)`), which matches
   how real Oberon code is actually written and removes the ambiguity outright.
4. **Unsigned hex integer literal**, `U` suffix as a sibling of the existing `H` suffix (e.g.
   `016C0U`), used throughout for raw machine-code words passed to `SYSTEM.INLINE` and hex
   bit-mask constants (7 source files, confirmed via `grep --include='*.mod'` after first ruling
   out false positives from grepping compiled `.OBJ`/binary siblings in the same directories).
   `integer` gained `token(seq(digit, repeat(hex_digit), 'U'))` as a third choice arm.
5. **Curly-brace external code names**, `{"Alerts.AlertDummy"}` on a procedure heading — a
   curly-brace sibling of Oberon-A's existing square-bracket `external_code_names`
   (`["_Foo"]`). Only 3 occurrences in this root but blocked full-file parses. New
   `curly_external_code_names: $ => seq('{', $.string, repeat(seq(',', $.string)), '}')`,
   added to `procedure_heading`'s existing `choice($.vector_offset, $.square_vector_offset,
   $.external_code_names, ...)` slot. No conflict with `vector_offset` (also `{`-led) since
   their first token differs (string vs. ident).
6. **`param_offset` varargs marker.** Fixing #5 above still left `Alerts.mod` failing at the
   same procedure heading — bisected to `data{9}..: SYSTEM.ADDRESS`, a trailing `..` after a
   parameter's curly-brace `param_offset`, mirroring the `..` that Oberon-A's square-bracket
   `reg_spec` already supports (round 14) but `param_offset` never gained. `param_offset` grew
   `optional('..')` after its closing `}`.

New corpus tests, one per construct: `module.txt` "module begin close end";
`statements.txt` "AmigaOberon typed set constructor" and "AmigaOberon unsigned hex literal";
`procedures.txt` "Procedure With Curly External Code Name" and "Procedure With Param Offset
Varargs Marker". `tree-sitter test`: 62/62 green (56 before this round + 6 new).

**Impact:** `sweep_corpus.py` 66.41% → 69.95% (526 → 554 of 792 passing), +28 files. Root
breakdown of the delta: `amiga-oberon-31` 92 → 73 (-19, all six fixes), but `oberon-a` 72 → 67
(-5), `stj` 59 → 57 (-2), and `voc` 43 → 41 (-2) also dropped — confirms fix #3 (the real/range
lexer bug) was genuinely cross-dialect, not AmigaOberon-specific, unlike fixes #1/#2/#4/#5/#6.

M1 is still below its ≥95% exit criterion (69.95%). `amiga-oberon-31`'s remaining 73 failures:
60 are `STRUCT`/`UNTRACED` (Phase 2 scope, unchanged), 13 are not. Sampled one of the 13
(`Demos/Sparks.mod`) far enough to find the next lead: `Ciapra[0BFE001H]: SHORTSET;` in a
`VAR` section — a square-bracket absolute hardware-address annotation on a variable
declaration, structurally new (no existing `var_decl` grammar slot for it) but not implemented
this round.

## M1.4 continued — hardware-address vars, D/E scale-factor bug, designator/actual_params ambiguity (round 19, 2026-08-11)

Started from round 18's known lead (`Sparks.mod`'s `Ciapra[0BFE001H]: SHORTSET;`), fixed it,
then kept sampling `amiga-oberon-31`'s remaining non-`STRUCT` failures. Two of the three fixes
this round turned out to be cross-dialect, one of them (fix 3) the single largest-impact fix to
date.

1. **Absolute hardware-address variable annotation**, `ident[hexInteger]: type` in a `VAR`
   section, e.g. `Ciapra[0BFE001H]: SHORTSET;` (AmigaOberon custom-chip register mapping).
   Confirmed via corpus grep: always exactly one identifier, never a comma list (4 occurrences,
   `Sparks.mod`/`Sparks2.mod`). Rather than folding into the shared `ident_list` (used by
   `field_list`/`fp_section` too, which never carry this), added a sibling
   `addressed_ident: $ => seq($.ident_def, $.address)` alternative inside `variable_decl`
   (`choice($.ident_list, $.addressed_ident)`), with `address: $ => seq('[', $.integer, ']')`.
   New test in `declarations.txt`. No conflicts — matches round 18's `param_offset`/`reg_spec`
   precedent of duplicating `ident_list`'s shape rather than editing the shared rule.

2. **Scale-factor lexer bug** (`D`/`E` real-number suffix), found while isolating fix 1's file
   further: `LongRealConversions.mod`'s `trans.Pow(n,0.1D)` failed to parse. `docs/
   language-baseline.md`'s `ScaleFactor = ("E"|"D") ["+"|"-"] digit {digit}` was only half
   implemented — the grammar had `E` only (no `D`), and required a mandatory sign. Corpus grep
   across all four roots found real usage diverges from the baseline's own EBNF in two ways:
   unsigned exponents with digits present (`9.22337177E18`, oberon-a), and AmigaOberon's `D`
   (LONGREAL literal) marker used consistently *bare*, no sign or digits at all
   (`3.141592653589793D`, both `amiga-oberon-31` and `oberon-a`). Fix: `scale_factor` gained
   `D` as a second choice alongside `E`, and made both the sign and the `digit {digit}` tail
   optional (`seq(choice('E','D'), optional(seq(optional(choice('+','-')), digit,
   repeat(digit))))`). Two new tests in a new `numbers.txt` corpus file (no prior file covered
   bare `real`/`scale_factor` literals directly).
   **Impact:** 70.20% → 73.48% (556 → 582), **+26 files** — cross-dialect: `voc` 41→26 (-15),
   `stj` 57→48 (-9), `amiga-oberon-31` 71→69 (-2). Confirms the baseline EBNF's own literal
   shape can still be wrong against real usage, same lesson as round 18's real/range fix.

3. **`designator`/`actual_params` ambiguity** — the round's deepest fix. `COMPLEX.mod`,
   `VECTOR.mod`, `SecureDos.mod`, `STRING.mod` all failed on a type-guard immediately followed
   by a selector or another call, e.g. `n(COMPLEX).Norm()` / `np(LockNode).lock`. Root cause:
   the report's own grammar has `designator = qualident {selector}` (`selector` includes the
   type-guard form `"(" qualident ")"`) and separately `factor = ... designator
   [ActualParameters] | ...` (`ActualParameters = "(" [ExpList] ")"`), bolted onto the *end* of
   designator only, once. A parenthesized single bare identifier is exactly the same token
   sequence for both — the real Oberon-2 grammar is genuinely ambiguous here, and real
   compilers resolve it via the symbol table (is the name a type?), which this syntax-only
   grammar has no access to. tree-sitter's default LALR resolution deterministically always
   picked `actual_params` (never `selector`), so once `(COMPLEX)` was consumed as a "call",
   nothing in the grammar allowed the following `.Norm()` to attach anywhere — hence `ERROR`.
   Confirmed via a minimal repro (`n(T).val` inside a plain procedure body, no receiver needed)
   that this was unconditional, not context-sensitive, and that adding `conflicts:
   [[$.selector, $.actual_params]]` alone (their old, separate homes — `selector` inside
   `designator`'s own `repeat`, `actual_params` bolted on afterward in `factor`) did nothing;
   tree-sitter reported it "unnecessary" both times (no automaton-level fork was ever being
   built between them in that shape).
   Fix: moved `actual_params` **into** `designator`'s own repeat, as a `choice` sibling of
   `selector` (`repeat(choice($.selector, $.actual_params))`), removed the now-redundant
   trailing `optional($.actual_params)` from `factor` and `procedure_call` (both now just use
   `$.designator` directly). This lets guards, field accesses and calls interleave and chain
   arbitrarily, matching corpus reality; kept `conflicts: [[$.selector, $.actual_params]]`
   declared (still reported "unnecessary" by `tree-sitter generate`, meaning no GLR fork is
   actually needed even now — the ambiguous case apparently never reaches a genuine automaton
   conflict, some other tree-sitter-internal resolution already picks a workable parse; kept
   the declaration anyway as in-code documentation of the known ambiguity). Reshapes the AST:
   `actual_params` is now a child of `designator` (alongside `selector`) instead of a sibling
   of it under `factor`/`procedure_call` — updated the 5 existing call-site assertions across
   `statements.txt` via `tree-sitter test --update`, read back to confirm pure reshaping (same
   content, no `ERROR`/`MISSING`), no other tests affected.
   **Impact:** 73.48% → 77.90% (582 → 617), **+35 files**, the single largest one-fix gain to
   date — cross-dialect: `voc` 26→10 (-16), `oberon-a` 67→60 (-7), `stj` 48→40 (-8),
   `amiga-oberon-31` 69→65 (-4).

4. **NBSP (U+00A0) not treated as whitespace.** Found while re-sampling `amiga-oberon-31`'s
   remaining non-`STRUCT` failures after fix 3: `BasicTypes.mod` had a literal Latin-1 `0xA0`
   byte used as inter-token whitespace right after a procedure heading's `;` (before `BEGIN`).
   `extras`' plain `/\s/` regex doesn't match it. Grepped all four roots for the raw byte
   (`LC_ALL=C grep $'\xa0'`): 10 files in `amiga-oberon-31`, 2 in `oberon-a` — mostly inside
   comment prose (German text, already opaque to the grammar via the external scanner) but a
   few, like `BasicTypes.mod`, `Lists.mod`, `FArrays.mod`, use it as bare inter-token
   whitespace. Fix: `extras`' whitespace regex widened to `/[\s ]/`.
   **Impact:** 77.90% → 78.16% (617 → 619), +2 files (`BasicTypes.mod` and one `oberon-a` file).

**Known remaining issue, not fixed this round:** `Lists.mod` and `FArrays.mod` still fail after
fix 4, at the same NBSP-adjacent procedure heading — but only when the NBSP is *followed by a
comment* before `BEGIN` (isolated via a minimal repro: NBSP alone before `BEGIN` parses fine;
comment alone before `BEGIN` parses fine; NBSP **and** comment together, even on a plain
receiver-less `PROCEDURE Add; <NBSP>\n(* comment *)\nBEGIN...END Add;`, fails). This reproduces
regardless of receiver/assignable-mark. Suspected cause: interaction between the `extras`-level
ambiguity (two different whitespace-token shapes, regex vs. external-scanner comment) and the
pre-existing `procedure_decl`/`definition_proc_decl` GLR fork (round 12) — extra lexer states
introduced by the NBSP token appear to tip that fork's resolution the wrong way when a comment
immediately follows. Not chased further this round (only 2 files affected, and the mechanism
looked like a deeper tree-sitter GLR/extras interaction rather than a simple grammar-shape
fix) — a lead for whoever samples `amiga-oberon-31` next, worth minimizing further (does a
*plain* extra `\s\s` double-space + comment combo also trigger it, ruling out NBSP specifically
and pointing at "any two different extras token kinds back to back"?) before attempting a fix.

`tree-sitter test`: 65/65 green (63 before this round + 2 new: 1 `addressed_ident` case in
`declarations.txt`, plus a new `numbers.txt` file with 2 cases for the scale-factor fix; the 5
pre-existing `actual_params` assertions in `statements.txt` were reshaped, not added).

M1 is still below its ≥95% exit criterion (78.16%). Post-round-19 failure counts by root:
`amiga-oberon-31` 64 (4 non-`STRUCT`: `Break.mod`, `FArrays.mod`, `Lists.mod`,
`linkedlists.mod`), `oberon-a` 59, `stj` 40, `voc` 10. `voc` dropped the most this round (41→10)
and is now the smallest-by-far root remaining — worth a dedicated fresh sampling pass next,
alongside finishing `amiga-oberon-31`'s last 4 non-`STRUCT` files.

## M1.4 continued — round 20 (2026-08-11)

Fixed round 19's known lead first, then ran `voc`'s first dedicated sampling pass (never
sampled before — 10 failures, smallest root by far), fixing 6 of its 10 files in one shot with
a single new rule. Asked the user how to scope `Break.mod`'s dual-header conditional
compilation; they chose to defer the decision rather than implement or formally scope out this
round, so it stays an open lead. 78.16% → 79.29% (619 → 628/792), +9 files.

1. **The NBSP+comment GLR bug, root cause found and fixed.** Round 19 left `Lists.mod` and
   `FArrays.mod` failing at a procedure heading where an NBSP is immediately followed by a
   comment before `BEGIN`, mechanism unknown. Minimized to a 9-line repro (`PROCEDURE
   Add*(x:INTEGER);<NBSP>\n(* comment *)\n\nBEGIN...END`) and confirmed via `src/scanner.c`
   read-through: the external scanner's own `is_space()` — used to skip leading whitespace
   *before* checking whether a comment starts — only recognizes `' ' '\t' '\n' '\r' '\v'
   '\f'`, not NBSP (`0xa0`), even though `grammar.js`'s `extras` regex (`/[\s ]/`, added
   round 19) does. When the external scanner is probed at a position starting exactly on an
   NBSP (which happens constantly, since `comment`/`pragma`/`bracket_pragma` are extras and so
   almost always "valid" at every lex decision), its whitespace-skip loop doesn't advance past
   the NBSP, sees a non-`(`/non-`<` character, and declines (returns `false`) — normally
   harmless (tree-sitter falls back to the internal regex extra for the single NBSP character,
   then re-probes and finds the comment fine on the next attempt), but the two-different
   whitespace-recognition boundaries created by this mismatch line up badly with the
   pre-existing `procedure_decl`/`definition_proc_decl` GLR fork (round 12) at exactly this
   spot, corrupting which fork survives. Fix: added `|| c == 0xa0` to `is_space()`, with a
   comment noting it must stay in sync with `extras`. One-line fix once the mechanism was
   understood; minimizing (binary-searching a real corpus file down from 158 lines to a 9-line
   synthetic repro, confirming NBSP-alone and comment-alone both parse fine in isolation) took
   most of the time. New test in `comments.txt` (`"NBSP before comment before BEGIN"`) with a
   real `\xa0` byte in the source, added via Python (not the `Edit` tool, to avoid the "did
   `Edit` normalize my literal NBSP" uncertainty round 19 flagged).
   **Impact:** 78.16% → 78.54% (619 → 622), **+3 files**, all `amiga-oberon-31`: `Lists.mod`,
   `FArrays.mod`, and the previously-undiagnosed `linkedlists.mod` (turned out to be the same
   bug — found via a fresh top-down binary search of the file, not yet known to be the same
   issue going in).

2. **voc's bodiless "external C procedure" heading**, `voc`'s first dedicated sampling pass.
   `PROCEDURE -ident [formal_params] [": " type] "C source string";` — a `-` mark right after
   `PROCEDURE` (same slot as Oberon-A's `*` assignable-procedure mark, but a different meaning:
   this dialect uses it to mark a procedure whose body is a literal C-source string spliced
   into voc's generated C output at each call site, instead of `BEGIN...END`). Confirmed via
   corpus grep: 56 occurrences across 6 files (`oocX11.Mod`, `oocXYplane.Mod`, `oocXutil.Mod`,
   both `oocwrapperlibc.Mod` copies, `ulmSysStat.Mod`), always exactly this shape — no
   receiver, no body, string always present and always immediately before the final `;` (some
   headings span 10+ lines of multi-line formal params, which produced false negatives in an
   early "does a string appear before the first `;`" grep check until the search window was
   widened past the first `;` inside the parameter list itself). New rule, added as a fourth
   `procedure_decls` alternative alongside `definition_proc_decl` (same structural family —
   bodiless heading): `external_proc_decl: $ => seq($.kProcedure, '-', $.ident_def,
   optional($.formal_params), $.string, ';')`. No conflicts reported. New test in
   `procedures.txt` (`"voc external C procedure"`).
   **Impact:** 78.54% → 79.29% (622 → 628), **+6 files**, all `voc`: both `oocwrapperlibc.Mod`
   copies, `oocX11.Mod`, `oocXYplane.Mod`, `oocXutil.Mod`, `ulmSysStat.Mod` — cleared 6 of
   `voc`'s 10 failures in one fix, the highest hit-rate of any fix so far.

**Scoping question raised and deferred (not implemented, not scoped out):** `Break.mod` and
`NoGuru.mod` (2/792, both `amiga-oberon-31`) use `(* $IF X *) MODULE A; (* $ELSE *) MODULE B;
(* $END *)` — two alternate top-level module headers guarded by conditional-compilation
pragma-comments, confirmed genuinely structural (the `module` rule only expects one `MODULE
ident ;`; grepped all `$IF` usage in the root first — 11 files use `$IF` for ordinary
conditional imports/pragmas, which already parse fine since pragma-comments are opaque extras,
but only these two duplicate the module header itself). Asked the user whether to scope this
out to Phase 2 (STRUCT-style), implement it now, or defer the decision; they chose to defer —
it remains an open lead in `NEXT.md`, not a resolved scoping decision.

**New leads found in `voc`'s remaining 4 failures, not attempted this round:**
- `MultiArrayRiders.Mod` and `MultiArrays.Mod` both have free-text documentation/usage notes
  appended *after* the module's closing `END Module.` — not valid Oberon syntax at all (real
  Oberon compilers stop reading at the closing `.`), e.g. `MultiArrays.Test\nCompiler.Compile
  \xc MultiArrays.Mod  ~`. The grammar's top-level `module` rule requires EOF right after
  `module_footer`; tolerating trailing content would need a deliberate "ignore the rest of the
  file" escape hatch (e.g. an `optional` catch-all token matching to EOF), which is a genuine
  design question (what should M2's lossless serializer do with such a trailing span?) — not a
  routine grammar addition, flag before implementing.
- `ethUnicode.Mod` has literal **binary** bytes after its `END ethUnicode.` (a serialized
  Native-Oberon font/timestamp object, `Oberon10.Scn.Fnt`/`TimeStamps.New` visible in the raw
  bytes) — not text at all, not parseable by any grammar extension; almost certainly wants the
  same "ignore trailing content" mechanism as the two files above, if one gets built.
- `ulmRandomGenerators.Mod` fails on `1. - real` — a bare real-number literal with **zero**
  digits after the decimal point, used in an ordinary arithmetic expression. This directly
  collides with round 18's fix (`real`'s fractional part was made to *require* ≥1 digit
  specifically to keep `2..4` lexing as `integer(2)` + range(`..`) + `integer(4)` rather than
  greedily eating the first `.` into `real`). Relaxing the fractional digit count back to
  optional was confirmed (by hand-tracing tree-sitter's maximal-munch DFA walk) to reopen
  exactly that regression: for `2..4`, `real` would match `2.` (now a complete 2-char token)
  before the DFA discovers there's no valid continuation past the second `.`, so maximal munch
  keeps the longer `real` match over the 1-char `integer`, breaking the range case again. The
  two facts (`1.` must lex as a complete real; `2..4` must not) can't both be satisfied by a
  single regex-only token, since tree-sitter's internal lexer (Rust `regex` crate) has no
  lookahead — this needs the same technique already used for comments/pragmas: move (at least
  the ambiguous tail of) real-number lexing into the **external scanner**, where
  `lexer->lookahead` after consuming the first `.` can check "is the next char a digit (keep
  consuming as real), another `.` (abort, let `integer` + `..` win instead), or neither (accept
  `N.` as a complete real)". Not attempted this round — a non-trivial change to a token used in
  nearly every file (real regression risk), confirmed needed for only this one currently-failing
  file so far (an initial corpus grep for bare `N.` looked like ~150 occurrences across the
  corpus but turned out to be almost entirely comment prose — e.g. "June 1990." — once sampled
  by hand; the true prevalence in actual code is unknown and worth checking properly, e.g. by
  grepping only outside `(* ... *)` spans, before investing in the external-scanner change).

`tree-sitter test`: 67/67 green (65 before this round + 2 new: `"NBSP before comment before
BEGIN"` in `comments.txt`, `"voc external C procedure"` in `procedures.txt`).

M1 is still below its ≥95% exit criterion (79.29%). Post-round-20 failure counts by root:
`amiga-oberon-31` 61 (`Break.mod`/`NoGuru.mod` dual-header lead still open, rest is `STRUCT`),
`oberon-a` 59, `stj` 40, `voc` 4 (2 trailing-garbage, 1 binary, 1 bare-real lexer gap — all
documented above). `oberon-a` and `stj` haven't had a dedicated sampling pass since rounds 17
(`stj`) and never (`oberon-a` was last touched round 14, not a full sampling pass) — worth
picking one for round 21 now that `voc` is nearly clear.
