# Errors and mitigations

Mistakes made while building, and what stopped them from recurring. Newest round last.

## Round 1 — 2026-08-10

### Workspace member listed before its manifest existed

`Cargo.toml` listed `crates/xoft-cli` as a member while only `crates/xoft-core` had been
written. Every `cargo` invocation then failed with `failed to load manifest for workspace
member`, including the ones meant to show the *core* tests failing — which briefly looked like
a broken core rather than a missing file.

**Mitigation:** write each member's `Cargo.toml` in the same step that adds it to
`workspace.members`, never before. If `cargo` reports a manifest error, fix that before reading
anything else in the output as a real result.

### Assumed the handoff document's corpus description

The handoff scoped Phase 1 against "real Standard Oberon sources". Measuring the actual files
on disk before planning showed a corpus that a Standard Oberon grammar cannot parse at all —
three encodings, CRLF, nested comments, `DEFINITION` modules, `INLINE` assembly. Had this been
taken on trust, M1 would have been declared complete against a grammar that fails on the
majority of real input.

**Mitigation:** measure the corpus before writing the grammar milestone, not after. The
`corpus manifest` command exists so the numbers stay checkable rather than remembered, and M1's
exit criterion is expressed as a percentage of the *real* corpus.

### Nearly shipped a vacuous acceptance test

`serialize(parse(s)) == s`, taken from the handoff, is satisfied by returning the input
unchanged — the test would have passed on day one and stayed green through every grammar bug.

**Mitigation:** decision D4 replaces it with a byte-coverage assertion (every byte belongs to
exactly one leaf or one trivia gap, zero `ERROR`/`MISSING`). When an acceptance criterion is
written down, check what the laziest passing implementation would be before adopting it.

## Round 4 — 2026-08-10

### Widening every `statement_seq` element to `optional` broke `tree-sitter generate`

Implementing the empty-statement fix literally as written in `NEXT.md` ("make each element of
the sequence `optional($.statement)`") produced `statement_seq: seq(optional($.statement),
repeat(seq(';', optional($.statement))))` — a rule that can match zero tokens, which
`tree-sitter generate` rejects with "The rule ... matches the empty string" rather than a
conflict or a silent wrong parse.

**Mitigation:** run `tree-sitter generate` immediately after any change that adds `optional()`
around every alternative/element of a rule, before writing tests against it — the failure mode
is a hard generate error, not a subtle parse bug, so it's cheap to catch immediately and
expensive to debug later if buried under other changes. See `docs/insights.md` round 4 for the
`choice`-of-two-non-empty-branches fix.

## Round 6 — 2026-08-10

### External scanner only got one chance per token, and leading whitespace burned it

First version of `scanner.c` checked `lexer->lookahead != '('` and returned `false` immediately
otherwise. This works for a comment that is the very first byte of the file, and fails for every
other comment in existence — confirmed by `tree-sitter test`, where even the pre-existing
two-comment corpus case (unchanged since M1.1) regressed to an `ERROR`, and by
`tree-sitter parse --debug`, which showed `lex_external` being called exactly once at the
position *before* the newline preceding a comment, declining (lookahead is `'\n'`, not `'('`),
and then `lex_internal` skipping that whitespace and committing to matching a *real* grammar
token (literal `(` used for expression grouping) instead of ever re-trying the external scanner.
Tree-sitter does not loop "skip one whitespace char, retry external scanner, repeat" — the
external scanner is consulted once per token boundary, before the internal DFA's own
whitespace-skipping runs.

**Mitigation:** an external scanner that has to coexist with a plain `/\s/` extra must skip its
own leading whitespace (`while (is_space(lexer->lookahead)) lexer->advance(lexer, true);`, the
`skip=true` argument marks the chars as not part of the token) before checking for the construct
it's actually looking for. Confirmed via `tree-sitter parse --debug`, not guessed — the debug
lex trace showing zero `consume character` lines for the failing `lex_external` call was the
proof that it never even reached the whitespace, not that the whitespace confused it.

## Round 8 — 2026-08-10

### Hand-wrote an expected S-expression from memory instead of generating it, got the shape wrong

Wrote the "Multi Digit Hex Literal" test's expected tree by hand (flat `(qualident (ident)
(ident))` for `S.INLINE`, and `actual_params` wrapping `expression` nodes directly) based on
guessing the shape from the rule names in `grammar.js`. `tree-sitter test` failed immediately —
the real shape is `(designator (qualident (ident)) (selector (ident)))` for a dotted qualified
name, and `actual_params` wraps its arguments in an `expression_list` node. Both were visible in
neighboring tests in the same file (`statements.txt` already had a `selector`/`expression_list`
example a few hundred lines up) but weren't checked before writing the new case by hand.

**Mitigation:** established practice in this repo (see round 4/5's insights) is to generate the
expected tree via `tree-sitter test --update` against real input and read it back, specifically
to avoid this. Reverted to that for the fix. When hand-writing is unavoidable, grep the same test
file for a structurally similar existing case (same rule combination) and copy its shape rather
than reconstructing it from `grammar.js` rule definitions alone — the generated node shape
depends on precedence/hiding choices in the grammar that aren't always obvious from the rule
text.

## Round 19 — 2026-08-11

### Declared a `conflicts` entry to fix an ambiguity, but the rules hadn't moved yet — wasted a cycle on a no-op

While diagnosing the `designator`/`actual_params` type-guard ambiguity (see `docs/insights.md`
round 19), the first attempt was `conflicts: $ => [[$.selector, $.actual_params]]` with the two
rules left in their *original* positions (`selector` inside `designator`'s `repeat`,
`actual_params` bolted onto `factor`/`procedure_call` afterward). `tree-sitter generate`
reported the declaration "unnecessary" and the minimal repro still failed identically —
because in that shape, the two rules are never actually offered as alternatives at the same
parser state; there was nothing for GLR to fork between. The declaration looked plausible
(both rules' first token is `(`) but conflict analysis operates on the automaton's actual
states, not on "these two things start with the same character."

**Mitigation:** when `tree-sitter generate` calls a declared conflict "unnecessary," believe it
immediately rather than re-testing the same declaration a second time hoping the warning was
stale — it means the automaton genuinely has no fork there, so the fix has to change what
states exist (here: moving `actual_params` into the same `repeat` as `selector`, so they're
truly siblings at one choice point), not add a bigger or differently-worded conflict list. This
was working correctly the first time it was tried (see the round-19 log above) — the mistake
was doubting the "unnecessary" warning enough to spend a second cycle confirming it rather than
moving straight to a grammar-shape change.

### Edit tool `old_string` silently failed to match text containing a literal NBSP character

Twice this round, an `Edit` call with `old_string` typed to *look* like the target line (e.g.
`extras: $ => [$.comment, $.pragma, $.bracket_pragma, /[\s ]/],`) failed with "String to
replace not found," even immediately after a successful edit had written that exact line. The
line actually contained a literal U+00A0 (non-breaking space) character inside the regex
(intentionally, to match the corpus's NBSP-as-whitespace bytes) — indistinguishable from a
plain space when read back visually, but a byte-exact mismatch against a hand-typed ASCII space
in the tool call.

**Mitigation:** when a byte a rule needs to match is a non-obvious/invisible Unicode character
(NBSP, zero-width chars, smart quotes), don't retype it by hand in a subsequent `Edit` call —
either use a `\uXXXX` escape in the replacement text (unambiguous, greppable) or drive the
substitution through a small Python/Bash script that references the character by codepoint,
as was eventually done here (`python3` one-liner replacing the exact old string read back from
the file). Confirm the result by reading the byte content back (`od -c` or `repr()` in Python),
not by eyeballing the file.

### Round 20: a hand-typed "matching" repro used a regular space where the bug needed an actual NBSP

While minimizing the NBSP+comment repro, several synthetic test files (`/tmp/x9.mod`,
`/tmp/x10.mod`, `/tmp/xC.mod`, …) were typed by hand via heredocs to match a failing file's
shape, including its trailing whitespace — but heredoc-typed whitespace is an ordinary ASCII
space, not the NBSP the real bug depended on. Every one of these hand-typed variants parsed
fine, which briefly looked like the failure required extra context (blank lines, comment
length, import blocks) beyond just "NBSP before comment" — a wrong lead chased through several
iterations before noticing (via `python3 -c "print(repr(...))"` on both files) that the two
"identical" files differed in exactly one invisible byte.

**Mitigation:** this is the same class of mistake `docs/errors.md`'s existing NBSP entry
already warns about (Edit `old_string` silently missing a literal NBSP), but on the *write*
side this time, not just the edit-match side: whenever a repro's minimality depends on a
specific non-ASCII byte, construct the repro file programmatically (Python string with an
explicit `\xa0`/` `) from the very first attempt, never by hand-typing a look-alike
character into a heredoc — and confirm with `repr()` before trusting a "doesn't reproduce"
result as signal rather than as a typo.

### Round 20: a corpus grep for "does a string appear before the first `;`" was truncated by inner `;`s in multi-line formal parameter lists

Checking whether every `PROCEDURE -ident` occurrence in `voc` had the expected trailing C
string before its heading's terminating `;` used a regex that grabbed text up to the *first*
`;` after the match. For single-line headings this is the real terminator; for multi-line
headings (formal parameters separated by `;`, e.g. `oocX11.Mod`'s `XCreateImage`) it stopped at
the first parameter separator instead, several hundred characters before the actual string —
producing 20 false "no string found" results that looked like exceptions to the pattern being
investigated, right before the fix was otherwise ready to write.

**Mitigation:** when scanning multi-line constructs for a trailing marker, don't assume the
first occurrence of the terminator character is the real one — either widen the search window
generously past what a single-line case would need (a few hundred extra characters cost
nothing) or match structurally (balance parens) rather than by first-occurrence of a character
that also appears inside the construct itself.

### Round 21: the already-documented "raw `grep -r` skips Latin-1 files" lesson was forgotten and had to be rediscovered mid-round

Checking underscore-identifier prevalence across the four corpus roots, the first `grep -rlE`
pass (no `-a`) reported only 1 matching file in `oberon-a` — implausibly low given the failing
file being investigated (`EAGUI.mod`) alone had 53 matches. The undercount was silently caused
by the exact issue already recorded in `NEXT.md`/`docs/insights.md` from round 18 (`grep -r`
over the raw corpus skips Latin-1 files unless given `-a`) — a lesson that was read at the start
of this round but not actively checked against before running a fresh grep, only noticed because
the result looked obviously wrong (one file, when direct inspection already showed 53 hits in a
single file) rather than from applying the rule proactively.

**Mitigation:** having a lesson recorded is not the same as applying it — when writing a *new*
corpus-wide `grep -r`/`grep -rl` command against these roots specifically, default to including
`-a` every time rather than adding it reactively after a suspiciously-low count; treat "surprise
zero or near-zero hit count for something known to be common" as itself a signal to re-check
the command against the known Latin-1 pitfall before trusting the number.
