# Next task

**M1.1 — vendor the base tree-sitter grammar and get it building under tree-sitter 0.26.**

Everything a fresh session needs to start cold is below.

## What to do

1. Copy [`viegasfh/tree-sitter-oberon-2`](https://github.com/viegasfh/tree-sitter-oberon-2)
   (MIT) into `grammars/tree-sitter-oberon2/`. Take `grammar.js`, `package.json`,
   `test/corpus/*`, `LICENSE`. Do **not** take the checked-in `src/parser.c` — regenerate it.
2. Keep the upstream `LICENSE` and add a short `NOTICE` or README line recording the fork
   origin and commit.
3. `tree-sitter generate` with the installed CLI (0.26.11) and fix whatever the newer CLI
   complains about — the grammar was written against roughly 0.20.
4. `tree-sitter test` — the five upstream corpus files (`comments`, `declarations`, `module`,
   `procedures`, `records`) must pass before anything is changed.
5. Also lift `queries/highlights.scm` from
   [`geekstakulus/tree-sitter-oberon-07`](https://github.com/geekstakulus/tree-sitter-oberon-07)
   (MIT, same skeleton) and adapt node names — that is M1.5, cheap to do in the same pass.

**Minimum model: Haiku.** This is mechanical. If the regeneration raises grammar conflicts that
need rule redesign rather than mechanical fixes, that is M1.2 work — stop and escalate to
Sonnet rather than improvising rule changes here.

**Do not** start adding missing constructs in this task. M1.2a/b/c are separate, each scoped to
one section of the EBNF, deliberately so that each carries a small context.

## Definition of done

- `tree-sitter generate && tree-sitter test` green in `grammars/tree-sitter-oberon2/`
- upstream attribution present
- `src/parser.c` regenerated locally and either gitignored or committed deliberately (decide
  and record which — committing it makes CI cheaper, ignoring it keeps diffs readable;
  recommendation: **gitignore** it and generate in CI)

## Context a fresh session needs

- `docs/plan.md` — decisions D1–D8 and the milestone table. Read this first.
- `docs/language-baseline.md` — the normative Oberon-2 EBNF, already extracted; there is no
  need to fetch the report again.
- `docs/insights.md` — in particular the list of what the upstream grammar is missing, so the
  gaps are not rediscovered.
- `CLAUDE.md` — test-first rule and the end-of-round ritual.

## State of the tree

- M0 complete: workspace builds, 13 tests green, `corpus/manifest.json` lists 792 files.
- `crates/xoft-core` has one module (`corpus`); the codec, serializer and diagnostics do not
  exist yet — those are M2/M3.
- `crates/xoft-testbed` does not exist yet by design (M6).
- The corpus is *not* in this repo. `corpus/roots.toml` holds the four absolute paths; if the
  machine changes, that is the only file to edit.

## After M1.1

M1.2a (declarations — receivers, forward declarations), then M1.2b (statements — `WITH`,
`LOOP`, `EXIT`, `RETURN`, empty statements), then M1.2c (expressions and types), then M1.3 (the
external scanner for nested comments). M1.3 is the highest-risk item in M1; do not leave it
last if time is short.
