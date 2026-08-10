# Next task

**M1.4 continued — implement `POINTER TO ARRAY OF Type` and single-quoted character literals.**
Both were carried over unconfirmed since M1.3/M1.2c; round 10 confirmed both as real, isolated
`ERROR`-causing gaps (not guesses) and sized them as small — this is "go implement," not a
scoping question. The `ASSEMBLER`/`STRUCT`/bracket-pragma/brace-annotation cluster is now fully
resolved (done, done, done, or explicitly deferred to Phase 2) — these two are what's left in
the triage table.

## What's confirmed (do not re-derive, just verify before coding)

- **`POINTER TO ARRAY OF Type`** (36 corpus files) — `array_type` (`grammars/tree-sitter-oberon2/
  grammar.js`) is `"ARRAY" length {"," length} "OF" type`, `length` mandatory. Real files use a
  length-less form as a pointer's base type: `TYPE P = POINTER TO ARRAY OF INTEGER;`. Isolating
  that exact line alone produces a real `ERROR` node (checked round 10, not assumed).
  `formal_type` already has this exact shorthand (`repeat(seq($.kArray, $.kOf))`, added M1.2c for
  formal parameters) — the fix is almost certainly widening `array_type` the same way, i.e. an
  alternative with no `length` at all, not touching `formal_type`. Grep the 36 files first (`grep
  -rl "POINTER TO ARRAY OF"` across the corpus roots per `corpus/roots.toml`) for any shape that
  isn't this simple length-less case before assuming it's a one-line grammar change.
- **Single-quoted character literals** (127 corpus files by a noisy substring grep — re-check
  count with a tighter pattern before trusting it, e.g. requiring exactly one char between quotes
  if that's what the corpus actually uses) — confirmed round 10 via `Tetriz.mod`'s `ORD('4')`,
  `ORD(' ')`, `ORD('q')`: genuine char-literal syntax, not apostrophes inside comments. The
  report's `string` production (`docs/language-baseline.md`) only has `'"' {char} '"'` and
  `digit {hexdigit} 'X'` — no single-quote form, so this is a real dialect extension, not a typo
  in the existing rule. `string_literal` in `grammar.js` (top of file, the `const` block) is
  where this most likely slots in as a third `choice` alternative — check first whether the
  corpus's single-quoted form allows multiple characters between quotes (Oberon string-typed) or
  is always exactly one (Oberon-2's `CHAR`-typed literal is usually single-char in other Pascal-
  family dialects) — that decides whether it's the same `string` node or needs its own.

## Order and scope

Independent changes, no reason not to do both in one round — neither touches the other's rule.
Do whichever is faster to confirm first; suggest `POINTER TO ARRAY OF Type` first since it's
already half-precedented by `formal_type`'s existing shorthand.

## Definition of done

- `tree-sitter test` still green, plus one new corpus case per item using a real shape from the
  corpus (same practice every round since M1.4 has used — prefer `tree-sitter test --update`
  against a minimized real snippet, read back once to confirm no `ERROR`/`MISSING` nodes, per
  `docs/insights.md` round 3's note on trusting `--update`).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  section, same format as round 10's).
- Update the triage table in `docs/progress/m1-grammar.md` (round 10's version, end of file).
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md`'s "M1.4 continued — `ASSEMBLER` blocks" section (round 10) — full
  detail on what was confirmed about both carried-over items, and the exact corpus-impact numbers
  so far (29.29%, 232/792).
- `docs/insights.md` round 10 — a note on a zero-length-token trap in external scanners; not
  directly relevant to these two items (neither needs a scanner — both are plain grammar/lexer
  widenings, `choice`/`repeat` additions, no new external token) but worth knowing the general
  shape of external-scanner work already done in `src/scanner.c` if either item turns out to need
  more than expected.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 29.29%).

## State of the tree

- `grammar.js`: M1.1 base through M1.4's `ASSEMBLER` (round 10) — four external tokens
  (`comment`, `pragma`, `bracket_pragma`, `assembler_body`), `assembler_statement` in the
  `statement` choice. Neither `array_type`'s length-less form nor single-quoted char literals
  exist yet.
- `src/scanner.c`: four external tokens (`COMMENT`, `PRAGMA`, `BRACKET_PRAGMA`,
  `ASSEMBLER_BODY`). Neither of this round's two items needs a fifth — both are ordinary
  grammar/token widenings, not raw-scan-to-a-delimiter problems.
- `sweep_corpus.py`: unchanged this round, still the way to measure before/after. Baseline for
  the next round: 29.29% (232/792).
- Rust workspace untouched since M0 — this task doesn't touch it.
