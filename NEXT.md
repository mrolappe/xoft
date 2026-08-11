# Next task

**M3.2 — `xoft transpile` / `xoft check` + `codespan-reporting`.** Per `docs/plan.md` line 115:
"`xoft transpile` / `xoft check` + `codespan-reporting`, charset applied at render time | D3's
display layer lives here and nowhere else." M2 and M3.1 are both done; this is the first thing to
land in `xoft-cli` (I/O + rendering) rather than `xoft-core`.

## What's confirmed (do not re-derive, just verify before coding)

- `xoft-core::diagnostic::diagnostics(tree) -> Vec<Diagnostic>` exists and is real (M3.1);
  `Diagnostic { start_byte, end_byte, message }` — byte spans only, no line/column, by design
  (`crates/xoft-core/src/diagnostic.rs`).
- `xoft-core::rule::RuleRegistry` exists and is real (M2.4) but is empty in Phase 1 — `check`
  should probably run diagnostics + an (empty) `RuleRegistry::run` and merge both `Vec<Diagnostic>`
  lists, even though the registry contributes nothing yet, so the wiring is already correct once
  M5 populates it.
- `xoft-core::codec::Document` (M2.1) does the byte↔char bijection (D3); `docs/plan.md`'s "charset
  applied at render time" line means the display layer (this milestone, in `xoft-cli`) is what
  decides how a `Document`'s codepoints get shown to the user — `xoft-core` itself renders no text
  (`CLAUDE.md`'s design rule) and must stay untouched by this milestone except as a consumer.
- `xoft-cli` already has one subcommand family (`xoft corpus manifest`,
  `crates/xoft-cli/src/main.rs` + `src/manifest.rs`) — `clap` derive, `Subcommand` enum nested
  under `Command`. `transpile`/`check` should follow the same shape: new top-level `Command`
  variants (not nested under `Corpus`), with their own `src/{transpile,check}.rs` or similar,
  parallel to `manifest.rs`.
- `codespan-reporting` is not yet a dependency anywhere in the workspace — this milestone adds it
  to `xoft-cli`'s `Cargo.toml` (it belongs in the CLI crate, not `xoft-core`, per the no-I/O rule).
  It renders from byte or line/col spans plus a `SimpleFiles`/`Files` source; `Diagnostic`'s byte
  spans should slot in directly, no new span type needed on the `xoft-core` side.
- `xoft check` and `xoft transpile` are two different commands per the plan line, not one command
  with a flag — worth confirming what `transpile` actually does yet, since `docs/plan.md`'s
  transpile-proper machinery (dialect mapping rules, template splicing) doesn't exist until M5.
  **Ask the user before coding**: for Phase 1 (no dialect rules registered), should `xoft
  transpile` just be `check` plus a lossless round-trip / `strip_comments` pass (i.e. exercise
  M2's serializer end-to-end from the CLI), or should it be stubbed/deferred until M5 gives it
  real work to do? This wasn't settled in `docs/plan.md` or any prior round.

## Definition of done

- A failing-then-passing test, TDD per `CLAUDE.md`. Given `xoft-cli` is I/O-facing, tests likely
  invoke the binary or its library functions against a fixture file/tempfile (see
  `crates/xoft-cli/tests/manifest.rs` for the existing pattern) and assert on rendered diagnostic
  output (message text, span markers) for at least one clean file and one broken file.
- Update `docs/progress/m3-diagnostics-cli.md`'s M3.2 section (currently "not started").
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` only if something genuinely mistake-worthy came up.

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments,diagnostic,rule}.rs` + `build.rs`:
  all green. **M2 is done** (round 29) — its only open item, the full-corpus byte-identical
  round-trip exit measurement, explicitly waits on M4's corpus runner and isn't blocking anything
  else.
- `crates/xoft-cli/src/{manifest,main,lib}.rs`: unchanged, `xoft corpus manifest` still the only
  command.
- `grammar.js`/`src/scanner.c`: unchanged since round 24, M1 is frozen unless a new corpus gap
  surfaces.
- `cargo test --workspace`: green, 31 tests in `xoft-core` (29 → 31 this round, M2.4) + `xoft-cli`
  unchanged.
