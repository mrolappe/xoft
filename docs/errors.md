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
