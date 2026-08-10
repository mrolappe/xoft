# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures in `stj` and `voc` (largest
unsampled territory relative to size), find the next cluster, implement it test-first.**

Round 16 fixed Oberon-A's "assignable procedure" `*` mark (`PROCEDURE* [sysflag] Name`), jumping
the pass rate from 54.42% to 60.61%. This round starts cold again: there is no known next
construct, only the same method used since round 8.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 480/792 (60.61%), up from 431/792 (54.42%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings.
- **Round 16's fix**: `procedure_heading` in `grammar.js` gained `optional($.kStar)` between
  `$.kProcedure` and `optional($.sysflag)` — Oberon-A's "assignable procedure" mark, a `*`
  immediately after the `PROCEDURE` keyword (not the normal export mark, which is after the
  identifier). Documented in `docs/OC.doc`'s "AssignableProcs" node. Reused the existing `kStar`
  token, no scanner change.
- **Post-round-16 failure counts by root**: `stj` 105, `amiga-oberon-31` 92, `oberon-a` 72,
  `voc` 43 (out of `/tmp/sweep_v2.txt`, now stale — regenerate before trusting exact numbers,
  but the *relative* picture — `stj` and `voc` essentially untouched across 16 rounds while
  `oberon-a` has had four rounds of fixes aimed at it — is the reason to sample there next).
  Neither `stj` nor `voc` has had a dedicated sampling pass yet; every round 8–16 fix originated
  from `oberon-a` or `amiga-oberon-31` samples. `voc` is public Oberon-2 (no dialect docs to
  lean on, per `corpus/roots.toml`), `stj` is Atari ST Oberon (no `*.doc`/`*.txt` manual
  confirmed present as of round 14 — check again, don't assume).
- **A "cluster looks cleared" claim from a stale round needs a fresh per-file `grep -A1` check,
  not just a re-run of the aggregate number** (round 16 insight) — round 15 aimed at
  `oberon-a/source/amiga/*.mod` and round 16 found 31 files in that exact directory still
  failing on an unrelated construct. Whenever `NEXT.md` claims a directory "may now be mostly
  clear," verify with `grep -A1 "^  <root>/<dir>" /tmp/sweep_v.txt | wc -l` before trusting it.
- **A narrow single-line `(ERROR [n,0]-[n,end])` landing exactly on a keyword the grammar already
  has a rule for is a fixed-order/cardinality bug signature, not a new construct** (round 15
  insight) — check whether the existing rule's `optional`/`seq`/`repeat` shape matches the
  baseline EBNF's actual repetition/ordering before assuming something new is missing.
- **A mark character can mean different things in different grammar positions** (round 16
  insight) — Oberon-A's assignable-procedure `*` (before the ident, after `PROCEDURE`) reuses the
  same `kStar` token as the export mark (after the ident) but is a wholly different grammar slot.
  Don't assume every occurrence of a mark character maps to one rule; check position, not just
  token identity, and cross-check the dialect's own manual before guessing meaning from position
  alone.
- **Oberon-A ships its own compiler manual** (`Oberon-A/docs/OC.doc`, per `corpus/roots.toml`'s
  `oberon-a` path) with a formal `$`-prefixed EBNF for its dialect extensions and worked
  examples — it's plain text wrapped in AmigaGuide markup (`@node`/`@endnode`), `grep -na` works
  fine directly on it. Check for a `*.doc`/`*.txt` manual in a corpus root before
  reverse-engineering a dialect construct's grammar from corpus samples alone. Not every root has
  one (`amiga-oberon-31`, `stj` don't as of round 14 — re-verify for `stj` if picking there).
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` — always
  `grep -rla` (or `-a` with whatever other flags) when grepping corpus roots directly;
  `sweep_corpus.py` itself already transcodes.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension (the `STRUCT`/`ASSEMBLER` pattern) or flagging it to the user,
  grep `docs/language-baseline.md` for it first — if it's already there, it's just an
  incomplete/incorrect grammar rule, implement it directly.
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12) because
  `definition_proc_decl` is reachable both inside `DEFINITION` modules and inside plain
  `MODULE`s via `procedure_decls`. If a new task reuses another rule across two enclosing
  contexts that share a token prefix, expect `tree-sitter generate` to report the same class of
  "Unresolved conflict" error — its suggested fix (add a `conflicts` entry) is normally correct.

## How to find the next cluster (reproduction, same method as rounds 8–16)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first to confirm stj/voc are still the least-sampled:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
# Then skim stj/voc entries for short, whole-file "(ERROR [0, 0] - [N, 0])" failures — those are
# usually a single early construct the grammar doesn't know at all, easiest to isolate. Also
# check narrow single-line spans landing on a known keyword (round 15's signature) before
# assuming a wholly new construct.
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
  when guessed from `grammar.js` alone).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–16).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 16 — the assignable-procedure-mark fix and the
  "cleared cluster still needs a fresh per-file check" lesson.
- `docs/insights.md` rounds 15–16 — the narrow-ERROR-on-known-keyword signature, the
  outer-brace repetition-reading lesson, the "verify cleared clusters per-file" lesson, and the
  "same mark character, different grammar position" lesson.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 60.61%).

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
  Round 16 added: `procedure_heading` now has `optional($.kStar)` right after `$.kProcedure`
  (Oberon-A's assignable-procedure mark), reusing the existing `kStar` token.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **60.61% (480/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
