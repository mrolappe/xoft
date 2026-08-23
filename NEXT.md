# Next task

**M7 — Phase 2 plan.** Tagged **Opus** in `docs/plan.md` line 147: "Written from the corpus
report, the allowlist and the measured Oberon-X cost." **M6 is now fully done** (M6.1 + M6.2 +
M6.3) — this is a planning/writing task, not an implementation round: read the three inputs
below and produce a Phase 2 plan document (no existing template for its shape/location in this
repo yet — that's part of what M7 needs to decide, alongside its actual content).

## The three inputs M7 is written from

1. **The corpus report** — `reports/corpus-report.json` (checked in, from M4.1's `xoft corpus
   run`): 766/792 files (96.72%) parse and round-trip cleanly, 26 files (3.28%) allowlisted,
   under D8's 5% cap. `corpus/allowlist.toml` has one grounded one-line reason per excluded file
   — read these before writing anything about "what Phase 1 didn't cover," since the reasons are
   already categorized (stub files, one-off corpus artifacts, conditional-compilation
   preprocessing that's structurally inexpressible with a single parse tree — see `PROGRESS.md`
   round 25's write-up for why that last category was scoped out rather than chased).
2. **The allowlist** — same file, `corpus/allowlist.toml`. The recurring theme across its 26
   entries (per round 25) is Amiga Oberon's `$IF`/`$ELSE`/`$END` conditional-compilation
   preprocessor, which no single Oberon-2 parse tree can represent both branches of — this is
   probably the single most important "what would Phase 2 need to solve" data point.
3. **The measured Oberon-X cost** — `docs/progress/m5-oberon-x.md`'s exit write-up: injectivity,
   not feature size, predicts round-trip cost. An additive construct (`UNLESS`) round-trips
   byte-identically both directions; an alias (`DO`/`BEGIN`) is one-way, and `X→2→X` only
   normalizes rather than reproducing the original spelling. Any Phase 2 dialect-experiment
   estimate should reason in these terms (is the proposed construct additive or an alias-style
   rename?) rather than by feature complexity.

## Not yet decided (ask the user before writing, per this project's "ambiguous syntax, ask" rule
extended to planning ambiguity — same pattern M6.1–M6.3 each followed)

- **Where does the M7 plan document live?** No precedent in this repo — `docs/plan.md` is the
  existing Phase 1 plan (decisions D1–D8, milestone table); M7's output could be a new
  `docs/plan-phase2.md`, an appended section to `docs/plan.md` itself, or something else. Ask
  before creating a new top-level doc file.
- **What Phase 2 scope is actually being planned?** `docs/plan.md`'s own text only says M7
  produces "a Phase 2 plan" — it doesn't enumerate what Phase 2 covers (more dialects? the
  conditional-compilation preprocessing deferred at M1/M4? something else entirely, e.g. a
  language-server protocol layer building on M6's testbed?). This is the single biggest open
  question — resolve it with the user first, since everything else in M7 depends on scope.

## Not in scope

Implementation of anything Phase 2 ends up deciding — M7 is the plan, not the work.

## State of the tree

- `cargo test --workspace` green: `xoft-core` 44, `xoft-cli` 15, `xoft-testbed` 9.
- `cargo clippy --workspace --all-targets` clean.
- `testbed-ui`: `npx tsc --noEmit` clean, `npm run build` succeeds (bundles both grammar
  `.wasm`s + web-tree-sitter's own runtime `.wasm`). `npm install` has been run in
  `testbed-ui/` on this machine.
- `tree-sitter test` unchanged by M6 (85 + 89).
- Not verified: `cargo tauri dev` in a real window (no display server in this environment) —
  standing limitation since M6.1, now also covers M6.3's semantic-token coloring and
  click-to-jump/click-to-select behavior, which have only been verified via `tsc`/`vite build`
  and a standalone Node probe of the parsing+query layer (see `docs/progress/m6-testbed.md`'s
  M6.3 section).
