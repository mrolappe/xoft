# Progress

Overall state of the xoft MVP. One line per milestone; details in the per-phase files.

| Milestone | State | Detail |
|---|---|---|
| M0 — Foundations | ✅ done | [docs/progress/m0-foundations.md](docs/progress/m0-foundations.md) |
| M1 — Grammar | 🟨 in progress (M1.1, M1.2a, M1.2b, M1.2c, M1.3, M1.5 done; M1.4 sweep tool + `INLINE` + bracket pragmas + brace annotations + `ASSEMBLER` + `POINTER TO ARRAY OF Type` + single-quoted strings + bodiless procedure headings + `CASE...ELSE` + Oberon-A system flags/square-bracket library calls + repeated/interleaved decl sections + Oberon-A assignable-procedure mark + STJ-Oberon `AND`/`NOT` keyword operators done, ≥95% exit criterion not yet met — 66.41%, see below) | [docs/progress/m1-grammar.md](docs/progress/m1-grammar.md) |
| M2 — Core: lossless parse/serialize | ⬜ not started | — |
| M3 — Diagnostics and CLI | ⬜ not started | — |
| M4 — Corpus runner | ⬜ not started | — |
| M5 — Toy dialect Oberon-X | ⬜ not started | — |
| M6 — Testbed | ⬜ not started | — |
| M7 — Phase 2 plan | ⬜ not started | — |

**Next task:** see [NEXT.md](NEXT.md).

Cross-cutting records: [insights](docs/insights.md) · [errors and mitigations](docs/errors.md) ·
[plan and decisions](docs/plan.md) · [language baseline](docs/language-baseline.md)

## Rounds

| # | Date | Covered |
|---|---|---|
| 1 | 2026-08-10 | Project bootstrap, M0 complete |
| 2 | 2026-08-10 | M1.1 base grammar vendored and building; M1.5 highlights.scm |
| 3 | 2026-08-10 | M1.2a — receivers, forward declarations, DEFINITION module header |
| 4 | 2026-08-10 | M1.2b — WITH, LOOP, EXIT, RETURN statements, empty statements, CASE label ranges confirmed |
| 5 | 2026-08-10 | M1.2c — IS, SET, open arrays confirmed as already working; procedure types fixed in `formal_type`; incidental fix for `field_list_seq` trailing-semicolon gap |
| 6 | 2026-08-10 | M1.3 — external C scanner for nested comments + `(*$…*)` pragma node; `.gitignore` fix so `scanner.c` is tracked |
| 7 | 2026-08-10 | Infra: generated tree-sitter output moved to `gen-src/` (gitignored), `src/` now holds only hand-written `scanner.c` plus symlinks into `gen-src/` so `tree-sitter test`/`parse` keep working unmodified |
| 8 | 2026-08-10 | M1.4 — corpus sweep script (`sweep_corpus.py`), first honest full-corpus number (15.78% → 21.97%); fixed `ELSIF`/`ELSEIF` keyword typo, hex-integer-literal token bug (this is what actually blocked `INLINE`, which needed no new grammar), two AmigaOberon `IMPORT` rename/re-export variants; triaged (not fixed) bracket pragmas (212 files), `STRUCT`, brace-annotated procedures, `ASSEMBLER` |
| 9 | 2026-08-10 | M1.4 continued — scoping decision flagged to and resolved by user (`STRUCT` deferred to Phase 2, `ASSEMBLER` deferred pending scanner work); implemented `<* ... *>` bracket pragmas (third external scanner token) and AmigaOberon brace annotations (`vector_offset`, `param_offset`); 21.97% → 27.15% (174 → 215/792) |
| 10 | 2026-08-10 | M1.4 continued — implemented `ASSEMBLER` blocks (fourth external scanner token, word-boundary raw-scan to `END`); 27.15% → 29.29% (215 → 232/792); confirmed the two carried-over items (`POINTER TO ARRAY OF Type`, single-quoted char literals) as real `ERROR`-causing gaps, not yet implemented |
| 11 | 2026-08-10 | M1.4 continued — implemented `POINTER TO ARRAY OF Type` (length made optional in `array_type`) and single-quoted strings (widened `string_literal`, not a separate `CHAR` type — round 10's "no single-quote form in the report" claim was wrong, corrected against `docs/language-baseline.md` and corpus FourCC evidence); 29.29% → 30.68% (232 → 243/792); triage table from rounds 9/10 now fully resolved |
| 12 | 2026-08-10 | M1.4 continued — implemented AmigaOberon's bodiless procedure heading (`Interfaces/*.mod` system-call wrappers), reusing `definition_proc_decl` as a third `procedure_decls` alternative and adding a `conflicts` declaration for the resulting GLR ambiguity with `procedure_decl`; 30.68% → 36.36% (243 → 288/792) |
| 13 | 2026-08-10 | M1.4 continued — implemented `CASE ... ELSE ... END`, a normative Oberon-2 EBNF construct the grammar was simply missing (not a dialect scoping question); `case_statement` gained the same optional `ELSE` arm `if_statement` already had; 36.36% → 39.39% (288 → 312/792) |
| 14 | 2026-08-10 | M1.4 continued — implemented Oberon-A's square-bracket dialect family from `Oberon-A/docs/OC.doc`: `sysflag` on `MODULE`/`POINTER`/`RECORD`/`PROCEDURE`, `square_vector_offset` and `external_code_names` on procedure headings, `reg_spec` (with vararg `..` marker) on formal parameters; 39.39% → 41.41% (312 → 328/792) |
| 15 | 2026-08-10 | M1.4 continued — fixed `DeclSeq` to allow repeated/interleaved `CONST`/`TYPE`/`VAR` sections per the normative baseline EBNF's outer `{}` (was fixed-order, one of each); a plain grammar bug, not a dialect extension; 41.41% → 54.42% (328 → 431/792), +103 files, largest single-round gain to date |
| 16 | 2026-08-10 | M1.4 continued — implemented Oberon-A's "assignable procedure" mark (`PROCEDURE* [sysflag] ident`, `*` right after the `PROCEDURE` keyword, per `docs/OC.doc` "AssignableProcs"); reused the existing `kStar` token, no scanner change; 54.42% → 60.61% (431 → 480/792), +49 files |
| 17 | 2026-08-10 | M1.4 continued — first sampling pass over `stj` (Atari ST Oberon); implemented `AND`/`NOT` as textual synonyms for `&`/`~` (STJ dialect extension, confirmed via corpus grep and the compiler's own embedded keyword table); two new keyword tokens, no scanner change; 60.61% → 66.41% (480 → 526/792), +46 files, all in `stj` |
