# Next task

**Pick a direction for M1's tail: declare M1 fully done and move to M2, or take on
conditional-compilation preprocessing as a scoped Phase-2/M1.6 item.** There is no more
low-risk grammar work left in the corpus — round 25 fully re-triaged `oberon-a`'s remaining 19
failures and found every one traces to an already-scoped-out category (see below). This is a
genuine fork in the road, not a triage task — start the next session by asking the user which
way to go, don't default to either.

## What's confirmed (do not re-derive, just verify before coding)

- **Current state**: `sweep_corpus.py` passes 766/792 (96.72%), unchanged since round 24.
  Per-root: `stj` 0 (done), `amiga-oberon-31` 3, `oberon-a` 19, `voc` 4. M1's ≥95% exit
  criterion has been met since round 23.
- **`oberon-a`'s 19 failures are fully triaged, none are actionable grammar work** (round 25,
  see `docs/progress/m1-grammar.md` round 25 for the full diagnostic trail):
  - 8 "moved to..." stub files (87/68-byte placeholders).
  - 3 already-known scoped items: `OberonLib.mod` (dual pragma-guarded `MODULE` header),
    `OBumpRevMsg.mod`/`ErrorMessages.mod` (`\"` string-escape, dialect-specific design not yet
    picked).
  - 5 one-off single-stray-byte corpus artifacts, each unique to its own file corpus-wide:
    `HelloWorld.mod`/`Skeleton.mod` (trailing NUL — **do not try to fix this via `extras` or
    `is_space()`, it hangs the entire parser**, see below), `AsciiTexts.Mod` (trailing 0xFE),
    `amiga/Intuition.mod` (a mid-file 0x08 backspace byte, diagnosed via bisection — was
    originally miscategorized as "huge span, undiagnosed" because tree-sitter's error recovery
    inflated it into an `ERROR [3266,0]-[4666,0]` span full of spurious `ASSEMBLER_BODY`
    matches; the real cause is this one byte).
  - 1 malformed-preamble file: `Obsolete/GTEvents.mod` (bare `@DATABASE`/`@NODE` AmigaGuide
    autodoc header before the `MODULE` keyword, no enclosing comment).
  - 4 files confirmed as **conditional-compilation preprocessing**, the same feature as the
    already-deferred dual-`MODULE`-header item, just in different surface syntax: `Kernel.mod`
    (bracket-pragma `<*IF DEBUG1 THEN*>` around two full `MODULE Kernel [...]` headers),
    `IntuiPointerDemo.mod` (comment-embedded `$IF`/`$ELSE`/`$END` around a whole duplicated
    `IMPORT`/`VAR` section), `amiga/Utility.mod` (bracket-pragma around two statements with no
    `;` separator) — plus `amiga-oberon-31`'s already-deferred `Break.mod`/`NoGuru.mod`
    (comment-embedded form, dual `MODULE` headers). In every case both branches' full code is
    present unconditionally in the source; no single Oberon-2 parse tree can represent that —
    it needs an actual text-preprocessing pass keyed on pragma-defined symbols
    (`DEBUG1`/`OberonA`/`SMALLDATA`/`RESIDENT`/`BreakRq`/...) to pick one branch before parsing.
  User confirmed (round 25): declare `oberon-a` done at 19 remaining, don't implement
  preprocessing this round, don't chase the one-off bytes.
- **`voc`'s 4 remaining failures** (unchanged since round 20): two trailing-garbage files (free
  text/binary appended after `END Module.`), one bare-real-literal lexer ambiguity (`1.`
  colliding with the `2..4` range fix), not re-sampled this round.
- **`amiga-oberon-31`'s 3 remaining failures**: `Module/Break.mod`, `Module/NoGuru.mod` — same
  conditional-compilation feature as above, confirmed again this round. Stays scoped out of M1
  unless/until preprocessing is implemented.

## Hard constraint learned this round — read before touching `extras` or `scanner.c`'s `is_space()`

**Never add byte value 0 (NUL) to whitespace/extras tolerance.** tree-sitter uses lookahead
value 0 as its own internal EOF sentinel (both the external-scanner API and the generated
internal lexer). Treating it as skippable content makes the skip-loop treat EOF as "more
whitespace," and `advance()` at EOF never changes `lookahead`, so the loop spins forever — this
hangs *every* parse, not just ones containing a NUL byte. Round 25 hit this, caught it (a
trivial one-line file hung too), and fully reverted before it reached a commit. Full trail:
`docs/errors.md` round 25, `docs/insights.md` round 25, `docs/checklist.md`.

## If the user picks "declare M1 done, move to M2"

Read `docs/plan.md`'s M2 scope (lossless parse/serialize) before starting. `docs/progress/`
doesn't have an `m2-core.md` file yet — create it following the `m0-foundations.md`/
`m1-grammar.md` pattern. The Rust workspace (`crates/xoft-core`, `crates/xoft-cli`) has been
untouched since M0 — M2 is where it starts moving again.

## If the user picks "scope conditional-compilation preprocessing"

This is a bigger design question than prior M1 rounds, not a quick grammar tweak — start by
re-reading `CLAUDE.md`'s design rule (`xoft-core` performs no I/O) and `docs/plan.md`'s D1-D8
decisions before proposing an approach, since tracking pragma-defined symbol state and
selecting a branch is a different kind of work than anything M1 has done so far (it's closer to
a preprocessing/expansion pass than a parse-tree shape). Don't start coding without walking the
user through the design tradeoff first (parse both branches into the tree with a symbol-state
annotation vs. an actual textual preprocessing pass ahead of tree-sitter) — this is exactly the
"ambiguous syntax, ask" situation `CLAUDE.md` calls out.

## How to find/reproduce a specific corpus failure (reproduction method, unchanged since round 8)

```sh
cd grammars/tree-sitter-oberon2
python3 sweep_corpus.py -v > /tmp/sweep_v.txt   # -v prints tree-sitter's stdout per failure
grep -oE "^  [a-zA-Z0-9_.-]+/" /tmp/sweep_v.txt | sort | uniq -c | sort -rn
python3 -c "
from pathlib import Path
p = Path('<absolute corpus path from roots.toml>/<relative path from sweep output>')
Path('/tmp/x.mod').write_text(p.read_text(encoding='latin-1'), encoding='utf-8')
"
timeout 10 tree-sitter parse /tmp/x.mod | grep -n "ERROR\|MISSING"   # find EVERY error node, not just the summary line
```

**Always wrap `tree-sitter parse`/`tree-sitter test`/any tree-sitter CLI call in `timeout N`**
(N ~10s for one file, ~60s for the full suite) — round 25 hung a scanner bug into an unguarded
20+ minute spin before anyone noticed; a timeout turns that into an immediate, obvious signal
that the just-made change broke something. Never run these commands bare.

Corpus files are Latin-1 except `voc` (UTF-8); always transcode Latin-1 roots before feeding to
`tree-sitter parse`. Always `grep -a` (or `grep -na`) against these roots — a suspiciously
low/zero hit count on a plain `grep -r` is a signal to re-check with `-a` before trusting it.

**Round 25's lesson for a large/misleading `ERROR` span**: don't trust that a huge span means a
big genuine gap — tree-sitter's error recovery can invoke *every* external-scanner token type
once real parsing fails (confirmed here: `ASSEMBLER_BODY` fired and greedily consumed hundreds
of lines with no `ASSEMBLER` keyword anywhere in the file). If a span looks implausibly large for
what's visually at the start of it, truncate the file at real declaration boundaries and re-parse
progressively smaller/closer real-content chunks (wrapped in a minimal synthetic
`MODULE Test; ... END Test.`) to bisect down to the true single-point cause before concluding
it's a "huge span" construct.

## Definition of done (for whichever direction gets picked)

- If M2: a first failing test for lossless parse/serialize round-tripping, per the test-first
  method in `CLAUDE.md`, plus an `m2-core.md` progress file.
- If preprocessing: a design writeup/decision recorded in `docs/plan.md` (a new D-numbered
  decision) before any grammar/scanner code, since this changes what "the grammar" is
  responsible for.
- Either way: update `PROGRESS.md`'s round table, and `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` per the usual end-of-round ritual.

## State of the tree

- `grammar.js`/`src/scanner.c`: **unchanged since round 24** — round 25's NUL-byte attempt was
  fully reverted (`git diff` against HEAD is empty for both files).
- `sweep_corpus.py`: unchanged. Baseline for the next round: **96.72% (766/792)**, `stj` at
  **100%**.
- Rust workspace untouched since M0 — not expected to be touched unless M2 is picked.
