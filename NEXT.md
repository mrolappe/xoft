# Next task

**M1.3 — external C scanner for nested comments.**

Everything a fresh session needs to start cold is below.

## What to do

Working tree: `grammars/tree-sitter-oberon2/`. Run `tree-sitter generate && tree-sitter test`
from that directory after every change (`src/` is gitignored — generate it locally, it's not
there after a fresh checkout).

The current `comment` rule is a flat regex token:
`token(seq(/[(][*]([^*]*[*]+[^)*])*[^*]*[*]+[)]/))` — matches one `(* ... *)` pair, cannot
express nesting. Report §3.6 (normative, not a dialect quirk — see `docs/language-baseline.md`):
*"Comments may be inserted between any two symbols in a program. They are arbitrary character
sequences opened by the bracket `(*` and closed by `*)`. Comments may be nested."* 48 corpus
files actually nest comments (25 Oberon-A, 13 AmigaOberon, 10 STJ).

1. Add `src/scanner.c` (tree-sitter external scanner C API — `tree_sitter_oberon2_external_scanner_create/destroy/serialize/deserialize/scan`)
   that depth-counts `(*`/`*)` pairs. Reference: tree-sitter's own docs on external scanners, and
   any existing nested-comment scanner in the tree-sitter ecosystem (e.g. Lua, Pascal, Haskell
   grammars all have this exact problem — `{-# ... #-}` in Haskell nests the same way) is worth
   reading for the shape, not copying verbatim (different comment delimiters).
2. Wire it into `grammar.js`: `externals: $ => [$.comment]` (or a dedicated token name the
   scanner reports), replacing the current regex-token `comment` rule.
3. `(*$ ... *)` pragmas (Oberon-A/AmigaOberon/STJ) are, per D1 in `docs/plan.md`, "a distinct
   node kind, lexically a comment" — check whether the existing grammar already special-cases
   this anywhere (grep `grammar.js` for `pragma` — as of M1.2c it does not) and whether it's
   actually in scope for M1.3 or a separate item; the plan doc's D1 table lists it as a lexical
   superset requirement but doesn't assign it to a specific milestone. If out of scope, leave a
   note rather than silently dropping it.
4. `binding.gyp` / `package.json` may need updating so `tree-sitter generate` picks up the new
   `src/scanner.c` — check what upstream `viegasfh/tree-sitter-oberon-2` or a sibling grammar
   with an external scanner does for the node binding wiring, since M1 doesn't currently build a
   node addon (no `node-gyp rebuild` has been run yet in this project).

## Definition of done

- `tree-sitter test` still green (33/33 before this round; add corpus cases for nested comments,
  at minimum a `(* outer (* inner *) still outer *)` case and a `(*$ pragma *)` case if that's
  picked up too).
- Re-run the spot-check files (from `grammars/tree-sitter-oberon2`):
  `tree-sitter parse "/Users/mrolappe/studio/oberon-a-fs-uae-env/Oberon-A/source/ProjectOberon/Viewers.Mod"`,
  `.../git-repos/voc/src/library/misc/MultiArrays.Mod` (this is the file expected to finally lose
  most of its `ERROR` regions — it currently fails extensively, largely attributable to nested
  comments per M1.2c's spot-check), and `.../git-repos/voc/src/library/v4/Printer.Mod` (still
  expected to retain a couple of unrelated errors — a `<*STANDARD-*>`-style pragma and at least
  one that looks like a single-quoted string literal, `string_literal` in `grammar.js` currently
  only matches `"..."` not `'...'` even though `docs/language-baseline.md`'s lexical section
  documents both as legal — that's a separate, not-yet-triaged gap, not M1.3 scope unless picked
  up incidentally).
- No changes outside `grammars/tree-sitter-oberon2/` — this task doesn't touch `crates/`.

## Context a fresh session needs

- `docs/plan.md` — D1 (lexical superset scope), D2 (base grammar fork), milestone breakdown.
- `docs/language-baseline.md` — §3.6 comment nesting is normative; the "Comments" section has the
  exact quoted wording to build the scanner against.
- `docs/insights.md` round 1 — "Nested comments are normative Oberon-2, not a dialect quirk", the
  file-count breakdown by corpus root.
- `docs/insights.md` round 5 — the "misdiagnosed error" and "two shapes for the same syntax"
  lessons; worth re-applying the same discipline (isolate the construct, check every rule that's
  supposed to reach it) rather than assuming this milestone's scope is only the scanner file.
- `docs/progress/m1-grammar.md` — M1.2c's exact spot-check results, so this round doesn't
  re-diagnose the same residual errors from scratch.
- `CLAUDE.md` — test-first rule and the end-of-round ritual.

## State of the tree

- `grammars/tree-sitter-oberon2/grammar.js`: M1.1 base + M1.2a (receivers, forward decls,
  `DEFINITION` header) + M1.2b (`WITH`, `LOOP`, `EXIT`, `RETURN` as statements, empty statements)
  + M1.2c (procedure types in formal params; `field_list_seq` trailing-`;` fix). 33/33
  `tree-sitter test` green.
- Known, not-yet-fixed gaps, out of scope for M1.3 unless picked up incidentally:
  - `string_literal` regex only matches double-quoted strings (`"..."`), not single-quoted
    (`'...'`) even though the EBNF's lexical section allows both — flagged during this round's
    `Printer.Mod` spot-check, not yet confirmed as the actual cause (just a plausible one; verify
    before fixing).
- `queries/highlights.scm` (M1.5) not expected to need changes for M1.3's external-scanner-based
  comment token, since the node kind name (`comment`) doesn't have to change — check after.
- Rust workspace untouched since M0 — this task doesn't touch it.

## After M1.3

M1 is then feature-complete against `docs/plan.md`'s decision D1 scope. Re-run the full corpus
manifest / a full-corpus parse sweep (not just the five spot-check files used through M1.2) to
get a real `ERROR`-free percentage before declaring M1 done and moving to M2 (lossless
parse/serialize in `xoft-core`).
