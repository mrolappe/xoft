# xoft — MVP plan

**xoft** is a workbench for designing Oberon-flavored dialects: a lossless, bidirectional
transpiler between Oberon-2 and an open-ended set of experimental dialects, where comments and
formatting survive the round trip by default.

Derived from `oberon-transpiler-handoff.md`, amended after grilling the project owner. Three of
the handoff document's premises did not survive:

1. **The goal is dialect experimentation, not one fixed transpiler.** Several Oberon-flavored
   dialects will be designed over time. The metric that matters is *how cheap is one dialect
   experiment* — which inverts the handoff's §4.2 typed-union-AST recommendation, since that
   design turns every experiment into a Rust refactor.
2. **The corpus is real, local, and is not Standard Oberon** (see below). A pure Oberon-2
   grammar cannot parse it.
3. **The handoff's acceptance criterion is vacuous.** `serialize(parse(s)) == s` is satisfied by
   `s.clone()`. The invariant with teeth is *byte coverage*.

## Corpus (measured)

| Root alias | Files | Size | Line ends | Encoding | Nested comments | `(*$` | `DEFINITION` |
|---|---|---|---|---|---|---|---|
| `oberon-a` | 237 `.mod` | 2.4 MB | LF | Latin-1 (185 non-UTF-8) | 25 files | 9 | 1 |
| `stj` | 194 `.mod` + 112 `.def` | 1.8 MB | CRLF | Atari/CP437-ish (33) | 10 files | 39 | 72 |
| `amiga-oberon-31` | 122 `.mod` | 1.2 MB | LF | Latin-1 (108) | 13 files | 78 | 0 |
| `voc` | 127 `.mod` | 1.4 MB | LF | UTF-8 (4 exceptions) | — | — | — |

792 files. `LOOP`/`EXIT`/`WITH`/`RETURN`-as-statement and type-bound `PROCEDURE (r: T)` are all
heavily used; `INLINE` assembly appears in ~22 files. Absolute paths live only in
`corpus/roots.toml`; everything else refers to files by `(root alias, relative path)`.

## Decisions

| # | Decision |
|---|---|
| D1 | Grammar = full Oberon-2 **+ lexical superset** that swallows dialect files without `ERROR`: nested comments, `(*$…*)` as a comment subtype, `INLINE` block as an opaque token, `DEFINITION` module header. Understand the core, tolerate the rest. |
| D2 | Base grammar = fork of [`viegasfh/tree-sitter-oberon-2`](https://github.com/viegasfh/tree-sitter-oberon-2) (MIT, 500 lines, ~60% usable). Lift `queries/highlights.scm` from [`geekstakulus/tree-sitter-oberon-07`](https://github.com/geekstakulus/tree-sitter-oberon-07) (MIT). Both need regenerating against tree-sitter 0.26. |
| D3 | Encoding = **byte↔U+0000-00FF bijection** in the core; parse the mapped text, map back on output. Byte-identity by construction for any single-byte charset, zero charset tables. The real charset is a per-file **display** attribute applied by CLI/testbed only. Safe because Oberon identifiers are ASCII — high bytes occur only in comments and strings. |
| D4 | Serializer = **token-walk + byte-coverage assertion**: emit leaf text and inter-leaf gaps; assert output == input bytes, zero `ERROR`/`MISSING`, and that every byte is covered by exactly one leaf or one gap, gaps containing only whitespace and comments. |
| D5 | Mapping layer = **untyped CST + `.scm` queries**, transform by text splicing. No typed union AST, no `type-sitter`. A new dialect = grammar overlay + query file + small transform fn. Queries double as the catalog encoding and the corpus-frequency counter. |
| D6 | Phase 1 scope = M0–M6. **Not built:** pretty-printer, typed AST, `proptest`, `cargo-fuzz`, `tracing` subscribers, LSP server, TextMate grammar, editor integrations. |
| D7 | Phase 1 includes one toy dialect **Oberon-X** (one keyword rename + one added construct) as a grammar-inheritance overlay, with two mapping rules and bidirectional round-trip tests — so the MVP measures the cost of a dialect experiment. |
| D8 | Done = zero `ERROR`/`MISSING` + byte-identical round-trip on 100% of the corpus **minus `corpus/allowlist.toml`** (capped at 5% of files), each entry carrying a one-line reason. The allowlist is the Phase 2 backlog. |

## Layout

```
crates/xoft-core/      # codec, CST wrapper, serializer, rule registry, Diagnostic — no I/O
crates/xoft-cli/       # binary `xoft`: corpus | transpile | check
crates/xoft-testbed/   # Tauri app (M6)
grammars/tree-sitter-oberon2/    # base grammar + external scanner
grammars/tree-sitter-oberon-x/   # toy dialect, extends the base
corpus/roots.toml      # the only file holding absolute paths
corpus/manifest.json   # generated inventory: root, relative path, sha256, facts
corpus/allowlist.toml  # excluded files + reason (D8)
corpus/cases/          # hand-written per-construct minimal cases
docs/language-baseline.md        # pinned Oberon-2 + normative EBNF
docs/catalog.md                  # dialect-difference catalog (scaffolding)
reports/corpus-report.json       # deterministic, checked in
```

**Design rule:** `xoft-core` performs no I/O and renders no text. All output paths consume
structured data from it.

## Milestones

Model column = the **minimum** model that can do the task well. Haiku for mechanical work
against a fully specified target; Sonnet for spec-driven implementation needing local judgment;
Opus only where a wrong choice is expensive to reverse.

### M0 — Foundations ✅

| Task | Model | State |
|---|---|---|
| M0.1 Cargo workspace + crate skeletons | Haiku | done |
| M0.2 Corpus manifest generator | Haiku | done — 792 files |
| M0.3 `docs/language-baseline.md` with normative EBNF | Sonnet | done |
| M0.4 Corpus provenance + licensing | Sonnet | done — `corpus/roots.toml` |

### M1 — Grammar

| Task | Model | Notes |
|---|---|---|
| M1.1 Vendor the base grammar, regenerate under tree-sitter 0.26, existing tests green | Haiku | MIT attribution in the vendored dir |
| M1.2a Declarations: type-bound `PROCEDURE (r: T) M*`, `ForwardDecl` `^`, `DEFINITION` header | Sonnet | receives the declaration EBNF only |
| M1.2b Statements: `WITH`, `LOOP`, `EXIT`, `RETURN`, `CASE` label ranges, empty statements | Sonnet | receives the statement EBNF only |
| M1.2c Expressions/types: `IS`, `SET` literals with ranges, open arrays, procedure types | Sonnet | receives the expression/type EBNF only |
| M1.3 External C scanner: nested comments + `(*$…*)` pragma node | Sonnet | ~80 lines; reference `tree-sitter-pascal`. Escalate to Opus only if scanner state serialization misbehaves |
| M1.4 Lexical superset (`INLINE` opaque token), iterated against real parse failures | Sonnet | needs a parse-only corpus script or M4.1 |
| M1.5 `queries/highlights.scm` | Haiku | port from the Oberon-07 grammar |

M1.2 is split three ways so each task carries one EBNF fragment plus the matching grammar
section rather than the whole report — the sections are disjoint, so there is no coordination
cost.

**Exit:** ≥95% of corpus files parse with zero `ERROR`/`MISSING`; one `tree-sitter test` case
per construct.

### M2 — Core: parse and serialize losslessly

| Task | Model | Notes |
|---|---|---|
| M2.1 `codec.rs`: byte↔char bijection + `Document` | Haiku | ~30 lines; property test over all 256 byte values |
| M2.2 Token-walk serializer + byte-coverage assertion (D4) | Sonnet | must report *which* byte range is uncovered |
| M2.3 `strip_comments` | Sonnet | pragma comments are kept — they are semantics. Output must re-parse |
| M2.4 Rule registry: query-driven traversal, empty in Phase 1 | Sonnet | shape only; filled in M5 |

**Exit:** byte-identical round-trip on 100% of non-allowlisted corpus.

### M3 — Diagnostics and CLI

| Task | Model | Notes |
|---|---|---|
| M3.1 `Diagnostic` + `ERROR`/`MISSING` walk with context-based message upgrading | Sonnet | byte spans throughout; a `MISSING` node's kind *is* the message |
| M3.2 `xoft transpile` / `xoft check` + `codespan-reporting`, charset applied at render time | Sonnet | D3's display layer lives here and nowhere else |
| M3.3 Broken-source fixtures + `insta` snapshots | Haiku | ~8 hand-written broken files |

### M4 — Corpus runner

| Task | Model | Notes |
|---|---|---|
| M4.1 `xoft corpus run` → `reports/corpus-report.json`, honoring the allowlist | Sonnet | sorted keys, relative paths, no timestamps; metrics: parse %, round-trip %, failure histogram, per-root breakdown |
| M4.2 CI: fail on undiffed report change | Haiku | `cargo test` + `corpus run` + `git diff --exit-code reports/` |

**Exit:** report byte-stable across consecutive runs.

### M5 — Toy dialect Oberon-X

| Task | Model | Notes |
|---|---|---|
| M5.1 `grammars/tree-sitter-oberon-x/grammar.js` extending the base | Sonnet | suggested: `BEGIN` → `DO`, plus `UNLESS Expr DO StatementSeq END` |
| M5.2 Two mapping rules + emit path: template splicing with inherited indentation | **Opus** | the one seam with lasting cost; deliberately not a Wadler/Oppen printer |
| M5.3 Bidirectional round-trip tests `X→2→X`, `2→X→2` | Haiku | golden files in `corpus/cases/` |

**Exit:** a measured answer to "what does one dialect experiment cost?"

### M6 — Testbed

| Task | Model | Notes |
|---|---|---|
| M6.1 Tauri shell linking `xoft-core`; `transpile`, `roundtrip_check`, `list_corpus` | Sonnet | serde types shared with the CLI |
| M6.2 Vite + Monaco `DiffEditor` frontend and controls | Haiku | fully specified layout |
| M6.3 `web-tree-sitter` highlighting, `ERROR` nodes marked, clickable diagnostics | Sonnet | semantic-tokens provider, not Monarch |

### M7 — Phase 2 plan

Written from the corpus report, the allowlist and the measured Oberon-X cost. **Opus.**

## Delegation packets

Each delegated task receives only what it needs:

- **Grammar tasks:** the relevant EBNF fragment from `docs/language-baseline.md`, the matching
  section of `grammar.js`, the `tree-sitter test` corpus format, and 2–3 real corpus snippets.
  Not the whole report, not the other grammar sections.
- **Core tasks:** the type signatures and the invariant to satisfy. Not the grammar, not the corpus.
- **CLI/testbed tasks:** the serde types and the command surface. Not the serializer internals.
- **Haiku tasks:** exact paths, exact dependencies, exact expected output. A Haiku task that
  needs judgment has been under-specified — respecify rather than escalate.

## Verification

```sh
cargo test --workspace                    # units, snapshots, coverage assertion
tree-sitter test                          # in each grammar dir
cargo run -p xoft-cli -- corpus manifest  # rebuild inventory (792 files)
cargo run -p xoft-cli -- corpus run       # deterministic; run twice and diff
```
