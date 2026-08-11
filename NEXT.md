# Next task

**M1.4 continued — pick `oberon-a` (59 failures, last touched round 14, never had a full
sampling pass) or `stj` (40 failures, last sampled round 17) for a fresh sampling pass; `voc`'s
remaining 4 failures are all design questions, not sampling leads (see below), so don't pick
`voc` again without first resolving one of those.**

Round 20 fixed round 19's known lead (the NBSP+comment GLR bug — root cause found, not just a
GLR mystery) plus ran `voc`'s first-ever dedicated sampling pass, clearing 6 of its 10 failures
with one new rule — 78.16% → 79.29% (619 → 628/792), +9 files this round.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 628/792 (79.29%), up from 619/792 (78.16%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings. This is
  now `amiga-oberon-31`'s entire remaining cluster (60 files) except the 1 open lead below.
- **`Break.mod`/`NoGuru.mod`'s dual pragma-guarded `MODULE` headers — scoping question raised,
  user explicitly deferred the decision, still open.** 2 files in `amiga-oberon-31`:
  `(* $IF X *) MODULE A; (* $ELSE *) MODULE B; (* $END *)` — two alternate top-level module
  headers. Genuinely structural (the `module` rule only expects one `MODULE ident ;`), not a
  simple lexical fix. When picked up again, re-ask rather than assume a default (the user chose
  "skip for now, revisit later" over "scope out" or "implement now" — that was not a decision to
  scope it out, just a deferral).
- **`voc`'s remaining 4 failures are all design questions, confirmed diagnosed, none are a
  routine grammar addition:**
  - `MultiArrayRiders.Mod`, `MultiArrays.Mod`: free-text documentation appended *after*
    `END Module.` (not valid Oberon — real compilers stop reading at the closing `.`). Needs a
    deliberate "ignore trailing content past module_footer" escape hatch if ever fixed — flag
    to the user first (what should M2's lossless serializer do with such a span?), same as the
    `Break.mod` precedent. Don't attempt without asking.
  - `ethUnicode.Mod`: literal **binary** bytes after `END ethUnicode.` (a serialized Native
    Oberon font/timestamp object). Not parseable text at all. Same "ignore trailing content"
    mechanism as above would cover it, if built — not on its own worth building for one file.
  - `ulmRandomGenerators.Mod`: `1. - real` — bare real literal, zero digits after the decimal
    point, colliding with round 18's `2..4` range-operator fix (which made the fractional digit
    *mandatory* specifically to stop `real` from greedily eating `2..4`'s first `.`). Confirmed
    (by hand-tracing tree-sitter's maximal-munch DFA) that relaxing the digit requirement back
    to optional reopens the `2..4` regression — the two corpus facts are mutually exclusive for
    a lookahead-free regex token. The real fix needs the **external scanner** (genuine
    lookahead: after consuming the first `.`, peek one more character — digit → keep consuming
    as real; another `.` → abort, let `integer`+`range` win; anything else → accept `N.` as a
    complete real). Not attempted — touches a token used in nearly every file (real regression
    risk) for a benefit confirmed on only this one file so far. **Before investing in this:**
    check actual prevalence properly (a raw grep for bare `N.` found ~150 hits across 59 files,
    but sampling showed nearly all are inside comment prose like "June 1990." — the true
    in-code prevalence is unknown; check against other roots' *live* parse-failure locations,
    not raw text grep, before deciding whether this is worth the external-scanner work).
- **A cross-dialect grammar bug can hide behind one root's cluster** (rounds 18–20, confirmed
  three times now) — after any fix, always re-tally failures by root before assuming the delta
  is isolated to the root you were sampling.
- **When a parser bug looks like "mysterious GLR behavior," read the hand-written scanner code
  before attributing it to tree-sitter internals** (round 20's biggest process lesson). Round
  19 guessed the NBSP+comment bug was some GLR-internal fork-tipping effect and left it
  unfixed; round 20 found the actual cause in five minutes by reading `src/scanner.c`:
  `is_space()` (used to skip leading whitespace before checking for a comment) didn't include
  NBSP, while `grammar.js`'s `extras` regex did — two independent whitespace definitions that
  had drifted apart. `grep -n "is_space\|extras" grammar.js src/scanner.c` whenever `extras` or
  whitespace handling changes, to keep both definitions in sync going forward.
- **A raw corpus-text grep for a lexical pattern can be dominated by comment prose** (round 20)
  — a pattern that could plausibly appear in an English sentence (e.g. "digit followed by a
  bare `.`", which matches "June 1990.") needs several matches sampled by hand before trusting
  the hit count as a measure of in-code prevalence, since comments are opaque to the grammar
  and grep can't tell the difference.
- **tree-sitter has no lookahead/lookbehind** in its internal regex-based lexer (Rust `regex`
  crate excludes it by design) — confirmed concretely again this round (the `1.`/`2..4`
  conflict). The external scanner is the only place genuine lookahead is available
  (`lexer->lookahead` after `advance()`, with "return `false`" as the rollback escape hatch —
  already the pattern used for comment/pragma/bracket_pragma detection).
- **A dialect's own EBNF documentation can still be wrong against real corpus usage** even for a
  construct it already names (round 19) — grep the corpus even when the baseline "already
  covers it."
- **An unscoped grep over a corpus root can match compiled binaries alongside source** (round
  18) — use `--include='*.mod'`/`--include='*.Mod'` (voc's extension is capitalized) when
  grepping a root that mixes source and compiled artifacts.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a`. For a
  specific *byte* (not a text pattern), `LC_ALL=C grep -rla $'\xXX'` is what actually works —
  plain `grep -rlaP '\xXX'` interprets the pattern as UTF-8 and won't match a raw Latin-1 byte
  the same way.
- **When constructing a minimal repro that depends on a specific non-ASCII byte (NBSP, etc.),
  build the file programmatically from the start** (round 20 error log) — a hand-typed heredoc
  will silently substitute a look-alike ASCII character, producing a false "doesn't reproduce"
  result. Confirm with `python3 -c "print(repr(...))"`, don't eyeball it.
- **When scanning a multi-line construct for a trailing marker via regex, don't stop at the
  first occurrence of the terminator character** (round 20 error log) — it may appear inside
  the construct itself (e.g. `;` separating formal parameters across a multi-line procedure
  heading) before the real terminator. Widen the search window generously or match
  structurally.
- **`procedure_decls` has 4 alternatives now**: `procedure_decl`, `forward_decl`,
  `definition_proc_decl`, `external_proc_decl` (round 20, voc's `PROCEDURE -ident ... "C
  string";`). `conflicts` still has two entries: `[procedure_decl, definition_proc_decl]`
  (round 12) and `[selector, actual_params]` (round 19, documentation-only per tree-sitter).

## How to find the next cluster (reproduction, same method as rounds 8–20)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
grep "^  oberon-a/" /tmp/sweep_v.txt | sed 's#^  oberon-a/##'    # or stj/
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # find the ERROR/MISSING node, read the surrounding source
```

Corpus files are Latin-1 except `voc` (UTF-8); always transcode Latin-1 roots before feeding to
`tree-sitter parse`.

Cross-check any candidate against `docs/language-baseline.md` first, and check for a
`*.doc`/`*.txt` compiler manual before inferring a dialect construct's shape purely from usage.
Only flag a scoping question to the user (round 9's `STRUCT`, round 20's `Break.mod`) when the
construct is genuinely absent from the baseline *and* structural (new type kind, new statement
form, tolerating multiple top-level headers, or — round 20's trailing-content files — content
that isn't Oberon syntax at all) needing more than a sibling-rule addition.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  filled either via `tree-sitter test --update` and read back to confirm no `ERROR`/`MISSING`
  nodes, or hand-written by copying a structurally identical existing test's shape verbatim.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–20). After any fix, re-tally failures by root — this
  has been cross-dialect three rounds running now.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 20 — the NBSP+comment scanner fix (root cause, not just
  the symptom), the `external_proc_decl` rule and its 6-file impact, the three still-open `voc`
  leads (trailing content x2, bare-real lexer gap), and the `Break.mod` deferral.
- `docs/insights.md` round 20 (four new entries) — the "read the scanner before blaming GLR"
  lesson, the comment-prose-pollutes-grep lesson, the `1.`/`2..4` lookahead-wall confirmation,
  and (in the errors log) the NBSP-heredoc and multi-line-terminator mistakes.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 79.29%).

## State of the tree

- `grammar.js`: unchanged from round 19's description except: `procedure_decls` gained a fourth
  choice arm, `$.external_proc_decl` (round 20); new rule `external_proc_decl: $ => seq(
  $.kProcedure, '-', $.ident_def, optional($.formal_params), $.string, ';')`. `extras`,
  `conflicts`, and everything else as round 19 left them.
- `src/scanner.c`: **round 20 changed `is_space()`** — now includes `0xa0` (NBSP) alongside the
  ASCII whitespace set, with a comment noting it must stay in sync with `grammar.js`'s `extras`
  regex. First scanner change since round 10. Still four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`) — no new token kinds added, this was a one-line fix inside
  the existing whitespace-skip helper.
- `sweep_corpus.py`: unchanged. Baseline for the next round: **79.29% (628/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
