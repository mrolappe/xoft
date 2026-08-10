# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures to find the next cluster, then
implement it test-first.**

Round 12 closed out the bodiless-procedure-heading task (AmigaOberon `Interfaces/*.mod`) with no
follow-on candidate queued — same situation round 11 left round 12 in. This round starts cold:
there is no known next construct, only a method for finding one.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 288/792 (36.36%), up from 243/792 (30.68%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still far off, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are already scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed round 11's triage table) — don't rediscover these as "new" findings and
  don't fix them as part of whatever cluster you find this round unless the user explicitly
  reopens the scoping decision.
- **`procedure_decls` now has a `conflicts` declaration** (`grammar.js`, added round 12:
  `conflicts: $ => [[$.procedure_decl, $.definition_proc_decl]]`) because `definition_proc_decl`
  is reachable both inside `DEFINITION` modules and, since round 12, inside plain `MODULE`s via
  `procedure_decls`. If a new task reuses another rule across two enclosing contexts that share a
  token prefix, expect `tree-sitter generate` to report the same class of "Unresolved conflict"
  error — its suggested fix (add a `conflicts` entry) is normally correct; see
  `docs/insights.md` round 12 for why.

## How to find the next cluster (reproduction, same method as rounds 8–12)

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

Corpus files are Latin-1; always transcode before feeding to `tree-sitter parse`, and strip
comments/strings before grepping free-form corpus text for a construct's frequency (round 11's
insight — apostrophes in comment prose swamp naive `'...'` pairing).

Cross-check any candidate against `docs/language-baseline.md` (the normative Oberon-2 EBNF) and
`docs/plan.md` D1 (lexical superset scope) before assuming it's in-scope for M1 — if it looks like
a structural/semantic extension rather than a lexical one, flag the scoping question to the user
the way round 9 did for `STRUCT`/`ASSEMBLER`, rather than deciding unilaterally.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case for whatever construct is
  implemented, filled via `tree-sitter test --update` and read back to confirm no `ERROR`/
  `MISSING` nodes.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–12).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 12 — the `conflicts` mechanism and when it's needed.
- `docs/insights.md` round 12 — the GLR-ambiguity takeaway and the "heuristic count is a floor,
  not the actual impact" takeaway (don't try to predict `sweep_corpus.py`'s delta before
  measuring it).
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 36.36%).

## State of the tree

- `grammar.js`: `procedure_decls` is now `choice(seq(procedure_decl, ';'), seq(forward_decl,
  ';'), definition_proc_decl)` — three procedure-declaration shapes (bodied, `^`-forward, and
  bodiless-heading) all reachable inside a plain `MODULE`. A top-level `conflicts: $ => [...]`
  field exists for the first time (round 12).
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **36.36% (288/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
