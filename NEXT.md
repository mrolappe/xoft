# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures to find the next cluster, then
implement it test-first.**

Round 14 closed out the Oberon-A "system flags"/square-bracket library-call cluster with no
follow-on candidate fully scoped — same situation every round since round 11 has left the next
round in. This round starts cold: there is no known next construct, only a method for finding
one and one concrete lead (below) worth checking first.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 328/792 (41.41%), up from 312/792 (39.39%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still far off, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings.
- **Concrete lead, not yet investigated**: `oberon-a/source/amiga/*.mod` — round 14 unblocked the
  whole-file `MODULE [n]`/library-call ERROR at the *top* of these files, but most of them
  (53 sampled, e.g. `ASL.mod`, `AmigaGuide.mod`, `BootBlock.mod`, `Bullet.mod`, `CDDevice.mod`)
  still fail, now on much narrower/later ERROR spans (single-line to ~10-line, not whole-file —
  see the delta in `/tmp/amiga_sweep.txt` if still present, otherwise re-run). This is the
  natural first thing to check this round: sample a few of these files' *new* failure points
  before sampling fresh corpus territory — they may be one more small construct away from
  passing, which would be a cheap, high-count win (there are ~85 files in that directory alone).
- **Oberon-A ships its own compiler manual** (`Oberon-A/docs/OC.doc`, per `corpus/roots.toml`'s
  `oberon-a` path) with a formal `$`-prefixed EBNF for its dialect extensions and worked
  examples — round 14's insight: check for a `*.doc`/`*.txt` manual in a corpus root before
  reverse-engineering a dialect construct's grammar from corpus samples alone. Not every root has
  one (`amiga-oberon-31`, `stj` don't), but `oberon-a` does and it's authoritative.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` (round 14
  insight) — files with high-bit bytes read as "binary" and get excluded from `-rl` counts,
  giving implausibly low frequency numbers. Always `grep -rla` (or `-a` with whatever other
  flags) when grepping corpus roots directly for a construct's frequency; `sweep_corpus.py`
  itself already transcodes, so this only bites ad-hoc `grep` exploration.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension (the `STRUCT`/`ASSEMBLER` pattern) or flagging it to the user,
  grep `docs/language-baseline.md` for it first — if it's already there, it's just an incomplete
  grammar rule, implement it directly (round 13's insight, still applies).
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12) because
  `definition_proc_decl` is reachable both inside `DEFINITION` modules and inside plain
  `MODULE`s via `procedure_decls`. If a new task reuses another rule across two enclosing
  contexts that share a token prefix, expect `tree-sitter generate` to report the same class of
  "Unresolved conflict" error — its suggested fix (add a `conflicts` entry) is normally correct.

## How to find the next cluster (reproduction, same method as rounds 8–14)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Skim /tmp/sweep_v.txt for short, whole-file "(ERROR [0, 0] - [N, 0])" failures — those are
# usually a single early construct the grammar doesn't know at all, easiest to isolate.
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
`grep -a` (not just `-r`) when grepping raw corpus text directly (round 14's insight — high-bit
bytes read as binary and get silently skipped otherwise).

Cross-check any candidate against `docs/language-baseline.md` (the normative Oberon-2 EBNF)
first, and check for a `*.doc`/`*.txt` compiler manual in the relevant corpus root (per
`corpus/roots.toml`) before inferring a dialect construct's shape purely from usage — round 14's
`Oberon-A/docs/OC.doc` had the exact formal grammar and worked examples for that round's
cluster. Only flag a scoping question to the user the way round 9 did for `STRUCT`/`ASSEMBLER`
when the construct is genuinely absent from the baseline (a structural/semantic dialect
extension, not a lexical or already-normative one) — check `docs/plan.md` D1 in that case.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case for whatever construct is
  implemented, filled either via `tree-sitter test --update` and read back to confirm no
  `ERROR`/`MISSING` nodes, or hand-written by copying a structurally identical existing test's
  shape verbatim (legitimate when the shape is already visible elsewhere in the same file, not
  when guessed from `grammar.js` alone — see round 8's and round 13's insights).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–14).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 14 — the "check the corpus root for its own compiler
  manual before reverse-engineering from samples" method, and the `oberon-a/source/amiga/*.mod`
  lead above.
- `docs/insights.md` rounds 12–14 — the GLR-ambiguity/`conflicts` mechanism, "heuristic count is
  a floor, not the actual impact," the baseline-first-not-scoping-question check, the
  copy-a-neighboring-test shortcut, the compiler-manual-before-corpus-archaeology method, and the
  `grep -a` Latin-1 gotcha.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 41.41%).

## State of the tree

- `grammar.js`: `case_statement` supports `[ELSE StatementSeq]` (round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` (round 12),
  with a top-level `conflicts: $ => [...]` for the resulting ambiguity. Round 14 added: `sysflag`
  (`"[" integer "]"`, on `module_header`/`pointer_type`/`record_type`/`procedure_heading`),
  `square_vector_offset` and `external_code_names` (both alternatives in `procedure_heading`'s
  post-`ident_def` slot alongside the existing curly-brace `vector_offset`), and `reg_spec`
  (`"[" integer "]" [".."]`, alternative to `param_offset` in `fp_section`) — all Oberon-A
  square-bracket dialect extensions, no scanner changes, no new conflicts.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **41.41% (328/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
