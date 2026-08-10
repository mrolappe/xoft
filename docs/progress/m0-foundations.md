# M0 — Foundations ✅

Completed in round 1 (2026-08-10).

**Exit criteria:** repo builds; corpus present and inventoried; language baseline pinned. All met.

## M0.1 — Workspace

`Cargo.toml` workspace with two members:

- `crates/xoft-core` — pure library. Currently one module, `corpus`, holding `FileFacts`.
- `crates/xoft-cli` — binary `xoft`, plus a `lib.rs` so integration tests can reach the
  internals (a bin-only crate cannot be imported from `tests/`).

`crates/xoft-testbed` was **not** created. It is a Tauri app due in M6; an empty stub now would
be a non-building workspace member for five milestones.

## M0.2 — Corpus manifest

`xoft corpus manifest` reads `corpus/roots.toml` and writes `corpus/manifest.json`.

```
   amiga-oberon-31   122 files     1205 KB
          oberon-a   237 files     2440 KB
               stj   306 files     1768 KB
               voc   127 files     1473 KB
             total   792 files
```

Split of responsibility follows the core design rule: `xoft_core::corpus::FileFacts::classify`
is pure (bytes in, facts out — sha256, byte count, line-ending class, UTF-8 vs high-bytes,
tabs); walking and writing live in `xoft_cli::manifest`.

Determinism is enforced by test, not by convention: `serialization_is_byte_stable_across_runs`
builds the manifest twice and compares, and also asserts the absolute root path never appears
in the output. Entries are sorted by `(root alias, relative path)` because filesystem order is
not stable across machines.

Tests: 9 in `xoft-core` (classification), 4 in `xoft-cli` (manifest). All written before the
implementation and observed failing first.

## M0.3 — Language baseline

`docs/language-baseline.md` pins **Oberon-2** (Mössenböck/Wirth, ETH, March 1995 revision) and
reproduces Appendix B's EBNF verbatim, plus the §3 lexical rules. This file is the normative
reference for M1; grammar subtasks receive fragments of it rather than the whole report.

Oberon-07 was rejected as baseline: it removes `LOOP`, `EXIT` and `WITH`, which the corpus uses
in 47, 45 and 14 Oberon-A files respectively.

## M0.4 — Provenance and licensing

Recorded per root in `corpus/roots.toml`. Corpus sources are **not** vendored into this
repository: Oberon-A, AmigaOberon 3.1 and STJ-Oberon are archived third-party code held locally
for reference, and voc is GPL-3.0. The manifest pins each file's sha256, so a run is
reproducible without redistributing anything.
