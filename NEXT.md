# Next task

**M6.3 — `web-tree-sitter` highlighting, `ERROR` nodes marked, clickable diagnostics.** Tagged
**Sonnet** in `docs/plan.md` line 143 ("semantic-tokens provider, not Monarch").

## What M6.2 already built (reuse, don't reimplement)

`testbed-ui/` is now a real Vite + TypeScript app (`crates/xoft-testbed/` unchanged backend
except one new command) — see `docs/progress/m6-testbed.md`'s M6.2 section for the full round
writeup:

- **`src/main.ts`** wires one `monaco.editor.createDiffEditor` (`diffEditor`, module-scope) as
  both the editable source pane (`originalModel`) and the diff view (`modifiedModel`, holds
  `transpile`'s `output`, read-only). Diagnostics render as a plain `<ul id="diagnostics">` fed
  by `TranspileResult.diagnostics` inside `renderDiagnostics()`. M6.3's "clickable diagnostics"
  and "`ERROR` nodes marked" both extend this same file — `renderDiagnostics` is the natural
  place to add click handlers, and the DiffEditor's `originalModel`/Monaco's decoration API
  (`monaco.editor.createDecorationsCollection` or `IModelDeltaDecoration`) is how `ERROR` spans
  would get marked.
- **`Diagnostic.start_byte`/`end_byte` are still raw byte offsets**, not Monaco
  `{lineNumber, column}` and not JS string indices (UTF-16 code units) — M6.2 deliberately
  deferred this conversion (decision 2 from the M6.2 planning round). This is now M6.3's
  problem to solve: `xoft_core::codec::Document` already maps each byte to one `char`
  (D3's bijection), so a byte offset → Monaco position conversion needs that codec, not a
  naive `text.slice(byteOffset)`. Decide whether the conversion happens in Rust (a new command
  or a widened `Diagnostic`) or in TS (shipping enough of the byte↔char mapping to the
  frontend) — this is exactly the kind of thing to confirm with the user before coding, per this
  project's "ambiguous syntax, ask" rule (applies to design ambiguity too, not just grammar).
- **`web-tree-sitter`** (the WASM build of tree-sitter, for in-browser parsing) is not wired up
  anywhere yet — M6.2 never parses in the frontend at all, it only calls the three/four backend
  commands. M6.3 is the first round that needs a `.wasm` grammar artifact
  (`tree-sitter build --wasm` on `grammars/tree-sitter-oberon2/` and/or
  `tree-sitter-oberon-x/`) and a way to load it into the Vite app (likely another `?url`-style
  Vite asset import, or copying into `public/`). No existing precedent in this repo for shipping
  a `.wasm` asset through Vite — expect this to need its own small investigation, not just
  "add the npm package."
- **Backend security boundary, already closed**: `list_corpus`/`read_corpus_file` no longer
  accept `roots_toml` over IPC at all (`lib.rs`'s wrappers read `corpus/roots.toml` from disk
  themselves). Don't reintroduce a `roots_toml` parameter on any new M6.3 command without
  re-reading `docs/insights.md` round 39 first — the lesson there (a new command mirroring an
  existing parameter shape inherits that shape's trust boundary, not just its convenience)
  applies directly if M6.3 adds another IPC command.
- `cargo tauri dev` still hasn't been verified in a real window in this environment (no display
  server) — M6.1 and M6.2 both flagged this. Worth doing on a machine with a display before or
  during M6.3, since M6.3 is the first round that would visibly show `ERROR`-node highlighting
  and click-to-diagnostic behavior — those are much harder to verify by reading code than a
  plain DiffEditor.

## Real decisions to make before coding

`docs/plan.md` line 143 only says "`web-tree-sitter` highlighting, `ERROR` nodes marked,
clickable diagnostics." Likely open questions, not yet resolved with the user:

1. **Highlighting scope**: full syntax highlighting (a Monaco semantic-tokens provider driving
   `web-tree-sitter`'s parse tree, per the plan's own "semantic-tokens provider, not Monarch"
   note) for both grammars (`tree-sitter-oberon2`, `tree-sitter-oberon-x`), or just enough to
   mark `ERROR` node spans distinctly? These could be two separate, independently-shippable
   pieces of work.
2. **Byte offset → Monaco position conversion** (carried over from M6.2, see above): where does
   it live, Rust or TS?
3. **Click behavior**: does clicking a diagnostic in the list jump the editor's cursor/selection
   to that span (needs the conversion above either way), or does it also need reverse
   navigation (clicking a squiggle in the editor highlights the matching list entry)?
4. **`.wasm` asset delivery**: how does the grammar's compiled WASM get from
   `grammars/tree-sitter-oberon2/` into something Vite serves — build step, checked-in artifact,
   or generated at `npm run build` time alongside the existing `tree-sitter generate` CI step?

## Not in scope

CI wiring for `xoft-testbed` (deferred twice now, M6.1 and M6.2); M7.

## State of the tree

- `cargo test --workspace` green: `xoft-core` 38, `xoft-cli` 15, `xoft-testbed` 8.
- `cargo clippy --workspace --all-targets` clean.
- `testbed-ui`: `npx tsc --noEmit` clean, `npm run build` (Vite) succeeds. `npm install` has been
  run in `testbed-ui/` on this machine — `node_modules/`/`dist/` both gitignored (added to
  `.gitignore` this round).
- `tree-sitter test` unchanged by M6.2 (85 + 89, untouched this round).
- Not verified: `cargo tauri dev` in a real window (no display server here, same limitation
  since M6.1).
