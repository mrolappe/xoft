# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures to find the next cluster, then
implement it test-first.**

Round 13 closed out the `CASE ... ELSE ... END` task with no follow-on candidate queued — same
situation every round since round 11 has left the next round in. This round starts cold: there is
no known next construct, only a method for finding one.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 312/792 (39.39%), up from 288/792 (36.36%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still far off, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are already scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed rounds 11 and 12's triage tables) — don't rediscover these as "new"
  findings and don't fix them as part of whatever cluster you find this round unless the user
  explicitly reopens the scoping decision. Round 13 re-hit these in `amiga-oberon-31/Interfaces/*.mod`
  samples and skipped them correctly — expect to keep seeing them in the `-v` output since they're
  common in that corpus root.
- **Not every unhandled construct is a scoping question.** Round 13's `CASE...ELSE` looked like a
  new-dialect finding but was actually already normative Oberon-2
  (`docs/language-baseline.md` line 94 has always had `[ELSE StatementSeq]` in the case
  statement). Before treating a gap as an out-of-scope dialect extension (the `STRUCT`/`ASSEMBLER`
  pattern) or flagging it to the user, grep `docs/language-baseline.md` for it first — if it's
  already there, it's just an incomplete grammar rule, implement it directly. See
  `docs/insights.md` round 13.
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12:
  `conflicts: $ => [[$.procedure_decl, $.definition_proc_decl]]`) because `definition_proc_decl`
  is reachable both inside `DEFINITION` modules and inside plain `MODULE`s via `procedure_decls`.
  If a new task reuses another rule across two enclosing contexts that share a token prefix,
  expect `tree-sitter generate` to report the same class of "Unresolved conflict" error — its
  suggested fix (add a `conflicts` entry) is normally correct; see `docs/insights.md` round 12
  for why.

## How to find the next cluster (reproduction, same method as rounds 8–13)

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

Cross-check any candidate against `docs/language-baseline.md` (the normative Oberon-2 EBNF) first
— round 13's insight: if it's already normative, just implement it, no need to ask. Only flag a
scoping question to the user the way round 9 did for `STRUCT`/`ASSEMBLER` when the construct is
genuinely absent from the baseline (a structural/semantic dialect extension, not a lexical or
already-normative one) — check `docs/plan.md` D1 (lexical superset scope) in that case.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case for whatever construct is
  implemented, filled either via `tree-sitter test --update` and read back to confirm no
  `ERROR`/`MISSING` nodes, or hand-written by copying a structurally identical existing test's
  shape verbatim (round 13's approach — legitimate when the shape is already visible elsewhere in
  the same file, not when guessed from `grammar.js` alone; see round 8's and round 13's insights).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–13).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 13 — the "check the baseline before treating a gap as a
  scoping question" method.
- `docs/insights.md` rounds 12–13 — the GLR-ambiguity/`conflicts` mechanism, "heuristic count is a
  floor, not the actual impact," the baseline-first-not-scoping-question check, and the
  copy-a-neighboring-test shortcut.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 39.39%).

## State of the tree

- `grammar.js`: `case_statement` now supports `[ELSE statement_seq]` before `kEnd`, matching
  `if_statement`'s existing `ELSE` handling (added round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` — three
  procedure-declaration shapes reachable inside a plain `MODULE` (round 12). A top-level
  `conflicts: $ => [...]` field exists (round 12) for the `procedure_decl`/`definition_proc_decl`
  ambiguity this creates.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **39.39% (312/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
