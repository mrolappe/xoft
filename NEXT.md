# Next task

**M1.4 continued — implement the absolute hardware-address variable annotation
(`Ciapra[0BFE001H]: SHORTSET;`) found at the end of round 18 in `amiga-oberon-31/Demos/Sparks.mod`,
then keep sampling `amiga-oberon-31`'s remaining 13 non-`STRUCT` failures and `stj`/`voc` for the
next cluster.**

Round 18 implemented six fixes in `amiga-oberon-31` (module `CLOSE` sections, `LONGSET`/
`SHORTSET` typed set constructors, a cross-dialect real-number/range lexer bug, unsigned-hex `U`
integer suffix, curly-brace `external_code_names`, `param_offset` varargs marker), jumping the
pass rate from 66.41% to 69.95%. This round starts with a known lead already isolated.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 554/792 (69.95%), up from 526/792 (66.41%) last
  round. M1's exit criterion is ≥95% (`docs/plan.md`, D8) — still below it.
- **`STRUCT` and `UNTRACED POINTER`** are still scoped **out** of M1 to Phase 2 (round 9
  decision, reaffirmed every round since) — don't rediscover these as "new" findings. 60 of
  `amiga-oberon-31`'s remaining 73 failures are this; skip files containing either string when
  looking for the next cluster (`grep -qa "STRUCT\|UNTRACED" <file>`).
- **The known next lead**: `amiga-oberon-31/Demos/Sparks.mod` line 25,
  `Ciapra[0BFE001H]: SHORTSET;` inside a `VAR` section — a square-bracket absolute
  hardware-address annotation directly after a variable's identifier, before the `:`. Not yet
  implemented; check `grammar.js`'s current `var_decl`/`variable_decl` rule shape before editing
  (round 18 didn't touch this area). Grep the corpus for how common this is and whether the
  bracketed value is always a hex integer or can be other forms before designing the grammar
  rule; check for a scanner/doc source with the same technique as round 17 (`.OBJ` embedded
  string tables) if no plain `.doc`/`.txt` manual turns up.
- **Post-round-18 failure counts by root**: `amiga-oberon-31` 73 (13 non-`STRUCT`), `oberon-a`
  67, `stj` 57, `voc` 41 (regenerate before trusting exact numbers). `amiga-oberon-31` is still
  the best per-file yield (13 concentrated, identifiable failures) but `stj` (57, last dedicated
  round was 17) and `voc` (41, never dedicated) remain unsampled-relative-to-size and are fair
  alternatives if the `Sparks.mod` lead turns out to be a dead end or scoped out.
- **A grammar bug can be cross-dialect even when found while working one dialect's cluster**
  (round 18 insight) — the real-number/range lexer fix (`2..4` mis-tokenized) dropped failures
  in `oberon-a`/`stj`/`voc` too, not just `amiga-oberon-31`. After any fix, always re-tally
  failures by root before assuming the delta is isolated to the root you were sampling.
- **tree-sitter has no lookahead/lookbehind** (Rust's `regex` crate excludes it by design) — an
  ambiguous-lexeme bug can't be patched with a negative-lookahead regex trick the way you might
  in PCRE. The fix has to change what the grammar's token *rule* accepts (round 18: require ≥1
  digit after `real`'s decimal point) after confirming via corpus grep that the stricter
  language is still a superset of everything actually written.
- **Fixing one construct can leave a file failing at the exact same error location** if a second,
  unrelated construct sits immediately after it in the same span (round 18: `Alerts.mod`'s
  `curly_external_code_names` fix didn't move the error until `param_offset`'s `..` varargs
  marker was also added). Don't conclude a fix "didn't work" from an unchanged error location —
  isolate the fix alone in a minimal repro first to confirm it in isolation, then look for a
  second construct stacked in the same span.
- **An unscoped grep over a corpus root can match compiled binaries alongside source** (round 18:
  `grep -rlas` for the `U` hex suffix hit dozens of `.OBJ` files with coincidentally matching
  garbled bytes). Use `--include='*.mod'` (or the dialect's actual source extension) when
  grepping a root that mixes source and compiled artifacts — most of these retro-Oberon roots do.
- **A `MISSING` node at a column that doesn't obviously match its named token is a signal the
  grammar has no rule at all for some nearby operator/keyword** (round 17 insight) — bisect a
  minimal repro rather than trusting the reported location literally.
- **A dialect's own compiled binaries in the corpus can double as a keyword-list source** when no
  `.doc`/`.txt` manual exists (round 17 insight) — `grep -a` over `.OBJ`/binary files sometimes
  surfaces an embedded plaintext string table.
- **Lexical keyword synonyms for an operator the grammar already models don't need a scoping
  conversation with the user** — only structural extensions (new type kind, new statement form
  needing scanner work) do. `amiga-oberon-31`'s round-18 fixes (CLOSE section, typed sets,
  U-suffix, curly names, param varargs) were all judged in-scope directly under this same
  reasoning — none needed scanner work, all reused existing rules/tokens as siblings.
- **A "cluster looks cleared" claim from a stale round needs a fresh per-file `grep -A1` check,
  not just a re-run of the aggregate number** (round 16 insight).
- **A narrow single-line `(ERROR [n,0]-[n,end])` landing exactly on a keyword the grammar already
  has a rule for is a fixed-order/cardinality bug signature, not a new construct** (round 15
  insight).
- **Oberon-A ships its own compiler manual** (`Oberon-A/docs/OC.doc`). `stj` and `amiga-oberon-31`
  have no manual (confirmed both, via `.OBJ` embedded keyword tables / direct corpus inference
  instead) — check for one in `voc` too if working there, don't assume absent.
- **`grep -r` over the raw corpus silently skips Latin-1 files** unless given `-a` — always
  `grep -rla` (or `-a` with whatever other flags) when grepping corpus roots directly;
  `sweep_corpus.py` itself already transcodes.
- **Not every unhandled construct is a scoping question.** Before treating a gap as an
  out-of-scope dialect extension, grep `docs/language-baseline.md` for it first.
- **`procedure_decls` has a `conflicts` declaration** (`grammar.js`, added round 12).

## How to find the next cluster (reproduction, same method as rounds 8–18)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
# Tally failures by root first:
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
# Filter out STRUCT/UNTRACED (still Phase 2 scope) when sampling amiga-oberon-31:
grep "^  amiga-oberon-31/" /tmp/sweep_v.txt | sed 's#^  amiga-oberon-31/##' | while read -r f; do
  grep -qa "STRUCT\|UNTRACED" "<amiga-oberon-31 absolute path from roots.toml>/$f" || echo "$f"
done
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
tree-sitter parse /tmp/x.mod   # find the ERROR/MISSING node, read the surrounding source
```

Corpus files are Latin-1; always transcode before feeding to `tree-sitter parse`, and use
`grep -a` (not just `-r`, and add `--include=<ext>` to exclude compiled binaries — round 18
insight) when grepping raw corpus text directly.

Cross-check any candidate against `docs/language-baseline.md` first, and check for a
`*.doc`/`*.txt` compiler manual (or an `.OBJ` binary's embedded string table, round 17) before
inferring a dialect construct's shape purely from usage. Only flag a scoping question to the
user the way round 9 did for `STRUCT`/`ASSEMBLER` when the construct is genuinely absent from the
baseline *and* structural (new type kind, new statement form needing scanner work) — round 18's
six fixes all stayed in-scope without asking, following round 17's precedent.

## Definition of done

- `tree-sitter test` still green, plus at least one new corpus case per construct implemented,
  filled either via `tree-sitter test --update` and read back to confirm no `ERROR`/`MISSING`
  nodes, or hand-written by copying a structurally identical existing test's shape verbatim
  (grep the same test file for a similar existing case first — round 8's mitigation, still the
  right move; round 18 needed it for two of its five new tests).
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md` (new
  round section, same format as rounds 9–18). After any fix, re-tally failures by root — round
  18's real/range fix turned out to be cross-dialect even though found while sampling one root.
- Update `PROGRESS.md`'s round table and M1 line with the new percentage.
- No changes outside `grammars/tree-sitter-oberon2/`.

## Context a fresh session needs

- `docs/progress/m1-grammar.md` round 18 — all six fixes, the real/range lexer bug diagnosis
  (tree-sitter has no lookahead — the grammar-shape fix instead), and the `Sparks.mod` lead for
  the next round's address-annotation construct.
- `docs/insights.md` rounds 15–18 — the narrow-ERROR-on-known-keyword signature, the
  MISSING-node-at-unrelated-column signature, the ".OBJ binaries as keyword-list source" trick,
  "lexical synonym doesn't need scoping conversation", the lookahead-free-tree-sitter
  disambiguation approach, the "one fix can be gated behind a second construct in the same span"
  trap, and the "grep can match compiled binaries" trap.
- `docs/plan.md` — D1 (lexical superset scope), D8 (allowlist cap, done criterion), M1's exit
  criterion (≥95%, currently 69.95%).

## State of the tree

- `grammar.js`: `case_statement` supports `[ELSE StatementSeq]` (round 13). `procedure_decls` is
  `choice(seq(procedure_decl, ';'), seq(forward_decl, ';'), definition_proc_decl)` (round 12),
  with a top-level `conflicts: $ => [...]`. Round 14 added: `sysflag`, `square_vector_offset`,
  `external_code_names`, `reg_spec`. Round 15: all three declaration-sequence sites use
  `repeat(choice($.const_decls, $.type_decls, $.variable_decls))`. Round 16:
  `procedure_heading` has `optional($.kStar)` right after `$.kProcedure`. Round 17 added:
  `mul_operator` gained `$.kAnd`; `factor` gained `seq($.kNot, $.factor)`; keyword tokens
  `kAnd => 'AND'`, `kNot => 'NOT'`. Round 18 added: `module`'s `BEGIN` arm gained
  `optional(seq($.kClose, optional($.statement_seq)))`, new token `kClose => 'CLOSE'`; `factor`
  gained `$.typed_set` (`typed_set: $ => seq($.qualident, $.set)`); `real`'s fractional part
  changed from `repeat(digit)` to `digit, repeat(digit)` (requires ≥1 digit after the `.`);
  `integer` gained a third choice arm `token(seq(digit, repeat(hex_digit), 'U'))`;
  `procedure_heading`'s annotation slot gained `$.curly_external_code_names`
  (`seq('{', $.string, repeat(seq(',', $.string)), '}')`); `param_offset` gained a trailing
  `optional('..')`.
- `src/scanner.c`: unchanged since round 10 — four external tokens (`COMMENT`, `PRAGMA`,
  `BRACKET_PRAGMA`, `ASSEMBLER_BODY`).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **69.95% (554/792)**.
- Rust workspace untouched since M0 — not expected to be touched by grammar-only rounds.
