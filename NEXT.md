# Next task

**M1.4 continued — triage the remaining corpus gap.** M1.4's original scope (build a sweep
script, get an honest number, resolve `INLINE`) is done: 15.78% → 21.97% `ERROR`-free, and
`INLINE` needed no grammar change at all (see below). What's left is bigger than `INLINE` ever
was and needs a scoping decision, not a blind implement.

## What happened last round

Full details in `docs/progress/m1-grammar.md`'s M1.4 section — read it before starting, this is
just the summary. `grammars/tree-sitter-oberon2/sweep_corpus.py` exists and works: `cd
grammars/tree-sitter-oberon2 && python3 sweep_corpus.py` (add `-v` for per-file error detail).
Takes ~1-2 minutes over all 792 files.

Fixed, all real bugs, all with new corpus tests, `tree-sitter test` 39/39 green:

1. `kElseif` matched literal `'ELSEIF'` instead of `'ELSIF'` — every `IF...ELSIF` in the corpus
   was silently failing. One-character... well, one-word fix.
2. Hex-integer token only matched a single hex digit (`hex_digit, 'H'` instead of `digit,
   repeat(hex_digit), 'H'`) — this is what was actually blocking `SYSTEM.INLINE(02F0EH,...)`,
   not missing block syntax. `INLINE` turned out to be an ordinary procedure call, not a special
   token; the dialect-extension table's "opaque token, contents unparsed" characterization was
   an unchecked assumption. No grammar rule added for `INLINE` itself — the hex fix was the whole
   fix. If a future round finds this characterization repeated elsewhere in `docs/`, correct it.
3. `import` gained two AmigaOberon-only rename/re-export variants: `IMPORT e * := Exec` (a `*`
   re-export marker, always paired with `:=`) and `IMPORT e: Exec` (plain `:` as an alternate
   rename operator — a different AmigaOberon compiler-version dialect than the `*` one, they
   don't seem to mix in the same file). Neither is in `docs/language-baseline.md`.

## What's triaged but not fixed

Ranked by corpus impact (substring grep across all 792 files, so an upper bound — a file can
fail for a different, earlier reason and never reach these constructs):

| Pattern | Files | What it is |
|---|---|---|
| `<* ... *>` bracket pragmas | 212 (27%) | Different delimiter from the `(*$…*)` pragma M1.3 built. Not in `docs/language-baseline.md`. Example: `<*STANDARD-*>`, `<* MAIN- *> <*$ NilChk- *>` — looks like it can hold either bare flags or `$`-prefixed sub-pragmas, needs real investigation. |
| `STRUCT` record variant | 43 | AmigaOberon C-interop type: `Point2D = STRUCT x,y: INTEGER; ... END`. Not `RECORD`, not in the baseline EBNF at all — this is a genuinely new type-declaration alternative, not a lexical tweak. |
| `PROCEDURE ... *{base,-N}(...)` / `param{N}` brace annotations | 42 | AmigaOberon library-vector-offset metadata on procedure/parameter declarations, e.g. `PROCEDURE ResetBattClock *{base,-6}();`. Undiscovered before this round. |
| `ASSEMBLER` blocks | 32 (STJ only) | Looks like `PROCEDURE ... ASSEMBLER ... END`, same family as `INLINE` conceptually but this one may genuinely be block syntax (unconfirmed — not minimized yet, don't assume, check first, same as `INLINE`'s lesson). |
| `POINTER TO ARRAY OF Type` | not re-measured | Carried over from M1.3. `array_type` needs a length-less `ARRAY OF` alternative like `formal_type` already has. |
| Single-quoted strings | not re-measured | Carried over from M1.2c. Still just "plausible", never confirmed as an actual failure cause — check before fixing. |

The bracket-pragma cluster alone (212 files) is bigger than everything M1.1–M1.4 has fixed
combined. Per D8, the allowlist is capped at 5% of the corpus (≈40 files) — this cannot be
closed by allowlisting, most of it has to become grammar or be ruled explicitly out of D1's
scope.

## Decision needed before implementing anything

This is a scoping call, not a technical one — flag it to the user rather than picking silently:

- Is the bracket-pragma family (`<* ... *>`) in scope for D1's "lexical superset," same tier as
  the `(*$…*)` pragma M1.3 already built? If yes, it's probably the highest-value single fix
  available (212 files) and a reasonable next task on its own.
- Is `STRUCT` in scope? It's a genuine type-system extension (a second record-like type), not a
  lexical tweak — arguably bigger than D1 was scoped for ("full Oberon-2 + lexical superset").
  Might belong in Phase 2 instead (`corpus/allowlist.toml`'s stated purpose) rather than M1.
- Same question for `ASSEMBLER` and the brace annotations — investigate their real syntax first
  (grep the STJ/AmigaOberon roots per `corpus/roots.toml`, same method used for `INLINE` and the
  import variants this round) before even asking the scoping question, since "what it actually
  is" changes the answer (a call vs. a block vs. an annotation are three different amounts of
  grammar work).

Whatever gets picked, **confirm real syntax from the corpus before writing a grammar rule** —
this was the single most valuable habit from this round (see `docs/insights.md` round 8, "A
task's stated cause can be wrong even when its stated construct is right"). Don't trust
`docs/language-baseline.md`'s dialect-extension table's characterization of *how* something
works, only that it exists.

## Definition of done (whatever gets picked)

- `tree-sitter test` still green, plus new corpus cases for whatever gets added.
- Re-run `sweep_corpus.py` before/after, record the delta in `docs/progress/m1-grammar.md`
  (same table format as this round's M1.4 section).
- Update the triage table above (or move it into `docs/progress/m1-grammar.md` permanently) so
  the next round doesn't have to regrep from scratch.
- No changes outside `grammars/tree-sitter-oberon2/` unless the scoping decision explicitly
  needs `corpus/allowlist.toml` (M1's own crate territory).

## Context a fresh session needs

- `docs/progress/m1-grammar.md`'s M1.4 section — full detail on what was fixed, why, and the
  exact corpus-impact numbers for each triaged pattern.
- `docs/insights.md` round 8 — four lessons, all still live: breadth vs. depth measurement,
  confirm-before-coding paying off again, grep-the-keyword-table when a mass-failure has no
  obvious cause, and a mechanistically-plausible theory (encoding) that turned out to change
  zero files (still worth the five minutes it took to disprove).
- `docs/errors.md` round 8 — hand-writing an expected S-expression from a rule name instead of
  generating it or copying a neighboring example got the tree shape wrong; `tree-sitter test
  --update` against real input is the established, faster way.
- `docs/plan.md` — D1 (lexical superset scope — this is exactly what's now ambiguous), D8 (done
  criterion, allowlist cap), M1's exit criterion (≥95%, currently 21.97%).
- `docs/language-baseline.md` — the dialect-extension table (now stale: `INLINE`'s "opaque
  token" description was wrong, and bracket pragmas / `STRUCT` / brace annotations aren't in the
  table at all — needs an update once next round settles what's in scope).

## State of the tree

- `grammar.js`: M1.1 base through M1.3 (see prior `NEXT.md` history) + M1.4 (`ELSIF` typo fix,
  hex-literal token fix, `import` re-export/colon-rename variants). No new node kinds added this
  round, only widened existing rules and fixed two token bugs.
- `grammars/tree-sitter-oberon2/sweep_corpus.py` — new, committed. Transcodes `encoding:
  "high-bytes"` corpus files (Latin-1 → UTF-8) before parsing since tree-sitter's CLI always
  reads UTF-8; this didn't change any pass/fail outcome this round but is still correct and
  needed for 330/792 files.
- Rust workspace untouched since M0 — this task doesn't touch it.
