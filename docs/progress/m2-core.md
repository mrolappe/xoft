# M2 — Core: parse and serialize losslessly

## M2.1 — `codec.rs`: byte↔char bijection + `Document` ✅ (round 26, 2026-08-11)

`crates/xoft-core/src/codec.rs`. `Document::from_bytes` maps each input byte to the Unicode
codepoint of the same value (D3); `Document::to_bytes` maps back. ~20 lines. Tested in
`crates/xoft-core/tests/codec.rs`: round-trips all 256 byte values in one property-style test,
plus a typical-source-file case and a one-char-per-byte sanity check. Written before the
implementation (TDD) — first run confirmed red (`codec` module didn't exist), then green.

## Grammar linkage (new — not its own plan.md line item, but required before M2.2)

`crates/xoft-core/build.rs` compiles the vendored grammar's `gen-src/parser.c` and
`src/scanner.c` (via the `cc` crate) directly into `xoft-core`; `crates/xoft-core/src/grammar.rs`
exposes the result as a `tree_sitter::Language` via `tree_sitter_language::LanguageFn::from_raw`.
This is build-time C compilation, not runtime I/O, so it doesn't violate the core's no-I/O design
rule — the crate still takes text in and returns structured data out. Tested in
`crates/xoft-core/tests/grammar.rs`: parses a trivial module, asserts zero `ERROR` nodes.

Picked the direct-link approach (compile the grammar's C sources straight into `xoft-core`) over
a separate `-sys` crate, since `docs/plan.md`'s Layout section has no such crate and one grammar
today doesn't justify the extra indirection.

## M2.2 — Token-walk serializer + byte-coverage assertion (D4) ✅ (round 26, 2026-08-11)

`crates/xoft-core/src/serialize.rs`. `walk(tree, text) -> Vec<Span>` collects every zero-child
node (leaf) in source order and interleaves each with the gap of text before it (`Span::Leaf` /
`Span::Gap`); `reconstruct(&[Span]) -> String` concatenates them back. No `Result`/error type —
by construction the walk's cursor only advances, so gap+leaf spans always partition the input
exactly; a coverage *failure* isn't a runtime state this algorithm can reach, so a fallible
signature would have been unrequested complexity (see the checklist's "vacuous acceptance test"
lesson). Comments turned out to already be leaves (see `docs/insights.md` round 26), so gaps hold
only real whitespace — the test `every_gap_is_whitespace_only` gives this the teeth a bare
round-trip check wouldn't have, since a naive "just return the input" implementation can't pass a
type signature that requires deriving output from the tree, but a *lazy* implementation that
ignores the tree in its body could — this test would catch a leaf-collector that silently skips a
subtree, since real content would then leak into a gap.

Tested in `crates/xoft-core/tests/serialize.rs`, all written before the implementation:

- `reconstructs_the_source_exactly` — basic walk/reconstruct round trip.
- `every_gap_is_whitespace_only` — the real invariant, see above.
- `round_trips_through_the_codec_byte_identically` — the actual M2/D8 invariant end to end:
  original bytes → `Document` → parse → `walk` → `reconstruct` → `Document` → bytes, including a
  high byte (0xE9) inside a comment, which is where the codec actually earns its keep (ASCII-only
  test input wouldn't exercise D3 at all).
- `a_syntax_error_is_detected` — `tree.root_node().has_error()` sanity check (M3's diagnostics
  will build on this, not duplicated here).

**Ad hoc corpus smoke check** (not committed — a throwaway `examples/roundtrip_smoke.rs`, deleted
after use; full corpus automation is M4's job, not M2's): ran the byte round-trip + `has_error`
check against the first 60 files of each of the 4 corpus roots (240 files total, ~30% of the
792-file corpus). Every file's `rt_ok` was `true` — the serializer never once produced a
byte-mismatched reconstruction, on clean files or on files with grammar-level `ERROR` nodes alike
(tree-sitter's error-recovery nodes still have valid byte spans, so the walk covers them fine).
`has_error=true` showed up on exactly the files M1 already knows about and has scoped out:
`AsciiTexts.Mod`/`Skeleton.mod`/`IntuiPointerDemo.mod`/`GTEvents.mod` (oberon-a),
`Break.mod`/`NoGuru.mod`/`OberonLib.mod` (amiga-oberon-31), plus 3 previously-unsampled voc files
(`MultiArrays.Mod`, `MultiArrayRiders.Mod`, `ethUnicode.Mod` — not yet cross-checked against
NEXT.md's round-20 voc failure list, since voc wasn't swept past the first 60 files this round).
No new grammar gap found; M2's serializer is confirmed orthogonal to M1's parse coverage, as
expected.

## M2.3 — `strip_comments` ✅ (round 27, 2026-08-11)

`crates/xoft-core/src/strip_comments.rs`. `strip_comments(tree, text) -> String` reuses
`serialize::collect_leaves` (made `pub(crate)`) to walk the same leaves M2.2's `walk` does, but
drops any leaf whose `kind()` is `"comment"` instead of keeping it — `pragma` and
`bracket_pragma` are separate node kinds (confirmed via `codegraph_explore` against
`scanner.c`'s `ts_external_scanner_symbol_map` and `grammar.js`'s `externals`), so they pass
through unfiltered without any text-sniffing for `$`. A removed comment's bytes are replaced
with a single space rather than deleted outright — a comment with no other whitespace around it
(e.g. `THEN(*c*)y`) is its only token separator, and deleting it outright would fuse the two
neighboring tokens into one (`THENy`), breaking the "output must re-parse" contract from
`docs/plan.md`. No `Result`/error type, same reasoning as M2.2's `walk`: the leaf walk always
covers the full input by construction, so there's no runtime failure mode to signal.

Tested in `crates/xoft-core/tests/strip_comments.rs`, all written before the implementation:

- `removes_an_ordinary_comment` — basic deletion.
- `keeps_a_pragma_comment` / `keeps_a_bracket_pragma` — both pragma surface syntaxes survive.
- `output_still_parses_with_zero_errors` — the plan's actual contract.
- `does_not_merge_two_tokens_when_the_comment_was_their_only_separator` — the token-fusion edge
  case above; without the space substitution this test fails (`THENy` lexes as one identifier,
  `has_error()` on re-parse).

## M2.4 — Rule registry (empty in Phase 1) ✅ (round 29, 2026-08-11)

`crates/xoft-core/src/rule.rs`. `Rule` is a trait with one method,
`check(&self, tree: &Tree, text: &str) -> Vec<Diagnostic>`; `RuleRegistry` holds
`Vec<Box<dyn Rule>>`, `register` pushes one, `run` flat-maps `check` over all of them into a
single `Vec<Diagnostic>`. Empty by construction — Phase 1 registers nothing; M5 (Oberon-X) is
what actually populates it. Asked the user whether `check` needs `text` alongside `tree`
(`docs/plan.md`'s "query-driven traversal" line doesn't say); confirmed yes — a node only carries
byte spans, and a rule that wants to inspect or compare the actual source text (an identifier's
spelling, a literal's value) would otherwise need the caller to re-slice for it, plus adding the
parameter later would be a breaking change to every future rule.

"Query-driven traversal" (the plan's own phrase) isn't implemented yet — there's no real rule to
drive with a `tree_sitter::Query` until M5 defines one, so the trait shape doesn't presuppose it;
a rule is free to use `Query`/`QueryCursor` or a hand-rolled walk internally, `check`'s signature
doesn't care.

Tested in `crates/xoft-core/tests/rule_registry.rs`, written before the implementation (TDD):

- `empty_registry_runs_zero_rules` — the Phase-1 shape itself: nothing registered, `run` returns
  `[]`.
- `a_registered_rule_is_actually_run` — a trivial `AlwaysFlagsRoot` rule that unconditionally
  returns one `Diagnostic` proves the wiring (register → run → collect) actually invokes `check`
  and returns what it produced, not just that the types compile.

**Exit criterion (byte-identical round-trip on 100% of non-allowlisted corpus)**: not yet
measured at full-corpus scale — that requires M4's corpus runner (`xoft corpus run`), which
doesn't exist yet. The ad hoc sample above is a strong signal, not the exit measurement itself.
