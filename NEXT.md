# Next task

**Continue M2: implement M2.3 `strip_comments`.** M1 is done (766/792, 96.72%, declared done
round 26). M2.1 (codec) and M2.2 (grammar linkage + token-walk serializer + byte-coverage
assertion) are done, TDD, all green — see `docs/progress/m2-core.md` for the full writeup. M2.4
(rule registry) is meant to stay empty shape-only in Phase 1 per `docs/plan.md`, so M2.3 is the
next real work.

## What's confirmed (do not re-derive, just verify before coding)

- `crates/xoft-core/src/codec.rs` — `Document::from_bytes`/`to_bytes`, byte↔char bijection (D3).
  Tested in `tests/codec.rs`.
- `crates/xoft-core/src/grammar.rs` + `build.rs` — links the vendored grammar
  (`grammars/tree-sitter-oberon2/gen-src/parser.c` + `src/scanner.c`, compiled via the `cc`
  crate directly into `xoft-core`) and exposes `grammar::language() -> tree_sitter::Language`.
  Tested in `tests/grammar.rs`.
- `crates/xoft-core/src/serialize.rs` — `walk(tree, text) -> Vec<Span>` (`Span::Leaf`/`Span::Gap`)
  + `reconstruct(&[Span]) -> String`. No fallible/`Result` API — the walk's cursor only advances,
  so full coverage is guaranteed by construction, not something that can fail at runtime; adding
  a `Result` type for an unreachable error would have been the exact "vacuous acceptance test"
  shape `docs/checklist.md` already warns about. Tested in `tests/serialize.rs`, 4 tests,
  including the actual M2/D8 end-to-end invariant (bytes → `Document` → parse → `walk` →
  `reconstruct` → `Document` → bytes, byte-identical, exercised with a high byte inside a
  comment).
- **Comments are real leaf nodes in the tree, not invisible extras** — confirmed empirically
  (`tree-sitter parse` on a snippet with a comment shows a `comment` node as a normal sibling).
  So `walk`'s "collect every zero-child node" already treats comments as leaves; gaps hold only
  actual whitespace, no comment-detection logic needed. See `docs/insights.md` round 26 before
  re-deriving this.
- Ad hoc, uncommitted 240-file corpus sample (60 files × 4 roots, via a throwaway
  `examples/roundtrip_smoke.rs` deleted after use — recreate the same way if useful, don't leave
  it committed) found `rt_ok = true` on every single file, including ones with M1-known `ERROR`
  nodes. `has_error = true` appeared on exactly the files M1 already scoped out (oberon-a's
  `AsciiTexts.Mod`/`Skeleton.mod`/`IntuiPointerDemo.mod`/`GTEvents.mod`, amiga-oberon-31's
  `Break.mod`/`NoGuru.mod`/`OberonLib.mod`) plus **3 voc files not yet cross-checked against the
  round-20 voc failure list** (`MultiArrays.Mod`, `MultiArrayRiders.Mod`, `ethUnicode.Mod`) —
  voc wasn't swept past its first 60 files this round, worth a quick check if voc comes up again.

## M2.3 — `strip_comments`

Per `docs/plan.md`: "pragma comments are kept — they are semantics. Output must re-parse."

- A comment is any leaf node of kind `comment` (confirmed node kind name via the M2.2 smoke
  test's tree dump — verify again with `tree-sitter parse` on a fresh snippet before coding, node
  kind names are cheap to re-check and expensive to get wrong).
- A **pragma** is the `(*$ ... *)`-style comment subtype from M1.3 — check `grammar.js` for
  whether it's a distinct node kind (e.g. `pragma`) or just a `comment` node with recognizable
  inner text; if it's already a distinct node kind, `strip_comments` should skip removing pragma
  nodes specifically, not text-sniff for `$`.
- "Output must re-parse" implies the function's contract: take a `Document`/text, produce a new
  text with ordinary comments removed (pragmas kept), and that new text must itself parse with
  zero `ERROR`/`MISSING` through `grammar::language()`. That's the shape of the first failing
  test — write it before any implementation, per `CLAUDE.md`'s test-first method.
- Removing a comment's bytes changes byte offsets for everything after it in that file, so this
  almost certainly cannot reuse `serialize::walk`'s `Span` list as-is without re-thinking what
  "gap" means once a leaf (the comment) is deleted rather than kept. Design this fresh rather than
  forcing M2.2's shape onto it — different problem (M2.2: reconstruct unchanged; M2.3: transform
  then re-verify-by-reparsing).

## Definition of done

- A failing-then-passing test for `strip_comments`, TDD per `CLAUDE.md`.
- Update `docs/progress/m2-core.md`'s M2.3 section (currently "not started").
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` if anything genuinely mistake-worthy came up (round 26 had none — API
  friction like a `u32`/`usize` mismatch on `Node::child` isn't a repeated-mistake pattern, don't
  manufacture an entry just to fill the ritual).

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize}.rs` + `build.rs`: new this round, all green.
- `grammar.js`/`src/scanner.c` (the tree-sitter grammar itself): unchanged since round 24, M1 is
  frozen unless a new corpus gap surfaces.
- `cargo test --workspace`: green, 20 tests across `xoft-core` (17) and `xoft-cli` (unchanged).
