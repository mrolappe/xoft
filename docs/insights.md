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

## Round 7 — 2026-08-10

### `tree-sitter test`/`parse` hardcode `src/parser.c`; `generate -o` doesn't help them

`tree-sitter generate -o <dir>` redirects where `generate` *writes* its output, but `test` and
`parse` (checked `--help` on both — no `--src-dir` or similar) always read the compiled parser
from `<grammar-path>/src/parser.c`. Wanting generated files physically outside `src/` (so `src/`
can hold only hand-written source, like `scanner.c`) while keeping the standard `tree-sitter
test` workflow working means the two requirements can't both be satisfied by CLI flags alone.
Symlinks resolve it: `gen-src/` is the real `-o` target (gitignored), `src/` holds `scanner.c`
plus symlinks (`parser.c`, `grammar.json`, `node-types.json`, `tree_sitter/`) pointing into
`gen-src/`. The symlinks are themselves tiny, real, trackable files — set up once, not
regenerated per `tree-sitter generate` run, since they just point at paths whose contents change
underneath them.

## Round 8 — 2026-08-10

### Spot-checks (3-5 files) and a full-corpus sweep (792 files) can disagree by 80 points

M1.1 through M1.3's progress notes all read as "still one `ERROR` region" or "down from 46 to
28" on the same handful of files, which reads like M1 is nearly done. The first full-corpus
sweep this round put the real number at 15.78%. Depth on a few files (does *this* file's error
count go down) and breadth across the corpus (does *this fraction of files* parse clean) are
different measurements, and a milestone's exit criterion ("≥95% of corpus files") is a breadth
number — a round of spot-checks alone cannot tell you how close you are to it, only whether the
specific thing you just changed helped the specific files you tried it on. Build the breadth
measurement (even a throwaway one) before trusting a "looks nearly done" impression from depth
checks, especially right before a milestone's exit criterion is supposed to be evaluated.

### A task's stated cause can be wrong even when its stated construct is right

`NEXT.md` was correct that `INLINE` appears in the corpus and needed grammar attention, and
correct that its concrete syntax "has to come from the corpus itself." It was wrong about what
kind of thing `INLINE` is: `docs/language-baseline.md`'s dialect table called it "opaque token,
contents unparsed," implying block syntax needing a scanner or special token rule (the same
shape as the nested-comment/pragma work in M1.3). The corpus showed `SYSTEM.INLINE(...)` is an
ordinary procedure call — no new grammar surface at all, just a lexer bug (hex literals) in a
token it happens to use constantly. The instruction to "confirm the real syntax before writing a
rule to swallow it" (already in `NEXT.md`) was exactly the right guard and paid off immediately —
worth treating as the default move whenever a task brief characterizes an unconfirmed construct's
*shape* (block vs. call vs. token), not just its existence.

### A `token()` typo is invisible until you hunt for it — grep the keyword table when a common construct mass-fails

`kElseif: $ => 'ELSEIF'` compiled clean, generated clean, and every existing test stayed green,
because no test happened to exercise `ELSIF` (`docs/language-baseline.md`'s own spelling) at all
— `tree-sitter test`'s 35/35 gave zero signal that anything was wrong. It only surfaced because
the corpus sweep's failure list was dominated by a construct with no obvious single cause, which
prompted grepping `grammar.js` for how `ELSIF` is actually spelled there. A keyword table is a
good place to `grep` the source doc's reserved-word list against literally, once a mass-failure
pattern doesn't point at anything more specific — it's a cheap check that a "why is *everything*
failing" moment doesn't reach for by default.

### A plausible root cause can be exactly wrong, and the disproof is cheap enough to always run

`[0,0]`-to-EOF `ERROR` spans clustered suspiciously with `encoding: "high-bytes"` manifest
entries (Latin-1 `©` in banner comments), and tree-sitter's CLI does always read UTF-8 — a clean
mechanistic story. Adding transcoding to the sweep script and re-running was maybe five minutes
of work and it changed zero files from fail to pass: every one of those files has a real syntax
cause early on (this round's brace-annotation discovery, for one), and the Latin-1 byte was just
incidentally nearby in the same banner-comment header most of these interface files share. Same
lesson as round 5's and round 6's misattributed `ERROR` causes, generalized one step further:
even a mechanistically-sound theory needs the "does fixing it change the count" check before it's
trusted, not just theories that were guessed under time pressure.

### Post-round-8 correction: "AmigaOberon 3.1 is an Oberon-2 implementation" was never checked

`docs/language-baseline.md` asserted all three dialect roots (Oberon-A, AmigaOberon, STJ) are
"Oberon-2 implementations with additions" from the project's start (M0/M1.1), and every round
since inherited that framing without questioning it — including this one, until asked directly.
A quick corpus check (grep for type-bound-procedure receivers, record type extension, `WITH` —
all genuinely Oberon-2-only constructs, absent from the original 1988/1990 Oberon report)
shows Oberon-A and STJ use these constructs regularly (11/237 and 68/306 files for receivers
alone) while AmigaOberon barely does (1/122). Combined with this round's finds — `STRUCT`,
`{base,-N}` brace annotations, `IMPORT ident: M` colon-rename — none of which resemble Oberon-2
extensions, AmigaOberon looks much more like it's rooted in the original Oberon report with its
own Amiga/C-interop extensions layered on, not Oberon-2. **Confirmed against the primary
source**, not just inferred: the 1990 AmigaOberon manual itself
(<https://archive.org/details/amiga-oberon>, found after the user pointed at it) cites only
`[nw:or]: Niklaus Wirth, Revised Oberon Report` in its bibliography, never the Oberon-2 report —
and "Amiga Oberon 2.0" appearing throughout the text turned out to be the product's own version
number, not a language-spec reference (worth ruling out explicitly; it reads exactly like an
Oberon-2 citation out of context). Corrected in `docs/language-baseline.md` and
`corpus/roots.toml`'s origin string with the citation.

**Lesson:** a claim written once at project bootstrap (M0/M1.1, before there was a corpus to
check it against) can survive unchallenged through every later round simply because nothing in
the normal workflow re-examines foundational assumptions — only "does this file still parse"
gets checked repeatedly, not "is the premise about what this file even is still right." Worth
periodically asking whether a load-bearing claim in `docs/plan.md` or `docs/language-baseline.md`
has ever actually been checked against the corpus, versus just asserted early and repeated.

### Round 9: an error message is a primary source for a dialect's own grammar

Confirming the bracket-pragma sub-forms didn't need corpus archaeology at all — Oberon-A's own
`ErrorMessages.mod` contains the string `"Pragma must start with '<*$'"`, i.e. the compiler
itself documents that `$` is the canonical pragma marker and (by implication) bare `<* FLAG *>`
is a tolerated shorthand. When a dialect's original compiler source is sitting right there in the
corpus root, its diagnostic strings, error catalogs (`.ct`/`.cd` files in Oberon-A's case), and
comments are a higher-confidence source than reverse-engineering the grammar from usage samples
alone — worth a targeted grep before assuming corpus-sample-only triage is as good as it gets.

### Round 9: "confirm real syntax before coding" scales down to "confirm it's simple," not just "confirm it's real"

`ASSEMBLER` and brace annotations were both unconfirmed going into this round. Investigating both
before asking the user to scope anything (per `NEXT.md`'s explicit instruction) didn't just
avoid a wrong implementation — it changed the shape of the scoping question itself: brace
annotations turned out cheap enough that "should we do this" barely needed asking, while
`ASSEMBLER`'s confirmed raw-assembly-block shape immediately signalled "this needs scanner work,
size it separately." The investigation step produces the options the scoping question offers,
not just a yes/no on the item as originally described.

## Round 10 — 2026-08-10

### An external-scanner delimiter search needs an explicit "consumed at least one byte" guard

Scanning for a closing `"END"` as a whole word (not a substring — `"SEND"`/`"ENDIF"` must not
match) means checking that the byte before `E` isn't an identifier character. On the very first
loop iteration that check trivially passes (nothing precedes the start of the scan), so if the
body happens to be empty (`ASSEMBLER` immediately followed by `END`, not observed in this corpus
but not ruled out by the grammar either), the scanner would call `mark_end` at the very position
it started — a zero-length token. Tree-sitter external scanners that emit a zero-length token can
put the parser in an infinite loop (same token offered forever, position never advances). Fixed
by unconditionally consuming the first byte as body content before entering the boundary-checking
loop, so `mark_end` can never land at the scan's starting position. Worth treating as a standing
check for any new "raw-scan to a delimiter" external token, not just this one: does the empty-body
case (delimiter immediately at the start) produce a zero-length match, and if so, is that
prevented structurally rather than by assuming the corpus never triggers it.

## Round 11 — 2026-08-10

### A round's own memory of "the report" is not the report — check the doc file, not the last summary

Round 10's `docs/progress/m1-grammar.md` entry asserted "the report's `string` production ... has
no single-quote form at all." `docs/language-baseline.md` — the actual normative EBNF this
project keeps for exactly this purpose — has read `string = '"' {char} '"' | "'" {char} "'".`
since its first commit (checked with `git log -p`). Round 10 never opened that file for this
specific claim; it reasoned from a half-remembered shape of the classic Oberon report instead of
the copy sitting in the repo. The corpus grep it ran to "confirm" the claim was also mis-read: a
naive `'.'`-pairing regex is dominated by false positives from English contractions in comment
prose ("don't", "it's"), and the few genuine single-char matches it sampled (`ORD('4')` etc.)
were real but not representative — real *multi-character* single-quoted strings exist too
(AmigaOberon FourCC tags like `'KICK'`, format strings like `'%%%dld'`), just outnumbered in a
raw count by comment noise. Stripping `(* ... *)` comments and `"..."` strings out of the corpus
text *before* grepping for `'...'` is what surfaces them.

**Takeaway:** when a NEXT.md/progress-doc claim names a specific grammar production or file
content ("the report has no X"), re-derive it from the primary doc/file before coding against it
— a prior round's confident-sounding summary is a claim written under the same corpus-reading
constraints as this round, not a verified fact. And when a substring grep over free-form prose
(comments, string bodies) feeds a "how common is this" number, strip the prose-bearing regions
first — a `'quote'`-pairing count on raw text is mostly measuring apostrophes in English
sentences, not the construct being searched for.

## Round 12 — 2026-08-10

### Reusing a rule across two enclosing contexts can create a GLR ambiguity tree-sitter names for you

Splicing `definition_proc_decl` (previously only reachable inside `DEFINITION` modules) into
`procedure_decls` (reachable inside plain `MODULE`s and nested procedure bodies) made
`PROCEDURE ident ... ';'` ambiguous with the start of `procedure_decl` (`PROCEDURE ... ';'
procedure_body ident`) — both share the exact same prefix, and nothing after the `';'` tells the
parser with bounded lookahead which one it's in until it either finds a `procedure_body`'s
`kEnd`/`ident` closing sequence or runs out of things that could be one. `tree-sitter generate`
does not just fail on this — its error message names the two conflicting rules and prints the
exact `conflicts: $ => [[...]]` line to add. Trust that message over trying to manually restructure
the grammar to avoid the ambiguity; GLR resolving it at parse time (trying both, keeping the one
that completes) is the intended mechanism, not a workaround.

### A "files entirely matching pattern X" heuristic count is a floor, not the actual impact

`NEXT.md` estimated 125 corpus files as "entirely bodiless declarations" (contains `PROCEDURE`,
no `BEGIN`, no `STRUCT`) and flagged that this undercounts files mixing bodied/bodiless
procedures or pairing `STRUCT` elsewhere with an unrelated bodiless procedure. The actual
`sweep_corpus.py` delta was +45 passing files — real, but the heuristic gives no way to predict
the number in advance beyond "some more than files matching cleanly." When a NEXT.md count is
explicitly caveated as an undercount, don't round it up by guesswork either; just implement and
re-measure, per the task's existing "before/after" instruction.

## Round 13 — 2026-08-10

### Not every corpus-observed gap is a scoping question — check the baseline before asking

Rounds 9 and 11 both encountered constructs the grammar didn't handle and treated the "is this
in scope for M1" question carefully: round 9 flagged `STRUCT`/`ASSEMBLER` to the user because
they're AmigaOberon-specific dialect extensions, absent from `docs/language-baseline.md`
entirely. This round's `CASE ... ELSE ... END` looked superficially similar (an unhandled
construct found via corpus sampling) but turned out to already be in the *normative* Oberon-2
EBNF (`docs/language-baseline.md` line 94: `Case Statement = ... [ELSE StatementSeq] END`) — the
grammar was simply incomplete against its own stated baseline, not facing a dialect-scope
decision. The distinguishing check is cheap and should run before treating any new-looking
construct as a scoping question: grep the baseline doc for it first. If it's already normative,
just implement it; only escalate when the baseline doc doesn't have it at all.

### A structurally identical neighboring rule is a legitimate substitute for `--update`

Round 8's insight ("hand-writing an expected S-expression from rule names alone will get the
shape wrong, generate it instead") doesn't forbid hand-writing in general — it forbids
*guessing* the shape from `grammar.js` alone. This round's new `ELSE` arm in `case_statement`
produces the exact same `(kElse) (statement_seq ...)` shape that `if_statement`'s existing tests
already show a few hundred lines up in the same file, and the surrounding `case_clause` shape
was already in the immediately preceding test. Copying both verbatim and hand-assembling the new
test passed on the first `tree-sitter test` run — no `--update` round-trip needed. The rule is
"don't guess a shape you haven't seen," not "always regenerate."

## Round 14 — 2026-08-10

### The compiler's own manual beats corpus archaeology when one exists

Rounds up to this point derived every dialect construct's grammar from grep-ing the corpus and
inferring the shape from examples. `Oberon-A/docs/OC.doc` — the actual Oberon-A compiler's
reference manual, sitting right next to the corpus root in `roots.toml`'s `oberon-a` path — has
a `$`-prefixed formal EBNF for every dialect extension it defines (`ModuleHeading`,
`LibCallHeading`, `RegParameters`, `RegSpec`), plus worked examples (`OpenLibrary`,
`CoerceMethodA`) that became this round's corpus tests verbatim. Checking for a dialect's own
docs directory before reverse-engineering its grammar from source samples would have shortened
several earlier rounds (the round 9/12 brace-annotation and bracket-pragma work both had to
infer shapes purely from usage). Not every corpus root ships one — `amiga-oberon-31` and `stj`
don't have an equivalent — but `oberon-a` does, and it's worth a `find <root> -iname '*.doc'`
before falling back to corpus grep for future oberon-a-rooted gaps.

### `grep -r` silently skips Latin-1 corpus files unless given `-a`

A frequency-count `grep -rl` across the whole `oberon-a` root came back with implausibly low
counts (1 file where the real count was 77) — `grep` treats any file containing a byte sequence
it can't classify as text (the corpus's Latin-1 high-bit bytes, same encoding issue as round
6's insight, but hitting the *search* side this time rather than string-pairing) as binary and
skips it under `-r` unless `-a` (treat as text) is also passed. Round 6 already knew the corpus
is Latin-1 for `tree-sitter parse` transcoding purposes; this round is the reminder that the
same fact applies to any `grep -r` frequency count over the raw corpus, not just to feeding
files to the parser. Always add `-a` when grepping corpus roots directly (not through
`sweep_corpus.py`, which transcodes).

### Two dialects' delimiters for the "same" concept are worth two grammar rules, not one

AmigaOberon's curly-brace `{base,-54}` and Oberon-A's square-bracket `[base,-552]` vector-offset
annotations describe the same underlying concept (a library base variable plus a negative vector
offset) but are never mixed within a corpus root and have slightly different grammars (Oberon-A's
leading `-` is optional per its own EBNF, AmigaOberon's is mandatory). Modeled as two sibling
rules (`vector_offset`, `square_vector_offset`) rather than one rule accepting either delimiter —
collapsing them would blur a real dialect distinction for no parsing benefit, since the two never
compete for the same input.

### A narrow `(ERROR [n,0]-[n,end])` on a bare section keyword is a fixed-order/cardinality bug, not a new construct

Round 15's `oberon-a/source/amiga/*.mod` failures all had the same shape: a single-line ERROR
spanning exactly one keyword (`CONST`, `TYPE`) mid-file, not a whole-file or multi-hundred-line
span. That shape — the parser accepted everything before the keyword and everything the keyword
*would* introduce, it just wasn't expecting the keyword to reappear in that position — is the
signature of a repetition/ordering bug in an already-modeled rule, not an unhandled shape.
Worth checking `grammar.js`'s existing rule for a hardcoded `optional(...)`/fixed sequence before
assuming a new rule is needed, whenever a failure span is this narrow and lands exactly on a
keyword the grammar clearly already knows.

### The normative EBNF's outer braces are easy to miss when reading it as "one CONST, one TYPE, one VAR"

`DeclSeq = { CONST {ConstDecl ";"} | TYPE {TypeDecl ";"} | VAR {VarDecl ";"}} {ProcDecl ";" |
ForwardDecl ";"}` has *two* levels of repetition: the inner `{}` repeats declarations within one
section, the outer `{}` repeats whole sections, in any order, any number of times (including
zero). It's easy to transcribe this as "CONST section, then TYPE section, then VAR section, each
optional" (what `grammar.js` had) and miss that the outer brace makes the whole group repeatable
— the visual nesting doesn't make the two `{}` levels obvious at a glance. Worth re-reading a
baseline EBNF rule's brace nesting literally, not from memory of "how Oberon declaration order
usually looks", when a corpus sample defies grammar.js's current shape.

### "Cleared" clusters need a per-file re-check, not just a trust of the aggregate number

Round 15's fix was aimed at `oberon-a/source/amiga/*.mod`, and the round ended without
re-sampling that directory specifically — only the aggregate pass rate was re-run. Round 16
found 31 of that directory's ~121 files (`.mod` count varies) still failing, on a wholly
different construct. A fix that targets one cluster's *symptom* (a narrow ERROR span) can still
leave a second, unrelated construct blocking the same files further down — the aggregate number
going up doesn't mean a specific directory named in `NEXT.md` is actually clear. Worth spending
one `grep -A1` pass over the directory in question before assuming it's exhausted and moving on.

### An export mark and a "must-be-assignable" mark can look like the same token in two positions

Oberon's normative export mark is `ident "*"` (after the identifier). Oberon-A's "assignable
procedure" mark reuses the same `*` character but sits *before* the identifier, directly after
`PROCEDURE` (`PROCEDURE* [sysflag] Name`) — a different grammar position with a different meaning
(assignability to a procedure variable, not export), documented in `docs/OC.doc`'s
"AssignableProcs" node with the explicit rule "unless they are marked as exported" (i.e. the two
marks are alternatives in practice, never combined — confirmed by grep, zero files in the corpus
use both). Implemented as a second `optional($.kStar)` in `procedure_heading`, reusing the
existing `kStar` token rather than inventing a new one — same lexeme, different grammar slot, no
scanner or token changes needed.

### A `MISSING` node at an unrelated column can mean "operator the grammar has no rule for at all"

STJ-Oberon's `IF (byte >= 0) AND (byte < 20H) THEN ...` reported `(MISSING "*" [56, 18] -
[56, 18])` — a column that, read literally, pointed at whitespace between `0)` and `AND`, nowhere
near anything resembling a missing `*`. The real problem was that `AND` isn't a token the grammar
knows anywhere, so GLR error recovery picked some plausible-looking continuation (inserting a
virtual `*`) to keep going, and reported *that* location rather than the token it actually choked
on. Bisecting a minimal repro (deleting clauses one at a time until the error disappeared) found
the true cause faster than trusting the reported location. Worth remembering: an unexplained
`MISSING` node whose named token doesn't obviously belong at that column is a cue to check for an
entirely unhandled operator/keyword nearby, not to go looking for a subtle cardinality bug at the
literal reported position (round 15's narrow-`ERROR`-on-known-keyword signature is a *different*
case — that one does land where the real problem is).

### A dialect's own compiled binaries can double as a source of truth for its keyword list

Two `.OBJ` files in the `stj` corpus root (STJ-Oberon's own compiler binaries, `MAKE2PAR.OBJ`,
`OCSTAT.OBJ`) embed a plaintext keyword/error-message table readable via `grep -a` — e.g.
`... MOD NIL VAR CASE ... AND NOT ASSEMBLER FOR BY ...`. This confirmed `AND`/`NOT` are the
compiler's own reserved words, not an idiosyncrasy of one corpus author's coding style, without
needing a `.doc`/`.txt` manual (which `stj` doesn't have, per round 14/16's check). Worth grepping
`.OBJ`/binary files in a corpus root with `-a` when a dialect has no manual — compiled tool
binaries sometimes carry their own string tables as corroborating evidence.

### Lexical keyword synonyms for an existing operator don't need a scoping conversation

`STRUCT`/`ASSEMBLER` (round 9) needed a user decision because they're structural dialect
extensions — a new type kind, a new statement form requiring scanner work. `AND`/`NOT` as textual
alternatives to `&`/`~` are neither: same semantics, same grammar position, one new keyword token
each, squarely inside D1's "lexical superset" scope. Implemented directly, same as round 13's
`CASE...ELSE` (already-normative) — the distinguishing question is "does this need new structure
or scanner work", not "is this dialect-specific".

### tree-sitter has no lookahead/lookbehind — disambiguate ambiguous lexemes by tightening the grammar, not the regex

The real-number-vs-range bug (round 18: `2..4` mis-tokenized as `real` "2." + "." + "4") looks
like a textbook job for a negative lookahead (`2\.(?!\.)`) after the decimal point. tree-sitter
can't express that: it compiles token rules through Rust's `regex` crate, which excludes
lookaround by design (it's incompatible with the linear-time guarantee). The fix instead has to
change what the grammar *accepts* — here, requiring at least one digit after the `.` so `real`
stops being a candidate match for `2.` in the first place. Before reaching for this kind of
grammar-shape fix, check whether the stricter language is actually true to the corpus (grepped
all four roots for genuine bare-`N.`-reals first — found none, all matches were false positives
from identifiers containing digits) — tightening a token rule is only safe when nothing real
relies on the looser form.

### One new construct can be gated behind another — don't stop at the first fix that changes the error location

AmigaOberon's `Alerts.mod` kept failing at the exact same line after adding
`curly_external_code_names` (round 18) — the error span was identical before and after. Isolating
a minimal repro of *just* the fix confirmed it worked in isolation; the file still failed because
a second, unrelated construct (`data{9}..: SYSTEM.ADDRESS`, `param_offset` needing the same `..`
varargs marker `reg_spec` already had) sat immediately after it on the same procedure heading.
Two constructs stacked in the same span read like one fix "not working" when it's really two
fixes needed — bisect with a reduced repro of the fix alone before concluding it failed.

### Grepping a corpus root for a suffix/token pattern can accidentally match compiled binaries in the same directory

A plain `grep -rlas '[0-9A-F]+U\b'` over `amiga-oberon-31` (round 18, chasing the `U` hex suffix)
matched dozens of `.OBJ`/binary files whose garbled byte content coincidentally contains the
pattern — noise that looked like a much bigger cluster than it was. Adding `--include='*.mod'`
cut it down to the 7 genuine source-file hits. Worth remembering as the inverse of round 17's
"grep binaries on purpose for a keyword table" trick: when the corpus root mixes source and
compiled artifacts (common in these retro-Oberon roots), an unscoped grep can't tell the
difference and will over-count.

### A dialect's own EBNF documentation can still be wrong against real usage — grep the corpus even when the baseline "already covers it"

Round 19's scale-factor bug looked, at first glance, like the grammar was simply missing `D`
(only `E` was implemented) — an easy add-the-missing-choice-arm fix. But `docs/
language-baseline.md`'s own `ScaleFactor = ("E"|"D") ["+"|"-"] digit {digit}` requires a sign
and at least one exponent digit whenever a scale factor appears at all, and real corpus usage
violates *that* too: `9.22337177E18` (oberon-a) has no sign, and AmigaOberon's `D`
(LONGREAL-literal marker, `3.141592653589793D`) is used consistently bare, no sign or digits
at all. Trusting the baseline document's EBNF as sufficient — rather than re-grepping actual
usage even for a construct the baseline already names — would have produced a fix that still
left both patterns as `ERROR`. Same lesson as round 18's real/range fix (grep first), but this
time the wrong assumption was "the baseline documents it correctly" rather than "the grammar
implements the baseline correctly" — two different places the same trust can misfire.

### A genuinely ambiguous language construct (no lookahead/backtracking can fix it) may still have a workable syntax-only resolution — check where tree-sitter's default LALR choice lands before reaching for GLR

Oberon-2's `designator [ActualParameters]` vs. `selector`'s `"(" qualident ")"` type guard are
textually identical for a parenthesized single bare identifier — real compilers resolve this
via the symbol table (is the name a type?), which a syntax-only tree-sitter grammar doesn't
have. Round 19 initially assumed this needed GLR (`conflicts: [[$.selector,
$.actual_params]]`), but tree-sitter reported the declaration "unnecessary" both times tried
(before and after restructuring) — no automaton-level fork was ever being built, meaning
tree-sitter's default resolution was already deterministic, just consistently wrong for the
type-guard-then-selector case (`n(COMPLEX).Norm()` always parsed the `(COMPLEX)` as a call,
leaving `.Norm()` nowhere to attach → `ERROR`). The actual fix wasn't forcing GLR at all — it
was noticing that `actual_params` lived in the wrong *place* in the grammar (bolted onto
`factor`/`procedure_call` as a single trailing slot after designator, instead of inside
designator's own `repeat` alongside `selector`) so nothing could ever follow a call. Moving it
into the same repeat let guards/fields/calls interleave and chain freely, at the cost of
occasionally mislabeling which one a lone parenthesized identifier "really" is — acceptable
since M1's exit criterion is parse success, not semantic-correct node labeling. Cross-dialect
impact confirmed the fix was structural, not AmigaOberon-specific (`voc` -16, `oberon-a` -7,
`stj` -8, `amiga-oberon-31` -4) — the single largest one-fix gain of the project so far.
Lesson: when a construct looks like a textbook GLR case, try the `conflicts` declaration
*first* and trust "unnecessary" if tree-sitter says so — chasing GLR further when the generator
insists there's no fork wastes effort the grammar-shape question would have resolved faster.

### `extras` token-kind mixing can destabilize an unrelated, pre-existing GLR fork — not chased to a fix

Round 19 found (but didn't fix) a case where a bare NBSP (U+00A0) extras token followed by a
comment extras token, sitting between a procedure heading's `;` and its `BEGIN`, tips the
`procedure_decl`/`definition_proc_decl` GLR fork (round 12) toward the wrong (bodiless) branch
— even for a plain receiver-less procedure with no `*`/sysflag/anything else unusual. Isolated
via a minimal repro: NBSP alone before `BEGIN` parses fine; a comment alone before `BEGIN`
parses fine; the *combination* fails. This smells like a tree-sitter GLR-internal interaction
(more lexer states from the second extras-token kind altering which fork the parser commits to
first) rather than anything expressible as a grammar-shape fix. Left as a documented lead
(`Lists.mod`, `FArrays.mod`) rather than chased — worth minimizing further before the next
attempt (does *any* two-different-extras-kinds-back-to-back combination trigger it, not just
NBSP+comment specifically?).

**Round 20 correction: it was not a GLR-internal interaction, it was a concrete, findable
scanner bug.** The mechanism guessed above was never verified against the actual scanner
code — round 20 read `src/scanner.c` and found the real cause in five minutes: the external
scanner's own `is_space()` (used to skip leading whitespace before checking whether a comment
starts) didn't include NBSP, while `grammar.js`'s `extras` regex (added the same round 19) did.
Two independent definitions of "whitespace" existing in the same grammar — one in the
hand-written C scanner, one in the generated-grammar regex — will drift apart the moment either
one is edited without the other, and the drift only surfaces as a symptom several layers away
(here: a GLR fork that looks unrelated to whitespace at all). Lesson: when extending what counts
as an "extra" (whitespace/skippable content), grep for *every* place whitespace is defined
before declaring done — `grep -n "is_space\|extras" grammar.js src/scanner.c` would have caught
this in round 19 directly, instead of requiring a round-20 investigation that started from "GLR
looks broken" and worked backward. More generally: before attributing a parser bug to
tree-sitter's GLR machinery being mysterious, read the hand-written scanner code first — GLR
*looks* nondeterministic from the outside, but the actual nondeterminism triggers are almost
always a concrete, readable line of C.

### A corpus grep for a token pattern can be dominated by comment prose — sample matches by hand before trusting a prevalence count

Round 20 grepped the corpus for bare `N.` (a digit run followed by a literal `.` with no
trailing digit, e.g. `1.`) to gauge how common a newly-found lexer gap was, expecting a handful
of hits. It found ~150 across 59 files and treated that as a strong signal the fix was
high-value — but sampling the actual matches showed nearly all of them were inside comment
prose (sentences ending in a number: "June 1990.", "Defaults to 10.", "otherwise it returns
-1.") where the text never reaches the lexer's `real`/`integer` tokens at all (comments are
opaque external-scanner tokens). A regex over raw source text can't distinguish "inside a
comment" from "inside code" the way the grammar itself can — only one confirmed occurrence
(`ulmRandomGenerators.Mod`'s `1. - real`) was actually in live code. Lesson: a prevalence grep
against raw corpus text is a lead, not a measurement, whenever the pattern could plausibly
appear in prose — always sample several actual matches by hand (or better, check against
current parse-failure locations, which are guaranteed to be in live code) before sizing the
investment around the raw hit count.

### Two corpus facts can both be true and still be mutually exclusive for a lookahead-free lexer — that's a real wall, not a missing regex trick

Round 18 made `real`'s fractional digit mandatory specifically so `2..4` lexes as
`integer`+`range`+`integer` instead of greedily eating the first `.` into `real`. Round 20 found
a *different* file relying on the opposite: `1.` as a complete real literal with zero fractional
digits, in a context where no range operator ever follows. Both are genuine, confirmed corpus
usage; a single regex-based token cannot satisfy both, because tree-sitter's internal lexer
(Rust `regex` crate) has no lookahead to ask "is the character after this `.` another `.`, a
digit, or neither" before committing to how many characters the token consumes — confirming
(again, more concretely than the general note already in `NEXT.md`) that this class of
ambiguity needs the external scanner's genuine one-character lookahead (`lexer->lookahead` after
`advance()`, with the established "return false to roll back" escape hatch used for
comment/pragma detection), not a cleverer regex. Not yet implemented (see `NEXT.md`) — recorded
here so the next attempt doesn't re-discover "there's no regex fix for this" from scratch.

## Round 21 — 2026-08-11

### A whole-file `ERROR` wrapping otherwise-valid children is a different diagnostic signature than a localized `ERROR` deep in the tree — read it accordingly

Several round-21 failures showed `(ERROR [0,0] - [N,0] (comment ...) (module_header ...)
(import_list ...) (const_decls ...) ...)` — every child individually well-formed (correct node
types, correct nesting), just sitting as flat siblings directly under one `ERROR` at the very
top instead of wrapped in a `module` node. That shape means the *outermost* rule failed to
reduce even though every piece it's built from parsed fine — i.e., look at what's structurally
different about the very first or very last child (or, as in fix 3 this round, the exact
column where a child stops short), not at "some construct deep inside is unsupported." This is
the opposite diagnostic move from a localized `(ERROR [x,y]-[x,y+1])` nested several levels
down, which does mean a specific token/construct at that exact position is the problem. Telling
the two apart first (one `grep -n "ERROR\|MISSING"` on the full parse tree, not just the
summary line) saved real time this round — e.g. it immediately ruled out "the reg_spec/
square_vector_offset params are wrong" for `MathIEEESingBas.mod` (those parsed as valid
children under the flat `ERROR`) and pointed at "something about the file's outer structure"
instead.

### Finding a GLR combinatorial-blowup bug: bisect by repetition count, not by construct

`MathIEEESingBas.mod` (12 bodiless procedure headings) failed whole-file; a hand-guess at which
specific procedure heading's syntax was wrong would have been slow and wrong (it wasn't any
single heading's syntax — every one parsed fine alone). Instead: built a minimal synthetic file
with N copies of the *same* trivial bodyless-heading template followed by one real body-having
procedure, and swept N from 1 upward. It passed through N=7 and failed at N=8, exactly and
reproducibly. That number is itself the diagnostic: a real syntax gap fails at every N≥1 (or
never), while a threshold that only appears past a specific count is the signature of GLR
ambiguity whose live-branch count grows with repetition (here: how many ways N consecutive
anchor-less — no `END` — declarations could nest inside each other, a combinatorial/Catalan-like
count). Once the shape (count-triggered, not content-triggered) was confirmed, isolating *which*
grammar position allowed the anchor-less alternative to recurse (`procedure_body`'s nested
declaration repeat, reusing the full module-level `procedure_decls` including bodyless variants)
was a five-minute grep, not more guessing. General lesson: when a parser failure's boundary
tracks a *count* rather than a specific token, suspect GLR ambiguity that compounds with
repetition, and confirm by varying the count in isolation before touching the grammar.

### A grammar rule reused in two structurally different positions can be correct in one and wrong in the other — check baseline EBNF for each position separately, not just once

`procedure_decls` (all four bodyless/body-having variants) is valid at the *module* level per
the dialect extensions already implemented in earlier rounds. Round 12 reused it verbatim for
procedure_body's *nested* declarations too, reasoning "it's the same kind of declaration." But
`docs/language-baseline.md`'s own `DeclSeq` EBNF (line 73) already specifies nested declarations
as strictly `ProcDecl ";" | ForwardDecl ";"` — narrower than what's legal at the module level in
every dialect variant this grammar supports. The reuse wasn't wrong when it was written (it
didn't cause a visible failure until enough consecutive bodyless headings existed in one corpus
file to trigger the GLR blowup), but it was never actually licensed by the baseline for that
*position*. Lesson: when reusing a rule in a second position, re-check the baseline EBNF for
that specific position rather than assuming "legal at the outer level" transfers to "legal when
nested" — Oberon's own grammar deliberately allows less inside a procedure body than at module
scope.

### Two corpus dialect idioms can want opposite lexer behavior for the same byte sequence — recognize the `Break.mod`-shaped question early and stop, rather than pick a side

`\"` (backslash-quote) inside a double-quoted string appears in two corpus files wanting it
read as an *escaped quote* (FlexCat-generated catalog text, `ErrorMessages.mod`/
`OBumpRevMsg.mod`) and in two other, currently-*passing* files relying on it being a complete
*one-character string* with no escape processing at all (`ulmPrint.Mod`/`Printer.Mod`'s `"\"`
idiom — standard Oberon-2 has no string-escape syntax, so a bare `\` is just an ordinary
character and `"` always closes regardless of what precedes it). Grepping for the pattern across
*all four* corpus roots before writing any regex (not just the two failing files) surfaced the
conflict immediately — the same "check prevalence and direction before assuming a fix is safe"
discipline as round 20's comment-prose lesson, but here the payoff was catching a genuine
one-fix-breaks-another-file conflict rather than just an inflated hit count. This is
structurally the same category as round 20's `Break.mod` dual-header question (deferred, not
scoped out, not implemented) — a case where the corpus itself contains contradictory evidence
about the dialect's actual rules, and no amount of additional sampling resolves it without a
human call. Recognizing the shape early (two *already-confirmed*, *opposite* readings of the
same syntax) is faster than what round 20 needed to reach the same kind of stop, since it didn't
require a scoping conversation to notice — just grepping both directions before coding.

### A dialect's own compiler manual, when the corpus root has one, is a primary source worth checking before inferring semantics from usage alone

Round 22 found `stj`'s `PROCEDURE~` (assignable nested procedure) and `RETURN^` (non-local
return) constructs from corpus grep alone first, and had already correctly guessed their rough
shape and even their approximate semantics from context (both patterns only ever wrap a nested
procedure assigned to a callback variable). But `OBERON_I/DOC/STJ-OBN.TXT` — the compiler's own
manual, sitting right in the corpus root next to the source — spelled out both features exactly,
by name, with the author's own worked examples ("Assignment procedures", "Extended return from
procedures"), removing all doubt and confirming the guess was right before writing a line of
grammar. `NEXT.md`'s existing guidance ("check for a `*.doc`/`*.txt` compiler manual before
inferring a dialect construct's shape purely from usage") already said to do this — this round
is the first time it actually paid off with a hit, converting what would otherwise have been
"grammar addition based on corpus inference" into "grammar addition confirmed against a primary
source." Lesson: when a root's manual exists, read the relevant section *before* finalizing a
fix based on corpus-only inference, not just as a courtesy check afterward — it can also reveal
a *documented but not-yet-corpus-confirmed* companion feature (the manual's `a := b := proc()`
chained-assignment example, supported in the same fix even though the sampled corpus only
exercised the parenthesized-nested form).

### `tree-sitter generate`'s conflict-resolution suggestions name the exact colliding symbols — pair those, not their containing rules

Adding STJ's `PROCEDURE-` trap-bound heading (a bare `kMinus` token in `procedure_heading`'s
mark slot) collided with voc's pre-existing `external_proc_decl`, which spells its own leading
mark as an inline `'-'` literal rather than the named `$.kMinus` rule — same text, two different
grammar symbols, unresolvable by the parser without a `conflicts` entry. The first attempt
declared `[$.external_proc_decl, $.procedure_heading]` (the two *rules* whose expansions
actually diverge) and `tree-sitter generate` still reported the identical unresolved-conflict
error, unchanged. The generator's own error message had already named the precise pair —
`external_proc_decl`, `kMinus` — as its suggested resolution #4; using that literal pairing
instead of the higher-level rules resolved it immediately. Lesson: when `tree-sitter generate`
suggests "add a conflict for these rules: `X`, `Y`," pair exactly `X` and `Y`, not whatever
higher-level rule happens to contain them — the conflict lives at the specific symbols colliding
in the parse table, and a plausible-looking substitute one level up doesn't fix it even when it
sounds like the same idea.

### A synthetic hand-written test source can accidentally be grammar-ambiguous in a way the real corpus source never is — check the generated tree, not just "0 failures"

Testing the new `PROCEDURE~` nested-procedure mark, a first hand-written test source
(`PROCEDURE Outer; PROCEDURE~ Inner(...); BEGIN ... END Inner; BEGIN ... END Outer.`) parsed
successfully via `tree-sitter test --update` — but the generated tree showed `Outer` had been
parsed as a *bodiless* `definition_proc_decl` (reusing round 20's AmigaOberon precedent) with
`Inner` promoted to a *second, independent* module-level procedure, not nested inside `Outer` at
all. The source was genuinely ambiguous as written: ending the file with `END Outer.` (a single
`END` before the closing period) let GLR find an entirely different but still fully valid parse
of the same bytes, one that happened to not exercise nesting at all. The real corpus source
(`LIBRARY.PRJ/LINKEDLI.MOD`) never has this shape because it always has a receiver and a
distinct enclosing procedure `END name;` followed by further module content — accidentally
unambiguous by construction, not by any grammar guarantee. Lesson: after `tree-sitter test
--update` reports success on a hand-written (not corpus-copied) source, read the generated tree
before trusting it — "parses with 0 errors" is not the same as "parses the way I intended,"
especially for any construct that reuses an existing bodiless-heading alternative elsewhere in
the same rule.

### A stale scoping decision is worth re-sampling before restating it, not just re-affirming

Round 9 scoped `STRUCT`/`UNTRACED POINTER` out of M1 with the reasoning "a genuine second
record-like type, bigger than D1's lexical-superset scope" — and every round since (12 through
22) restated that call without re-checking it, per `NEXT.md`'s own "not a new question, just
restating" framing. When the user asked what implementing it would actually entail (round 23),
sampling the real corpus directly (`Module/Objects.mod`, `OberonLib.mod`, `DataTypes.mod`, ~15
occurrences read in full) showed the round-9 characterization was simply wrong: `STRUCT` is
structurally near-identical to `RECORD` (same field-list-seq, same optional-parens slot, same
`END` terminator), a same-tier addition to constructs already implemented (`CASE...ELSE`,
`POINTER TO ARRAY OF Type`). The *scoping call*, not just the *scoping question*, had gone
stale — a decision recorded as "confirmed, don't re-derive" in a handoff doc is still only as
good as the sampling that produced it, and a decision that's been carried forward unexamined for
13 rounds is a candidate for a fresh look, not just a citation. Lesson: when a scoping decision
is old enough that nobody has re-sampled the actual corpus since it was made, treat "user is
asking about it again" as a trigger to re-derive from source, not just to restate the prior
answer — the cost of one more `grep`/read pass is far lower than the cost of leaving real,
same-tier grammar work misfiled as "Phase 2" indefinitely.

### A near-identical dialect keyword can still have a structurally different shape — read the actual line before pattern-matching from its sibling

While implementing `UNTRACED POINTER TO Type`, `BPOINTER TO Type` turned up in the same
re-tally pass (`Interfaces/Dos.mod`). Because it *looks* like a sibling of `UNTRACED` (both are
AmigaOberon pointer-type keywords found in the same grep sweep), the first draft modeled it the
same way — an optional modifier keyword ahead of the mandatory `kPointer` — which would have
required `BPOINTER POINTER TO Type` to parse, a shape that appears nowhere in the corpus.
Re-reading the actual corpus line (`FileLockPtr* = BPOINTER TO FileLock;`) before running
`--update` showed `BPOINTER` fully replaces `POINTER`, it never co-occurs with it. Lesson: two
dialect keywords found together and superficially similar (same grep, same corpus root, same
"pointer" semantics) can still have unrelated grammar shapes — verify each one's actual token
sequence against real source individually, don't extrapolate the second from the first's
already-confirmed shape.

## Round 24 — 2026-08-11

### A single whole-file `ERROR` span can hide more than one independent, unrelated defect

Round 23 left 5 `amiga-oberon-31` files undiagnosed, found while re-tallying that round's
`STRUCT`/`UNTRACED`/`BPOINTER` fix. It would have been easy to assume all 5 shared one cause —
they surfaced together, in the same root, right after a round whose theme was pointer/struct
dialect extensions. They didn't: 4 failed because `module`'s declaration/procedure structure
couldn't repeat a `(CONST|TYPE|VAR)*` block after a `PROCEDURE*` block had already started; the
5th (`GarbageCollector.mod`) failed for *that* reason too, but even after fixing it, still had a
second, entirely unrelated defect (a fixed-length `ARRAY` used as a formal parameter type,
appearing exactly once in the whole corpus). A single `tree-sitter parse` run showing one
`ERROR [0,0]-[N,0]` span doesn't mean one root cause — GLR error recovery folds everything after
the first real failure into one node, so a file can still have a second bug hiding behind the
first. Lesson: after fixing what looks like the cause of a whole-file `ERROR`, re-parse before
declaring the file done — don't assume the fix was complete just because it explains why the
`ERROR` node started where it did.

### A one-shot "declarations, then procedures" grammar shape is a narrower reading of the EBNF than the corpus needs

The baseline EBNF's `DeclarationSequence` (and this grammar's `module` rule) has
`[CONST...] [TYPE...] [VAR...] {ProcedureDeclaration}` — read literally, one declarations block
followed by one procedures block. Round 15 already found the *inner* three sections need to
repeat (`CONST`/`TYPE`/`VAR` interleaved, not one of each) and fixed that. This round found the
*outer* structure needs the same treatment: the whole `(CONST|TYPE|VAR)*` group and the
`PROCEDURE*` group can themselves interleave, repeatedly. Both fixes were the same shape of
mistake — reading the EBNF's grouping as an upper bound on repetition instead of confirming
against the corpus — worth checking any other still-single-instance EBNF group (this grammar has
already been burned by this twice now) before assuming it's actually singular in the dialects
this project targets.

## Round 25 — 2026-08-11

### A byte value can be reserved by the tool's own protocol, not just by the grammar

Chasing two `oberon-a` files with a stray trailing NUL byte, the instinct was to treat it like
round 19/20's NBSP fix — a real byte that happens to look like whitespace, tolerable via
`extras` and the external scanner's `is_space()`. NUL is different in kind: tree-sitter uses
lookahead value `0` as its own internal EOF sentinel, in both the external-scanner API and the
generated internal lexer. Treating it as ordinary skippable content means EOF itself now looks
like "more whitespace to skip," and the skip-loop that calls `advance()` never terminates,
because advancing past EOF doesn't change the lookahead. This hung on every input, not just the
target files — even a one-line trivial module. The general lesson: before extending a
grammar/scanner to tolerate a raw byte value, check whether that value has a reserved meaning in
the *tool's* protocol (EOF sentinels, escape/control bytes used by the parsing engine itself),
not only whether it collides with another grammar rule. A byte that's safe to add to `\s`-style
tolerance in one lexer generator can be unsafe in another for reasons that have nothing to do
with the language being parsed.

### One dialect feature can wear two different surface syntaxes across files

`amiga-oberon-31`'s `Break.mod`/`NoGuru.mod` (round 20/23, dual `MODULE` headers) and
`oberon-a`'s `Kernel.mod`/`Utility.mod`/`IntuiPointerDemo.mod` (round 25) turned out to be the
same underlying feature — Amiga Oberon's conditional-compilation preprocessor — diagnosed
independently, three rounds apart, because the surface syntax differs: some files wrap the
directives in a plain `(* $IF x *)`/`(* $ELSE *)`/`(* $END *)` comment convention, others use the
bare bracket-pragma form `<*IF x THEN*>`/`<*ELSE*>`/`<*END*>` this grammar already tokenizes for
an unrelated purpose (compiler-hint pragmas). Recognizing "this is the same scoped-out item, not
a new one" required reading the actual bytes of both, not pattern-matching on which token forms
appeared. When two failures in different corpus roots produce a structurally similar symptom
(two full declarations/statements present unconditionally, no separator between them), check
whether they're the same feature in a different dialect's spelling before treating either as a
one-off.

## Round 26 — 2026-08-11

### Extras still produce real tree nodes — comments are leaves, not invisible

Before writing the M2.2 token-walk serializer, the assumption was that `extras` (whitespace,
comments) vanish from the tree entirely, so the walk would need explicit comment-detection logic
to keep them out of "uncovered gap" territory. Parsing `MODULE M; (* a comment *)\nEND M.\n` and
inspecting the raw tree showed a `comment` node as a normal sibling in `module`'s children,
alongside `module_header`/`module_footer` — extras are marked `is_extra` internally but still
appear as ordinary nodes with real byte spans. Consequence: a plain "collect every zero-child
node as a leaf" walk already includes comments as leaves for free; gaps end up holding only
actual whitespace, with no special-casing needed. Worth re-confirming empirically (one
`tree-sitter parse` call) rather than assuming either way before designing a tree-walk.

### Staying inside the codec's mapped-text domain sidesteps an offset-conversion class of bugs

D3's byte↔char codec means the string handed to tree-sitter is not byte-identical to the
original file for any byte >= 0x80 (one original byte becomes a 2-byte UTF-8 sequence in the
mapped text), so tree-sitter's `start_byte()`/`end_byte()` are offsets into the *mapped* text,
not the original bytes. The instinct was to build a byte-offset→original-byte-index table to
convert every node span back before reconstructing. Unnecessary: as long as every step of the
walk (slicing, concatenating) stays inside the mapped text's own `&str` domain and the codec's
`to_bytes()` is only invoked once, at the very end, on the fully-reconstructed mapped-text
`String`, no conversion table is ever needed — `Document`'s bijection is one char per original
byte by construction, so char-domain operations on the mapped text are automatically
original-byte-domain operations. Recognizing which domain a design actually needs to operate in
avoided writing an offset-mapping module that the checklist's "check what the laziest passing
implementation would look like" instinct would have flagged as unrequested complexity anyway.

### A missing statement separator becomes an `ERROR` node, not a `MISSING ";"` — and the CLI's default tree view hides real `MISSING` nodes

Two surprises from probing round 28's `Diagnostic` walk against the real parser before writing
tests. First: intuition says "missing `;`" should surface as a `MISSING ";"` node, the same shape
as a missing closing bracket. It doesn't — GLR error recovery instead lets the next statement's
value misparse as part of the current one, producing an `ERROR` node whose immediate parent is
`"assignment"`. A genuinely missing token (unbalanced `(`, no `)`) *does* produce a real
zero-width `MISSING ")"` node. Which shape you get depends on how the grammar's own recovery
happens to continue, not on any general rule — don't assume one broken-source category always
manifests the same way; probe each one. Second, and more procedurally important: `tree-sitter
parse`'s default S-expression output doesn't render `MISSING` nodes inline in the tree at all —
they only show up in the CLI's one-line trailing error summary (`(MISSING ")" [3, 13] - [3, 13])`
after the `Parse:` timing line). Reading only the pretty-printed tree would have concluded there
was no `MISSING` node in that case. `Node::is_missing()` via the Rust API sees it correctly
regardless — when checking whether a construct produces a `MISSING` node, verify through a small
Rust probe (`node.is_missing()`), not by eyeballing `tree-sitter parse`'s default tree dump.

## Round 30 — 2026-08-11

### Keep a rendering layer's structured output alongside its rendered text, not instead of it

`check.rs`'s `CheckResult` carries both `diagnostics: Vec<Diagnostic>` and `rendered: String`,
even though the CLI only ever prints `rendered`. The alternative — have `check_source` return just
the rendered string, since that's all `main.rs` needs — would have forced every test (and later,
`transpile.rs`, which needs `diagnostics.is_empty()` to decide its own exit code) to re-derive
facts by string-matching the rendered output (`rendered.contains("assignment")`, `rendered.len()
== 0` as a proxy for "no diagnostics"). That's fragile in the exact way this round's own `-->` vs.
`┌─` mistake demonstrates: rendered text is a presentation detail of whichever library does the
rendering, not a stable contract. Once a codespan-reporting version bump or config change alters
the glyphs, only the presentation assertions break, not the ones about what was actually found.
Worth remembering for M3.3 (snapshot tests against rendered text are exactly the right tool there)
and M6 (the Tauri bridge will want the structured list, not scraped text) — this is a case where
returning "both the data and its one current view of the data" from a rendering boundary is the
right shape, not redundant.

## Round 31 — 2026-08-11

### This grammar's `ERROR`-node recovery is not scoped per-mistake; `MISSING`-node insertion is

Trying to build M3.3's "two diagnostics in one file" fixture surfaced a real property of the
parser's error recovery, not just a fixture-design nuisance. A `MISSING` node (a single token GLR
can insert with zero width, e.g. a missing `)`) stays localized — the rest of the file parses
normally around it, and two independent `MISSING`-producing mistakes in two independent
constructs (two procedure headings, each missing a parameter type) really do produce two separate
diagnostics. An `ERROR` node (real tokens get skipped/re-synced around) does not: once triggered,
it swallows every subsequent token — including a second, unrelated procedure's entire body — up to
whatever the *outermost* still-valid resumption point is (usually the module's final `END`), even
when the source contains two obviously distinct mistakes a human would report separately. Five
source variants were tried (two broken assignments, two broken procedures with `RETURN`
statements, etc.) before landing on one that avoided `ERROR`-recovery entirely. Consequence for
future diagnostic work: "how many diagnostics does a broken file produce" is not predictable from
counting mistakes in the source — it depends on whether each mistake's local recovery stays a
`MISSING` insertion or escalates to an `ERROR` span, and one `ERROR` span can hide arbitrarily many
downstream real mistakes from ever being reported at all (relevant if M5 or later ever wants
"report every error in the file" as a UX goal — this grammar's recovery doesn't give that for
free).

### A whole-file `ERROR` span localizes fast by truncating lines, not by reading

M4.1's real corpus run left one file (`voc/ulm/ulmRandomGenerators.Mod`, 421 lines) reporting a
single `ERROR` node spanning line 1 to EOF — the least informative diagnostic shape, since it
gives no hint which of hundreds of lines actually caused it. Rather than reading the file
looking for something suspicious, bisected mechanically: `head -n N` the source, append a
throwaway `END X.` to keep the truncated prefix independently parseable, run `xoft check`,
binary-search `N`. Six `check` calls (50/100/150/200 → 142/144/146) narrowed 421 lines to a
14-line window, which then read as an obvious cause on inspection: `(1. - real - real)`, a
bare-decimal real literal with no digit after the `.`. This generalizes past this one file —
any single-`ERROR`-spanning-everything diagnostic is a bisection target, not a reading
assignment, since the span itself carries zero localization information once recovery gives up
on the whole file.

### A grammar comment's "no real-world corpus code relies on X" is a claim about corpus coverage *at the time it was written*, not a permanent fact

`grammar.js`'s `real` rule requires ≥1 digit after `.` specifically to avoid `..`-range
ambiguity, with a comment justifying the narrowing: "no real-world corpus code relies on [bare
`2.`]." That was true against however much of the corpus M1's rounds had sampled by the time it
was written. M4.1's full-corpus, round-trip-inclusive sweep (the first time every file in all
four roots was actually run, not just M1's incremental sampling passes) found the one
counter-example (`ulmRandomGenerators.Mod`'s `1.`) that claim didn't anticipate. Neither M1 nor
this round's narrowing decision was wrong given the evidence available at the time — the lesson
is that a scoping comment grounded in "the corpus doesn't do this" is only as strong as how much
of the corpus was actually checked, and a later, more exhaustive pass over the same corpus can
legitimately falsify it. Treat such comments as a recorded belief with a known evidence horizon,
worth re-checking (not just re-trusting) whenever a more complete sweep becomes possible —
which is exactly why M4.1 asked the user rather than silently allowlisting or silently
reopening M1 on its own judgment.

## Round 35 — 2026-08-18

### M5's own exit criterion ("what does a dialect experiment cost?") is being measured by M5.1 itself, not just its eventual total

Of M5.1's actual work, the two new grammar rules (`unless_statement`, the `kBegin`/`kDo` choice)
were each a few lines and correct on the first `tree-sitter generate` — no grammar-shape
iteration at all, unlike M1's 26 rounds of corpus-driven back-and-forth. Nearly all the round's
friction was fork *mechanics* that have nothing to do with the dialect's actual grammar: a
dangling-symlink collision from copying round 7's `gen-src`/`src` convention without its
generated contents, and a link failure from the generated parser and the hand-written scanner
disagreeing on a renamed symbol prefix. Both are now one-line checklist entries (`docs/checklist.
md`), so the *next* grammar fork (M6? Phase 2?) should be near-zero-friction — meaning M5.1's own
elapsed cost overstates the marginal cost of a second dialect experiment once the fork recipe is
known. Worth remembering when M5's exit criterion finally gets written up: separate "cost of the
grammar change itself" from "one-time cost of learning to fork this repo's grammar layout," since
only the former recurs per dialect.

## Round 36 — 2026-08-18

### A synonym is not invertible; an added construct is. That distinction *is* M5's cost measurement

M5.2's brief asserted `X→2→X` byte-identity "in both directions... achievable, not just
aspirational," with an argument that only actually covered Rule B. It does not hold for Rule A,
and the reason is structural rather than an emit-path shortcoming: `BEGIN` and `DO` are synonyms
in Oberon-X, so two Oberon-X spellings map onto one Oberon-2 spelling. The function is
many-to-one; the inverse does not exist. The tempting repair — make 2→X rewrite `BEGIN`→`DO` so
the mapping looks symmetric — does not recover anything, it just relocates the loss onto Oberon-X
sources that happen to spell it `BEGIN`. Two spellings cannot survive a trip through one.

Rule B has the opposite shape and round-trips exactly, for a reason worth naming: `UNLESS` is an
*added* construct, so the Oberon-2 form it lowers to (`IF ~(E) THEN …`) is a distinguishable
sub-language of Oberon-2, and the reverse rule can be written to match precisely that image and
nothing else. Every `IF` that Rule B did not produce is left alone — which is exactly what keeps
`2→X→2` byte-identical too, since lifting a random `IF` would not be reversible either.

So for M5's exit criterion ("what does one dialect experiment cost?"), the axis that predicts
lossless round-tripping is not how big the feature is but whether it is **additive** (new
construct, new syntax nothing else uses → bijective) or an **alias** (a second spelling of
something that already exists → lossy in the lowering direction, permanently). A dialect built
only from additive constructs round-trips byte-identically; a dialect with aliases has a
normalizing direction and always will. This is cheap to know up front and expensive to discover
in M5.3's golden files, so it belongs in the design conversation for the *next* dialect.

### "Inherited indentation" is a property you get by not having a layout engine, not one you build

`docs/plan.md` M5.2's phrase "template splicing with inherited indentation" reads like a feature
to implement — find the splice point's indentation, reuse it. It turned out to be the *absence*
of a feature. M2's serializer already partitions the source into leaves and gaps, and every gap
is whitespace plus comments by construction (D4). Generalizing `walk` into `walk_with`, which
offers only *leaves* to the caller's rewrite closure and passes gaps through untouched, makes
inherited indentation unfalsifiable: there is no code path that could compute a different
indentation, because there is no code that computes indentation at all. The whole mapping layer
is ~150 lines with no notion of a line, a column, or a nesting depth.

The generalization also cost nothing structurally — `walk` became a one-line delegation to
`walk_with`, so the gap-cursor logic (the fiddly part, with its `cursor.max(end)` overlap guard)
still exists in exactly one place and both callers are proven against it by M2's existing tests.
Worth remembering the shape: when a new consumer needs "the same traversal but with a hook,"
parameterizing the existing traversal beats a parallel one, and the tell is that the old function
survives as a trivial default argument.

## Round 37 — 2026-08-19

### One boolean, derived from an existing distinction, replaced a third golden file per case

M5.3's golden-file table needed to express four different expected outputs per fixture (`X→2`,
`2→X`, `2→X→2`, `X→2→X`) across two groups whose `2→X` and `X→2→X` answers differ (Rule B reaches
back to the `.x.mod` content; Rule A reaches the `.2.mod` content, since `BEGIN` is already valid
Oberon-X and Rule A never fires in the 2→X direction). The instinct was a third file per case
holding the "reached on the way back up" text. Unnecessary: that value is just the `.x.mod`
content for Rule B and the `.2.mod` content for Rule A, i.e. a function of the same `lossy`
distinction round 36 already named. One `lossy: bool` field on the `Case` struct, and all four
assertions derive their expected value from it — no new file, no new concept, just naming what
was already implied by "additive vs. alias" and computing from there.

### A file-driven suite promoting fully-implemented behavior has no red phase, and that's fine

M5.3's fixtures are byte-identical copies of M5.2's unit-test `const` strings; the code under
test (`to_oberon2`/`to_oberon_x`) shipped a round earlier. The integration test passed on its
first run — no failing state to see. TDD's red-green cycle governs *production* code; a test
suite whose entire purpose is promoting existing, already-red/green-verified behavior to a
different (file-driven, I/O-backed) consumer is not skipping TDD, it's outside its scope. Worth
naming so a future round doesn't manufacture an artificial red phase (e.g. temporarily breaking
`mapping.rs`) just to satisfy the letter of the discipline.

## Round 38 — 2026-08-23

### An IPC command is an injection-relevant sink even when it only ever "reads files"

`list_corpus` takes a `roots.toml`'s *content* from the webview and hands the paths inside it
straight to `WalkDir`. Nothing here executes a shell command or renders HTML, so it doesn't look
like the kind of thing CLAUDE.md's "name the output sink and injection class" rule is about — but
the sink is real: it's the local filesystem, and the "untrusted data" is whatever the webview
process sends over IPC, which Tauri's own threat model treats as a lower-trust boundary than the
Rust backend, precisely because a future frontend dependency or a loaded remote resource could
run script there that the backend never sees coming. `xoft_cli::manifest::build` doing this same
walk for the CLI is not a vulnerability — a user runs it against a `roots.toml` they wrote,
there's no boundary crossing. Wiring the *identical* function up to `#[tauri::command]` changes
who gets to supply the path, and that change is the thing worth reviewing, not the function
itself. General form: porting an existing, already-trusted function to a new caller can move it
across a trust boundary without changing a line of its own code — review the new caller's trust
level, not just whether the function's logic is already tested and correct.

### `cargo tauri init` hardcodes its output directory name

`-d <dir>` picks *where* to scaffold, but the scaffolded Rust crate is always created as
`<dir>/src-tauri` — there's no flag to name that subdirectory anything else. Getting the crate to
live at `crates/xoft-testbed/` (this repo's `crates/<name>/` convention, not Tauri's own
`<project>/src-tauri` convention) meant running `-d crates` and renaming `crates/src-tauri` →
`crates/xoft-testbed` immediately after, then fixing every place the scaffold had baked in the
old name (`Cargo.toml`'s `[package].name`/`[lib].name`, `main.rs`'s `app_lib::run()` call). Worth
knowing before the *next* `cargo tauri init` in this repo (M6.2's frontend, if it ever needs its
own scaffold step): the rename is mechanical but has to happen before anything else references
the generated names, not after.

## Round 39 — 2026-08-23

### A new IPC command that mirrors an existing command's parameter shape inherits that command's already-acknowledged trust-boundary gap, not just its convenience

M6.2's `read_corpus_file` was designed (per the approved plan) to take `roots_toml: &str` as a
parameter, deliberately "mirroring `list_corpus`'s existing pattern... same bundled string
reused for both calls." That mirroring was meant purely as a testability/consistency win, but it
silently reproduced M6.1's acknowledged, not-yet-fixed finding on the *new* command too: at the
`#[tauri::command]` boundary, both commands accepted whatever `roots_toml` text the IPC caller
supplied, so a compromised webview could still name arbitrary filesystem roots — the frontend's
switch to a build-time-bundled `corpus/roots.toml` (closing the finding for the *default* code
path) didn't change what the Rust command itself was willing to accept from any caller.
Caught by treating "does this new command's design closes the round's stated security decision"
as its own question during the security-review pass, not by assuming a pattern-matched design
was safe because the thing it was patterned on had already been discussed.

The actual fix separated two things that had been living in one signature: `commands::list_corpus`/
`read_corpus_file` (the plain, Tauri-free, *unit-tested* functions) correctly keep `roots_toml`
as a parameter — that's what lets them be tested against synthetic fixtures without touching the
real machine-local corpus. The `#[tauri::command]` wrappers in `lib.rs` — the actual IPC-facing
trust boundary — now read `corpus/roots.toml` from disk themselves and no longer expose
`roots_toml` as a caller-suppliable argument at all. Testability and trust-boundary enforcement
turned out to want the boundary drawn in two different places in the same file, not one.

**General form:** when a new IPC command's design is justified by "mirrors an existing command's
pattern," check whether the existing pattern has any open/acknowledged security caveat before
copying its parameter shape — matching the pattern can mean matching the caveat, and a plan
review that only checks "is this internally consistent with the existing code" won't surface
that on its own.
