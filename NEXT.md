# Next task

**M1.4 — `INLINE` opaque token + a parse-only corpus sweep.**

Everything a fresh session needs to start cold is below.

## What to do

Working tree: `grammars/tree-sitter-oberon2/`. Run `tree-sitter generate && tree-sitter test`
from that directory after every change (`src/parser.c`, `src/grammar.json`,
`src/node-types.json`, `src/tree_sitter/` are gitignored, generated locally; `src/scanner.c` is
real source and IS committed — see this round's `.gitignore` fix in `docs/errors.md` round 6 if
that distinction looks surprising).

Per `docs/plan.md`'s M1.4 row, this task "needs a parse-only corpus script or M4.1" — M4 (the
real corpus runner) doesn't exist yet, and building it fully is out of scope here. Write the
minimal version: a script (shell or a small Rust/Python one-off, doesn't need to live in
`xoft-cli`) that reads `corpus/manifest.json` (792 files, `cargo run -p xoft-cli -- corpus
manifest` regenerates it if stale), runs `tree-sitter parse <file>` against each, and reports
the `ERROR`-free percentage plus the file list that still fails. `docs/plan.md`'s M1 exit
criterion is "≥95% of corpus files parse with zero `ERROR`/`MISSING`" — this task's job is to
get an honest number for the first time (M1.1–M1.3 have only ever spot-checked 3–5 files) and
then close as much of the gap as `INLINE` accounts for.

1. Build the parse-only sweep script first (this is the "test" this round is test-first
   against, in spirit — you can't know if `INLINE` is fixed without a real pass-rate number
   before and after). Doesn't need to be polished; it's a throwaway tool, not a deliverable.
2. Run it once before touching the grammar. Record the baseline percentage and skim the failure
   list for patterns — `docs/plan.md` says `INLINE` appears in ~22 corpus files (Oberon-A,
   AmigaOberon), so it should show up as a cluster, not the whole gap.
3. Find a real `INLINE` block in the corpus (grep the Oberon-A or AmigaOberon roots — see
   `corpus/roots.toml` for paths) to confirm its actual syntax before writing a rule to swallow
   it: `docs/language-baseline.md` describes it as "opaque token, contents unparsed" but doesn't
   give the delimiter syntax, since it's a dialect extension, not in the normative EBNF.
4. Add it to `grammar.js` as an opaque token (likely another external-scanner token if it needs
   balanced-delimiter or keyword-terminated matching, or a plain regex `token()` if it's simpler
   than that — check the real syntax before assuming it needs scanner.c changes).
5. Re-run the sweep script. Compare before/after like this round's `git stash` before/after
   `ERROR`-count check (see `docs/insights.md` round 6) — cheap and answers "did this actually
   help" directly instead of trusting the corpus-test pass alone.

## Known gaps to triage during the sweep (not necessarily this round's job to fix)

Discovered by this round's (M1.3) spot-checks, confirmed real but out of that round's scope
(comments/pragmas only). The corpus sweep from step 1 will surface these plus others
systematically — decide file-by-file whether each is worth fixing now or logging for later,
same as `INLINE` itself:

- **`POINTER TO ARRAY OF Type`** (open array as a pointer's base type, no explicit `length`) —
  confirmed via `TYPE P = POINTER TO ARRAY OF INTEGER;` in isolation, still `ERROR`.
  `array_type` requires a `length` between `ARRAY` and `OF`; only `formal_type` (M1.2c) has the
  length-less shorthand, and only for formal parameters. Root cause of most of
  `MultiArrays.Mod`'s 28 remaining `ERROR` regions (see `docs/progress/m1-grammar.md` M1.3).
- **`string_literal` only matches double-quoted strings**, not single-quoted (`'...'`), even
  though `docs/language-baseline.md`'s lexical section allows both. Flagged in M1.2c's
  `Printer.Mod` spot-check, still not confirmed as an actual parse failure cause (plausible, not
  verified) — verify with an isolated case before fixing.
- **`<*STANDARD-*>`-style bracket pragmas** (`Printer.Mod`) — a different delimiter
  (`<* ... *>`) from the `(*$…*)` pragma M1.3 just implemented. Not in `docs/language-baseline.md`
  at all; if the corpus sweep shows this is common, it may need its own D1-style scoping
  decision (is this in scope for the lexical superset, or an allowlist entry per D8?) rather than
  being silently folded into M1.4.

## Definition of done

- The sweep script exists somewhere reproducible (doesn't need to be committed if it's truly
  throwaway, but committing it under, say, `grammars/tree-sitter-oberon2/` or a `scripts/`
  directory is probably worth it — M4 will want the same shape of tool later).
- `tree-sitter test` still green (35/35 before this round), plus new corpus cases for whatever
  `INLINE` syntax turns out to be.
- A recorded before/after `ERROR`-free percentage across the full 792-file corpus, not just the
  3-5 spot-check files used through M1.1–M1.3.
- No changes outside `grammars/tree-sitter-oberon2/` (and the throwaway sweep script, wherever
  it ends up) — this task doesn't touch `crates/`.

## Context a fresh session needs

- `docs/plan.md` — D1 (lexical superset scope), D8 (done criterion + allowlist), the M1 exit
  criterion ("≥95% ... zero ERROR/MISSING"), M1.4's row (needs a corpus script).
- `docs/language-baseline.md` — the dialect-extension table (INLINE row), and note that INLINE's
  actual syntax isn't in this doc (it's not normative EBNF) — has to come from the corpus itself.
- `docs/progress/m1-grammar.md` — M1.3's exact spot-check numbers and the newly-discovered
  `POINTER TO ARRAY OF Type` gap, so this round doesn't re-discover it from scratch.
- `docs/insights.md` round 6 — the `git stash` before/after technique, and the "an ERROR-cause
  attribution needs to be checked, not trusted" lesson (repeated across two rounds now — this
  round should not add a third instance).
- `docs/errors.md` round 6 — the external-scanner leading-whitespace gotcha, relevant if
  `INLINE` also needs an external scanner (e.g. if it's not a simple regex-matchable token).
- `CLAUDE.md` — test-first rule and the end-of-round ritual.

## State of the tree

- `grammar.js`: M1.1 base + M1.2a (receivers, forward decls, `DEFINITION` header) + M1.2b
  (`WITH`, `LOOP`, `EXIT`, `RETURN` as statements, empty statements) + M1.2c (procedure types in
  formal params; `field_list_seq` trailing-`;` fix) + M1.3 (external scanner: nested comments,
  `(*$…*)` pragma as a distinct node kind). 35/35 `tree-sitter test` green.
- `src/scanner.c` now exists and is tracked in git (fixed this round — previously `src/` was
  entirely gitignored, which was fine when everything in it was generated).
- `queries/highlights.scm` (M1.5) has `(pragma) @comment` alongside `(comment) @comment`.
- Rust workspace untouched since M0 — this task doesn't touch it.

## After M1.4

M1 should be at or near its exit criterion (≥95% `ERROR`-free). If the sweep shows it isn't,
the remaining gap needs to be triaged (fix vs. `corpus/allowlist.toml` entry per D8) before M1
can be declared done and M2 (lossless parse/serialize in `xoft-core`) starts.
