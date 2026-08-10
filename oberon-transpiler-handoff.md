# Oberon Transpiler — Handoff Document

**Purpose:** Hand off to a coding agent for detailed planning and implementation of Phase 1 (MVP).
**Status:** Architecture decisions made; Phase 1 scoped; Phase 2+ deliberately unplanned.
**Date:** 2026-08-10

---

## 1. Long-Term Project Goal

Build a **bidirectional transpiler** between an **Amiga Oberon-2 dialect** and a **custom Oberon dialect**.

Hard requirement across the whole project: **comments and formatting are preserved by default**, with an option to strip comments. This constraint drives most architecture decisions below — in particular the choice of a lossless concrete syntax tree over a conventional AST-only pipeline.

Additional goals:
- Editor/IDE integration for **Emacs**, **IntelliJ IDEA (Community Edition)**, and **VS Code**
- A **local testbed application** for side-by-side comparison of input and output source, with syntax highlighting
- Tree-sitter grammars are wanted independently, for additional tooling beyond the transpiler

---

## 2. Phase 1 (MVP) Scope — Read This First

**Only Phase 1 is planned. Do not plan or implement beyond it.** Planning of Phase 2 is itself the final task of Phase 1 (see §10).

### 2.1 Radical simplification for the MVP

- **Source language = target language = Standard Oberon.** No Amiga Oberon, no custom dialect yet.
- Consequently the **source and target AST/CST are identical**, and the **mapping layer is the identity function**.
- Consequently the testbed's comparison view shows **identical code on both sides** during Phase 1. This is expected and correct.

### 2.2 Why an identity transpiler is a real deliverable, not a placeholder

The hardest and highest-risk part of this project is **lossless round-tripping** — parsing arbitrary real-world source and reproducing it byte-for-byte, with comments and formatting intact. An identity transpiler tests exactly that, with zero dialect complexity as a confounder.

Phase 1 succeeds when `serialize(parse(source)) == source` byte-identically across an entire real corpus. Everything after that (dialect divergence, mapping rules, lossy-construct handling) builds on infrastructure already proven correct.

### 2.3 First task before writing code

**Pin the exact language definition.** "Standard Oberon" is ambiguous — Oberon (1988), Oberon-2 (1991), Oberon-07 (2007+, several revisions). Choose one, record the decision and the specific report/revision in `docs/language-baseline.md`, and use it as the normative reference for the grammar. Recommendation: **Oberon-2**, since it is the closest common ancestor of the eventual Amiga dialect, but confirm with the project owner.

---

## 3. Technology Decisions (with rationale)

### 3.1 Implementation language: Rust

Alternatives evaluated: Go (original proposal), TypeScript, Kotlin/JVM, OCaml, Python.

**Rust chosen because:**
- Enums + exhaustive pattern matching are the right tool for tree transformations — this is the core work of a transpiler, and the compiler enforces that every dialect-specific node is explicitly handled
- Best-in-class tree-sitter bindings (official)
- Ships as a **single dependency-free binary** — the same core crate serves CLI, LSP server, and testbed backend, and end users need neither Node nor a JVM
- Mature ecosystem for the exact sub-problems here: lossless syntax trees (rowan), diagnostics rendering (ariadne/codespan-reporting), snapshot testing (insta), property testing (proptest)

**Go was rejected** primarily because it lacks sum types and pattern matching (verbose type-switch-heavy transformation code) and has a weak WASM story. Note that WASM is *not* currently required (testbed is local-only), so this is a secondary concern.

**TypeScript is the credible fallback** if development velocity turns out to matter more than deployment ergonomics: everything (core, testbed, LSP server, VS Code extension) lives in one ecosystem. The cost is weaker AST modelling and a Node runtime dependency for Emacs/IntelliJ users.

### 3.2 Parser: tree-sitter

Alternatives evaluated: hand-written recursive descent + rowan, chumsky, lalrpop, pest, ANTLR.

Note: **no production-ready ANTLR Rust target exists** (`antlr-rust` is a poorly maintained community project), so existing ANTLR knowledge does not transfer directly. It transfers conceptually to grammar authoring in general.

**tree-sitter chosen because:**
- Produces a **lossless CST with byte spans** natively — exactly what the formatting-preservation requirement needs
- Incremental reparsing (relevant for editor/LSP use)
- The grammar is wanted anyway for other tooling; writing a second grammar for a different parser would mean maintaining two grammars for the same dialects — the worst possible outcome
- Excellent authoring tooling (see §7)
- Query language (`.scm`) doubles as the formal encoding of the dialect-difference catalog (see §5.2)

**Known cost — weak error diagnostics.** tree-sitter's error recovery is optimized for editors: it yields `ERROR` and `MISSING` nodes, not precise diagnoses. Mitigation strategy in §6.2. This is acceptable because transpiler inputs are usually already-compiling source.

**Typed access layer.** tree-sitter nodes are untyped. `tree-sitter generate` emits `node-types.json`; crates such as `type-sitter` can generate typed Rust wrappers from it. **Verify maturity before depending on it**; otherwise write a thin hand-rolled accessor layer (`fn condition(&self) -> Option<Expr>` per node kind) — mechanical work, roughly 800–1500 lines for a full dialect.

### 3.3 Parser-variant evaluation (deferred, optional)

An earlier discussion considered implementing several parser variants to compare them. **This is now deferred and likely unnecessary**, since tree-sitter is required anyway. If it is revisited, do it as a **spike, not as a permanent abstraction layer**: a small representative language fragment (module header, procedure declaration, IF/WHILE, expressions with operator precedence, comments in every position) implemented per variant in separate branches against identical tests. A permanent `trait Frontend` abstraction would require an intermediate CST type plus per-variant adapters, and would flatten exactly the differences being evaluated.

Trigger for revisiting: if tree-sitter error quality proves unworkable in practice, or if lossless serialization turns out to be harder than expected.

---

## 4. Core Architecture

```
source text
    │
    ▼
tree-sitter parser (per dialect)  ──►  lossless CST (all tokens + trivia, byte spans)
    │
    ▼
typed AST view over the CST
    │
    ▼
mapping layer (rule registry)  ──►  IDENTITY IN PHASE 1
    │
    ▼
serializer:
   • unchanged subtrees  → byte-identical splice from source CST
   • transformed nodes   → pretty-printer (Wadler/Oppen; `pretty` crate)
    │
    ▼
target text
```

### 4.1 Crate layout (proposed)

```
crates/
  oberon-core/        # parsing, CST, AST view, mapping registry, serializer, diagnostics
                      # NO I/O, no CLI, no LSP — pure library
  oberon-cli/         # binary: transpile, check, corpus-run
  oberon-testbed/     # Tauri app (src-tauri + frontend/)
grammars/
  tree-sitter-oberon/ # grammar.js, queries/highlights.scm, queries/catalog/*.scm, test/corpus/
corpus/               # real-world Oberon sources for testing
docs/
  language-baseline.md
  catalog.md          # dialect-difference catalog (empty scaffolding in Phase 1)
reports/
  corpus-report.json  # checked-in, deterministic (see §6.3)
```

Design rule: **`oberon-core` performs no I/O and renders no text.** All output paths (CLI, LSP, testbed, tests) consume structured data from it.

### 4.2 Dialect-marked nodes (design for Phase 2+, prepare in Phase 1)

The AST is designed as the **union of all dialects**. Constructs existing in only one dialect become their own variants, carrying provenance in the type:

```rust
enum Statement {
    // shared core language
    Assignment { .. },
    If { .. },
    // Amiga-only
    AmigaOnly(AmigaStatement),
    // custom-dialect-only
    CustomOnly(CustomStatement),
}
```

The type checker then forces the mapping layer to handle every foreign node explicitly (translate / reject / pass through with annotation). An alternative design (a `dialect: DialectSet` attribute per node) is less type-safe.

**In Phase 1 these variants are empty or absent** — but the enum shape and the mapping-layer traversal should be structured so that adding them later does not require restructuring.

---

## 5. Mapping Layer and Difference Catalog (Phase 2+ design; scaffold only in Phase 1)

### 5.1 Mapping rules

A rule is a function `AST pattern → AST pattern` for one construct, with declared properties:

- **Direction:** bidirectional (one rule, both ways) or two unidirectional rules
- **Fidelity:** bijective (A→B→A reproduces the original exactly) vs. lossy (e.g. an Amiga pragma with no counterpart, preserved as an annotated comment so the reverse direction can reconstruct it)
- **Level:** lexical (keyword renaming) / syntactic (restructuring) / semantic (library-call substitution; may require a symbol table)

Suggested shape:

```rust
trait Rule {
    fn catalog_id(&self) -> CatalogId;
    fn applies(&self, node: &Node) -> bool;
    fn transform(&self, node: Node) -> Result<Node, MappingError>;
}
```

Rules live in a registry, applied by tree traversal. Non-bijective rules must leave metadata on the target node (e.g. a special comment `(*$origin: ... *)`) to keep round-trips possible.

### 5.2 The difference catalog

A hybrid artifact that grows with the project, in three layers:

1. **Structured prose** (`docs/catalog.md`): construct, behaviour in dialect A, behaviour in B, translatability (bijective / lossy / impossible), example pair
2. **Formal encoding as tree-sitter queries** (`grammars/.../queries/catalog/NNN.scm`): each catalog entry gets a query that detects the construct. This makes the catalog **machine-evaluable over the corpus** ("#017 occurs in 43 files") and the same query serves as the detection side of the mapping rule. Catalog, statistics and transformation share one definition.
3. **Test pairs** (`corpus/catalog/case_017.a.mod` / `case_017.b.mod`): the normative, executable specification of each rule

Catalog IDs appear in rule implementations, diagnostics, and corpus reports.

Prose alone rots; the query + test-pair form is authoritative.

### 5.3 Incremental build-out of the mapping layer (Phase 2+)

1. **Stage zero:** identity mapping for the shared core language; every dialect-specific construct raises a clean `missing rule (#ID)` diagnostic — *this is exactly what Phase 1 delivers*
2. Prioritize catalog entries by real frequency in the corpus (the corpus runner counts which missing rules fire how often)
3. One rule per iteration: catalog entry → test pair (golden files) → implement rule → round-trip test → merge
4. Metric: share of corpus files that pass end-to-end cleanly
5. Lossy/impossible constructs last, with a deliberate decision each (annotate / warn / abort)

---

## 6. Cross-Cutting Infrastructure (build this in Phase 1)

### 6.1 Diagnostics

**Never render text directly from the core.** Define one diagnostic type:

```rust
struct Diagnostic {
    severity: Severity,
    code: Option<CatalogId>,        // link into the difference catalog
    message: String,
    labels: Vec<(Range<usize>, String)>,   // byte spans
    help: Option<String>,
}
```

Four consumers, one source:
- **CLI** → rendered via **ariadne** or **codespan-reporting** (rustc-style terminal output with source excerpts and underlines). `codespan-reporting` is leaner and more stable; `ariadne` offers richer rendering (multiple overlapping labels, relationship arrows) — useful later for "declaration here, untranslatable use there"
- **LSP** → converted to `lsp_types::Diagnostic`. **Pitfall: LSP counts in UTF-16 code units**, so byte spans must be converted to UTF-16 line/column
- **Testbed** → JSON to the frontend, rendered as a clickable list that jumps to the position in the Monaco editor
- **Golden-file tests** → rendered output checked in as snapshots

tree-sitter provides byte spans directly (`node.byte_range()`); carry them through consistently into the mapping layer.

Target output quality:

```
error: construct has no counterpart in the target dialect [catalog #017]
   ┌─ corpus/gadgets.mod:42:12
   │
42 │   SYSTEM.PUTREG(0, adr);
   │          ^^^^^^ Amiga-specific register access
   │
   = help: use --allow-lossy to preserve it as a comment
```

### 6.2 Turning tree-sitter errors into useful diagnostics

tree-sitter marks problems two ways, to be handled differently:

- **`MISSING`** (`node.is_missing()`): the parser knows exactly which token was absent and inserts it virtually. `node.kind()` *is* the diagnosis — "`END` expected". These are the best messages available.
- **`ERROR`** (`node.is_error()`): unrecoverable; the span covers the un-parsed region. Upgrade these using context:

```rust
match (err.parent().map(|p| p.kind()), err.prev_sibling().map(|s| s.kind())) {
    (Some("procedure_declaration"), Some("formal_parameters")) =>
        "invalid element in procedure body",
    (Some("if_statement"), Some("expression")) =>
        "`THEN` expected",
    _ => "syntax error",
}
```

Implementation: `tree.root_node().has_error()` as a fast pre-check, then a cursor walk collecting `ERROR`/`MISSING` into `Diagnostic` values. A query `(ERROR) @syntax.error` also works and composes with the catalog queries.

### 6.3 Observability

For this project, observability means **progress and regression measurement over the corpus**, not runtime monitoring. Four things to build in from the start:

1. **Diagnostics as data, never `println!`** — see §6.1
2. **Structured event sink in the core, not output.** Use `tracing`: spans per phase (parse / transform / print) and per file; the mapping layer emits structured events ("rule #017 applied", "rule missing for construct X", "lossy transformation"). Subscribers differ per frontend (CLI: text; testbed: JSON; corpus run: aggregation)
3. **Corpus runner as its own tool, from week 1.** Runs over all corpus files and emits a machine-readable report: share parseable, share fully transpilable, share losslessly round-trippable, histogram of missing rules by frequency, timings. This report is both the prioritization basis and the progress metric
4. **Report history in Git.** Check in the aggregated report so every diff shows which files a change improved or broke — effectively a corpus-level golden file; regressions surface in review

**Critical for (4): determinism.** No hash-map iteration order, no absolute paths, no timestamps in the report.

Optional later: per-phase timing (`tracing` + `criterion` benchmarks); a log-level switch in the LSP server so users can produce useful bug reports.

### 6.4 Testing strategy

- **Round-trip invariants, tested separately:**
  - `serialize(parse(s)) == s` — lossless identity (the Phase 1 acceptance criterion)
  - `parse(serialize(parse(s))) == parse(s)` — idempotence
  - Later, for bijective mappings: `B→A(A→B(x)) == x`
- **Corpus:** real-world Oberon sources plus hand-written minimal cases per language construct
- **Property-based testing** with `proptest`: generate random ASTs, print, re-parse, compare — finds cases nobody writes by hand
- **Fuzzing** (`cargo-fuzz`) against the parser: must never panic, only report errors
- **Golden files / snapshot tests** with **`insta`**: expected outputs (transpiled code, rendered diagnostics, AST dumps) are checked in; on intentional changes, regenerate via `cargo insta review` and review the diff. Preferred over inline assertions because large outputs stay maintainable and changes are visible in Git. `expect-test` is an alternative.
- **`tree-sitter test`**: the grammar's own corpus format (source + expected S-expression side by side) — the fastest write/verify cycle for grammar work. Use this for grammar-level cases, not the Rust test suite.

---

## 7. Grammar Authoring Tooling

Use these from day one; the cycle is far shorter than going through the testbed app.

- **`tree-sitter playground`** — local web UI with source pane, live parse tree, and a **query editor**; clicking a node highlights the source and vice versa. Also the right place to develop catalog queries.
- **`tree-sitter parse <file>`** — S-expression output; `--debug` shows parser steps, `--debug-graph` renders the parse stack via Graphviz (essential for conflicts)
- **`tree-sitter test`** — grammar corpus tests (see §6.4)
- **`tree-sitter generate`** — reports conflicts with the involved rules; resolve via `conflicts` / `precedences` in the grammar
- **`tree-sitter highlight`** — verifies `highlights.scm` in the terminal without an editor
- **Editor inspectors** — Neovim `:InspectTree` and `:EditQuery` (live query development), Emacs `treesit-explore-mode`
- Existing community Oberon-family grammars are worth reviewing as a starting point

Expected grammar size for one Oberon dialect: roughly 600–900 lines of `grammar.js`.

---

## 8. Testbed Application

**Decision: Tauri** (Rust core + system webview frontend).

### 8.1 Why Tauri, and what "core linked directly" means

A Tauri app is a **native Rust process** that links `oberon-core` as an ordinary crate dependency — no WASM, no HTTP, no subprocess — plus a **system webview** (WebKit / WebView2) for the UI. The two are connected by annotated Rust functions:

```rust
#[tauri::command]
fn transpile(src: String, dir: Direction, strip_comments: bool)
    -> Result<TranspileResult, Vec<Diagnostic>> { ... }
```

Callable from the frontend as `await invoke("transpile", { src, dir, stripComments })`. Arguments and return values are serialized via Serde but stay in-process — no ports, no CORS, no server startup. File access and native file dialogs happen on the Rust side with full permissions.

Result: the core exists once as a crate and is linked identically by CLI, LSP server, and testbed. No serialization format between testbed and core, no version drift.

**Fallback if Tauri is too much infrastructure:** the same UI as a static Vite page against a `localhost` HTTP server embedded in the transpiler binary. Less integrated, trivially debuggable.

### 8.2 Frontend

**Monaco** (the VS Code editor as a web component), using its built-in `DiffEditor` — side-by-side panes with synchronized scrolling out of the box. CodeMirror 6 is the lighter alternative if Monaco proves heavy.

### 8.3 Syntax highlighting in the testbed: use `web-tree-sitter`, not Shiki

Two options were considered:

- **Shiki** — a highlighter that consumes real **TextMate grammars** (the same JSON/plist files VS Code uses) and applies VS Code themes, via Oniguruma compiled to WASM. Note it is primarily a *static* highlighter (source in, colored HTML out); wiring it into Monaco requires `@shikijs/monaco`, since Monaco natively knows only its own Monarch syntax.
- **`web-tree-sitter`** — tree-sitter compiled to WASM, driven by your own grammar. **Recommended for the testbed.**

`web-tree-sitter` wins here because the testbed then highlights using *exactly the parser the transpiler uses*: divergence between display and processing is impossible, `ERROR` nodes can be shown in red, and catalog queries can mark untranslatable constructs visually *before* transpiling. A TextMate grammar cannot do this in principle.

Integration outline (roughly one day of work):

1. **Build:** `tree-sitter build --wasm` → `tree-sitter-oberon.wasm`; plus `tree-sitter.wasm` (the runtime) from the `web-tree-sitter` npm package
2. **Init:**
   ```js
   await Parser.init();
   const parser = new Parser();
   parser.setLanguage(await Parser.Language.load("/tree-sitter-oberon.wasm"));
   ```
3. **Highlighting:** load `highlights.scm` (the same file Neovim/Emacs use), compile as a query; `query.captures(tree.rootNode)` yields nodes plus capture names (`@keyword`, `@type`, `@comment`)
4. **Monaco binding:** Monarch is unusable here. Either `monaco.languages.registerDocumentSemanticTokensProvider` (translate captures into Monaco's semantic-token format; themes apply normally — the cleaner route) or `editor.createDecorationsCollection` with CSS classes per capture (simpler, but you define colors yourself)
5. **On edit:** `tree.edit(delta)` using position data from Monaco's `onDidChangeModelContent`, then `parser.parse(newText, tree)` — incremental, sub-millisecond

### 8.4 Feature set for Phase 1

- Pick a corpus file from a list, or open a file via native dialog
- Direction switch A→B / B→A (**both directions identical in Phase 1**; wire the control anyway)
- Option "strip comments"
- Right pane shows output; diagnostics list (from the structured `Diagnostic` type) with click-to-jump to the source position
- **"Check round-trip" button:** run A→B→A and diff against the original — exposes fidelity loss immediately. In Phase 1 this is the primary function of the app.
- Syntax highlighting on both panes via `web-tree-sitter`, `ERROR` nodes visibly marked

---

## 9. Editor Integration (design context — mostly Phase 2+)

Not part of Phase 1, but the architecture must not preclude it.

Three independent artifacts:

| Concern | Artifact | Notes |
|---|---|---|
| Syntax highlighting | **TextMate grammar** (JSON) | Covers VS Code **and** IntelliJ **and** Monaco. Written by hand, but see below. |
| | tree-sitter grammar | Covers Emacs 29+, Neovim, Helix, Zed — already exists |
| Semantics (errors, go-to-def, formatting, "transpile file" command) | **LSP server** (`tower-lsp` or `lsp-server`) | One Rust binary, no user-side runtime dependency |
| VS Code packaging | TypeScript extension shell | Thin wrapper that ships the server binary |

### 9.1 Editor-specific findings

- **Emacs** — `eglot` (built in since Emacs 29) or `lsp-mode`. Unproblematic. Add a lightweight major mode: either regex-based font-lock or tree-sitter-based.
- **VS Code** — reference case. TextMate for basic highlighting, LSP **semantic tokens** from the real parser for precise classification.
- **IntelliJ IDEA Community Edition** —
  - **Highlighting:** the bundled JetBrains "TextMate Bundles" plugin is available in CE and enabled by default. Users import the bundle via *Settings → Editor → TextMate Bundles* by pointing at a folder. No third-party plugin needed, but also no marketplace installation — manual folder setup. A trivial IntelliJ plugin bundling the grammar could improve distribution later.
  - **LSP:** JetBrains' built-in LSP API is **paid-editions only**. CE requires the **LSP4IJ** community plugin (Red Hat), which can define servers declaratively through its UI. This is an extra dependency for users.
  - ⚠️ **Verify both points against current versions before planning on them** — this area is close to the knowledge cutoff. Semantic-token support in LSP4IJ in particular was a late-arriving feature.

### 9.2 Deriving the TextMate grammar from the tree-sitter grammar

No production-ready tool exists, and full automation is impossible in principle (TextMate is a regex state machine with begin/end pairs; tree-sitter is a GLR parser).

**Partial generation works well:** extract all string literals from `grammar.json` (keywords, operators, punctuation) with a ~50-line script and emit the keyword alternation:

```json
{ "name": "keyword.control.oberon",
  "match": "\\b(BEGIN|END|IF|THEN|...)\\b" }   ← generated
```

Comments, strings, numbers and identifiers are written by hand once (~10 rules, stable thereafter). The generated part then tracks dialect extensions automatically.

Anti-drift test: run both grammars over a file containing every keyword once and assert TextMate misses none.

---

## 10. Phase 1 Work Plan

### M0 — Foundations
- Pin the language baseline (§2.3); write `docs/language-baseline.md`
- Set up the workspace per §4.1
- Assemble the **corpus**: real Standard Oberon sources plus hand-written per-construct minimal cases. Aim for breadth of formatting and comment placement, not just breadth of syntax. Record provenance and licensing of corpus files.
- Define the `Diagnostic` type and set up `tracing`

**Exit:** repo builds; corpus present and inventoried.

### M1 — Grammar
- `tree-sitter-oberon` grammar covering the pinned baseline
- Grammar corpus tests via `tree-sitter test`, one case per construct
- `queries/highlights.scm`
- Resolve all generate-time conflicts

**Exit:** every corpus file parses with zero `ERROR`/`MISSING` nodes.

### M2 — Core: parse and serialize losslessly
- Rust binding to the grammar; decide typed-wrapper approach (`type-sitter` vs. hand-rolled — evaluate `type-sitter` maturity first, timebox it)
- Serializer: byte-identical reconstruction from the CST
- `strip_comments` option
- Structure the mapping layer as an (empty) rule registry with a traversal, so Phase 2 slots in without restructuring

**Exit:** `serialize(parse(s)) == s` byte-identically for 100% of the corpus; stripped output re-parses cleanly.

### M3 — Diagnostics and CLI
- `ERROR`/`MISSING` → `Diagnostic` conversion with context-based message upgrading (§6.2)
- CLI: `transpile`, `check`
- Terminal rendering via ariadne or codespan-reporting
- A set of deliberately broken source files as diagnostic-quality test cases

**Exit:** snapshot tests (`insta`) cover rendered diagnostics for the broken-source set.

### M4 — Corpus runner and reporting
- `corpus-run` subcommand producing the deterministic JSON report (§6.3)
- Check in `reports/corpus-report.json`; wire it into CI so regressions appear as diffs
- Metrics: parseable %, lossless round-trip %, timings

**Exit:** report is deterministic across runs and machines; CI fails on unreviewed regressions.

### M5 — Test hardening
- `proptest` round-trip properties
- `cargo-fuzz` target for the parser
- Idempotence tests

**Exit:** fuzzing runs without panics for a defined budget.

### M6 — Testbed application
- Tauri shell with `oberon-core` linked directly; `transpile` and `roundtrip_check` commands
- Monaco `DiffEditor` frontend
- `web-tree-sitter` highlighting on both panes, `ERROR` nodes marked
- Corpus file picker + native file dialog; direction switch; strip-comments toggle; clickable diagnostics list; "check round-trip" button

**Exit:** a corpus file can be loaded, transpiled (identity), and visually verified as identical; a deliberately broken file shows highlighted errors and a usable diagnostics list.

### M7 — Plan Phase 2 (final task of Phase 1)
Using the corpus report and the experience from M1–M6, produce a detailed Phase 2 plan. Expected inputs to that plan:
- Which Amiga Oberon / custom-dialect divergences actually occur, and how often
- Whether tree-sitter's diagnostic quality is adequate or a second parser path is warranted
- Whether the typed-wrapper approach chosen in M2 scales
- Real measurements for grammar and serializer effort, to calibrate estimates

---

## 11. Explicitly Out of Scope for Phase 1

- Amiga Oberon dialect; custom dialect; any real mapping rules
- Dialect-marked AST variants (design for them, do not populate them)
- Symbol tables / semantic analysis
- Pretty-printer for newly generated nodes (only needed once transformations exist)
- LSP server
- TextMate grammar and its generator
- VS Code, IntelliJ, and Emacs integration
- Anything requiring WASM outside the testbed
- Web deployment of the testbed

---

## 12. Open Questions / To Verify

1. **Which Oberon revision** is the Phase 1 baseline? (blocking — resolve first)
2. `type-sitter` maturity for typed Rust node wrappers — timebox the evaluation
3. **LSP4IJ** current feature set, especially semantic tokens (Phase 2 concern, but affects planning)
4. IntelliJ CE TextMate bundle import workflow in the current version
5. eglot semantic-token support in the target Emacs version
6. Corpus sourcing: which real-world Standard Oberon sources are available, and under what licenses?
7. Should the CLI and testbed share a config file format (dialect selection, strip-comments defaults) from the start?

Items 3–5 are close to the knowledge cutoff of the source conversation and should be re-checked against current documentation rather than assumed.
