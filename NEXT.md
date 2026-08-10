# Next task

**M1.4 continued — implement AmigaOberon's bodiless procedure declaration
(`PROCEDURE Name *{base,-N}(params): RetType;` with no `BEGIN...END Name;` body).**

Round 11 closed out the last two carried-over items (`POINTER TO ARRAY OF Type`, single-quoted
strings). With the round 9/10 triage table now fully resolved, this round found the next cluster
by sampling `sweep_corpus.py`'s remaining failures directly (not from a stale note) — see
"How this was found" below for the reproduction steps a fresh session can repeat.

## What's confirmed (do not re-derive, just verify before coding)

- **Shape**: AmigaOberon's `Interfaces/*.mod` files (system-call wrapper modules, e.g. `Cia.mod`,
  `Potgo.mod`, `Console.mod`) declare procedures as headers only, terminated by `;` with **no**
  `BEGIN`/`END Name` body at all — not even an empty one. Real example (`Cia.mod`):
  ```oberon
  PROCEDURE AddICRVector *{base,- 6}(icrBit{0}    : SHORTINT;
                                     interrupt{9} : e.InterruptPtr):e.InterruptPtr;
  PROCEDURE RemICRVector *{base,-12}(icrBit{0}    : SHORTINT;
                                     interrupt{9} : e.InterruptPtr);
  ```
  Note the `{base,-N}` vector-offset annotation (already handled, round 9) commonly co-occurs but
  isn't the blocker — the missing body is.
- **Not the same as `forward_decl`**: `grammar.js`'s existing `forward_decl` rule
  (`"PROCEDURE" "^" ...`) requires a literal `^` marker. These declarations have no `^` — they're
  a *third* procedure-declaration shape, structurally identical to `definition_proc_decl`
  (`grammar.js` line 79, `procedure_heading ';'`, used today only inside `DEFINITION` modules) —
  but appearing inside a plain `MODULE ... END`.
- **Fix shape**: `procedure_decls` (`grammar.js` line ~134, currently
  `choice($.procedure_decl, $.forward_decl), ';'`) needs a third alternative for a bodiless
  heading — almost certainly reusable as `$.definition_proc_decl` itself (same rule, new call
  site) rather than a new node, since the shape is identical. Confirm there's no semantic reason
  the two need to stay visually distinct in the tree before reusing the node.
- **Size**: a heuristic scan (see reproduction below) found **125 corpus files** that contain
  `PROCEDURE` but no `BEGIN` at all and no `STRUCT` (STRUCT is a separate, already-deferred
  cluster — see below) — i.e. modules that are *entirely* bodiless declarations. That heuristic
  undercounts: files mixing bodied and bodiless procedures, or using `STRUCT` for another type
  while also having a bodiless procedure, aren't in that count. Re-measure with `sweep_corpus.py`
  before/after per usual practice, don't trust 125 as final.
- **Do not confuse with `STRUCT`**: `BootBlock.mod` also whole-file-errors, but its cause is
  `STRUCT` (already scoped out of M1 to Phase 2, round 9 decision) plus an unconfirmed `UNTRACED
  POINTER TO Type` modifier — a different file, different cause, don't fix it as part of this
  task. If `UNTRACED POINTER` turns out to be common on its own (not paired with `STRUCT`), that's
  a separate future candidate, not investigated this round.

## How this was found (reproduction, for a fresh session to trust this over re-deriving it)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Pick any short whole-file "(ERROR [0, 0] - [N, 0])" failure under an Interfaces/ path, e.g.
# amiga-oberon-31/Interfaces/Cia.mod, transcode it to UTF-8 (corpus is Latin-1) and inspect:
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/Interfaces/Cia.mod')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # shows procedure_heading nodes floating outside any
                                # procedure_decl/procedure_decls wrapper — the tell.
```

The 125-file estimate came from scanning `corpus/manifest.json` for files containing `PROCEDURE`
but neither `BEGIN` nor `STRUCT` — a cheap proxy for "this file's failure is the bodiless-header
shape, not something else." Re-derive if `manifest.json` or the corpus roots change.

## Definition of done

- `tree-sitter test` still green, plus one new corpus case using `Cia.mod`'s exact
  `PROCEDURE AddICRVector *{base,- 6}(...)...;` shape (or similarly minimal), filled via
  `tree-sitter test --update` and read back to confirm no `ERROR`/`MISSING` nodes.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as round 11's).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md`'s round 11 section — the corrected understanding of
  `string_literal` (it's a general alternate-delimiter string, not a `CHAR`-only literal) and the
  fully-resolved round 9/10 triage table, so this round doesn't re-open either.
- `docs/insights.md` round 11 — a reminder to check `docs/language-baseline.md` itself (not a
  prior round's summary of it) before asserting what "the report" does or doesn't allow, and to
  strip comments/strings before grepping free-form corpus text for a construct's frequency.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 30.68%).

## State of the tree

- `grammar.js`: `array_type`'s length list is now `optional(...)` (round 11); `string_literal` has
  three alternatives — double-quoted, single-quoted, and the `digit {hexdigit} 'X'` char-code
  form. `procedure_decls` still only accepts `procedure_decl` (heading + body) or `forward_decl`
  (`PROCEDURE ^ ...`) — the bodiless-no-caret shape above is not yet supported.
- `src/scanner.c`: unchanged this round — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`). This task doesn't need a fifth; it's a plain grammar
  widening like the two round-11 items.
- `sweep_corpus.py`: unchanged. Baseline for the next round: **30.68% (243/792)**.
- Rust workspace untouched since M0 — this task doesn't touch it.
