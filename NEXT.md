# Next task

**M6.2 — Vite + Monaco `DiffEditor` frontend and controls.** Tagged **Haiku** in `docs/plan.md`
line 142, on the premise that it "receives a fully specified layout" rather than inventing one.
That layout does not exist yet anywhere in this repo — writing it (or getting it from the user)
is real judgment work and belongs in *this* planning step, before the task is handed off, not
inside the Haiku task itself. See "Real decisions to make" below.

## What M6.1 already built (reuse, don't reimplement)

`crates/xoft-testbed/` (new workspace member, Rust/Tauri backend) exposes three
`#[tauri::command]`s over IPC — see `docs/progress/m6-testbed.md` for the full round writeup:

- **`list_corpus(rootsToml: string) -> Manifest`** — the command's own argument name is
  camelCase (Tauri's default convention for `#[tauri::command]` parameters), but the return
  value reuses `xoft_cli::manifest::Manifest` verbatim, whose fields are plain `#[derive(Serialize)]`
  with no `rename_all` — so the JSON that comes back is **snake_case**: `{ roots:
  RootSummary[], files: Entry[] }`, `Entry` is `{ root, path, bytes, sha256, line_endings,
  encoding, has_tabs }`. Deliberate, not an oversight: `Manifest`/`Entry`/`RootSummary` are the
  same types that produce the checked-in `corpus/manifest.json`/`reports/corpus-report.json`,
  so renaming their fields for the testbed's benefit would be a breaking change to those
  committed artifacts and to CI's `git diff --exit-code reports/` check — out of scope for a
  Tauri command that just reuses the type as-is.
- **`roundtrip_check(filename: string, raw: number[]) -> RoundtripResult`** — `raw` is the
  file's raw bytes as a plain JS array (Tauri deserializes it into `Vec<u8>`).
  `RoundtripResult` (new in M6.1, `commands.rs`) is also plain-derived, so also
  **snake_case**: `{ parse_ok, round_trip_ok, diagnostics }`. `Diagnostic` (`xoft_core::diagnostic`,
  gained `Serialize` this round) is `{ start_byte, end_byte, message }` — byte offsets into
  the *input* text, not line/column, and bytes not UTF-16 code units (see decision 2 below).
- **`transpile(direction: "oberon-x-to-oberon2" | "oberon2-to-oberon-x", text: string) ->
  TranspileResult`** — `TranspileResult` (new in M6.1) is `{ output: string, diagnostics:
  Diagnostic[] }`, same snake_case `Diagnostic` shape. This is the Oberon-X↔Oberon-2 dialect
  mapping (`xoft_core::mapping`), confirmed with the user in M6.1 — **not** the CLI's `xoft
  transpile` (check + lossless round-trip). This is what should drive the `DiffEditor`: left
  pane = input, right pane = `output`.
- Every field above is snake_case in the actual JSON (verified empirically this round, not
  assumed) — M6.2's TS types should match that rather than guessing camelCase. Command
  *argument* names (`rootsToml`, `filename`, `raw`, `direction`, `text`) are the one place
  Tauri does auto-convert to camelCase; `crates/xoft-testbed/src/commands.rs` and
  `crates/xoft-testbed/tests/commands.rs` have the exact shapes either way.
- `testbed-ui/index.html` is the current placeholder — a single static page, no bundler, three
  raw `invoke()` calls wired to buttons. M6.2 replaces this with the real Vite app; nothing in
  it is worth preserving except as a reference for the exact `invoke()` call shapes.
- `crates/xoft-testbed/tauri.conf.json`: `build.frontendDist` currently points straight at
  `../../testbed-ui` (the static file, no build step). Once Vite is introduced this almost
  certainly needs to change to `../../testbed-ui/dist` (or wherever Vite's `outDir` is
  configured to write) for `tauri build`, plus a `build.devUrl` (e.g.
  `http://localhost:1420`) and `build.beforeDevCommand`/`beforeBuildCommand` (`npm run dev` /
  `npm run build`, run from `testbed-ui/`) for `cargo tauri dev` to proxy to Vite's dev server
  instead of serving the static file directly. This is exactly the kind of "fully specified"
  detail M6.2 needs handed to it rather than deciding itself.
- `crates/xoft-testbed/capabilities/default.json` currently just has `"core:default"`. No
  Tauri fs/shell/dialog plugins are wired up (M6.1 didn't need any — the three commands are
  plain app commands, not plugin-backed), so M6.2 shouldn't need capability changes unless the
  frontend needs to trigger a native "open file" dialog rather than pasting text into a
  textarea.

## Real decisions to make before coding

`docs/plan.md` line 142 only says "Vite + Monaco `DiffEditor` frontend and controls." It does
not specify:

1. **Layout.** How many panes, and mapped to which command? A plausible minimal shape: one
   source textarea/editor pane, a direction toggle (`oberon-x-to-oberon2` /
   `oberon2-to-oberon-x`), a `DiffEditor` showing input vs. `transpile`'s `output`, and a
   separate diagnostics panel/list fed by `roundtrip_check` (and/or `transpile`'s own
   `diagnostics`) — but this is a guess, not a spec. `list_corpus`'s role in the layout is
   also unclear: a file picker sourced from a corpus manifest, or is it out of scope for the
   diff view entirely (a separate tab/panel)?
2. **Diagnostics presentation.** `Diagnostic.startByte`/`endByte` are byte offsets, not
   line/column. Monaco wants `{ lineNumber, column }` or a model offset (UTF-16 code units,
   not bytes — the codec in `xoft_core::codec::Document` maps each *byte* to one `char`, so
   for any file with bytes ≥ 0x80 a byte offset and a JS string index diverge). Decide the
   conversion approach (and whether it's even needed for M6.2, or diagnostics render as a
   plain list with the offending byte range as text, deferring inline squiggles to M6.3, which
   is explicitly scoped for "clickable diagnostics").
3. **`list_corpus`'s `rootsToml` input.** Does the UI let the user type/paste TOML, browse to
   a `roots.toml` file (needs a native file dialog — a new Tauri capability), or is it wired
   to this machine's `corpus/roots.toml` by a hardcoded relative path? The last is simplest
   but reintroduces the "absolute path" concern `corpus/roots.toml` was designed to avoid
   (D8's "the only file holding absolute paths").

Once these are pinned down (ask the user rather than guessing — same rule M6.1 followed), M6.2
really can be a fully-specified, Haiku-sized task: exact component tree, exact Tauri command
calls, exact styling (or none).

## Not in scope

M6.3's `web-tree-sitter` highlighting and clickable diagnostics, CI wiring for `xoft-testbed`
(deferred in M6.1, still deferred), M7.

## State of the tree

- `cargo test --workspace` green: `xoft-core` 38, `xoft-cli` 15, `xoft-testbed` 5 (new this
  round).
- `tree-sitter test` green in both grammar dirs (85 + 89), unchanged by M6.1.
- `cargo build -p xoft-testbed` succeeds; no `cargo tauri dev` verification was possible in
  this environment (no display server) — see `docs/progress/m6-testbed.md`'s "Not verified"
  note. Worth a real `cargo tauri dev` smoke test on a machine with a display before or during
  M6.2, since M6.2 is the first round that will actually exercise the JS↔Rust IPC boundary
  from real (not hand-typed) frontend code.
- One security-review finding from M6.1 is **acknowledged, not fixed**: `list_corpus` walks
  any filesystem path the webview supplies, with no scoping. Still fine today (static,
  first-party `testbed-ui/index.html`), but M6.2 is exactly the round where this stops being
  hypothetical — worth resolving decision 3 above with this in mind, and revisiting whether
  `list_corpus` needs a capability/allowlist restriction as part of M6.2 rather than leaving it
  deferred again.
- Node/npm (`npm`, `node`) and the Tauri CLI (`cargo tauri`) are already installed on this
  machine (confirmed during M6.1), but `testbed-ui/` has no `package.json` yet — M6.2 starts
  the Vite/npm tooling from nothing, same way M6.1 started Tauri/Rust tooling from nothing.
