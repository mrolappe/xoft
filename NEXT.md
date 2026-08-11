# Next task

**M1.4 continued — finish `amiga-oberon-31`'s last 4 non-`STRUCT` failures (`Break.mod`,
`FArrays.mod`, `Lists.mod`, `linkedlists.mod`), then pivot to a dedicated `voc` sampling pass
(now the smallest root by far — 10 failures, never sampled) or `stj`/`oberon-a` if `voc` proves
a dead end.**

Round 19 fixed round 18's known lead (`Sparks.mod`'s hardware-address annotation) plus two more
bugs found while sampling further: a `D`/`E` scale-factor literal bug (cross-dialect, +26
files) and a `designator`/`actual_params` type-guard-vs-call ambiguity (cross-dialect, +35
files, the largest single-fix gain to date) — 69.95% → 78.16% (554 → 619/792), +65 files this
round.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 619/792 (78.16%), up from 554/792 (69.95%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings. 60 of
  `amiga-oberon-31`'s 64 remaining failures are this; skip files containing either string when
  looking for the next cluster (`grep -qa "STRUCT\|UNTRACED" <file>`).
- **The known next leads in `amiga-oberon-31`** (4 non-`STRUCT` failures left):
  - `Module/FArrays.mod`, `Module/Lists.mod`: both fail at a procedure heading with a trailing
    NBSP (U+00A0) followed by a comment, before `BEGIN` — the exact same pattern round 19
    diagnosed but did NOT fix in `BasicTypes.mod` (that one file got fixed because it had no
    comment after the NBSP; these two do). Minimal repro confirmed: NBSP alone before `BEGIN`
    parses fine, a comment alone before `BEGIN` parses fine, the *combination* fails — even for
    a plain receiver-less `PROCEDURE Add;` with nothing else unusual. Suspected cause:
    interaction between `extras`' now-two-different token kinds (regex-matched NBSP vs.
    external-scanner comment) and the pre-existing `procedure_decl`/`definition_proc_decl` GLR
    fork (round 12) — see `docs/insights.md` round 19's last entry before starting here. Try
    minimizing further first (does *any* two-different-extras-kinds-back-to-back combination
    trigger it, e.g. a bracket pragma then a comment, or two consecutive plain-regex extras of
    different shapes?) before attempting a grammar fix — the mechanism isn't understood yet.
  - `Module/Break.mod`: fails at `[0,0]` (whole file) — starts with `(* $IF BreakRq *)\nMODULE
    BreakRq;\n(* $ELSE *)\nMODULE Break;\n(* $END *)`, i.e. two complete alternate `MODULE ...;`
    headers guarded by pragma-shaped comments, a conditional-compilation convention. Not yet
    diagnosed how far the actual failure goes beyond the two-headers issue (the module rule
    only expects one `MODULE ident ;`) — this may be a genuine scoping question (a new
    conditional-compilation construct, not merely a lexical synonym) worth flagging to the user
    the way round 9 did for `STRUCT`, since tolerating two alternate top-level headers is
    structural, not a simple sibling-rule addition. Check how common this `$IF`/`$ELSE`/`$END`
    pattern is across the corpus first (grep `\$IF\b` in comments) before deciding.
  - `Module/linkedlists.mod`: not yet diagnosed at all this round (found via the post-round
    non-`STRUCT` filter, error at `[149,42]-[199,0]`) — start here fresh.
- **`voc` is now the smallest root by far** (41 → 10 failures this round, via the cross-dialect
  `D`/`E` scale-factor and `designator`/`actual_params` fixes) and has never had a dedicated
  sampling pass (round 17 was the first for `stj`, round 12/18/19 for `amiga-oberon-31`, none
  yet for `voc`). With only 10 files left it may clear in one round — a strong alternative to
  `amiga-oberon-31`'s remaining cluster if that one stalls.
- **A parenthesized single bare identifier after a designator (`Foo(Bar)`) is genuinely
  ambiguous in Oberon-2 itself** (type guard vs. procedure call with one argument) — round 19
  resolved it structurally (folded `actual_params` into `designator`'s own `repeat` alongside
  `selector`) rather than via GLR (tree-sitter reported `conflicts: [[$.selector,
  $.actual_params]]` "unnecessary" both before and after the restructuring — no automaton fork
  was ever built). Don't re-attempt a `conflicts`-only fix for a similar-looking ambiguity
  without first checking whether the rules genuinely sit at the same choice point.
- **A dialect's own EBNF documentation (`docs/language-baseline.md`) can still be wrong against
  real corpus usage** even for a construct it already names (round 19: `ScaleFactor` requires a
  sign and digits, but `9.22337177E18` has no sign and `D`-suffix literals have neither) — grep
  the corpus even when the baseline "already covers it," don't just check whether the grammar
  implements what the baseline says.
- **A cross-dialect grammar bug can hide behind one root's cluster** (round 18 and 19 insight,
  now confirmed twice) — after any fix, always re-tally failures by root before assuming the
  delta is isolated to the root you were sampling. Round 19's two biggest fixes (`D`/`E` scale
  factor, `designator`/`actual_params`) were both found while sampling `amiga-oberon-31` but
  landed their largest impact in `voc` and `stj`.
- **tree-sitter has no lookahead/lookbehind** (Rust's `regex` crate excludes it by design) — an
  ambiguous-lexeme bug can't be patched with a negative-lookahead regex trick the way you might
  in PCRE. The fix has to change what the grammar's token *rule* accepts (round 18: require ≥1
  digit after `real`'s decimal point) after confirming via corpus grep that the stricter
  language is still a superset of everything actually written.
- **Fixing one construct can leave a file failing at the exact same error location** if a second,
  unrelated construct sits immediately after it in the same span (round 18). Don't conclude a
  fix "didn't work" from an unchanged error location — isolate the fix alone in a minimal repro
  first to confirm it in isolation, then look for a second construct stacked in the same span.
- **An unscoped grep over a corpus root can match compiled binaries alongside source** (round 18)
  — use `--include='*.mod'` when grepping a root that mixes source and compiled artifacts.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` — always
  `grep -rla` (or `-a` with whatever other flags) when grepping corpus roots directly. For a
  specific *byte* (not a text pattern), `LC_ALL=C grep -rla $'\xXX'` is what actually found
  round 19's NBSP occurrences — plain `grep -rlaP '\xXX'` interprets the pattern as UTF-8 and
  won't match a raw Latin-1 byte the same way.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension, grep `docs/language-baseline.md` for it first. (`Break.mod`'s
  `$IF`/`$ELSE`/`$END` conditional-compilation headers may be a genuine exception to this —
  they're structural, not lexical, so check corpus prevalence before deciding whether to flag.)
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12, still
  needed) pairing `procedure_decl`/`definition_proc_decl`. A second conflict entry,
  `[$.selector, $.actual_params]`, was added round 19 as in-code documentation of the known
  type-guard/call ambiguity even though tree-sitter says it's currently unnecessary (kept for
  when the grammar changes again in a way that does make it live).
- **When an `Edit` `old_string` doesn't match text you just wrote, suspect a non-ASCII
  character** (round 19: a literal NBSP inside a regex character class) rather than assuming
  the file changed underneath you — read the bytes back (`od -c` or Python `repr()`) before
  retyping.

## How to find the next cluster (reproduction, same method as rounds 8–19)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
# Filter out STRUCT/UNTRACED (still Phase 2 scope) when sampling amiga-oberon-31:
grep "^  amiga-oberon-31/" /tmp/sweep_v.txt | sed 's#^  amiga-oberon-31/##' | while read -r f; do
  grep -qa "STRUCT\|UNTRACED" "<amiga-oberon-31 absolute path from roots.toml>/$f" || echo "$f"
done
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # find the ERROR/MISSING node, read the surrounding source
```

Corpus files are Latin-1; always transcode before feeding to `tree-sitter parse`, and use
`grep -a` (add `--include=<ext>` to exclude compiled binaries) when grepping raw corpus text
directly. For a raw byte (not a printable pattern), use `LC_ALL=C grep -rla $'\xXX'`.

Cross-check any candidate against `docs/language-baseline.md` first, and check for a
`*.doc`/`*.txt` compiler manual (or an `.OBJ` binary's embedded string table, round 17) before
inferring a dialect construct's shape purely from usage. Only flag a scoping question to the
user the way round 9 did for `STRUCT`/`ASSEMBLER` when the construct is genuinely absent from
the baseline *and* structural (new type kind, new statement form, or — round 19's `Break.mod`
lead — tolerating multiple top-level headers) needing more than a sibling-rule addition.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  filled either via `tree-sitter test --update` and read back to confirm no `ERROR`/`MISSING`
  nodes, or hand-written by copying a structurally identical existing test's shape verbatim
  (grep the same test file for a similar existing case first).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–19). After any fix, re-tally failures by root — rounds
  18 and 19 both turned out to be cross-dialect even though found while sampling one root.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 19 — all four fixes (hardware-address vars, D/E scale
  factor, designator/actual_params ambiguity resolution, NBSP whitespace), the NBSP+comment GLR
  lead for `Lists.mod`/`FArrays.mod`, and the `Break.mod` conditional-compilation lead.
- `docs/insights.md` round 19 (four new entries) — the baseline-EBNF-can-still-be-wrong lesson,
  the "trust tree-sitter's 'unnecessary conflicts' warning" lesson, and the unfixed NBSP+extras
  GLR-interaction finding.
- `docs/errors.md` round 19 — the wasted cycle re-testing a `conflicts` declaration that was
  already confirmed unnecessary, and the NBSP-in-`Edit`-`old_string` mismatch.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 78.16%).

## State of the tree

- `grammar.js`: `case_statement` supports `[ELSE StatementSeq]` (round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` (round 12).
  `conflicts` now has two entries: `[procedure_decl, definition_proc_decl]` (round 12) and
  `[selector, actual_params]` (round 19, documentation-only per tree-sitter). Round 14 added:
  `sysflag`, `square_vector_offset`, `external_code_names`, `reg_spec`. Round 15: all three
  declaration-sequence sites use `repeat(choice($.const_decls, $.type_decls,
  $.variable_decls))`. Round 16: `procedure_heading` has `optional($.kStar)` right after
  `$.kProcedure`. Round 17 added: `mul_operator` gained `$.kAnd`; `factor` gained
  `seq($.kNot, $.factor)`; keyword tokens `kAnd => 'AND'`, `kNot => 'NOT'`. Round 18 added:
  `module`'s `BEGIN` arm gained `optional(seq($.kClose, optional($.statement_seq)))`, new token
  `kClose => 'CLOSE'`; `factor` gained `$.typed_set` (`typed_set: $ => seq($.qualident,
  $.set)`); `real`'s fractional part requires ≥1 digit after the `.`; `integer` gained a third
  choice arm for the `U` unsigned-hex suffix; `procedure_heading`'s annotation slot gained
  `$.curly_external_code_names`; `param_offset` gained a trailing `optional('..')`. **Round 19
  added:** `variable_decl` is now `choice($.ident_list, $.addressed_ident), ':', $.type` with
  new rules `addressed_ident: $ => seq($.ident_def, $.address)` and `address: $ => seq('[',
  $.integer, ']')`; `scale_factor` is now `seq(choice('E', 'D'), optional(seq(optional(choice(
  '+', '-')), digit, repeat(digit))))` (was `E`-only, mandatory sign); `extras`' whitespace
  regex is now `/[\s ]/` (was `/\s/`); **`designator` is now `prec.left(seq($.qualident,
  repeat(choice($.selector, $.actual_params))))`** (was `repeat($.selector)` only) — `factor`
  and `procedure_call` no longer have a separate trailing `optional($.actual_params)`, they're
  just `$.designator` now. This reshaped the AST: `actual_params` is a child of `designator`
  (sibling of `selector`) rather than a sibling of `designator` under `factor`/`procedure_call`.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **78.16% (619/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
