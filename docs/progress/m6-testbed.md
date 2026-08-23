# M6 — Testbed ⬜ in progress (M6.1 done, round 38, 2026-08-23)

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

Not started: M6.2 (Vite + Monaco `DiffEditor`), M6.3 (`web-tree-sitter` highlighting).
