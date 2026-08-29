# M6 — Testbed ✅ done (M6.1 + M6.2 + M6.3 done, round 40, 2026-08-23)

## M6.1 — Tauri shell linking `xoft-core` ✅ (round 38, 2026-08-23)

Scoping questions from `NEXT.md` resolved with the user before coding:

- **`transpile` means the dialect mapping** (`xoft_core::mapping::to_oberon2`/`to_oberon_x`),
  not the CLI's check+round-trip `transpile` — the testbed is the first consumer of M5's
  mapping rules outside their own test suite.
- **Crate layout: split.** Rust/Tauri backend at `crates/xoft-testbed/` (workspace member,
  keeps the existing convention); JS/TS frontend at top-level `testbed-ui/`, not nested under
  `crates/`.
- **CI: deferred.** `.github/workflows/ci.yml` untouched this round — a native Tauri
  toolchain is a heavier addition, left for once the testbed has more shape.

### What changed

`cargo tauri init --ci -d crates -A xoft-testbed -D ../../testbed-ui ...` scaffolded into
`crates/src-tauri/`, then renamed to `crates/xoft-testbed/` (the CLI hardcodes `src-tauri` as
the folder name, no flag to rename it directly). Package renamed `app`/`app_lib` →
`xoft-testbed`/`xoft_testbed_lib`; `Cargo.toml` switched to the workspace's
`version.workspace = true` etc. convention; scaffold's `tauri-plugin-log`/`log` dependencies
dropped (unused, not requested — the scaffold adds them by default). Added to the root
`Cargo.toml`'s `members` in the same step its own `Cargo.toml` was written (checklist item).

`crates/xoft-testbed/src/commands.rs` (new): the three command bodies as plain functions, no
`tauri` types in their signatures — testable directly, no runtime needed:

- `list_corpus(roots_toml: &str) -> anyhow::Result<Manifest>` — thin wrapper over
  `xoft_cli::manifest::build`, takes the TOML *content* (not a path) so the one
  `std::fs::read_to_string` call stays in the `#[tauri::command]` wrapper.
- `roundtrip_check(filename: &str, raw: &[u8]) -> RoundtripResult` — reuses
  `xoft_cli::transpile::transpile_source`; `parse_ok`/`round_trip_ok` computed exactly as
  `corpus_run.rs` already does (`diagnostics.is_empty()`, `output_bytes == raw`), not
  reimplemented.
- `transpile(direction: Direction, text: &str) -> TranspileResult` — `Direction` (serde
  kebab-case) picks the matching grammar + mapping function together
  (`language_oberon_x`+`to_oberon2` or `language`+`to_oberon_x`) so the two can't drift out of
  sync; diagnostics come from `xoft_core::diagnostic::diagnostics` on the *input* tree.

`xoft_core::diagnostic::Diagnostic` gained `#[derive(Serialize)]` (one line) so it can cross
the Tauri IPC boundary — `xoft-core` already depended on `serde` (`corpus::FileFacts`), so
this doesn't reopen the no-I/O design rule.

`crates/xoft-testbed/src/lib.rs`: three `#[tauri::command]` wrappers around `commands::*`,
mapping `Result`/`anyhow::Error` to `String` for IPC, registered via
`generate_handler![list_corpus, roundtrip_check, transpile]`. `tauri.conf.json`:
`frontendDist` → `../../testbed-ui`, `identifier` → `dev.xoft.testbed` (scaffold default was
the placeholder `com.tauri.dev`), `app.withGlobalTauri: true` so the static frontend can call
`window.__TAURI__.core.invoke` without an npm bundler — M6.2 owns Vite/Monaco, M6.1 only needs
proof the commands work end to end. `testbed-ui/index.html`: one static page, three sections
(one per command) with a textarea/button/`<pre>` each; output is written via `.textContent`,
never `.innerHTML`, so nothing a command returns (diagnostic messages, file paths) is
interpreted as markup.

### Tests

`crates/xoft-testbed/tests/commands.rs`, 5 tests, TDD — written against a not-yet-existing
`commands` module first (confirmed red: `cannot find module`), then `commands.rs` implemented
and all 5 passed on the first run. Fixtures are real, not invented: `corpus/cases/comment_gap.2.mod`
(clean round-trip), `crates/xoft-cli/tests/fixtures/broken/missing_semicolon.mod` (M3.3's
existing broken fixture, reused rather than hand-writing a new invalid source), and
`corpus/cases/unless_body.{x,2}.mod` (M5.3's golden pair, Rule B, exercised in both mapping
directions). `list_corpus`'s test builds a small ad hoc `roots.toml` string pointing at
`corpus/cases/` itself rather than the machine-local `corpus/roots.toml`, keeping the test
portable.

Verified `Direction`'s serde names in a throwaway test (deleted after) rather than assuming
`rename_all = "kebab-case"`'s exact output: `OberonXToOberon2` → `"oberon-x-to-oberon2"`,
`Oberon2ToOberonX` → `"oberon2-to-oberon-x"` — confirmed to match `testbed-ui/index.html`'s
`<select>` values before wiring them together.

`cargo test --workspace` green: `xoft-core` 38 (unchanged), `xoft-cli` 15 (unchanged),
`xoft-testbed` 5 (new). `cargo build -p xoft-testbed` succeeds (the Tauri binary target links
and compiles); `cargo clippy -p xoft-testbed --all-targets` clean.

**Not verified this round:** an actual `cargo tauri dev` window. This sandboxed environment has
no display server, so opening a real webview window and clicking through the three placeholder
buttons wasn't possible — the crate compiling and linking as a Tauri binary, plus all three
command bodies covered by integration tests calling them directly, is the coverage this round
actually got. Worth a manual `cargo tauri dev` pass on a machine with a display before trusting
the IPC wiring (argument (de)serialization across the JS↔Rust boundary in particular) beyond
what the tests prove.

### Security review finding (acknowledged, not fixed this round)

`list_corpus` takes `roots_toml` from the webview and passes whatever paths it names straight
to `xoft_cli::manifest::build`'s `WalkDir`, with no restriction to the app's own corpus roots.
Any JS running in the webview can point it at an arbitrary local directory and get back
filenames/sizes/sha256 for every `.mod`/`.def` file under it — a real capability newly exposed
over IPC, even though `manifest::build` itself is unchanged from M0.2 and the CLI has always
allowed the same thing from a locally-run `roots.toml` file. Today's `testbed-ui/index.html` is
fully static and first-party, so nothing untrusted can reach this yet, but Tauri's own security
model treats the webview as the trust boundary this command crosses without scoping.
**Deliberately deferred rather than fixed**: M6.1's scope is "callable and testable," and a real
fix (a Tauri fs-capability allowlist, or restricting `list_corpus` to paths under
`corpus/roots.toml`'s configured roots) is a design decision that fits better once M6.2/M6.3
define what the real frontend actually needs to send. Tracked here so it isn't silently
forgotten; revisit before M6.2 wires up anything beyond the static placeholder page, and
before this crate ever ships to anyone who isn't running it locally against their own corpus.

## M6.2 — Vite + Monaco `DiffEditor` frontend ✅ (round 39, 2026-08-23)

Four open questions from `NEXT.md` resolved with the user before coding:

- **Layout**: minimal DiffEditor screen (direction toggle + diff view + diagnostics list) plus a
  corpus file-picker sidebar (from `list_corpus`) that loads a selected file into the editor.
- **Diagnostics**: plain list this round (`start_byte`–`end_byte`: `message`); byte→Monaco
  line/column conversion stays M6.3's job.
- **`rootsToml` input**: not a runtime textarea/dialog — resolved by removing it from the IPC
  surface entirely (see security section below).
- **Security**: resolve M6.1's acknowledged `list_corpus` finding this round, not defer again.

### What changed

**Backend.** `crates/xoft-testbed/src/commands.rs` gained `read_corpus_file(roots_toml, root,
path) -> Result<Vec<u8>, String>` (TDD: red on the not-yet-existing function, then implemented),
closing the gap M6.1 didn't have — no prior command read a file's actual bytes, only metadata
(`Entry` has `path`/`sha256`/etc., never content). Output sink / injection class named per
`CLAUDE.md`'s rule: turning a `(root, path)` pair into a filesystem read is path traversal;
mitigated by canonicalizing both the resolved root and the joined target and rejecting unless
the target still starts with the root, tested directly (escaping `..`, unknown alias).

**Security, addressed not deferred.** M6.1's acknowledged finding was `list_corpus` accepting
an arbitrary, webview-supplied `roots_toml` — letting IPC callers pick which filesystem paths
get walked. The first pass at `read_corpus_file` (following the approved plan) mirrored
`list_corpus`'s existing `roots_toml: &str` parameter and inherited the identical gap (see
`docs/insights.md` round 39). Real fix: `commands::list_corpus`/`read_corpus_file` (the
Tauri-free, unit-tested layer) keep `roots_toml` as a parameter for testability against
synthetic fixtures, but the `#[tauri::command]` wrappers in `lib.rs` — the actual IPC trust
boundary — no longer accept it from the caller at all. They read this repo's own
`corpus/roots.toml` from disk themselves (`env!("CARGO_MANIFEST_DIR")`-relative), so a webview
can no longer supply a `roots_toml` value under any circumstances. `list_corpus`/
`read_corpus_file` are now zero/two-argument commands from the frontend's perspective
(`{ root, path }` only).

**Frontend.** `testbed-ui/` gained real Vite tooling from nothing (`package.json`,
`vite.config.ts`, `tsconfig.json`) — `monaco-editor` as the only runtime dependency, no
`@tauri-apps/api` (kept `window.__TAURI__.core.invoke`, already working via `withGlobalTauri`,
one fewer dependency). `src/main.ts`: one `monaco.editor.createDiffEditor` doubles as the source
editor (its *original* model is live/editable, loaded from the corpus picker or typed directly)
and the diff view (*modified* model holds `transpile`'s `output`, refreshed by a "Transpile"
button); diagnostics render as a plain `<ul>` fed by `TranspileResult.diagnostics`, all via
`textContent` (never `innerHTML` with untrusted content — `entry.path`, diagnostic messages, and
thrown errors all go through `textContent`, matching M6.1's precedent). Monaco's worker is wired
via Vite's native `?worker` import (`monaco-editor/esm/vs/editor/editor.worker?worker`) — no
`vite-plugin-monaco-editor` dependency needed for a single generic worker. `tauri.conf.json`
gained `build.devUrl`/`beforeDevCommand`/`beforeBuildCommand` (object form with `cwd:
"../../testbed-ui"`, field name is `script` not `command` — see `docs/errors.md` round 39) and
`frontendDist` → `../../testbed-ui/dist`. `capabilities/default.json` untouched (`core:default`
still sufficient — no fs plugin needed, the new command is a plain Rust file read, not a
JS-invocable fs-plugin call).

### Tests

`crates/xoft-testbed/tests/commands.rs`: 3 new tests for `read_corpus_file` (happy path against
the portable `corpus/cases/` fixture root, path-traversal rejection, unknown-alias rejection),
written red-then-green. `cargo test --workspace` green: `xoft-core` 38, `xoft-cli` 15 (both
unchanged), `xoft-testbed` 8 (5 → 8). `cargo clippy --workspace --all-targets` clean.

Frontend: `npx tsc --noEmit` clean, `npm run build` (Vite) succeeds — confirms the TypeScript
compiles and Monaco's worker/import wiring resolves. No JS test framework introduced for a
two-file frontend (disproportionate for this size); frontend correctness beyond "it builds" is
unverified in this environment.

**Not verified this round**: an actual `cargo tauri dev` window (same no-display-server
limitation as M6.1) — the UI's real behavior (corpus list rendering, DiffEditor updating,
diagnostics list) has not been clicked through in a real browser/webview. Worth a manual pass on
a machine with a display before trusting the wiring beyond what `tsc`/`vite build` prove.

## M6.3 — `web-tree-sitter` highlighting, `ERROR` nodes marked, clickable diagnostics ✅ (round 40, 2026-08-23)

Four open questions from `NEXT.md` resolved with the user before coding:

- **Highlighting scope**: full semantic-tokens coloring for both grammars (not just `ERROR`
  spans), per the plan's own "semantic-tokens provider, not Monarch" note.
- **Byte→position conversion**: in Rust, against the exact text each `Diagnostic` was parsed
  from.
- **Click behavior**: both directions — list click jumps the editor, and clicking an `ERROR`
  span in the editor selects the matching list entry.
- **`.wasm` delivery**: checked-in compiled artifacts, regenerated by a small script.

### What changed

**Backend.** New `xoft_core::position` module (no I/O, pure text computation — stays inside the
"`xoft-core` performs no I/O" rule): `byte_to_position(text, byte) -> Position { line, column }`,
1-based, TDD (6 tests including a high-byte case). `xoft_core::diagnostic::Diagnostic` itself
stays byte-only — the CLI's `codespan-reporting` rendering needs byte ranges, not positions, and
widening it would ripple into the CLI for no reason. Instead `xoft-testbed/src/commands.rs`
gained a testbed-local `PositionedDiagnostic { start: Position, end: Position, message }` and
`position_diagnostics(text, diagnostics)`, and `TranspileResult`/`RoundtripResult` now return
`Vec<PositionedDiagnostic>` — layering the position concern at the IPC boundary that actually
needs it, rather than at the shared core type every other consumer (CLI, corpus runner) also
uses. Caught by the checklist's own warning against hand-deriving `ERROR`-node shapes from
memory: a first-draft integration test guessed `missing_semicolon.mod`'s diagnostic would start
at line 5 (by analogy with docs/errors.md round 31's "lands in assignment" note); running it
red-first showed the real position is line 4, columns 8–10 (the ERROR node covers the trailing
`10`, not the next line's `b`) — fixed against the probed value, not the guess.

**Frontend — grammars as WASM.** `tree-sitter build --wasm` compiled both grammars (`tree-sitter
build --wasm -o ... grammars/tree-sitter-oberon2` / `-oberon-x`) — tree-sitter-cli 0.26 bundles
its own wasi-sdk download, no Emscripten/Docker install needed. Checked in at
`testbed-ui/src/grammars/{oberon2,oberon-x}.wasm` plus a copy of `highlights.scm` (both grammars
ship byte-identical query text; one copy suffices), regenerated by
`testbed-ui/scripts/build-wasm-grammars.sh`. The repo's blanket `*.wasm` `.gitignore` rule needed
a `!testbed-ui/src/grammars/*.wasm` exception — these three files are the deliberate checked-in
exception the user chose, not build noise. `web-tree-sitter@^0.26.13` added as a `testbed-ui`
runtime dependency; its own core runtime (`web-tree-sitter.wasm`) is imported via Vite's `?url`
asset handling alongside the two grammar `.wasm`s, so nothing needs a `public/` folder or
cross-directory `fs.allow` config.

**Frontend — semantic tokens (`src/highlighting.ts`, new).** A Monaco
`registerDocumentSemanticTokensProvider` per dialect (`"oberon2"`, `"oberon-x"` language ids),
computed by parsing the *live* model text with `web-tree-sitter` and running the grammar's own
`highlights.scm` query via `Query.captures` — reusing the existing, already-shipped query rather
than writing a second classification scheme. Query captures can nest (e.g. `(type) @type`
wrapping a `(qualident (ident) @type.builtin ...)` inside it); resolved with an
innermost-capture-wins walk (map each captured node's id to its capture name, then for every
*leaf* token walk up to the nearest captured ancestor-or-self) rather than porting tree-sitter's
own highlight-priority/region-splitting algorithm — simpler, and correct for this grammar's
mostly-single-leaf capture shapes (confirmed by probing both grammars directly with Node before
wiring Monaco, see below). Capture names map to a small custom `SemanticTokensLegend`
(`namespace`/`type`/`property`/`variable`/`function`/`keyword`/`operator`/`string`/`number`/
`comment` token types, `definition`/`defaultLibrary`/`readonly` modifiers) — using LSP-standard
names lets Monaco's built-in theme colors apply without writing custom theme rules.
`monaco.languages.SemanticTokensBuilder` doesn't exist in this monaco-editor version's shipped
typings, so its delta-encoding algorithm (5 `uint32`s per token: deltaLine,
deltaStartChar-or-absolute, length, tokenType, tokenModifiers) is a small hand-rolled class
instead. Tree-sitter node positions are UTF-8 *byte* offsets/columns; Monaco wants UTF-16
code-unit offsets — bridged with a `byteToCharMap` built once per parse (iterates codepoints via
`for...of`, tracking UTF-8 byte length and UTF-16 unit length per codepoint) rather than trusting
`Node.startPosition`'s byte-counted column directly.

**Frontend — diagnostics wiring (`src/main.ts`).** `PositionedDiagnostic`'s shape flows straight
into `monaco.editor.setModelMarkers` (native squiggly-underline + hover, no hand-rolled decoration
CSS) and into the existing diagnostics `<ul>`, now clickable. List→editor: click reveals and
selects the diagnostic's `monaco.Range` on the source editor. Editor→list: `onMouseDown` on the
source editor finds which diagnostic's range contains the click position
(`Range.containsPosition`) and highlights the matching `<li>` (CSS `.selected`, scrolled into
view). `originalModel`/`modifiedModel` get a real language id (`"oberon2"`/`"oberon-x"`, chosen
from the direction selector) instead of `undefined`, so the semantic-tokens providers actually
attach.

### Verification

`web-tree-sitter` parsing + the `highlights.scm` query were probed directly in Node (a throwaway
script, deleted before commit, not part of the diff) against both compiled grammar `.wasm`s
before wiring Monaco — confirmed zero-`ERROR` parses and inspected the actual capture names/spans
for a small Oberon-X source (`UNLESS`/`DO`) and a small Oberon-2 source (`CONST`/`VAR`/
expression), rather than trusting the query's behavior from reading it. This surfaced two
pre-existing `highlights.scm` coverage gaps, not introduced by this round and out of scope to fix
here: `kUnless` (the Oberon-X keyword) has no capture rule at all (the query predates M5's fork),
and bare identifier *reads* in an expression (e.g. the `x` in `y := x + 1`) aren't captured as
`@variable` — only assignment left-hand-sides and declaration sites are. Both simply render with
no semantic color (safe fallback), not a wrong color.

`cargo test --workspace` green: `xoft-core` 38 → 44 (6 new `position` tests), `xoft-cli` 15
unchanged, `xoft-testbed` 8 → 9 (1 new). `cargo clippy --workspace --all-targets` clean. Frontend:
`npx tsc --noEmit` clean, `npm run build` succeeds and bundles all three `.wasm` assets. Security
review of the round's diff (`security-review` skill, via a fresh sub-agent): no high-confidence
findings — the new WASM/query assets are our own checked-in build artifacts, not user-controlled
input, and all new DOM writes stay on `textContent`/`createElement`.

**Not verified this round**: an actual `cargo tauri dev` window, still (no display server in this
environment, same limitation as M6.1/M6.2) — semantic-token coloring and the click-to-jump/
click-to-select-diagnostic behavior have not been exercised in a real browser/webview, only
`tsc`/`vite build`/a standalone Node probe of the parsing+query layer. Worth a manual pass on a
machine with a display before trusting the interactive pieces beyond what's proven here.

### Addendum — manual `cargo tauri dev` pass, partial (2026-08-23, same day)

The "no display server" limitation above is **wrong for this machine** (this repo's usual dev
machine — Apple M2 Max, real built-in Retina display) and should not be repeated in a future
round without first just trying `cargo tauri dev` here. Confirmed by actually launching it:

- Real window opens (`xoft-testbed` shows as a visible process via `osascript`'s System Events
  process list).
- Corpus sidebar populates with real files from `amiga-oberon-31`.
- Clicking a corpus file loads its real content into the editor.
- A transpile ran and produced diff output in the modified pane.

**Still not confirmed** (the session ended — user closed the app — before this was checked
carefully): whether the semantic-token coloring actually renders any color at all. One early
screenshot, before any interaction, showed the sample source (`MODULE`/`BEGIN`/`UNLESS`/`DO`/
`END`) in plain black text with no visible keyword/type/string coloring — either the provider
isn't attaching, the theme isn't applying the legend's token types, or the screenshot simply
predates the semantic-tokens request completing (Monaco computes them asynchronously after the
model is set). Also unconfirmed: click-to-jump (list → editor) and click-to-select (editor →
list) — attempts to drive these via `osascript ... click at {x,y}` used the wrong coordinate
scaling (the Retina-pixel multiplier the screenshot tool reports, instead of that multiplier
halved again for logical points, which is what `osascript`'s `click at` expects on a 2x display)
and landed on the wrong UI elements before the pass was abandoned.

**To resume this check**: launch `cargo tauri dev` from the repo root, load a file with real
keyword/type/string content (not just the `UNLESS` sample), and look directly at whether tokens
are colored — no scripted clicking needed for that first check. For scripted interaction instead
of eyeballing, convert screenshot-image coordinates to `click at` coordinates as
`displayed_coord × (full_res_dimension / displayed_res_dimension) / 2` (not the raw multiplier
the screenshot tool states, which is Retina-pixel scale, not logical-point scale), and confirm
the target window is actually frontmost before clicking (`click at` hits whatever window is
topmost at that screen point system-wide, not scoped to a chosen process).

**M6 declared done** (M6.1 + M6.2 + M6.3 all complete; the manual interactive pass above is a
verification follow-up, not a blocker for M6's own exit). Next is M7 (Opus-tagged in
`docs/plan.md`: phase-2 plan written from the corpus report, the allowlist, and the measured
Oberon-X cost) — or finishing the manual verification above first, if the user prefers that
before moving on.

### Addendum — manual `cargo tauri dev` pass, completed (2026-08-26, round 41)

Finished the pass round 40 left incomplete. Both open checks now have a definitive answer, and
the pass surfaced two real bugs plus one incidental environment finding.

**1. Semantic-token coloring does not render — confirmed real bug, not a stale screenshot.**
Loaded `oberon-a`'s `examples/Oberon0/Oberon0.Mod` and `examples/Oberon0/GraphicElems0.Mod`
(real `MODULE`/`IMPORT`/`CONST`/`TYPE`/`POINTER TO`/`RECORD`/`VAR`/`PROCEDURE`/`BEGIN`/`IF`/
`ELSIF`/`REPEAT`/`NEW` variety), ran Transpile, waited for the async semantic-tokens request to
settle: every keyword, type, and string still renders in plain black, identical to identifiers.
Round 40's three candidate explanations (provider not attaching, theme not applying the legend's
token types, or the screenshot predating the async request) are narrowed to the first two — the
async-race explanation is ruled out, since this was checked well after load and after a
completed Transpile. Not root-caused this round (out of scope for a verification pass per
`CLAUDE.md`'s test-first method — needs its own TDD fix); worth checking first whether Monaco's
`registerDocumentSemanticTokensProvider` is actually being invoked for the `"oberon2"`/
`"oberon-x"` language ids at all (e.g. a legend/provider registration mismatch) before assuming
the query layer is at fault, since `docs/progress/m6-testbed.md`'s M6.3 section already verified
the parsing+query layer directly against both grammars with a throwaway Node script.

**2. Bidirectional click navigation — confirmed working, both directions.** Loaded
`examples/amok/IntuiPointer/IntuiPointerDemo.mod` (a real, already-known parse failure —
`corpus/allowlist.toml`'s `$IF OberonA`/`$ELSE`/`$END` conditional-compilation case), ran
Transpile, got one real diagnostic (`31:3-74:1: unexpected token in module body`). Clicking the
diagnostic list item selected the matching span in the editor (list→editor). Clicking inside that
span in the editor (a fresh corpus-file load first, to start from an unselected state)
highlighted the matching list item (editor→list). Both directions work as designed.

**3. Real bug: `manifest::build` aborts entirely on the first unreadable corpus root.**
`corpus/roots.toml`'s `amiga-oberon-31` path lives under `~/Nextcloud/...`, which wasn't
materialized on this machine's disk this session (Nextcloud selective sync gap, a machine-local
environment fact, not a repo bug) — `ls` on it returned "No such file or directory" even though
three of the four configured roots (`oberon-a`, `stj`, `voc`) resolve fine. `manifest::build`
(`crates/xoft-cli/src/manifest.rs:53`) loops over roots and uses `?` on `WalkDir`'s per-entry
`Result` inside the loop; the very first `io::Error` (root doesn't exist) short-circuits the
*entire* function, discarding every root already walked and every root still to come. Effect: one
bad machine-local path blanked the *whole* corpus sidebar (all four roots), not just the broken
one's section — confirmed by temporarily commenting out the `amiga-oberon-31` entry (reverted via
`git checkout` immediately after, `corpus/roots.toml`'s tracked content is unchanged), which let
the other three roots' ~200+ files populate normally. Fix (not applied this round, verification
only): `build` should collect a per-root `Result` and continue past a failing root, surfacing that
root's failure without discarding the others' file lists — same shape as `corpus_run.rs`'s
existing per-file outcome aggregation, applied one level up.

**4. Real bug: the diff editor's "original" (source) pane is not actually editable by a user.**
`main.ts`'s own comment says the original model is "the live, editable source (typed, or loaded
from the corpus picker)", and `diffEditor.getModifiedEditor().updateOptions({ readOnly: true })`
is called only on the *modified* side — but Monaco's `createDiffEditor` defaults
`originalEditable` to `false`, and `main.ts:35-38` never passes `originalEditable: true` in the
constructor options. Net effect: **both** panes reject keyboard input ("Cannot edit in read-only
editor"), confirmed by clicking into the original pane and typing. Loading a file via the corpus
picker still works, because `originalModel.setValue(...)` is a direct model mutation that bypasses
the editor's read-only UI gate — so the one interaction path this round's checklist could
fall back to (corpus click) masked the bug, while the documented fallback ("type something invalid
directly into the editor") is the one path that's actually broken. Fix (not applied — same
verification-only scope as #3): add `originalEditable: true` to the options object at
`testbed-ui/src/main.ts:35-37`.

**5. Incidental: the app window has no opaque background, letting other windows show through.**
Discovered while screenshotting for #1/#2, unrelated to what was being checked: `#corpus-list`'s
area (and the general `body`/`#app` background) render fully transparent where no content
draws — this Tauri window is apparently configured transparent (or defaults to it) and nothing in
`style.css` sets an opaque `background-color` on `body`/`#app`. In practice this meant whatever
window happened to be stacked behind `xoft-testbed` at the time (this session: a terminal, an
email client, a browser) was visible through the corpus sidebar's empty space. No source-code fix
attempted this round; noted here because it's a real, previously-undiscovered UI defect (not just
a screenshot artifact) and because it caused an incidental, transient exposure of unrelated
on-screen content during this verification session — those screenshots were deleted immediately
after being reviewed rather than kept. Fix, when picked up: an explicit `background: <theme
color>` on `body` in `testbed-ui/src/style.css` closes it with a one-line change.

`corpus/roots.toml` and `crates/xoft-testbed/Cargo.toml` (the latter's known incidental
`cargo build` rewrite, per the "State of the tree" note below) were both restored via `git
checkout` before this round ended; `git status` is clean. No source files changed this round —
pure verification, per the plan.

### Addendum — round 42 (2026-08-29): all four round-41 bugs fixed and re-verified

All four findings above are now fixed, test-first, and re-confirmed in a real `cargo tauri dev`
window. Order: the three small fixes first, then the open-ended coloring investigation, per the
user's explicit choice when this round started.

**Finding #3 (`manifest::build` abort-on-bad-root), fixed.** `crates/xoft-cli/src/manifest.rs`:
split the per-root walk into `walk_root(root: &Root) -> Result<(RootSummary, Vec<Entry>)>`; `build`
now loops over roots, collects each into `summaries`/`files` on `Ok`, or into a new
`Manifest.failures: Vec<RootFailure>` (`{ alias, error }`) on `Err`, and continues — same shape as
`corpus_run.rs`'s existing `aggregate` (per-file, one level down). `build`'s signature dropped
`Result` entirely: once every root-level failure is caught internally there's no remaining error
path, so the wrapper was never meaningful (illegal states unrepresentable). Three call sites
updated (`corpus_run.rs:147`, `main.rs:62`, `xoft-testbed/src/commands.rs:65`); the CLI's
`corpus manifest` command now also prints any failures to stderr. New test
`continues_past_an_unreadable_root` in `crates/xoft-cli/tests/manifest.rs`, written and confirmed
red (compile failure against the pre-refactor signature) before implementing.

**Finding #4 (source pane not editable), fixed, plus new Vitest infra.** Root cause was exactly as
diagnosed: `createDiffEditor` never passed `originalEditable: true`. The user asked for this fix to
get an automated regression test rather than a manual-only check, so this round also added Vitest
to `testbed-ui` (previously zero JS/TS tests existed). The real obstacle: `main.ts` is one
top-level imperative script — `document.getElementById`, `createDiffEditor`, `window.__TAURI__`
access, and `void loadCorpus()` all run immediately at module-import time, so importing it in any
test environment throws immediately outside a real Tauri window (confirmed empirically — see
below). Rather than restructure `main.ts` into an init function (bigger than this bug needed), the
diff editor's construction options moved into a new, zero-runtime-side-effect
`testbed-ui/src/editor-config.ts` (type-only `monaco-editor` import, so even Monaco's own runtime
never loads at test time) — `main.ts` now just imports `DIFF_EDITOR_OPTIONS` from it. Test written
and confirmed red first (`Cannot find module './editor-config'`) before the file existed.

**Finding #5 (transparent background), fixed.** One line: `background-color: Canvas;` on `body` in
`testbed-ui/src/style.css` — the CSS4 system-color keyword that follows the OS light/dark setting,
consistent with the file's existing `color-scheme: light dark`. No test added (pure-CSS visual
fix, matches the project's established manual-re-check scope for this kind of change; the user's
test-framework decision was scoped to finding #4 specifically).

**Finding #1 (semantic-token coloring), root-caused and fixed.** The `main.ts`-side hypothesis from
round 41 (Monaco's `semanticHighlighting.enabled` gate, off by default even with a correctly
registered provider) was right in substance but wrong in *where* to set it: a `.updateOptions()`
call on each constituent editor **after** `setModel` had no effect. Root cause, found by direct
inspection of `monaco-editor`'s bundled source
(`esm/vs/editor/contrib/semanticTokens/browser/documentSemanticTokens.js`): Monaco's
`DocumentSemanticTokensFeature` runs a **one-time** scan over all existing models at construction
(`modelService.getModels().forEach(model => { if (isSemanticColoringEnabled(...)) register(model) }`)
— this is what schedules a model to ever have `provideDocumentSemanticTokens` called on it at all.
That construction happens synchronously inside `createDiffEditor`, strictly before any code that
runs after it — so a later `updateOptions()` call is too late; the model is simply never
registered, and the provider is silently never invoked (confirmed with temporary `console.log`
instrumentation inside `provideDocumentSemanticTokens`/`getLegend`: zero calls, no exceptions).
`isSemanticColoringEnabled` does have a reactive path (`configurationService.onDidChangeConfiguration`
re-scans all models when `affectsConfiguration('editor.semanticHighlighting')`), but a diff editor's
constituent-editor `updateOptions()` calls did not trigger it in practice. Fix: pass
`"semanticHighlighting.enabled": true` **inside `createDiffEditor`'s own construction options
object** (`editor-config.ts`'s `DIFF_EDITOR_OPTIONS`, typed as
`editor.IDiffEditorConstructionOptions & editor.IGlobalEditorOptions` since the key is
`IGlobalEditorOptions`-only and TypeScript's `IDiffEditorConstructionOptions` doesn't declare it,
even though Monaco accepts and forwards it at runtime). Verified empirically, not just by reading
source — see below.

**How the coloring root cause was actually found**, since it's a reusable technique: `main.ts`
can't be loaded standalone (hard `window.__TAURI__` dependency, throws immediately in a plain
browser tab — confirmed by trying), and the real Tauri window can't be attached to with Chrome
DevTools Protocol (it's WKWebView, not Chromium) or easily instrumented from outside. Built a
throwaway `testbed-ui/debug.html` + `src/debug.ts` (deleted before commit, never part of the
diff) that exercises the exact same `registerHighlighting()` + `createDiffEditor(..., DIFF_EDITOR_OPTIONS)`
call shape as `main.ts`, minus the Tauri dependency, served by the same already-running Vite dev
server. Drove it headlessly with `Brave Browser --headless=new --remote-debugging-port=9333`
(already installed, no new dependency) and a small Node script talking raw Chrome DevTools
Protocol over Node's built-in `fetch`/`WebSocket` (no `puppeteer`/`playwright` — Node 22+'s native
`WebSocket` global is enough for `Runtime.enable`/`Page.reload`/`Runtime.evaluate`/console capture).
This let three things happen that the real Tauri window's AppleScript-driven workflow can't: read
`getComputedStyle(...).color` on every `.mtk*` span programmatically instead of eyeballing a
screenshot, see console output including temporary debug instrumentation, and catch uncaught
exceptions directly (`Runtime.exceptionThrown`) rather than a silently blank result. Once the fix
was confirmed this way (colored `mtk*` classes, non-black `getComputedStyle` colors), it was
re-verified visually in the real `cargo tauri dev` window with a real corpus file
(`examples/Oberon0/Oberon0.Mod`) as the final check, since rendered pixel color in the actual
target environment is still the thing that matters, not the headless proxy.

**Manual re-verification, real `cargo tauri dev` window** (AppleScript + `screencapture`, per
round 41's documented technique): corpus sidebar shows `oberon-a`'s full file list even though
`amiga-oberon-31` still doesn't resolve on this machine (Fix #3, no `roots.toml` edit needed this
time — the missing root was already the ambient condition); window background solid, no
stacked-window bleed-through (Fix #5); loaded `examples/Oberon0/Oberon0.Mod`, ran Transpile,
keywords/types rendered in color in both diff panes (Fix #1); clicked into the source pane and
typed — text inserted directly, confirming real keyboard editability (Fix #4).

`cargo test --workspace` green (`xoft-cli` gained 1 test), `cargo clippy --workspace --all-targets`
clean, `testbed-ui`: `npx tsc --noEmit` clean, `npm run build` succeeds, `npm test` (new) green (3
tests). `crates/xoft-testbed/Cargo.toml`'s known incidental `cargo build` rewrite (adds
`features = []`) was restored via `git checkout` before this round ended, same as round 41.
