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
