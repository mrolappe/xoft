# Next task

**M1's ≥95% exit criterion is now met (95.96%, 760/792). Ask the user whether to declare M1 done
and move to M2, or keep closing the remaining ~4% first** — this is a milestone-boundary
decision, not a routine "pick the next cluster" call, so don't just plow into M2 or into further
grammar rounds without checking. If the user wants to keep closing gaps, the un-diagnosed
5-file `amiga-oberon-31` cluster below is the best next lead (found this round, not yet looked
at); if not, M2 (`docs/plan.md`'s lossless parse/serialize core) is queued and unblocked.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 760/792 (95.96%), up from 707/792 (89.27%) last
  round. Per-root: `stj` 0 failures (done since round 22), `amiga-oberon-31` 8, `oberon-a` 20,
  `voc` 4.
- **This round's work** (user asked "what would tackling STRUCT/UNTRACED POINTER entail?" —
  sampling the real corpus showed round 9's "bigger than lexical-superset scope" call was
  stale, not still-correct, so it was implemented rather than re-deferred): AmigaOberon
  `STRUCT` (near-`RECORD` shape — same field-list-seq, same `END`, but its parenthesized slot
  is a C-struct-embedding *named field* `(ident: Type)`, not a bare base-type reference; also
  legal as an anonymous inline pointer target, so it joins `struct_type`'s choice, not just
  `type_decl`'s RHS), `UNTRACED POINTER TO Type` (new optional keyword ahead of `kPointer`), and
  `BPOINTER TO Type` (AmigaDOS's BCPL-relative pointer, found mid-round while re-tallying —
  replaces `POINTER` entirely, does *not* modify it like `UNTRACED` does; first attempt got this
  wrong, see `docs/errors.md`/`docs/checklist.md` round 23). +53 files, all in
  `amiga-oberon-31` except a few reused pieces elsewhere.
- **Two of the three round-20/21 scoping questions are resolved, one still open**:
  1. **Dual pragma-guarded `MODULE` headers** (`Break.mod`/`NoGuru.mod`/`OberonLib.mod`, 3
     files) — user confirmed round 23: **stays scoped out of M1, Phase 2 item.** Don't re-raise.
  2. **`\"` (backslash-quote) inside a string literal** — user confirmed round 23: make it
     **dialect/root-specific** rather than picking one reading (`oberon-a`'s
     `ErrorMessages.mod`/`OBumpRevMsg.mod` need escaped-quote handling; `voc`'s
     `ulmPrint.Mod`/`Printer.Mod`, currently passing, need the opposite, no-escape reading).
     **Not yet implemented** — needs a design decision (external scanner keyed on which corpus
     root/dialect is active, since `string_literal` is currently a single lexer-level choice
     with no such hook point) before coding. Affects 4 files total (2 currently failing, 2
     currently passing that must not regress).
  3. **`STRUCT`/`UNTRACED POINTER`** — done this round, no longer scoped out (see above). Remove
     any remaining references to it as a Phase 2 item if found stale elsewhere.
- **`amiga-oberon-31`'s 8 remaining failures**:
  - 3 are the still-deferred dual pragma-guarded `MODULE` headers (scoping question 1 above):
    `Module/Break.mod`, `Module/NoGuru.mod`, `Module/OberonLib.mod` (also occurs in `oberon-a`'s
    `source/Library/OberonLib.mod`, same construct, different file).
  - **5 are newly visible this round, found but not diagnosed**: `Interfaces/Commodities.mod`,
    `Interfaces/Rexx.mod`, `Interfaces/Utility.mod`, `Module/Concurrency.mod`,
    `Module/GarbageCollector.mod`. Each showed a whole-file `ERROR` span (`[0,0]-[N,0]` or
    similar) when spot-checked, which per `docs/errors.md`'s round-21 lesson means the
    *outermost* rule failed to reduce somewhere, not necessarily near where the span starts —
    re-run `tree-sitter parse` on each and search the full tree for the first real `ERROR`, don't
    assume it's related to `STRUCT`/`UNTRACED`/`BPOINTER` just because those were this round's
    theme (each `Interfaces/Dos.mod`-style near-miss already got fixed; these might be a
    different construct entirely).
- **`oberon-a`'s remaining 20 failures — unchanged since round 21/22, fully triaged**, see round
  21/22 entries in `docs/progress/m1-grammar.md` for the full breakdown (8 non-Oberon stub
  files, 2 trailing-NUL-byte files, `OberonLib.mod` per scoping question 1, `ErrorMessages.mod`/
  `OBumpRevMsg.mod` per scoping question 2, 7 not-yet-individually-triaged one-offs).
- **`voc`'s 4 remaining failures — unchanged since round 20, all deferred**: two
  trailing-garbage files, one bare-real-literal lexer ambiguity (`ulmRandomGenerators.Mod`),
  `ulmPrint.Mod`/`Printer.Mod`'s passing status is what scoping question 2 must not break.

## How to find the next cluster (reproduction, same method as rounds 8–23)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod | grep -n "ERROR\|MISSING"   # find EVERY error node, not just the summary line
```

Corpus files are Latin-1 except `voc` (UTF-8); always transcode Latin-1 roots before feeding to
`tree-sitter parse`.

## Definition of done (if the user picks "keep closing gaps")

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  via `tree-sitter test --update` (scoped with `-i "<test name>"`) and read back to confirm no
  `ERROR`/`MISSING` nodes — don't trust "0 errors" alone, especially for a hand-written (not
  corpus-copied) source.
- Before modeling a new dialect keyword on a sibling found in the same grep pass, read its own
  corpus line first — don't assume it shares the sibling's grammar shape (round 23's `BPOINTER`
  mistake).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md`.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 23 — `STRUCT`/`UNTRACED POINTER`/`BPOINTER` in full,
  including why round 9's original scoping call was wrong, not just old.
- `docs/insights.md` round 23 — the "re-sample a stale scoping decision instead of restating it"
  lesson and the "verify each sibling keyword's shape individually" lesson.
- `docs/errors.md`/`docs/checklist.md` round 23 — the `BPOINTER`-modeled-as-modifier mistake and
  its mitigation.
- `docs/plan.md` D8 and M1's exit criterion (`docs/plan.md` line ~96): "≥95% of corpus files
  parse with zero ERROR/MISSING; one `tree-sitter test` case per construct" — now met.

## State of the tree

- `grammar.js`: four changes from round 22's description:
  - `struct_type`'s choice gained a fifth arm, `$.amiga_struct_type`.
  - New `amiga_struct_type` rule: `kStruct ["(" field_list ")"] [field_list_seq] kEnd`.
  - `pointer_type` restructured from a single `seq` into a `choice` of two forms: the existing
    `POINTER`-headed form (now with an added `optional($.kUntraced)` prefix) and a new
    `kBPointer $.kTo $.type` form.
  - New keyword tokens `kStruct`, `kUntraced`, `kBPointer`.
- `src/scanner.c`: unchanged this round.
- `test/corpus/types.txt`: 6 new cases (`AmigaOberon Struct Type`, `AmigaOberon Empty Struct
  Type`, `AmigaOberon Struct Embedded Base Field`, `AmigaOberon Untraced Pointer`, `AmigaOberon
  Untraced Pointer To Anonymous Struct`, `AmigaOberon BCPL-Relative Pointer`), all copied from
  real corpus shapes, all read back after `--update` per the checklist rule.
- `sweep_corpus.py`: unchanged. Baseline for the next round: **95.96% (760/792)**, `stj` at
  **100%**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
