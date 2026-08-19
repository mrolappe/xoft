# Next task

**M6.1 — Tauri shell linking `xoft-core`; commands `transpile`, `roundtrip_check`,
`list_corpus`.** Tagged **Sonnet** in `docs/plan.md` line 141 ("serde types shared with the
CLI"). First task of M6 (Testbed) and the first task of this repo to touch a new toolchain —
no Tauri, Node, or frontend tooling exists here yet. M5 (Oberon-X toy dialect) is done; see
`docs/progress/m5-oberon-x.md` for its exit write-up if the testbed ever wants to demo the
dialect mapping.

## Scope for M6.1 specifically

Backend only. `docs/plan.md` splits the testbed into three tasks on purpose:

- M6.1 (this one): Tauri shell + the three commands, callable and testable, no real UI.
- M6.2 (Haiku): Vite + Monaco `DiffEditor` frontend — "fully specified layout" per the plan, i.e.
  a follow-up task will hand you the layout rather than you inventing one.
- M6.3 (Sonnet): `web-tree-sitter` highlighting + clickable diagnostics.

Do not build M6.2/M6.3 scope now. A bare Tauri window that can invoke the three commands (from
the dev console, a test, or a placeholder button) is enough to call M6.1 done.

## What already exists (reuse, don't reimplement)

- `crates/xoft-core`: `grammar::{language, language_oberon_x}`, `codec::Document`,
  `serialize::{walk, walk_with, reconstruct}`, `diagnostic::Diagnostic` + the walk that produces
  them, `mapping::{to_oberon2, to_oberon_x}`. No I/O, no rendering — reuse directly.
- `crates/xoft-cli/src/check.rs`: `check_source`/`check_file` — parse + diagnostics +
  `codespan-reporting` rendered text. `roundtrip_check` almost certainly wants the structured
  `CheckResult` (diagnostics + round-trip bool), not the rendered string.
- `crates/xoft-cli/src/transpile.rs`: `transpile_source`/`transpile_file` — check + lossless
  round-trip via the serializer. This is *not* the Oberon-X/Oberon-2 `mapping` transform (see
  `NEXT.md` history, M5.3 round 37 — "transpile" in this repo has meant "check + lossless
  round-trip" since round 30, mapping was deliberately never wired into the CLI). Decide up front
  whether the testbed's `transpile` command means the same thing, or means `mapping::to_oberon2`/
  `to_oberon_x` — the plan doesn't disambiguate and the name collision with the CLI's existing
  `transpile` is exactly the kind of ambiguity `CLAUDE.md` says to stop and ask about rather than
  guess.
- `crates/xoft-cli/src/manifest.rs`: `manifest::build` — walks corpus roots into a
  `Manifest`/`Entry` list with `serde` derives already in place. `list_corpus` is very likely a
  thin wrapper over this, reusing `RootsConfig`/`corpus/roots.toml` loading from
  `crates/xoft-cli/src/main.rs`'s `Corpus::Manifest` arm rather than re-deriving it.

## Real decisions to make before coding (ask if genuinely unclear, per `CLAUDE.md`)

1. **What does `transpile` mean for the testbed** — CLI's check+round-trip, or the Oberon-X↔
   Oberon-2 `mapping` functions? The `DiffEditor` mentioned in M6.2 strongly suggests showing an
   Oberon-X source next to its Oberon-2 mapping (or vice versa), which would mean the testbed is
   the *first* consumer of `mapping.rs` outside its own test suite — worth confirming rather than
   assuming.
2. **Crate layout**: `crates/xoft-testbed/` per `docs/plan.md` line 50. A Tauri app is normally
   two halves (Rust backend in `src-tauri/`, JS/TS frontend) — decide whether both halves live
   under `crates/xoft-testbed/` (keeping the "MVP source lives in `crates/`" convention) or the
   frontend gets its own top-level dir, before running `cargo tauri init`/`npm create tauri-app`.
   Add the new member to the root `Cargo.toml` workspace (see the checklist: "workspace member
   listed before its manifest existed" is a documented past mistake — write the crate's
   `Cargo.toml` in the same step you add it to `members`).
3. **CI**: `.github/workflows/ci.yml` currently runs `cargo test --workspace` plus the corpus
   fixture check. A Tauri crate pulls in a much heavier native toolchain (webview libs on
   Linux especially) — decide whether M6.1 wires CI for it now or defers, and say so explicitly
   in the round's progress notes either way, the way M4.2 vs. the real-corpus-in-CI question was
   handled.

## Not in scope

M6.2's frontend layout, M6.3's highlighting/diagnostics UI, M7 (Phase 2 plan, Opus-tagged,
written from the corpus report + allowlist + M5's measured dialect cost).

## State of the tree

- `cargo test --workspace` green: `xoft-core` 38, `xoft-cli` 15.
- `tree-sitter test` green in both grammar dirs (85 + 89).
- M5 is fully done (M5.1–M5.3); its exit finding — additive constructs round-trip losslessly,
  aliases don't, regardless of code size — is in `docs/progress/m5-oberon-x.md` and
  `docs/insights.md` round 36, worth a skim before designing any future dialect features the
  testbed might want to demo.
- No Tauri/Node tooling installed or vendored in this repo yet; M6.1 is starting that from
  nothing.
