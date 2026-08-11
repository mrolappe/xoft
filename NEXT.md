# Next task

**Sample `oberon-a`'s remaining 19 failures with a fresh triage pass** — same reproduction
method as rounds 8–24. This round's two dialect-neutral grammar widenings (module-level
decl/procedure interleaving, fixed-length array formal params) incidentally fixed one
`oberon-a` file (20 → 19) as a side effect, which means the round 21/22 category breakdown for
this root is now one file stale — re-derive it from the actual current failure list below
rather than trusting the old counts.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 766/792 (96.72%), up from 760/792 (95.96%) last
  round. Per-root: `stj` 0 (done since round 22), `amiga-oberon-31` **3** (down from 8 — this
  round's 5-file lead is fully resolved, see below), `oberon-a` 19, `voc` 4.
- **This round's work** (user chose "keep closing gaps" over declaring M1 done at the
  milestone-boundary check): diagnosed the 5 undiagnosed `amiga-oberon-31` files from round 23
  and found they were unrelated to that round's `STRUCT`/`UNTRACED`/`BPOINTER` theme — two
  separate, dialect-neutral grammar bugs:
  1. `module`'s declaration structure only allowed one `(CONST|TYPE|VAR)*` block followed by
     one `PROCEDURE*` block; this dialect interleaves them repeatedly
     (`TYPE...CONST...PROCEDURE PROCEDURE...TYPE...CONST...PROCEDURE...`). Fixed by merging
     into one `repeat(choice(const_decls, type_decls, variable_decls, procedure_decls))` — a
     strict superset of the old shape. Fixed 4 of the 5 files.
  2. `GarbageCollector.mod` (the 5th) also had an unrelated defect after fix 1:
     `formal_type`'s `{"ARRAY" "OF"}` shorthand had no room for a length, needed for one
     corpus-wide-unique fixed-length by-reference array parameter
     (`DuplicateOpenArray(VAR from,to: ARRAY 100000H OF SYSTEM.ADDRESS; ...)`). Widened to
     `optional($.length)`, reusing `array_type`'s existing `length` rule.
  Both fixes are dialect-neutral (not gated to `amiga-oberon-31`), which is why `oberon-a`
  picked up one incidental fix too. See `docs/progress/m1-grammar.md` round 24 and
  `docs/insights.md` round 24 (the "one `ERROR` span can hide two unrelated bugs" and "an EBNF
  group read as singular may still need `repeat()`" lessons) for the full diagnostic trail.
- **`amiga-oberon-31`'s remaining 3 failures are all the same, already-scoped-out item**:
  `Module/Break.mod`, `Module/NoGuru.mod` (also `oberon-a/source/Library/OberonLib.mod`, same
  construct) — dual pragma-guarded `MODULE` headers. User confirmed round 23: **stays scoped
  out of M1, Phase 2 item.** Don't re-raise; nothing left to investigate in this root for M1.
- **The `\"` (backslash-quote) string-escape scoping question is resolved but not
  implemented**: user confirmed round 23 it should be **dialect/root-specific**
  (`oberon-a`'s `ErrorMessages.mod`/`OBumpRevMsg.mod` need escaped-quote handling; `voc`'s
  `ulmPrint.Mod`/`Printer.Mod`, currently passing, need the opposite, no-escape reading).
  Needs a design decision (external scanner keyed on which corpus root/dialect is active,
  since `string_literal` is currently a single lexer-level choice with no such hook point)
  before coding — a reasonable alternative pick for a future round if `oberon-a`'s untriaged
  one-offs turn out to be low-value.

## `oberon-a`'s current 19 failures (2026-08-11, post-round-24 — re-triage before trusting old categories)

```
oberon-a/examples/Oberon0/AsciiTexts.Mod                    ERROR [114,15]-[114,17]  (single point)
oberon-a/examples/amok/IntuiPointer/IntuiPointerDemo.mod    ERROR [30,2]-[73,0]
oberon-a/source/AmigaUtil/Args.mod                          ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/AmigaUtil/BoopsiUtil.mod                    ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/AmigaUtil/HookUtil.mod                      ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/AmigaUtil/RexxUtil.mod                      ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/FPE/HelloWorld.mod                          ERROR [39,0]-[39,1]      (single point)
oberon-a/source/Kernel/Kernel.mod                           ERROR [49,0]-[1603,0]    (huge span)
oberon-a/source/Library/BigSets.mod                         ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/Library/OberonLib.mod                       ERROR [0,0]-[71,0]       (dual-header, scoped out)
oberon-a/source/Library/StdIO.mod                           ERROR [0,0]-[3,0]        (likely stub file)
oberon-a/source/Misc/Skeleton.mod                           ERROR [58,0]-[58,1]      (single point)
oberon-a/source/OBumpRev/OBumpRevMsg.mod                    ERROR [0,0]-[133,0]      (string-escape, scoped)
oberon-a/source/OEL/ErrorMessages.mod                       ERROR [0,0]-[567,0]      (string-escape, scoped)
oberon-a/source/Obsolete/GTEvents.mod                       ERROR [0,0]-[250,0]
oberon-a/source/ProjectOberon/Files.mod                     ERROR [0,0]-[2,0]        (likely stub file)
oberon-a/source/amiga/Intuition.mod                         ERROR [3266,0]-[4666,0]  (huge span)
oberon-a/source/amiga/Utility.mod                           ERROR [734,2]-[734,25]
oberon-a/source/framework/GTEvents.mod                      ERROR [0,0]-[3,0]        (likely stub file)
```

Round 21/22 previously triaged this root as: 8 non-Oberon "moved to..." stub files (the
`[0,0]-[3,0]`/`[0,0]-[2,0]` short spans above, 8 of them, still matches), 2 trailing-NUL-byte
files (not yet re-identified against the list above — one may now be gone, see this round's
work), `OberonLib.mod`/`ErrorMessages.mod`/`OBumpRevMsg.mod` (3 files, all scoped/deferred,
unchanged), and 7 not-yet-individually-triaged one-offs (also not yet re-confirmed — could now
be 6). **Don't trust these old sub-counts** — re-derive which of `AsciiTexts.Mod`,
`IntuiPointerDemo.mod`, `Kernel.mod`, `Skeleton.mod`, `HelloWorld.mod`, `Obsolete/GTEvents.mod`,
`amiga/Intuition.mod`, `amiga/Utility.mod`, `framework/GTEvents.mod` are trailing-NUL-byte
(check with `od -c <file> | tail`) vs genuinely untriaged before diagnosing further.

## How to find the next cluster (reproduction, same method as rounds 8–24)

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

**This round's lesson (see `docs/insights.md` round 24)**: a single top-level `ERROR` span can
hide more than one independent, unrelated defect — after a fix, re-parse before declaring a
file done, don't assume the `ERROR`'s start position explains the whole span. Also: don't
assume newly-visible failures share the theme of whatever the previous round just implemented
— verify by checking whether the same construct already parses correctly elsewhere in the same
file before diagnosing further.

## Definition of done (for whichever cluster gets picked)

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  via `tree-sitter test --update` (scoped with `-i "<test name>"`) and read back to confirm no
  `ERROR`/`MISSING` nodes — don't trust "0 errors" alone, especially for a hand-written (not
  corpus-copied) source.
- Before modeling a new construct on a sibling found in the same grep pass, read its own corpus
  line first — don't assume it shares the sibling's grammar shape (round 23's `BPOINTER`
  mistake).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md`.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 24 — the interleaving/formal-param fixes in full,
  including how the misdiagnosis (assuming round 23's theme) was ruled out.
- `docs/insights.md` round 24 — the "one ERROR span, multiple bugs" and "EBNF group read as
  singular may still need repeat()" lessons.
- `docs/plan.md` D8 and M1's exit criterion (`docs/plan.md` line ~96): "≥95% of corpus files
  parse with zero ERROR/MISSING; one `tree-sitter test` case per construct" — met since round
  23, still met (96.72%).

## State of the tree

- `grammar.js`: two changes from round 23's description:
  - `module`'s first `seq` branch: the separate `repeat(choice(const_decls, type_decls,
    variable_decls))` + `repeat($.procedure_decls)` merged into one
    `repeat(choice($.const_decls, $.type_decls, $.variable_decls, $.procedure_decls))`. The
    second (`DEFINITION` module) branch is unchanged — not touched, no corpus evidence found
    requiring it.
  - `formal_type`'s `repeat(seq($.kArray, $.kOf))` widened to `repeat(seq($.kArray,
    optional($.length), $.kOf))`.
  - No new keyword tokens, no scanner changes.
- `src/scanner.c`: unchanged this round.
- `test/corpus/declarations.txt`: +1 case ("AmigaOberon Decl Section After Procedure
  Declaration").
- `test/corpus/types.txt`: +1 case ("AmigaOberon Fixed-Length Array Formal Param").
- `sweep_corpus.py`: unchanged. Baseline for the next round: **96.72% (766/792)**, `stj` at
  **100%**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
