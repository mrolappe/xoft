# Next task

**M5.2 — Two mapping rules + emit path: template splicing with inherited indentation.** Per
`docs/plan.md` line 132, tagged **Opus** ("the one seam with lasting cost; deliberately not a
Wadler/Oppen printer") — the first task in this milestone that isn't Sonnet. Flag this to the
user before starting if the session isn't already on Opus.

## What M5.2 is for

Decision D7 (`docs/plan.md` line 42): Oberon-X exists as "a grammar-inheritance overlay, with two
mapping rules and bidirectional round-trip tests — so the MVP measures the cost of a dialect
experiment." M5.1 (done, round 35) built the overlay grammar. M5.2 is where that grammar's parsed
tree actually gets *consumed* — this is the first place `grammars/tree-sitter-oberon-x/` needs
`xoft-core` wiring (a second `tree_sitter::Language`, alongside the existing oberon2 one in
`grammar.rs`), per M5.1's `NEXT.md` open question, now resolved by reaching this task.

M5.3 (Haiku, not started) needs bidirectional round-trip tests — `X→2→X` and `2→X→2` — against
golden files, which M5.2's mapping rules + emit path must make possible.

## Open questions worth raising with the user before coding

- **What are the "two mapping rules," concretely?** M5.1 added two dialect features: `BEGIN`/`DO`
  synonymy and `UNLESS Expr DO StatementSeq END`. Is one mapping rule per feature (a `DO`↔`BEGIN`
  normalization plus an `UNLESS`↔`IF NOT ... THEN` rewrite), or some other split? `UNLESS Expr DO
  StatementSeq END` ⟷ `IF NOT (Expr) THEN StatementSeq END` is the structurally obvious rewrite
  for the harder of the two — confirm before assuming it's the intended target shape of the
  Oberon-2 side, since `NOT` availability/semantics may need re-checking against
  `docs/language-baseline.md`.
- **Is the mapping symmetric?** `2→X` direction: any Oberon-2 source is already valid Oberon-X
  input as-is (M5.1's synonym decision was chosen exactly so this holds), so `2→X→2` may reduce
  to M2's existing lossless round-trip with no new mapping logic — worth confirming as the easy
  half rather than building machinery for it. `X→2→X` is the real test of "template splicing with
  inherited indentation": mapping `UNLESS`/`DO` down to base Oberon-2 text, then mapping back up,
  needs to reconstruct something faithful to (though not necessarily byte-identical to, unlike
  M2/D4's plain-serialization guarantee) the original Oberon-X source. Confirm what "round-trip"
  means here before implementing — byte-identical, or structurally-equivalent-modulo-formatting?
- **"Template splicing with inherited indentation," concretely?** Plan.md's phrase implies emitting
  by locating the original source's indentation at the splice point and reusing it for inserted/
  rewritten text, rather than an independent pretty-printer computing layout from scratch (that's
  what "deliberately not a Wadler/Oppen printer" rules out). Confirm this reading, and whether it
  reuses M2's `serialize.rs` token-walk machinery (leaf text + inter-leaf gaps) as its base, or is
  a new code path.
- **Where do the mapping rules live?** `xoft-core`'s existing `Rule`/`RuleRegistry` (`rule.rs`,
  M2.4, currently empty — "no real rule exists until M5") is the obvious home per that module's own
  comment. Confirm the mapping rules are `Rule` impls, not a separate mechanism.

## What's confirmed (do not re-derive)

- M5.1 is done: `grammars/tree-sitter-oberon-x/` forked from `tree-sitter-oberon2/`, `BEGIN`/`DO`
  synonym + `UNLESS Expr DO StatementSeq END`, 89/89 `tree-sitter test` (85 inherited + 4 new).
  See `docs/progress/m5-oberon-x.md`. No `xoft-core`/`xoft-cli` changes yet — confined to
  `grammars/` as scoped.
- Two fork-specific traps already paid for, don't re-hit them when adding the second `Language` to
  `xoft-core`: bare `tree-sitter generate` needs `-o gen-src` explicit on a fresh grammar dir
  (`docs/errors.md` round 35), and `grammar.js`'s `name` field must match `src/scanner.c`'s
  `tree_sitter_<name>_external_scanner_*` symbol prefix or linking fails.
- `crates/xoft-core/build.rs` compiles `gen-src/parser.c` + `src/scanner.c` via `cc` directly for
  oberon2; wiring a second grammar will likely mean parameterizing this over both grammar dirs
  rather than duplicating the build script — read `build.rs`/`grammar.rs` before designing.
- `rule.rs`'s `Rule` trait: `check(&self, tree: &Tree, text: &str) -> Vec<Diagnostic>` (M2.4,
  round 29). Note the shape is diagnostic-producing, not tree-transforming — a mapping rule that
  needs to *emit* Oberon-2 text is not a drop-in `Rule` impl as-is; this tension is itself worth
  surfacing to the user rather than silently bending one concept to fit the other.

## Definition of done

- Scoping questions above resolved with the user before writing code.
- Usual end-of-round ritual: `PROGRESS.md` + `docs/progress/m5-oberon-x.md` (append M5.2 section),
  `docs/insights.md`/`docs/errors.md`/`docs/checklist.md` only if something genuinely
  mistake-worthy came up, `cargo test --workspace` plus `tree-sitter test` in
  `grammars/tree-sitter-oberon-x/`.

## State of the tree

- `crates/xoft-core/`, `crates/xoft-cli/`: unchanged since round 32 (M4.1) — M4.2 and M5.1 both
  touched no Rust code. `cargo test --workspace`: 28 `xoft-core`, 14 `xoft-cli`, green.
- `grammars/tree-sitter-oberon2/`: unchanged, still the base grammar.
- `grammars/tree-sitter-oberon-x/`: new this round (round 35) — `grammar.js` (forked +
  `unless_statement`/`kUnless`/`DO`-as-`BEGIN`-synonym), `test/corpus/oberon_x.txt` (4 cases),
  `NOTICE` (updated provenance), `package.json` (renamed), `src/scanner.c` (renamed external-
  scanner symbols). `sweep_corpus.py` dropped (real-corpus tool, not applicable). `gen-src/` is
  local/gitignored as usual — CI will need the same `tree-sitter generate -o gen-src` treatment
  `.github/workflows/ci.yml` already does for oberon2, once M5.2 makes `xoft-core` depend on it.
