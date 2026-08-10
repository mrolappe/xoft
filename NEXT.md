# Next task

**M1.4 continued — implement `ASSEMBLER` blocks.** Scoping is settled (round 9): `STRUCT` is
explicitly out of M1 (Phase 2), bracket pragmas and brace annotations are done. `ASSEMBLER` is
now the largest unimplemented cluster with confirmed real syntax — this is a "go implement it"
task, not a scoping question, unless something in the corpus contradicts the round-9
characterization below.

## What's confirmed (do not re-derive, just verify against the two files below before coding)

`ASSEMBLER` appears only in STJ-Oberon (32 files, e.g. `LIBRARY.PRJ/HALT.MOD`,
`LIBRARY.PRJ/QSORT.MOD` — grep `ASSEMBLER` under
`/Users/mrolappe/atari-retro-dev/c-drv/OBERON_I` per `corpus/roots.toml`'s `stj` root). Shape,
confirmed by reading real files:

```
ASSEMBLER
  MOVEM.L  D0-A7,registers
END;
```

used as a **statement** inside a procedure body (not a whole-procedure-body replacement — it
appears mid-`BEGIN...END`, followed by `;` and more statements). Content is raw M68K assembly:
opcodes with size suffixes (`MOVE.L`, `EXTB.L`), addressing modes with parens and dots
(`(A0,D0.L)`), immediates (`#1`), register lists (`D0-A7`) — none of this tokenizes as Oberon.
This is **not** expressible as a grammar rule over Oberon tokens; it needs the same technique
class as the nested-comment/pragma external scanner (`src/scanner.c`): raw-scan from `ASSEMBLER`
to the matching `END`.

## Why this is a scanner task, not a grammar rule

`grammar.js`'s `statement` choice can add an `assembler_statement` alternative, but the *body*
between `ASSEMBLER` and `END` has to come from the external scanner as one opaque token (like
`$.comment`/`$.pragma`/`$.bracket_pragma` already do), because:
- `#` isn't a valid token anywhere else in this grammar.
- `.` after an identifier collides with `selector`'s `.` (`A0.L` would otherwise try to parse as
  a designator selector).
- Register ranges (`D0-A7`) use `-` in a way no existing rule expects.

Confirm before coding whether the raw-scan should stop at the literal string `END` or needs to
be smarter (e.g. could `END` appear inside operand text in any of the 32 files? grep first). The
existing comment/pragma scanner in `src/scanner.c` is the closest precedent — read its
`COMMENT`/`PRAGMA`/`BRACKET_PRAGMA` handling in full before adding a fourth external token, and
decide whether `ASSEMBLER...END` should track nesting (unlikely — Oberon `END` doesn't nest
inside asm text) the way `(*...*)` does.

## Definition of done

- `tree-sitter test` still green, plus a new corpus case using a real shape from one of the two
  files above (same "copy the real corpus shape into the test" practice every round since M1.4
  has used).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  section, same format as round 9's "M1.4 continued").
- Update the triage table in `docs/progress/m1-grammar.md` (round 9's version, end of file).
- No changes outside `grammars/tree-sitter-oberon2/`.

## After that: re-measure the two carried-over items

Once `ASSEMBLER` is done or explicitly deferred again, two older items are still just "carried
over, not re-measured" and have never been confirmed as real failure causes on their own:

- `POINTER TO ARRAY OF Type` (carried over from M1.3) — `array_type` needs a length-less
  `ARRAY OF` alternative like `formal_type` already has, *if* this is actually blocking files
  (check via `sweep_corpus.py -v` failure list, don't assume).
- Single-quoted strings (carried over from M1.2c) — still just "plausible," never confirmed.

Both are cheap to check (grep + look at a sweep failure) relative to `ASSEMBLER`'s scanner work,
so worth a fast pass either before or after `ASSEMBLER` depending on which the next session finds
faster to disprove/confirm.

## Context a fresh session needs

- `docs/progress/m1-grammar.md`'s "M1.4 continued" section (round 9) — full detail on bracket
  pragmas and brace annotations, what was confirmed about `ASSEMBLER`, and the exact
  corpus-impact numbers.
- `docs/insights.md` round 9 — a dialect's own compiler source (error strings, catalogs) can be
  a higher-confidence source than corpus-sample triage alone; confirming real syntax also tells
  you *how much work* an item is, which is what actually shapes a scoping question.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 27.15%).
- `src/scanner.c` — the existing `COMMENT`/`PRAGMA`/`BRACKET_PRAGMA` external-token pattern is
  the direct precedent for whatever `ASSEMBLER` needs.

## State of the tree

- `grammar.js`: M1.1 base through M1.4 continued (round 9) — `bracket_pragma` external token,
  `vector_offset`/`param_offset` rules spliced into `procedure_heading`/`fp_section`. No node
  kinds added for `STRUCT` or `ASSEMBLER` — neither is implemented yet.
- `src/scanner.c`: three external tokens (`COMMENT`, `PRAGMA`, `BRACKET_PRAGMA`). `ASSEMBLER`
  would be a fourth.
- `sweep_corpus.py`: unchanged this round, still the way to measure before/after.
- Rust workspace untouched since M0 — this task doesn't touch it.
