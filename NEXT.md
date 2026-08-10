# Next task

**M1.4 continued — sample `sweep_corpus.py`'s remaining failures in `amiga-oberon-31` and `voc`
(neither has had a dedicated round since round 12 and never, respectively), find the next
cluster, implement it test-first.**

Round 17 implemented STJ-Oberon's `AND`/`NOT` keyword operators (textual synonyms for `&`/`~`),
jumping the pass rate from 60.61% to 66.41%. This round starts cold again: there is no known
next construct, only the same method used since round 8.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 526/792 (66.41%), up from 480/792 (60.61%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it, expect several more
  rounds of clustering.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings.
- **Round 17's fix**: `mul_operator` gained `$.kAnd` (`'AND'`) as a sibling of `'&'`; `factor`
  gained `seq($.kNot, $.factor)` as a sibling of `seq('~', $.factor)` — STJ-Oberon (Atari ST)
  accepts `AND`/`NOT` as textual synonyms for `&`/`~`, confirmed via corpus grep (both spellings
  coexist in the same files) and the compiler's own embedded keyword table inside two `.OBJ`
  binaries in the `stj` root. Two new keyword tokens, no scanner change, zero `tree-sitter
  generate` conflicts.
- **Post-round-17 failure counts by root**: `amiga-oberon-31` 92, `oberon-a` 72, `stj` 59, `voc`
  43 (regenerate before trusting exact numbers, but the relative picture holds: `stj` dropped
  from 105 → 59, the other three roots are untouched by round 17's fix since it was a `stj`-only
  keyword). `amiga-oberon-31`'s last dedicated sampling round was round 12 (bodiless procedure
  headings); `voc` has never had a dedicated sampling pass. Both are now better candidates than
  `stj`, though `stj` (59 files) may still have more clusters worth a quick look first since it's
  fresh in context.
- **A `MISSING` node at a column that doesn't obviously match its named token is a signal the
  grammar has no rule at all for some nearby operator/keyword** (round 17 insight) — GLR error
  recovery reports wherever it guessed a plausible continuation, not necessarily where the real
  problem is. Bisect a minimal repro (delete clauses one at a time) rather than trusting the
  reported location literally. This is a *different* signature from round 15's narrow
  single-line `ERROR` landing exactly on a known keyword — that one *does* land at the real
  problem (a fixed-order/cardinality bug in an existing rule); this one (`MISSING` on an
  unrelated token) usually means a wholly unhandled token nearby.
- **A dialect's own compiled binaries in the corpus can double as a keyword-list source** when no
  `.doc`/`.txt` manual exists — `grep -a` over `.OBJ`/binary files sometimes surfaces an embedded
  plaintext string table (round 17: `stj`'s `MAKE2PAR.OBJ`/`OCSTAT.OBJ` listed `AND`/`NOT`
  alongside `DIV`/`MOD` as reserved words).
- **Lexical keyword synonyms for an operator the grammar already models don't need a scoping
  conversation with the user** — only structural extensions (new type kind, new statement form
  needing scanner work, like `STRUCT`/`ASSEMBLER` in round 9) do. A same-semantics, same-position,
  new-token-only change is squarely inside D1's "lexical superset" scope; implement directly.
- **A "cluster looks cleared" claim from a stale round needs a fresh per-file `grep -A1` check,
  not just a re-run of the aggregate number** (round 16 insight) — verify with
  `grep -A1 "^  <root>/<dir>" /tmp/sweep_v.txt | wc -l` before trusting a directory is clear.
- **A narrow single-line `(ERROR [n,0]-[n,end])` landing exactly on a keyword the grammar already
  has a rule for is a fixed-order/cardinality bug signature, not a new construct** (round 15
  insight) — check whether the existing rule's `optional`/`seq`/`repeat` shape matches the
  baseline EBNF's actual repetition/ordering before assuming something new is missing.
- **A mark character can mean different things in different grammar positions** (round 16
  insight) — check position, not just token identity, and cross-check the dialect's own manual
  before guessing meaning from position alone.
- **Oberon-A ships its own compiler manual** (`Oberon-A/docs/OC.doc`, per `corpus/roots.toml`'s
  `oberon-a` path). `stj` has no manual (confirmed again round 17, via the `.OBJ` embedded
  keyword table instead) — check for one in `amiga-oberon-31`/`voc` too, don't assume absent.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` — always
  `grep -rla` (or `-a` with whatever other flags) when grepping corpus roots directly;
  `sweep_corpus.py` itself already transcodes.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension, grep `docs/language-baseline.md` for it first — if it's already
  there, it's just an incomplete/incorrect grammar rule, implement it directly.
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12). If a new
  task reuses another rule across two enclosing contexts that share a token prefix, expect
  `tree-sitter generate` to report the same class of "Unresolved conflict" error.

## How to find the next cluster (reproduction, same method as rounds 8–17)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first to confirm amiga-oberon-31/voc are still the least-recently sampled:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
# Then skim for short, whole-file "(ERROR [0, 0] - [N, 0])" failures — those are usually a single
# early construct the grammar doesn't know at all, easiest to isolate. Also check narrow
# single-line spans landing on a known keyword (round 15's signature), and MISSING nodes at a
# column that doesn't match their named token (round 17's signature — bisect a minimal repro).
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # find the ERROR/MISSING node, read the surrounding source
```

Corpus files are Latin-1; always transcode before feeding to `tree-sitter parse`, and use
`grep -a` (not just `-r`) when grepping raw corpus text directly.

Cross-check any candidate against `docs/language-baseline.md` (the normative Oberon-2 EBNF)
first — read brace nesting literally, not from memory — and check for a `*.doc`/`*.txt` compiler
manual (or, per round 17, an `.OBJ` binary's embedded string table) in the relevant corpus root
before inferring a dialect construct's shape purely from usage. Only flag a scoping question to
the user the way round 9 did for `STRUCT`/`ASSEMBLER` when the construct is genuinely absent from
the baseline *and* structural (a new type kind, a new statement form needing scanner work) — a
lexical keyword synonym for an existing operator (round 17's `AND`/`NOT`) does not need this.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case for whatever construct is
  implemented, filled either via `tree-sitter test --update` and read back to confirm no
  `ERROR`/`MISSING` nodes, or hand-written by copying a structurally identical existing test's
  shape verbatim.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–17).
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 17 — the `AND`/`NOT` keyword-synonym fix and the
  "MISSING node at unrelated column" bisection lesson.
- `docs/insights.md` rounds 15–17 — the narrow-ERROR-on-known-keyword signature, the
  MISSING-node-at-unrelated-column signature, the ".OBJ binaries as keyword-list source" trick,
  and the "lexical synonym doesn't need scoping conversation" distinction.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 66.41%).

## State of the tree

- `grammar.js`: `case_statement` supports `[ELSE StatementSeq]` (round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` (round 12),
  with a top-level `conflicts: $ => [...]`. Round 14 added: `sysflag`, `square_vector_offset`,
  `external_code_names`, `reg_spec` (Oberon-A square-bracket family). Round 15: all three
  declaration-sequence sites use `repeat(choice($.const_decls, $.type_decls,
  $.variable_decls))`. Round 16: `procedure_heading` has `optional($.kStar)` right after
  `$.kProcedure` (Oberon-A assignable-procedure mark). Round 17 added: `mul_operator` gained
  `$.kAnd` as a `'&'` sibling; `factor` gained `seq($.kNot, $.factor)` as a `'~'`-factor sibling;
  two new keyword tokens `kAnd => 'AND'`, `kNot => 'NOT'` in the keyword section.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **66.41% (526/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
