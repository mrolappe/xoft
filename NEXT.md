# Next task

**M1.4 continued — pick `stj` (40 failures, last dedicated sampling pass round 17, oldest
untouched cluster) for a fresh sampling pass.** `amiga-oberon-31`'s 61 failures are mostly
`STRUCT`/`UNTRACED POINTER`, already scoped out to Phase 2 since round 9 — low expected value
until then, aside from the still-open `Break.mod`/`NoGuru.mod` deferral below. `oberon-a`'s
remaining ~15 failures (of 21) are one-off stubs, NUL-byte artifacts, or already-diagnosed
deferrals (see below) — not worth further sampling right now.

Round 21 ran `oberon-a`'s first-ever dedicated sampling pass, clearing 38 of its 59 failures
across four fixes — 79.29% → 84.09% (628 → 666/792), the biggest single-round jump since round
15.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 666/792 (84.09%), up from 628/792 (79.29%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings. This is
  `amiga-oberon-31`'s entire remaining cluster (≈59 of its 61 files) except the 1 open lead
  below.
- **`Break.mod`/`NoGuru.mod`'s dual pragma-guarded `MODULE` headers — scoping question raised,
  user explicitly deferred the decision, still open.** 2 files in `amiga-oberon-31`:
  `(* $IF X *) MODULE A; (* $ELSE *) MODULE B; (* $END *)`. Re-ask rather than assume a default
  if picked up (the user chose "skip for now, revisit later," not "scope out" or "implement
  now"). **`oberon-a` has the same pattern too now**: `source/Library/OberonLib.mod` —
  `(* $IF OberonA *) MODULE OberonLib; (* $ELSE *) MODULE OAOberonLib; (* $END *)` — confirmed
  round 21, not yet raised with the user. When either dialect's version comes up, treat them as
  one scoping question, not two.
- **Genuine dialect ambiguity found in round 21, not attempted — needs a human call, not a
  guess:** `ErrorMessages.mod`/`OBumpRevMsg.mod` (`oberon-a`, both FlexCat-generated catalog
  files) embed C-template text as adjacent string literals containing `\"`, apparently wanting
  it read as an escaped quote. But `voc`'s `ulmPrint.Mod`/`Printer.Mod` (currently *passing*)
  rely on `"\"` being a complete one-backslash string with **no** escape processing — standard
  Oberon-2 has no string-escape syntax at all, so neither reading is "more correct." Making
  `string_literal` escape-aware to fix the first pair would break the second. Confirmed by
  direct grep in both directions across all four roots before writing any code (5 files total:
  2 `oberon-a`, 1 `amiga-oberon-31`, 2 `voc`). Flag to the user before attempting either
  direction — same category as `Break.mod` above, not a routine grammar addition.
- **`oberon-a` has 8 files that are not Oberon source at all** — one-line/two-line stub text
  ("This module is obsolete and has been moved to the directory OBERON-A:Source/Obsolete.") in
  place of real content: `source/AmigaUtil/Args.mod`, `BoopsiUtil.mod`, `HookUtil.mod`,
  `RexxUtil.mod`, `source/framework/GTEvents.mod`, `source/Library/BigSets.mod`, `StdIO.mod`,
  `source/ProjectOberon/Files.mod`. These will never parse as Oberon and aren't a grammar gap —
  don't count them toward M1's exit criterion denominator without a decision on whether
  `sweep_corpus.py` should exclude non-source files (not raised with the user yet; low
  priority, only 8/792 ≈ 1%).
- **`oberon-a` has 2 files with a trailing NUL byte (`\x00`) after the final comment**:
  `source/FPE/HelloWorld.mod`, `source/Misc/Skeleton.mod`. Neither `extras`' regex (`/[\s
   ]/`) nor the scanner's `is_space()` treat `\x00` as whitespace, so it's a lone `ERROR`
  token after an otherwise-complete parse. Likely disk-block padding from the original Amiga
  floppy image, not meaningful content. A one-line fix (add `\x00` to both, following the NBSP
  precedent from round 20) if worth it for 2 files — not attempted, low value, flagged here in
  case a future round wants the quick win.
- **`oberon-a` has ~8 remaining failures not yet individually triaged** (found but not sampled
  this round): `examples/amok/IntuiPointer/IntuiPointer.mod`, `IntuiPointerDemo.mod`,
  `examples/Oberon0/AsciiTexts.Mod`, `source/amiga/Intuition.mod`, `Utility.mod`,
  `source/Kernel/Kernel.mod`, `source/Obsolete/GTEvents.mod`, `source/OC/OCS.mod`. Each may be
  a one-off or may reveal another small cluster — worth a quick look if `oberon-a` is picked up
  again before `stj`, but round 20/21's precedent (pick the oldest fully-untouched root first)
  points at `stj` instead.
- **A whole-file `ERROR` node wrapping otherwise-fully-valid children (correct node types,
  correct nesting, just flattened as siblings instead of wrapped in `module`) means the
  *outermost* rule failed to reduce — look at the file's overall structure (first/last
  children, or the exact column a child stops short), not at "some construct deep inside is
  unsupported."** Different diagnostic signature than a localized `ERROR` several levels down
  in the tree, which does mean a specific token/position is the problem. Always run
  `tree-sitter parse <file> | grep -n "ERROR\|MISSING"` on the *full* tree, not just the one-line
  summary, to tell the two apart before investigating. (Round 21 insight — see
  `docs/insights.md` round 21 for the two bugs this caught: the nested-bodiless-decl GLR
  blowup, and the underscore-identifier gap.)
- **When a parser failure's boundary tracks a repetition *count* rather than a specific
  construct's content, suspect GLR ambiguity that compounds with repetition — confirm by
  building a synthetic file with N copies of the suspected construct and sweeping N, not by
  guessing which specific instance is "wrong."** Round 21's `MathIEEESingBas.mod` (12 bodyless
  procedure headings) failed whole-file; every individual heading parsed fine in isolation up to
  7 repetitions and failed reproducibly at 8. The trigger was `procedure_body`'s nested
  declaration repeat reusing the full module-level `procedure_decls` (including bodyless
  variants with no `END` to anchor GLR's nesting search) — narrowed to
  `repeat(choice(seq($.procedure_decl, ';'), seq($.forward_decl, ';')))`, matching
  `docs/language-baseline.md`'s own `DeclSeq` EBNF (line 73), which never licensed bodyless
  declarations to nest in the first place. **General pattern to watch for elsewhere in this
  grammar**: any other position where a rule reused from the module level includes an
  anchor-less (no closing/terminal token) alternative, nested inside a `repeat`, is a candidate
  for the same class of blowup.
- **When a grammar rule is reused in a second position (e.g. nested vs. top-level), re-check
  the baseline EBNF for that specific position** — "legal at module scope" doesn't imply "legal
  when nested," and Oberon's own grammar deliberately allows less inside a procedure body.
  (Round 21 — see above.)
- **A cross-dialect grammar bug can hide behind one root's cluster** (rounds 18–20, confirmed
  three times) — after any fix, always re-tally failures by root before assuming the delta is
  isolated to the root you were sampling. **Round 21 is the first round where this did NOT
  happen** — all four fixes stayed within `oberon-a`, `amiga-oberon-31`/`stj`/`voc` counts were
  unchanged before and after every fix. Still re-tally by root after every fix regardless; round
  21 confirms it's a check, not an assumption either way.
- **A raw corpus-text grep for a lexical pattern can be dominated by comment prose** (round 20)
  — sample matches by hand before trusting a hit count as a measure of in-code prevalence.
- **`grep -r` over the raw corpus silently skips Latin-1 files unless given `-a`.** Confirmed
  *again* in round 21 (see `docs/errors.md` round 21 — the lesson was already documented from
  round 18 and still had to be rediscovered mid-round from a suspiciously-low hit count).
  Default to `-a` on every fresh `grep -r`/`grep -rl` against these corpus roots, don't wait for
  a wrong-looking number to remind you.
- **tree-sitter has no lookahead/lookbehind** in its internal regex-based lexer (Rust `regex`
  crate excludes it by design) — the external scanner is the only place genuine lookahead is
  available (`lexer->lookahead` after `advance()`, "return false" as the rollback escape hatch).
  Relevant again if `voc`'s bare-real-literal lexer gap (`ulmRandomGenerators.Mod`, still open
  from round 20) is picked up.
- **`procedure_decls` has 4 alternatives** (`procedure_decl`, `forward_decl`,
  `definition_proc_decl`, `external_proc_decl`) at the *module* level — unchanged this round.
  **`procedure_body`'s nested declaration repeat is narrower** (round 21): only
  `procedure_decl`/`forward_decl`, no bodyless variants. `conflicts` still has two entries:
  `[procedure_decl, definition_proc_decl]` (round 12, now fires only at the module level, no
  longer compounds) and `[selector, actual_params]` (round 19, documentation-only).

## How to find the next cluster (reproduction, same method as rounds 8–21)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
grep "^  stj/" /tmp/sweep_v.txt | sed 's#^  stj/##'
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod | grep -n "ERROR\|MISSING"   # find EVERY error node, not just the summary line
```

Corpus files are Latin-1 except `voc` (UTF-8); always transcode Latin-1 roots before feeding to
`tree-sitter parse`. `stj`'s extension is capitalized (`.MOD`/`.mod` mixed) — check both when
grepping.

Cross-check any candidate against `docs/language-baseline.md` first, and check for a
`*.doc`/`*.txt` compiler manual before inferring a dialect construct's shape purely from usage
(`stj` doesn't have one confirmed yet — check `OBERON_I`'s root for a manual before assuming).
Only flag a scoping question to the user (round 9's `STRUCT`, round 20's `Break.mod`, round 21's
`\"` string-escape conflict) when the construct is genuinely ambiguous or structural, not a
routine grammar addition.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  filled either via `tree-sitter test --update` (scoped with `-i "<test name>"` to avoid
  reformatting the whole file) and read back to confirm no `ERROR`/`MISSING` nodes, or
  hand-written by copying a structurally identical existing test's shape verbatim.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–21). After any fix, re-tally failures by root.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 21 — all four fixes in full (adjacent string
  concatenation, the nested-bodiless-decl GLR blowup root-cause, the underscore-identifier gap,
  module-level external names), plus the `\"` string-escape ambiguity and the `OberonLib.mod`
  dual-header finding.
- `docs/insights.md` round 21 (four new entries) — the flat-vs-nested `ERROR` diagnostic
  signature, the bisect-by-repetition-count technique, the reuse-position EBNF-recheck lesson,
  and the two-conflicting-idioms pattern-recognition lesson.
- `docs/errors.md` round 21 — the Latin-1-grep lesson recurring despite being already
  documented (apply proactively, not reactively).
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 84.09%).

## State of the tree

- `grammar.js`: four changes from round 20's description, all in the `rules` object plus one
  token constant:
  - `identifier` token constant gained `'_'` as a continuation character (was `letter, digit`
    only).
  - `factor`'s `$.string` arm is now `seq($.string, repeat($.string))` (adjacent-string
    concatenation).
  - `module_header` gained `optional($.external_code_names)` before its terminal `;`.
  - `procedure_body`'s nested declaration repeat changed from `repeat($.procedure_decls)` to
    `repeat(choice(seq($.procedure_decl, ';'), seq($.forward_decl, ';')))` — narrower, excludes
    bodyless variants from the nested position (still fully legal at module scope, unchanged).
  - `conflicts`, `extras`, everything else as round 20 left them.
- `src/scanner.c`: unchanged since round 20 (NBSP in `is_space()`). No scanner changes this
  round — all four fixes were pure `grammar.js` changes.
- `sweep_corpus.py`: unchanged. Baseline for the next round: **84.09% (666/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
