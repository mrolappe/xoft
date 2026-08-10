# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures across corpus roots to find the
next cluster, then implement it test-first.**

Round 15 fixed a big, broadly-applicable grammar bug (repeated/interleaved `CONST`/`TYPE`/`VAR`
declaration sections), jumping the pass rate from 41.41% to 54.42% — the largest single-round
gain since the sweep tool existed. This round starts cold again: there is no known next
construct, only the same method used since round 8.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 431/792 (54.42%), up from 328/792 (41.41%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings.
- **Round 15's fix**: `grammar.js`'s `module`, `definition module`, and `procedure_body` rules
  now use `repeat(choice($.const_decls, $.type_decls, $.variable_decls))` instead of one
  `optional(...)` of each in fixed order — matches the normative `DeclSeq` EBNF's outer `{}`
  (declaration sections can repeat and interleave in any order/count, not just once each). This
  was a plain grammar bug against the baseline, not a dialect scoping question.
- **No fresh sampling has been done yet this round beyond the aggregate number.** The
  `oberon-a/source/amiga/*.mod` cluster round 14/15 focused on may now be mostly clear (round
  15's fix targeted exactly its failure shape), but that hasn't been re-checked file-by-file —
  worth a quick check before sampling fresh territory, in case a second blocking construct
  remains in that directory. If it's clear, move to unsampled roots (`amiga-oberon-31`, `stj`,
  `voc`) or unsampled parts of `oberon-a` (outside `source/amiga/`).
- **A narrow single-line `(ERROR [n,0]-[n,end])` landing exactly on a keyword the grammar already
  has a rule for is a fixed-order/cardinality bug signature, not a new construct** (round 15
  insight) — check whether the existing rule's `optional`/`seq`/`repeat` shape matches the
  baseline EBNF's actual repetition/ordering before assuming something new is missing. Re-read
  the EBNF's brace nesting literally (an outer `{}` around an alternation means the whole group
  repeats, easy to miss when skimming) rather than from memory of "how declarations usually look".
- **Oberon-A ships its own compiler manual** (`Oberon-A/docs/OC.doc`, per `corpus/roots.toml`'s
  `oberon-a` path) with a formal `$`-prefixed EBNF for its dialect extensions and worked
  examples (round 14's insight) — check for a `*.doc`/`*.txt` manual in a corpus root before
  reverse-engineering a dialect construct's grammar from corpus samples alone. Not every root has
  one (`amiga-oberon-31`, `stj` don't), but `oberon-a` does and it's authoritative.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` (round 14
  insight) — always `grep -rla` (or `-a` with whatever other flags) when grepping corpus roots
  directly; `sweep_corpus.py` itself already transcodes, so this only bites ad-hoc `grep`
  exploration.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension (the `STRUCT`/`ASSEMBLER` pattern) or flagging it to the user,
  grep `docs/language-baseline.md` for it first — if it's already there (even if easy to misread,
  like round 15's outer-brace repetition), it's just an incomplete/incorrect grammar rule,
  implement it directly (round 13's and 15's insight, still applies).
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12) because
  `definition_proc_decl` is reachable both inside `DEFINITION` modules and inside plain
  `MODULE`s via `procedure_decls`. If a new task reuses another rule across two enclosing
  contexts that share a token prefix, expect `tree-sitter generate` to report the same class of
  "Unresolved conflict" error — its suggested fix (add a `conflicts` entry) is normally correct.

## How to find the next cluster (reproduction, same method as rounds 8–15)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Skim /tmp/sweep_v.txt for short, whole-file "(ERROR [0, 0] - [N, 0])" failures — those are
# usually a single early construct the grammar doesn't know at all, easiest to isolate. Also
# check narrow single-line spans landing on a known keyword (round 15's signature) before
# assuming a wholly new construct.
# Pick a handful across different corpus roots (oberon-a, amiga-oberon-31, stj, voc, ...) so the
# cluster you find isn't an artifact of one codebase's house style.
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # find the ERROR node, read the surrounding source to see the
                                # actual unhandled shape
```

Corpus files are Latin-1; always transcode before feeding to `tree-sitter parse`, and use
`grep -a` (not just `-r`) when grepping raw corpus text directly.

Cross-check any candidate against `docs/language-baseline.md` (the normative Oberon-2 EBNF)
first — read brace nesting literally, not from memory — and check for a `*.doc`/`*.txt` compiler
manual in the relevant corpus root before inferring a dialect construct's shape purely from
usage. Only flag a scoping question to the user the way round 9 did for `STRUCT`/`ASSEMBLER`
when the construct is genuinely absent from the baseline (a structural/semantic dialect
extension, not a lexical or already-normative one) — check `docs/plan.md` D1 in that case.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case for whatever construct is
  implemented, filled either via `tree-sitter test --update` and read back to confirm no
  `ERROR`/`MISSING` nodes, or hand-written by copying a structurally identical existing test's
  shape verbatim (legitimate when the shape is already visible elsewhere in the same file, not
  when guessed from `grammar.js` alone — see round 8's and round 13's insights).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–15).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 15 — the "narrow single-line ERROR on a known keyword is a
  cardinality bug, not a new construct" method, and the outer-brace EBNF-reading lesson.
- `docs/insights.md` rounds 13–15 — the baseline-first-not-scoping-question check, the
  copy-a-neighboring-test shortcut, the compiler-manual-before-corpus-archaeology method, the
  `grep -a` Latin-1 gotcha, the narrow-ERROR-on-known-keyword signature, and the outer-brace
  repetition-reading lesson.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 54.42%).

## State of the tree

- `grammar.js`: `case_statement` supports `[ELSE StatementSeq]` (round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` (round 12),
  with a top-level `conflicts: $ => [...]` for the resulting ambiguity. Round 14 added: `sysflag`
  (`"[" integer "]"`, on `module_header`/`pointer_type`/`record_type`/`procedure_heading`),
  `square_vector_offset` and `external_code_names` (both alternatives in `procedure_heading`'s
  post-`ident_def` slot alongside the existing curly-brace `vector_offset`), and `reg_spec`
  (`"[" integer "]" [".."]`, alternative to `param_offset` in `fp_section`) — all Oberon-A
  square-bracket dialect extensions, no scanner changes, no new conflicts. Round 15 changed: all
  three declaration-sequence sites (`module`'s two `seq` branches, `procedure_body`) now use
  `repeat(choice($.const_decls, $.type_decls, $.variable_decls))` instead of one
  `optional(...)` of each — sections can repeat and interleave, matching the baseline EBNF.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **54.42% (431/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
