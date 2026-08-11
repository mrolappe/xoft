# Next task

**M1.4 continued — raise the three still-open scoping questions with the user, then pick
whichever root has fresh non-deferred work.** `stj` reached 0 failures in round 22 (full pass,
first root to get there). `amiga-oberon-31` (61 failures) and `oberon-a` (20 failures) both now
consist almost entirely of items already triaged in earlier rounds and either scoped out,
deferred pending a user decision, or individually low-value — there's very little "sample and
find a new cluster" work left in either without either (a) getting an answer on the open
questions below, or (b) picking off the handful of untriaged one-offs in `oberon-a` or `voc`'s 4
remaining files.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 707/792 (89.27%), up from 666/792 (84.09%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it. Per-root: `stj` 0
  (done), `amiga-oberon-31` 61, `oberon-a` 20, `voc` 4.
- **Three scoping questions are now open and blocking further progress**, none attempted, none
  scoped out — ask the user before touching any of them:
  1. **`Break.mod`/`NoGuru.mod`'s dual pragma-guarded `MODULE` headers** (round 20, user
     explicitly deferred — "skip for now, revisit later," not "scope out" or "implement now").
     2 files in `amiga-oberon-31`: `(* $IF X *) MODULE A; (* $ELSE *) MODULE B; (* $END *)`.
     `oberon-a` has the same pattern in `source/Library/OberonLib.mod` (confirmed round 21,
     not yet raised). Treat as one scoping question covering both roots' occurrences (3 files
     total), not two separate ones.
  2. **`\"` (backslash-quote) inside a string literal** (round 21, found but not attempted):
     `oberon-a`'s `ErrorMessages.mod`/`OBumpRevMsg.mod` (FlexCat-generated catalog files) need
     it read as an escaped quote; `voc`'s `ulmPrint.Mod`/`Printer.Mod` (currently *passing*)
     rely on `"\"` being a complete one-backslash string with **no** escape processing —
     standard Oberon-2 has no string-escape syntax at all, so neither reading is "more
     correct," and fixing one breaks the other. See `docs/insights.md` round 21's "Two corpus
     dialect idioms" entry for the full shape of this kind of question.
  3. **`STRUCT`/`UNTRACED POINTER`** (round 9, reaffirmed every round since) — still scoped
     **out** of M1 to Phase 2. This is `amiga-oberon-31`'s entire remaining cluster (≈59 of its
     61 files). Not a new question, just restating it's still the reason most of that root's
     count won't move without a Phase 2 kickoff — don't rediscover this as "new."
- **`oberon-a`'s remaining 20 failures, fully triaged, nothing new to find without picking one
  off individually**:
  - 8 non-Oberon stub files (never will parse, not a grammar gap): `source/AmigaUtil/Args.mod`,
    `BoopsiUtil.mod`, `HookUtil.mod`, `RexxUtil.mod`, `source/framework/GTEvents.mod`,
    `source/Library/BigSets.mod`, `StdIO.mod`, `source/ProjectOberon/Files.mod`. No decision
    yet on whether `sweep_corpus.py` should exclude non-source files from the denominator (not
    raised with the user, low priority, only 8/792 ≈ 1%).
  - 2 trailing-NUL-byte files: `source/FPE/HelloWorld.mod`, `source/Misc/Skeleton.mod`. Add
    `\x00` to `extras`' regex and the scanner's `is_space()` if picked up (same fix shape as
    round 20's NBSP precedent) — one-line, low value, 2 files.
  - `OberonLib.mod` — see scoping question 1 above.
  - `ErrorMessages.mod`/`OBumpRevMsg.mod` — see scoping question 2 above.
  - **7 not-yet-individually-triaged**: `examples/amok/IntuiPointer/IntuiPointer.mod`,
    `IntuiPointerDemo.mod`, `examples/Oberon0/AsciiTexts.Mod`, `source/amiga/Intuition.mod`,
    `Utility.mod`, `source/Kernel/Kernel.mod`, `source/Obsolete/GTEvents.mod`. (Round 21 also
    listed `source/OC/OCS.mod` here — that one was fixed as a cross-dialect side effect of
    round 22's `stj` work, confirmed via `diff` against the pre-round failure list, not
    assumed.) Worth a quick look if picked up.
- **`voc`'s 4 remaining failures** (unchanged since round 20, all deferred): two trailing-garbage
  files (free text/binary blob appended after `END Module.`) and one bare-real-literal lexer
  ambiguity (`ulmRandomGenerators.Mod`'s `1.` colliding with round 18's `2..4` range fix,
  needing external-scanner lookahead) — plus one more not yet individually named this round,
  re-check `voc/misc/MultiArrayRiders.Mod`, `MultiArrays.Mod`, `s3/ethUnicode.Mod` if picked up.
- **`tree-sitter generate`'s conflict-resolution suggestion names the exact colliding symbols
  — pair those, not their containing rules.** Round 22: `[$.external_proc_decl,
  $.procedure_heading]` did *not* resolve an unresolved-conflict error; `[$.external_proc_decl,
  $.kMinus]` (exactly what the generator's own error message suggested) did. See
  `docs/insights.md` round 22.
- **A dialect's own compiler manual (`*.doc`/`*.txt` in the corpus root) is worth reading before
  finalizing a fix based on corpus-only inference, not just as an afterthought** — `stj`'s
  `DOC/STJ-OBN.TXT` confirmed round 22's `PROCEDURE~`/`RETURN^`/assignment-expression fixes
  exactly, including a companion feature (`a := b := proc()` chaining) not yet seen bare in the
  sampled corpus. Check for a manual before inferring a dialect construct's shape purely from
  usage — this was already `NEXT.md` guidance from earlier rounds, round 22 is just the first
  time it actually had a hit.
- **After `tree-sitter test --update` succeeds on a *hand-written* (not corpus-copied) test
  source, read the generated tree before trusting it** — round 22's first `PROCEDURE~` test was
  accidentally ambiguous with an unrelated bodiless-heading rule and "0 errors" didn't catch it.
  Prefer copying a real corpus file's structural shape over a minimal invented one. See
  `docs/errors.md`/`docs/checklist.md` round 22.
- **A whole-file `ERROR` node wrapping otherwise-fully-valid children means the *outermost* rule
  failed to reduce** — look at the file's overall structure, not at "some construct deep inside
  is unsupported." Always run `tree-sitter parse <file> | grep -n "ERROR\|MISSING"` on the full
  tree. (Round 21 insight, still applies.)
- **`grep -r` over the raw corpus silently skips Latin-1 files unless given `-a`.** Default to
  `-a` on every fresh `grep -r`/`grep -rl` against these corpus roots.
- **`procedure_decls` has 4 alternatives** (`procedure_decl`, `forward_decl`,
  `definition_proc_decl`, `external_proc_decl`) at the *module* level. `procedure_body`'s nested
  declaration repeat is narrower: `procedure_decl`/`forward_decl` only, unchanged since round
  21. `conflicts` now has three entries: `[procedure_decl, definition_proc_decl]` (round 12),
  `[selector, actual_params]` (round 19, documentation-only per `tree-sitter generate`'s
  "unnecessary conflicts" warning — still expected, not a bug), `[external_proc_decl, kMinus]`
  (round 22, see above).

## How to find the next cluster (reproduction, same method as rounds 8–22)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
grep "^  oberon-a/" /tmp/sweep_v.txt | sed 's#^  oberon-a/##'
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod | grep -n "ERROR\|MISSING"   # find EVERY error node, not just the summary line
```

Corpus files are Latin-1 except `voc` (UTF-8); always transcode Latin-1 roots before feeding to
`tree-sitter parse`. `stj`'s extension is capitalized (`.MOD`/`.mod` mixed) — check both when
grepping (moot now that `stj` is at 0, but relevant if a regression shows up there later).

Cross-check any candidate against `docs/language-baseline.md` first, and check for a
`*.doc`/`*.txt` compiler manual in the corpus root before inferring a dialect construct's shape
purely from usage — `stj`'s `DOC/STJ-OBN.TXT` paid off directly in round 22. Only flag a scoping
question to the user when the construct is genuinely ambiguous or structural (the three above),
not a routine grammar addition.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  filled either via `tree-sitter test --update` (scoped with `-i "<test name>"` to avoid
  reformatting the whole file) and read back to confirm no `ERROR`/`MISSING` nodes, or
  hand-written by copying a structurally identical existing test's shape verbatim.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–22). After any fix, re-tally failures by root.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 22 — all six fixes in full (record-embedded procedure
  headings, trap-bound `PROCEDURE-` headings and the `kMinus`/`external_proc_decl` conflict
  fight, `PROCEDURE~`/`RETURN^` confirmed against the compiler manual, the assembler-comment
  scanner fix, CASE label widened to `ConstExpr`, assignment expressions), plus the cross-dialect
  bonus fix and the three still-open scoping questions listed above.
- `docs/insights.md` round 22 (three new entries) — the compiler-manual-as-primary-source
  lesson, the `tree-sitter generate` conflict-pairing lesson, and the ambiguous-hand-written-test
  lesson.
- `docs/errors.md`/`docs/checklist.md` round 22 — the two mistakes made this round (wrong
  conflict pairing on the first attempt, an accidentally-ambiguous hand-written test) and their
  mitigations.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 89.27%).

## State of the tree

- `grammar.js`: eight changes from round 21's description:
  - `field_list` gained a `$.procedure_heading` choice arm (STJ DEFINITION-module RECORD
    bodies).
  - `procedure_heading`'s mark slot widened from `optional($.kStar)` to
    `optional(choice($.kStar, $.kMinus, '~'))` (STJ `-`/`~` marks), and gained a trailing
    `optional($.trap_offset)`.
  - New `trap_offset = integer "," integer` rule.
  - `return_statement` gained a leading `optional('^')` before its existing
    `optional($.expression)`.
  - `label` replaced entirely: was `choice($.integer, $.string, $.qualident)`, now
    `$.const_expression`.
  - `assignment`'s RHS widened from `$.expression` to `choice($.assignment, $.expression)`
    (right-recursive chaining).
  - `factor` gained `seq('(', $.assignment, ')')` as a new alternative.
  - `conflicts` gained `[$.external_proc_decl, $.kMinus]` (third entry).
- `src/scanner.c`: one change — the `ASSEMBLER_BODY` raw-scan loop now skips `;`-to-newline
  Motorola-style comments before checking for the closing "END" word, so comment prose
  containing "END" doesn't falsely terminate the block. NBSP handling (round 20) unchanged.
- `sweep_corpus.py`: unchanged. Baseline for the next round: **89.27% (707/792)**, `stj` at
  **100%**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
