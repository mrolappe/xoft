# Next task

**Finish the M6.3 manual verification in a real `cargo tauri dev` window**, started and left
incomplete 2026-08-23 (user closed the app mid-check). Do this before M7 (below) unless the user
says otherwise.

**Correction to earlier rounds' docs**: M6.1/M6.2/M6.3 all state "no display server in this
environment" as the reason `cargo tauri dev` was never opened in a real window. That's wrong for
this machine (real built-in Retina display, confirmed by actually launching it) — don't repeat
that assumption; just run `cargo tauri dev` from the repo root.

## What's already confirmed (2026-08-23)

- Real window opens; `xoft-testbed` shows as a visible process.
- Corpus sidebar populates with real files (tried `amiga-oberon-31`).
- Clicking a corpus file loads its real content into the editor.
- A transpile ran and produced diff output in the modified pane.

## What's still unconfirmed — do these next

1. **Does semantic-token coloring actually render?** One early screenshot (sample source,
   `MODULE`/`BEGIN`/`UNLESS`/`DO`/`END`, before any interaction) showed plain black text, no
   visible keyword/type/string coloring. Could be: the provider isn't attaching, the theme isn't
   applying the legend's token types, or the screenshot just predates the async semantic-tokens
   request completing. **Check this first, by eye** — load a file with real keyword/type/string
   variety (not just the tiny `UNLESS` sample; e.g. anything from `amiga-oberon-31`'s
   `Interfaces/` has `CONST`/`TYPE`/`VAR`/procedure headers) and look at whether tokens are
   colored at all. No scripted clicking needed for this part.
2. **Click-to-jump** (click a diagnostic in the list → editor selection jumps to its span) and
   **click-to-select** (click an `ERROR`/`MISSING` squiggle in the editor → matching list item
   highlights). Needs a broken source loaded (e.g.
   `crates/xoft-cli/tests/fixtures/broken/missing_semicolon.mod`, or type something invalid
   directly into the editor) and a Transpile run first so diagnostics populate.

## Driving the app via `osascript`/`screencapture`, if scripting instead of eyeballing

Got this wrong once already this round — the coordinate math, not the concept:

- `screencapture -x <path>` full-screen capture; Read tool reports both the file's real pixel
  size and a "displayed at WxH, multiply by N" scale factor — that factor converts *displayed
  image coordinates* to **Retina pixel** coordinates, not to what `osascript` needs.
- `tell application "System Events" to click at {x, y}` expects **logical point** coordinates
  (half of Retina pixels on a 2x display — confirmed via `tell process "..." to get {position,
  size} of window`, which reports logical points).
- **Correct combined factor**: `displayed_coord × (full_res_dim / displayed_res_dim) / 2` — i.e.
  the screenshot tool's stated multiplier, halved again. Using the stated multiplier directly
  overshoots by 2x (this round's actual mistake).
- `click at {x,y}` clicks whatever window is topmost at that screen point **system-wide**, not
  scoped to a chosen process — bring the target window frontmost first and verify it actually
  came frontmost (don't trust `set frontmost of process "..." to true` silently; it errored with
  -10006 at least once this round) before clicking, or the click can land on an unrelated window
  (it landed on the Claude Code terminal once).

## After the manual pass: M7 — Phase 2 plan

Tagged **Opus** in `docs/plan.md` line 147: "Written from the corpus report, the allowlist and
the measured Oberon-X cost." **M6 is fully done** (M6.1 + M6.2 + M6.3, code-complete and tested;
the manual pass above is a verification follow-up, not a blocker for M6's own exit) — M7 is a
planning/writing task, not an implementation round: read the three inputs below and produce a
Phase 2 plan document (no existing template for its shape/location in this repo yet — that's
part of what M7 needs to decide, alongside its actual content).

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
   probably the single most important "what would Phase 2 need to solve" data point.
3. **The measured Oberon-X cost** — `docs/progress/m5-oberon-x.md`'s exit write-up: injectivity,
   not feature size, predicts round-trip cost. An additive construct (`UNLESS`) round-trips
   byte-identically both directions; an alias (`DO`/`BEGIN`) is one-way, and `X→2→X` only
   normalizes rather than reproducing the original spelling. Any Phase 2 dialect-experiment
   estimate should reason in these terms (is the proposed construct additive or an alias-style
   rename?) rather than by feature complexity.

### Not yet decided (ask the user before writing, per this project's "ambiguous syntax, ask"
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

- `cargo test --workspace` green: `xoft-core` 44, `xoft-cli` 15, `xoft-testbed` 9.
- `cargo clippy --workspace --all-targets` clean.
- `testbed-ui`: `npx tsc --noEmit` clean, `npm run build` succeeds (bundles both grammar
  `.wasm`s + web-tree-sitter's own runtime `.wasm`). `npm install` has been run in
  `testbed-ui/` on this machine.
- `tree-sitter test` unchanged by M6 (85 + 89).
- `cargo tauri dev` **does work in a real window on this machine** (see above) — partially
  exercised, not fully verified; see "What's still unconfirmed" above for exactly what's left.
- Note: running `cargo tauri dev`/`cargo build` may rewrite
  `crates/xoft-testbed/Cargo.toml`'s `tauri`/`tauri-build` dependency lines to add an explicit
  `features = []` — harmless (no behavior change) but incidental; `git checkout -- <file>` it
  rather than committing it, unless intentionally adding real features later.
