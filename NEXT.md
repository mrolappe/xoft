# Next task

**M1.2c — expressions/types: `IS`, `SET` literals, open arrays, procedure types.**

Everything a fresh session needs to start cold is below.

## What to do

Working tree: `grammars/tree-sitter-oberon2/grammar.js`. Run `tree-sitter generate &&
tree-sitter test` from that directory after every change (`src/` is gitignored — generate it
locally, it's not there after a fresh checkout).

1. **`SET` as a type** — `Type = Qualident | ARRAY ... | RECORD ... | POINTER TO Type |
   PROCEDURE [FormalPars]` in `docs/language-baseline.md` doesn't list `SET` as a builtin type
   keyword at all in this EBNF fragment — check the lexical/type section of
   `docs/language-baseline.md` again for where `SET` (and other builtins like `INTEGER`,
   `BOOLEAN`, `CHAR`, `REAL`, `BYTE`, `LONGINT`, etc.) are defined; they're likely just ordinary
   predeclared identifiers matched by `qualident`, not grammar keywords — confirm before adding
   a `kSet` token. `voc/src/library/v4/Printer.Mod` has `used: ARRAY 8 OF SET;` (grep hit from
   M1.2b's spot-check, still failing with `SET` unrecognized) — use it as a real test case
   either way.
2. **`IS` in expressions** — `relation` already includes `$.kIs` (grammar.js ~line 292) and
   `kIs` is defined. Check whether this already works end-to-end with a real corpus example
   (`x IS SomeType`) before assuming it needs work — same "confirm before coding" discipline as
   M1.2a/M1.2b. If it already works, this item may just need a corpus test, not a grammar
   change.
3. **Open arrays** (`ARRAY OF Type` in formal parameter positions, no length) — `formal_type`
   already has `repeat(seq($.kArray, $.kOf))` prefix (grammar.js ~line 217), which looks like
   it already covers this. Confirm against real corpus usage (`ARRAY OF CHAR` is extremely
   common — any STJ or Oberon-A file with a string parameter) before writing new grammar.
4. **Procedure types** — `procedure_type = "PROCEDURE" [formal_params]` already exists
   (grammar.js ~line 192-194) and is wired into `struct_type`. Confirm with a real corpus
   example of a `PROCEDURE(...)` type used as a field or parameter type (`voc/src/library/
   misc/MultiArrays.Mod` uses `f: PROCEDURE(s1,s2:SHORTINT): SHORTINT` as a parameter — grep
   confirmed this during M1.2b's spot-check) before assuming more work is needed.

**Read the grammar before writing anything** — several of these items may already be
implemented and this milestone's real work may be narrower than its name suggests (same lesson
as M1.2a's `DEFINITION` header and M1.2b's `CASE` label ranges, both of which turned out to
already exist). Grep the corpus for each construct, try parsing a real example, and only write
grammar.js changes for what's actually still broken.

## Definition of done

- `tree-sitter test` still green (27/27 before this round; add corpus cases per construct
  actually touched).
- Re-run the M1.2b spot-check files (from `grammars/tree-sitter-oberon2`):
  `tree-sitter parse "/Users/mrolappe/studio/oberon-a-fs-uae-env/Oberon-A/source/ProjectOberon/Viewers.Mod"`,
  `.../git-repos/voc/src/library/misc/MultiArrays.Mod` (still expected to fail on nested
  comments, M1.3 scope — that's fine), and `.../git-repos/voc/src/library/v4/Printer.Mod` (the
  `ARRAY 8 OF SET` one) — confirm the `SET`-related `ERROR` is gone and no new ones appeared.
- No changes outside `grammars/tree-sitter-oberon2/` — this task doesn't touch `crates/`.

## Context a fresh session needs

- `docs/plan.md` — milestone breakdown and decisions D1-D8.
- `docs/language-baseline.md` — the full EBNF; re-check the lexical section for how builtin
  types (`SET`, `INTEGER`, etc.) are actually specified, not just the `Type` production quoted
  above.
- `docs/insights.md` round 4 — the `tree-sitter test --update` workflow for generating corpus
  tests from real-shaped snippets without hand-writing trees, and the "widening every element to
  optional can make a rule match the empty string" gotcha (run `tree-sitter generate`
  immediately after any such change).
- `docs/progress/m1-grammar.md` — M1.2b's exact spot-check results (which real corpus files
  have which residual `ERROR` and why), so this round doesn't re-diagnose the same errors from
  scratch.
- `CLAUDE.md` — test-first rule and the end-of-round ritual.

## State of the tree

- `grammars/tree-sitter-oberon2/grammar.js`: M1.1 base + M1.2a (receivers, forward decls,
  `DEFINITION` header) + M1.2b (`WITH`, `LOOP`, `EXIT`, `RETURN` as statements, empty
  statements, confirmed `CASE` label ranges). 27/27 `tree-sitter test` green.
- Known, not-yet-fixed gap, same shape as the (now-fixed) statement one: `field_list_seq` (a
  `RECORD`'s fields) rejects a trailing `";"` before `END` — flagged during M1.2a, still open,
  out of scope for M1.2c unless picked up incidentally.
- `queries/highlights.scm` (M1.5) not expected to need changes for M1.2c's new/confirmed node
  kinds unless new keyword tokens are introduced (e.g. `kSet`, if that turns out to be needed)
  — check after.
- Rust workspace untouched since M0 — this task doesn't touch it.

## After M1.2c

M1.3 (external C scanner for nested comments — highest-risk item in M1, don't leave it last if
time is short; the `MultiArrays.Mod` whole-file parse failure from M1.2b's spot-check is a ready
real-world test case for it).
