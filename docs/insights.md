# Insights

Things learned that were not obvious beforehand. Newest round last.

## Round 1 — 2026-08-10

### The existing tree-sitter Oberon-2 grammar is a starting point, not a solution

`viegasfh/tree-sitter-oberon-2` (MIT, 500 lines, last touched 2023) covers roughly 60% of what
is needed. Confirmed missing against the report's EBNF:

- **Type-bound procedure receivers** — `PROCEDURE (r: T) M*`. `procedure_heading` is
  `kProcedure ident_def [formal_params]`, with no `Receiver`. Used in 38 Oberon-A, 70 STJ and
  21 AmigaOberon files, so this is not an edge case.
- **`WITH`, `LOOP`, `EXIT`, and `RETURN` as a statement** — the `statement` rule lists only
  assignment, call, `IF`, `CASE`, `WHILE`, `REPEAT`, `FOR`.
- **Forward declarations** (`PROCEDURE ^ …`).
- **Nested comments** — the comment token is a flat regex, which cannot nest.

Everything else (module structure, declarations, expressions, designators, precedence) is sound
and worth keeping. `geekstakulus/tree-sitter-oberon-07` is the same skeleton plus a
`queries/highlights.scm` worth porting.

### Nested comments are normative Oberon-2, not a dialect quirk

Report §3.6: *"Comments may be nested."* 48 corpus files actually do it (25 Oberon-A, 13
AmigaOberon, 10 STJ). A regex token cannot express nesting, so an external C scanner is
mandatory rather than a nice-to-have. This was the single most likely source of a late,
expensive surprise.

### Empty statements are legal and the corpus relies on them

In the EBNF, `Statement = [ … ]` — the whole production is optional. So `BEGIN ; END` and a
trailing `;` before `END` are both valid. A grammar that requires a statement will emit `ERROR`
nodes on real files for a reason that looks like a corpus problem rather than a grammar bug.

### `SYSTEM` is not a reserved word

It is an ordinary module identifier (report Appendix C). `SYSTEM.ADR(x)` parses as a plain
qualified designator with no grammar support at all. 167 Oberon-A and 169 STJ files import it —
none of that needs grammar work; the dialect-specific *procedures* are a Phase 2 catalog
concern, not a parsing one.

### The corpus's encodings are not one problem but three

Oberon-A and AmigaOberon are Latin-1 (`0xFC` = ü, `0xA9` = ©); STJ is an Atari codepage where
the same characters are `0x81`/`0x94`/`0x84` (CP437-like); voc is UTF-8. STJ is also uniformly
CRLF while everything else is LF. Any design that has to *know* the charset needs three tables
and a detection heuristic. Mapping bytes `0x00-0xFF` to `U+0000-U+00FF` instead is a total
bijection that needs none of that, and is safe precisely because Oberon identifiers are ASCII —
high bytes only ever appear inside comments and strings (decision D3).

### A bin-only crate cannot be tested from `tests/`

Rust integration tests can only import a library target. `xoft-cli` therefore has both
`src/main.rs` and `src/lib.rs`, with the logic in the lib and `main.rs` reduced to argument
parsing. Worth doing from the start rather than retrofitting.

### The STJ corpus exists twice on disk

`~/sandkasten/tmp-stj-oberon-prj/OBERON_I` and `~/atari-retro-dev/c-drv/OBERON_I` are identical
apart from a `.DS_Store`. `corpus/roots.toml` uses the `atari-retro-dev` copy; the other is
ignored. Without this, the corpus would have been double-counted at 1098 files.

### The upstream grammar needed zero rule changes for tree-sitter 0.26

`tree-sitter generate` on `viegasfh/tree-sitter-oberon-2` as-is, under CLI 0.26.11, produces
only warnings (ABI 14 fallback for lacking `tree-sitter.json`; one redundant `seq` in `comment`)
— no errors, no conflicts. The "written against ~0.20, expect breakage" premise in the M1.1 task
brief did not hold. Worth remembering when scoping future mechanical tasks: verify before
padding the estimate for a CLI-version gap.

## Round 3 — 2026-08-10

### `DEFINITION` isn't just an alternate `MODULE` keyword

The task brief framed it as "a second acceptable keyword where `module_header` currently
hardcodes `kModule`." Grepping the real STJ corpus first (per the task's own instruction)
showed that's wrong: procedure declarations inside a `DEFINITION` module have no body at all —
`PROCEDURE Open;` then straight to the next declaration, never `END ident`. A label on a task
brief is a hypothesis, not a spec; the EBNF-adjacent framing ("second keyword") undersold a
structurally different declaration form one level down. Cost nothing to check — one `grep -rn`
before writing any grammar.js — and would have shipped a grammar that still `ERROR`s on 70 of
112 real `.DEF` files if trusted at face value.

### The apparent receiver/formal_params conflict wasn't real

`NEXT.md` flagged a likely grammar conflict between `receiver` and `formal_params`, both
starting with `"("`. They never actually collide: `receiver` sits *before* the mandatory
`ident_def` in `procedure_heading` and `formal_params` sits *after* it, so a `"("` seen
immediately after `kProcedure` is unambiguously `receiver` (nothing else can start there, since
`ident_def` can't begin with `"("`). `tree-sitter generate` produced zero conflicts on the
first attempt. Worth remembering: a flagged risk from a handoff document is a place to look
carefully, not a guarantee of an actual problem — check by running the tool before spending
design effort pre-emptively working around a conflict that may not exist.

### A second gap in the same shape as the already-known one

The empty-statement gap (`StatementSeq` requires a `Statement`, but the EBNF makes the whole
production optional) has a sibling: `FieldList` inside a `RECORD` has the same issue — real
corpus files (`AES.DEF`, `PROCLIST.DEF`) put a trailing `";"` before `END` after the last field,
which `field_list_seq` (no trailing-separator alternative) rejects. Same root cause — a
grammar rule modeling a "list with separators" as `item {sep item}` when the source allows
`item {sep item} [sep]` — recurring in more than one place in the same EBNF. Worth checking all
list-like productions (`FieldList`, `StatementSeq`, `DeclSeq`, `CaseLabelList`, …) for the same
optional-trailing-separator shape in one pass when M1.2b picks this up, rather than
rediscovering each instance one corpus file at a time.

### `tree-sitter test --update` is safe to trust for generating corpus tests, if read back once

Writing new corpus test cases by hand (predicting the exact parenthesized tree tree-sitter will
produce) is slow and error-prone for anything beyond a trivial rule. Writing just the title and
source, with a placeholder `(module)` as the expected tree, then running `tree-sitter test
--update -i <pattern>`, fills in the real tree from the real parser. The one thing this doesn't
catch on its own: an `--update` run "succeeds" even if the actual tree contains an `ERROR` node
— the tool is recording what the grammar produces, not asserting it's correct. Read the
generated tree back once (grep for `ERROR`, and glance at whether the node shape matches intent)
before trusting it as a regression baseline.

### A rule that widens every element to `optional` can start matching the empty string

Fixing the empty-statement gap by writing `statement_seq: seq(optional($.statement),
repeat(seq(';', optional($.statement))))` (the literal reading of "each element is optional")
compiles to a rule that *can* match zero tokens — and `tree-sitter generate` refuses to build
any rule that can match the empty string, even one only ever used inside an outer `optional()`.
The fix is a two-branch `choice`: one branch anchored on a real `$.statement`, the other
`repeat1` over `;`-separated empty slots, so every branch consumes at least one token. The
general lesson: "make every element optional" and "make the whole rule able to match nothing"
are different asks, and only the caller (`optional($.statement_seq)`) needs the latter — an
inner rule that can match empty is a tree-sitter error regardless of how it is used.

### The already-known "RETURN only at the end" restriction wasn't real once checked

`procedure_body` had a hardcoded `optional(seq($.kReturn, $.expression))` appended after
`statement_seq`, modeling the classic Oberon restriction that `RETURN` appears once, at the
textual end of a procedure. Grepping the corpus for `RETURN` before assuming this held (same
discipline as M1.2a's `DEFINITION` lesson) found `Oberon-A/source/ol/OLPrefsStrings.mod:157-160`
— an early-return pattern, `RETURN` as the last statement of *each* branch of an `IF`, which is
mid-body, not end-of-procedure. Cross-checking `docs/language-baseline.md`'s `ProcDecl`
production confirmed the EBNF never had a separate `RETURN` slot in the first place — it's just
`DeclSeq [BEGIN StatementSeq] END ident`, and `RETURN [Expr]` is one of `Statement`'s ordinary
alternatives. The hardcoded field in `procedure_body` was modeling a restriction nothing in
this project's actual grammar asked for; removing it and adding `RETURN` as a normal statement
was strictly simpler *and* more correct, not a tradeoff.

### Two forks of the same grammar still diverge on field names

`geekstakulus/tree-sitter-oberon-07`'s `queries/highlights.scm` cannot be copied verbatim onto
`viegasfh/tree-sitter-oberon-2` even though both descend from the same EBNF-to-grammar shape and
share most rule names (`module_header`, `ident_def`, `qualident`, …). The 07 fork adds field
labels (`param:`, `paramtype:`, `returntype:`) and a `base_type` wrapper around builtin
qualidents that this grammar doesn't have. `tree-sitter query <file> <source>` will happily
report 0 matches instead of erroring when a query pattern's shape just never occurs — silent,
not loud. Always cross-check against `node-types.json` fields before trusting a ported query,
and smoke-test it on a real source file, not just the corpus.

## Round 5 — 2026-08-10

### A logged residual `ERROR` was misdiagnosed one round earlier

M1.2b's notes attributed `Printer.Mod`'s one residual `ERROR` to `SET` ("`ARRAY 8 OF SET`, SET
isn't a type yet"). It wasn't `SET` — isolating `used: ARRAY 8 OF SET;` alone showed it parses
clean; the error only appeared because that line sits inside a `RECORD` with a trailing `";"`
before `END`, the already-known `field_list_seq` gap logged back in round 3. A one-line note
written under time pressure ("this field triggers an error, and this field also happens to be
where the not-yet-supported construct lives") had fused two unrelated facts into one incorrect
causal claim. The general lesson: when a task brief attributes a real error to a specific
construct, isolate that construct alone (delete everything else around it) before trusting the
attribution — the M1.2a/b "confirm before coding" discipline applies to diagnosing existing
`ERROR`s, not just to deciding whether new grammar is needed.

### A rule can have two shapes for the same syntax, and only one gets fixed

`procedure_type` (`PROCEDURE [FormalPars]`) was already reachable from `struct_type`, so
`PROCEDURE(...)` worked fine as a `RECORD` field or a `TYPE` declaration's RHS. But
`fp_section` — a formal parameter's type — doesn't go through `type`/`struct_type` at all; it
goes through a separate, narrower `formal_type` rule (`{"ARRAY" "OF"} qualident`) that was never
widened when `procedure_type` was added. Same construct, two different grammar paths, only one
of which got the memo. Worth checking, when adding a new alternative to `Type`, whether every
production that's supposed to accept `Type` actually routes through the same rule — or has its
own narrower stand-in that also needs the addition.

### Both M1.2c grammar changes were structurally regression-proof

`formal_type` gained a `choice` alternative; `field_list_seq` gained a `repeat1` branch
alongside the existing one. Both are pure widenings — every string the old rule accepted, the
new rule still accepts, plus more. This means re-running the spot-check files didn't need a
byte-for-byte "before" `ERROR`-count comparison to rule out regressions on constructs the change
didn't touch: a `choice`/`repeat1` addition can only turn some previous `ERROR`s into successful
parses, never the reverse. Worth remembering as a fast regression argument for this class of
grammar change, in place of an expensive full-corpus diff.

## Round 6 — 2026-08-10

### `git stash` as a cheap oracle for "did my change actually help"

M1.3's real value is hard to see from `tree-sitter test` alone (corpus tests only prove the two
new cases work, not that the scanner improved anything on real files). `git stash` /
`git stash pop` around a `tree-sitter generate && tree-sitter parse <file>` gave an exact
before/after `ERROR`-region count on the same file with a two-command round trip — cheaper and
more reliable than trying to remember or reconstruct what the old regex-token comment rule did.
Worth reaching for whenever a change's effect is "fewer parse errors on real input" rather than
"this specific corpus case now passes" — the corpus test proves the mechanism, the stash-diff
proves the impact.

### A milestone-scoping question the plan doc had already answered

`NEXT.md` flagged "check whether the `(*$…*)` pragma is actually in scope for M1.3" as an open
question. `docs/plan.md`'s M1 milestone table had the answer on the same line as the scanner
task: "M1.3 External C scanner: nested comments + `(*$…*)` pragma node". The question was
answerable by re-reading the planning doc more carefully, not by making a judgment call — worth
checking the milestone table's own row text before treating an ambiguity flagged by a previous
round as still open.

### A previous round's causal attribution for a residual `ERROR` was wrong a second time

Round 5's insights already logged one misdiagnosed `ERROR` (attributed to `SET`, actually
`field_list_seq`). `NEXT.md` carried forward another one for this round: "`Printer.Mod`... a
nested comment... M1.3 scope". After M1.3, `Printer.Mod`'s `ERROR` count didn't move (5 before,
5 after) — and grepping the file directly for any `(*...(*` nesting pattern found none at all.
The construct named in the attribution never occurred in the file. Same lesson as round 5, worse
this time because the wrong attribution had already survived one full round unchallenged: an
`ERROR`-cause guess written under time pressure needs to be checked (isolate the construct, or
just grep for it) before it's repeated as fact in a later round's task brief.

### Depth-count simulation in a throwaway script beats staring at a giant diff

When `MultiArrays.Mod` still showed the entire file wrapped in one `(ERROR [0,0]-[747,0])` after
the scanner fix, the instinct was to suspect the new scanner. A ten-line Python script
re-implementing the same depth-counting algorithm over the actual file content (not the grammar,
just the raw bytes) proved the comments were perfectly balanced — 35 matched pairs, zero
unterminated. That ruled out the scanner in under a minute and pointed straight at a genuinely
separate cause (an unrelated `array_type` gap). Reproducing a suspect algorithm standalone
against the real input is a fast way to separate "my new code is broken" from "something else,
upstream or downstream, is broken" without adding printf/debug builds to the scanner itself.
