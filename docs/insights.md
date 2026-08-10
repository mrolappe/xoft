# Insights

Things learned that were not obvious beforehand. Newest round last.

## Round 1 — 2026-08-10

### The existing tree-sitter Oberon-2 grammar is a starting point, not a solution

`viegasfh/tree-sitter-oberon-2` (MIT, 500 lines, last touched 2023) covers roughly 60% of what
is needed. Confirmed missing against the report's EBNF:

- **Type-bound procedure receivers** — `PROCEDURE (r: T) M*`. `procedure_heading` is
  `kProcedure ident_def [formal_params]`, with no `Receiver`. Used in 38 Oberon-A, 70 STJ and
  21 AmigaOberon files, so this is not an edge case.
- **`WITH`, `LOOP`, `EXIT`, and `RETURN` as a statement** — the `statement` rule lists only
  assignment, call, `IF`, `CASE`, `WHILE`, `REPEAT`, `FOR`.
- **Forward declarations** (`PROCEDURE ^ …`).
- **Nested comments** — the comment token is a flat regex, which cannot nest.

Everything else (module structure, declarations, expressions, designators, precedence) is sound
and worth keeping. `geekstakulus/tree-sitter-oberon-07` is the same skeleton plus a
`queries/highlights.scm` worth porting.

### Nested comments are normative Oberon-2, not a dialect quirk

Report §3.6: *"Comments may be nested."* 48 corpus files actually do it (25 Oberon-A, 13
AmigaOberon, 10 STJ). A regex token cannot express nesting, so an external C scanner is
mandatory rather than a nice-to-have. This was the single most likely source of a late,
expensive surprise.

### Empty statements are legal and the corpus relies on them

In the EBNF, `Statement = [ … ]` — the whole production is optional. So `BEGIN ; END` and a
trailing `;` before `END` are both valid. A grammar that requires a statement will emit `ERROR`
nodes on real files for a reason that looks like a corpus problem rather than a grammar bug.

### `SYSTEM` is not a reserved word

It is an ordinary module identifier (report Appendix C). `SYSTEM.ADR(x)` parses as a plain
qualified designator with no grammar support at all. 167 Oberon-A and 169 STJ files import it —
none of that needs grammar work; the dialect-specific *procedures* are a Phase 2 catalog
concern, not a parsing one.

### The corpus's encodings are not one problem but three

Oberon-A and AmigaOberon are Latin-1 (`0xFC` = ü, `0xA9` = ©); STJ is an Atari codepage where
the same characters are `0x81`/`0x94`/`0x84` (CP437-like); voc is UTF-8. STJ is also uniformly
CRLF while everything else is LF. Any design that has to *know* the charset needs three tables
and a detection heuristic. Mapping bytes `0x00-0xFF` to `U+0000-U+00FF` instead is a total
bijection that needs none of that, and is safe precisely because Oberon identifiers are ASCII —
high bytes only ever appear inside comments and strings (decision D3).

### A bin-only crate cannot be tested from `tests/`

Rust integration tests can only import a library target. `xoft-cli` therefore has both
`src/main.rs` and `src/lib.rs`, with the logic in the lib and `main.rs` reduced to argument
parsing. Worth doing from the start rather than retrofitting.

### The STJ corpus exists twice on disk

`~/sandkasten/tmp-stj-oberon-prj/OBERON_I` and `~/atari-retro-dev/c-drv/OBERON_I` are identical
apart from a `.DS_Store`. `corpus/roots.toml` uses the `atari-retro-dev` copy; the other is
ignored. Without this, the corpus would have been double-counted at 1098 files.

### The upstream grammar needed zero rule changes for tree-sitter 0.26

`tree-sitter generate` on `viegasfh/tree-sitter-oberon-2` as-is, under CLI 0.26.11, produces
only warnings (ABI 14 fallback for lacking `tree-sitter.json`; one redundant `seq` in `comment`)
— no errors, no conflicts. The "written against ~0.20, expect breakage" premise in the M1.1 task
brief did not hold. Worth remembering when scoping future mechanical tasks: verify before
padding the estimate for a CLI-version gap.

## Round 3 — 2026-08-10

### `DEFINITION` isn't just an alternate `MODULE` keyword

The task brief framed it as "a second acceptable keyword where `module_header` currently
hardcodes `kModule`." Grepping the real STJ corpus first (per the task's own instruction)
showed that's wrong: procedure declarations inside a `DEFINITION` module have no body at all —
`PROCEDURE Open;` then straight to the next declaration, never `END ident`. A label on a task
brief is a hypothesis, not a spec; the EBNF-adjacent framing ("second keyword") undersold a
structurally different declaration form one level down. Cost nothing to check — one `grep -rn`
before writing any grammar.js — and would have shipped a grammar that still `ERROR`s on 70 of
112 real `.DEF` files if trusted at face value.

### The apparent receiver/formal_params conflict wasn't real

`NEXT.md` flagged a likely grammar conflict between `receiver` and `formal_params`, both
starting with `"("`. They never actually collide: `receiver` sits *before* the mandatory
`ident_def` in `procedure_heading` and `formal_params` sits *after* it, so a `"("` seen
immediately after `kProcedure` is unambiguously `receiver` (nothing else can start there, since
`ident_def` can't begin with `"("`). `tree-sitter generate` produced zero conflicts on the
first attempt. Worth remembering: a flagged risk from a handoff document is a place to look
carefully, not a guarantee of an actual problem — check by running the tool before spending
design effort pre-emptively working around a conflict that may not exist.

### A second gap in the same shape as the already-known one

The empty-statement gap (`StatementSeq` requires a `Statement`, but the EBNF makes the whole
production optional) has a sibling: `FieldList` inside a `RECORD` has the same issue — real
corpus files (`AES.DEF`, `PROCLIST.DEF`) put a trailing `";"` before `END` after the last field,
which `field_list_seq` (no trailing-separator alternative) rejects. Same root cause — a
grammar rule modeling a "list with separators" as `item {sep item}` when the source allows
`item {sep item} [sep]` — recurring in more than one place in the same EBNF. Worth checking all
list-like productions (`FieldList`, `StatementSeq`, `DeclSeq`, `CaseLabelList`, …) for the same
optional-trailing-separator shape in one pass when M1.2b picks this up, rather than
rediscovering each instance one corpus file at a time.

### Two forks of the same grammar still diverge on field names

`geekstakulus/tree-sitter-oberon-07`'s `queries/highlights.scm` cannot be copied verbatim onto
`viegasfh/tree-sitter-oberon-2` even though both descend from the same EBNF-to-grammar shape and
share most rule names (`module_header`, `ident_def`, `qualident`, …). The 07 fork adds field
labels (`param:`, `paramtype:`, `returntype:`) and a `base_type` wrapper around builtin
qualidents that this grammar doesn't have. `tree-sitter query <file> <source>` will happily
report 0 matches instead of erroring when a query pattern's shape just never occurs — silent,
not loud. Always cross-check against `node-types.json` fields before trusting a ported query,
and smoke-test it on a real source file, not just the corpus.
