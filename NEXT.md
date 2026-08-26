# Next task

M6.3's manual verification (started round 40, finished round 41, 2026-08-26) is **done**. Both
open checks now have a definitive answer, and the pass found three real bugs. See
`docs/progress/m6-testbed.md`'s round-41 addendum for full detail; summary below.

## What round 41 found

1. **Semantic-token coloring does not render, confirmed real (not a stale screenshot).** Checked
   with real keyword/type/string variety, well after the async semantic-tokens request would have
   settled, after a completed Transpile. Not root-caused yet. `docs/insights.md`'s round-41 entry
   narrows where to look first: round 40's Node probe already confirmed the tree-sitter+query
   layer produces correct captures for both grammars, so the bug is almost certainly in the
   Monaco-facing wiring `testbed-ui/src/highlighting.ts`/`main.ts` never got to verify standalone
   — start with whether `registerDocumentSemanticTokensProvider` actually fires for the model's
   language id (`"oberon2"`/`"oberon-x"`), and whether the legend's token-type names resolve to a
   real theme color, before re-checking the query layer.
2. **Bidirectional click navigation confirmed working**, both directions, tested against a real
   corpus parse failure. No further action needed.
3. **Real bug: `crates/xoft-cli/src/manifest.rs:53` `build()` aborts entirely on the first
   unreadable corpus root.** One bad machine-local path (this session: `amiga-oberon-31`'s
   Nextcloud folder, not currently synced to this machine — a local environment fact, not a repo
   bug) blanks the *whole* sidebar, discarding the other three roots' several-hundred files too.
   Fix: change the loop to collect a per-root `Result` and continue past a failing root (same
   shape as `corpus_run.rs`'s existing per-file outcome aggregation), surfacing just that root's
   failure in the returned `Manifest` rather than discarding everything. Needs a test-first repro
   (e.g. a `RootsConfig` with one nonexistent path among two/three valid ones, asserting the
   valid roots' files still appear).
4. **Real bug: the diff editor's "original" (source) pane is not user-editable.**
   `testbed-ui/src/main.ts:35-38` never sets `originalEditable: true` in `createDiffEditor`'s
   options; Monaco defaults it to `false`. The documented "editable source pane" / "type something
   invalid directly into the editor" workflow is currently broken for a real user — only
   programmatic `setValue()` (the corpus-file-picker path) works. One-line fix
   (`originalEditable: true` alongside the existing `automaticLayout: true`), but per this
   project's test-first method, needs whatever this frontend's equivalent of a regression check
   is (no JS test framework exists yet for this two-file frontend, per M6.2's note — decide
   whether this is the round that adds one, or whether a manual re-check is the accepted
   verification for a one-line Monaco option, same as M6.2/M6.3's own frontend testing scope).
5. **Incidental, not yet fixed: the app window has no opaque background.** `body`/`#app`/
   `#corpus-list` in `testbed-ui/src/style.css` never set a `background-color`, and the window
   renders transparent wherever nothing else draws — letting whatever's stacked behind it show
   through. Fix is a one-line `background: <color>` on `body`. Low priority functionally, but
   worth closing since it's a real UI defect (not just a screenshot artifact) and a recurrence
   risk for incidental exposure during any future manual/visual verification pass.

## Not yet decided — ask the user before starting

- **Fix all four now, or move to M7 first?** These are small, well-scoped bugs (#3 and #5 are
  each a few lines; #4 is one line plus its test decision; #1 needs investigation first). None of
  M7's three planning inputs (corpus report, allowlist, Oberon-X cost) depend on any of them being
  fixed. Ask whether to spend a round closing these out before M7, or defer them (tracked here and
  in `docs/progress/m6-testbed.md`) and start M7 planning now.
- **If fixing now: order?** #3 and #4 are independent one-file fixes with clear tests. #1 needs
  investigation before it's known how big a fix it is. #5 is cosmetic. Suggest #3, #4, #5 (all
  small, testable) before #1 (open-ended), but this is a judgment call worth confirming.

## M7 — Phase 2 plan (unchanged from before round 41; still fully scoped, still waiting on the
two questions below)

Tagged **Opus** in `docs/plan.md` line 147: "Written from the corpus report, the allowlist and
the measured Oberon-X cost." **M6 is fully done** (M6.1 + M6.2 + M6.3, code-complete and tested,
plus now a completed — not just partial — manual verification pass) — M7 is a planning/writing
task, not an implementation round: read the three inputs below and produce a Phase 2 plan document
(no existing template for its shape/location in this repo yet — that's part of what M7 needs to
decide, alongside its actual content).

### The three inputs M7 is written from

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
   probably the single most important "what would Phase 2 need to solve" data point. (Round 41
   loaded exactly one of these, `IntuiPointerDemo.mod`, while testing click-nav — a live,
   re-confirmed example if a concrete illustration is useful when writing the plan.)
3. **The measured Oberon-X cost** — `docs/progress/m5-oberon-x.md`'s exit write-up: injectivity,
   not feature size, predicts round-trip cost. An additive construct (`UNLESS`) round-trips
   byte-identically both directions; an alias (`DO`/`BEGIN`) is one-way, and `X→2→X` only
   normalizes rather than reproducing the original spelling. Any Phase 2 dialect-experiment
   estimate should reason in these terms (is the proposed construct additive or an alias-style
   rename?) rather than by feature complexity.

### Not yet decided for M7 (ask the user before writing, per this project's "ambiguous syntax, ask"
rule extended to planning ambiguity — same pattern M6.1–M6.3 each followed)

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

- `cargo test --workspace` green: `xoft-core` 44, `xoft-cli` 15, `xoft-testbed` 9 (unchanged by
  round 41 — pure verification, no source changed).
- `cargo clippy --workspace --all-targets` clean.
- `testbed-ui`: `npx tsc --noEmit` clean, `npm run build` succeeds (bundles both grammar
  `.wasm`s + web-tree-sitter's own runtime `.wasm`). `npm install` has been run in
  `testbed-ui/` on this machine.
- `tree-sitter test` unchanged by M6 (85 + 89).
- `cargo tauri dev` **fully verified working in a real window on this machine** — see round 41's
  findings above for what that verification actually found. Don't repeat the old "no display
  server" assumption; it was already wrong as of round 40's addendum and is now fully retired.
- `corpus/roots.toml`'s `amiga-oberon-31` entry currently points to a path that doesn't resolve on
  this machine (`~/Nextcloud/retro-comp/amiga/vamos-spielplatz/AmigaOberon3.1` — likely a
  Nextcloud selective-sync gap, not a repo issue). Re-sync that folder before relying on the full
  4-root corpus locally; `oberon-a`, `stj`, `voc` all resolve fine. This is exactly the condition
  that triggers bug #3 above.
- Note: running `cargo tauri dev`/`cargo build` may rewrite
  `crates/xoft-testbed/Cargo.toml`'s `tauri`/`tauri-build` dependency lines to add an explicit
  `features = []` — harmless (no behavior change) but incidental; `git checkout -- <file>` it
  rather than committing it, unless intentionally adding real features later.
