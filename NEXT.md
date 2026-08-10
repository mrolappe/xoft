# Next task

**M1.2a — declarations: type-bound procedure receivers, forward declarations, `DEFINITION`
module header.**

Everything a fresh session needs to start cold is below.

## What to do

Working tree: `grammars/tree-sitter-oberon2/grammar.js`. Run `tree-sitter generate &&
tree-sitter test` from that directory after every change (`src/` is gitignored — generate it
locally, it's not there after a fresh checkout).

1. **Receiver** on `ProcDecl` and `ForwardDecl`:
   `Receiver = "(" [VAR] ident ":" ident ")".`
   Add an optional `$.receiver` between `kProcedure` and `ident_def` in `procedure_heading`
   (currently `kProcedure, ident_def, optional(formal_params)` — see `grammar.js`). Watch for a
   grammar conflict against `formal_params`, which also starts with `"("` — both are optional
   and adjacent, so the parser needs to disambiguate on the `VAR`/ident-then-`:` shape inside.
2. **Forward declaration**: `ForwardDecl = PROCEDURE "^" [Receiver] IdentDef [FormalPars].`
   Add a `forward_decl` rule and fold it into wherever `procedure_decls` currently is (module
   and procedure_body both repeat `procedure_decls` — check whether forward decls belong in the
   same `repeat` or need their own, since the EBNF's `DeclSeq` allows both `ProcDecl ";"` and
   `ForwardDecl ";"` interleaved).
3. **`DEFINITION` module header** (STJ-Oberon variant, not in the Oberon-2 report — see
   `docs/language-baseline.md` lines 20–28): a second acceptable keyword where `module_header`
   currently hardcodes `kModule`. Grep the STJ corpus for a real example before guessing the
   syntax — don't invent it from the label alone.

## Definition of done

- `tree-sitter test` still green on the 5 upstream corpus files, plus new corpus cases you add
  for each construct (one file per construct is enough — `test/corpus/receivers.txt`,
  `test/corpus/forward_decl.txt`, `test/corpus/definition_module.txt`).
- `docs/insights.md` line 12–13 (receiver gap, "38 Oberon-A, 70 STJ and 21 AmigaOberon files")
  can be checked off — spot-check a real receiver method from one of those files parses clean
  with no `ERROR` node (`tree-sitter parse <file>`).
- No changes outside `grammars/tree-sitter-oberon2/` — this task doesn't touch `crates/`.

## Context a fresh session needs

- `docs/plan.md` line 85: `M1.2a Declarations: type-bound PROCEDURE (r: T) M*, ForwardDecl ^,
  DEFINITION header | Sonnet | receives the declaration EBNF only`. Model: Sonnet, not Haiku —
  this one needs grammar-conflict judgment, unlike M1.1.
- `docs/language-baseline.md` lines 39–49 — the `DeclSeq`/`ProcDecl`/`ForwardDecl`/`Receiver`
  EBNF fragment, already extracted, no need to fetch the report.
- `docs/insights.md` — "Type-bound procedure receivers" and "Empty statements are legal" entries
  (the latter is M1.2b's problem, not this one — don't fix it here, it'll conflict with that
  task's diff).
- `docs/progress/m1-grammar.md` — what M1.1/M1.5 already did.
- `CLAUDE.md` — test-first rule (write the corpus case, see it fail — `tree-sitter test` will
  show a parse error or wrong tree — then change `grammar.js`) and the end-of-round ritual.

## State of the tree

- `grammars/tree-sitter-oberon2/` vendored from `viegasfh/tree-sitter-oberon-2` (MIT), building
  clean under tree-sitter 0.26.11, 14/14 upstream corpus tests green. `NOTICE` has both fork
  attributions.
- `queries/highlights.scm` (M1.5) done — adapted from `tree-sitter-oberon-07`, hand-checked
  against this grammar's actual node names and fields (they diverge from upstream's query
  despite the shared skeleton — see `docs/insights.md`).
- Known, confirmed-by-hand gap (not this task): `Out.Ln;` immediately before `END` produces an
  `ERROR` node — `statement_seq` has no empty-statement alternative. That's M1.2b.
- Rust workspace (`crates/xoft-core`, `crates/xoft-cli`) untouched since M0 — this task doesn't
  touch it.

## After M1.2a

M1.2b (statements — `WITH`, `LOOP`, `EXIT`, `RETURN`, `CASE` label ranges, empty statements),
then M1.2c (expressions/types — `IS`, `SET` literals with ranges, open arrays, procedure types),
then M1.3 (external C scanner for nested comments — highest-risk item in M1, don't leave it
last if time is short).
