# M3 — Diagnostics and CLI

## M3.1 — `Diagnostic` + `ERROR`/`MISSING` walk with context-based message upgrading ✅ (round 28, 2026-08-11)

`crates/xoft-core/src/diagnostic.rs`. `diagnostics(tree) -> Vec<Diagnostic>`, `Diagnostic { start_byte,
end_byte, message }` — byte spans throughout, no line/column (that's a display concern for M3.2's
`codespan-reporting` layer, not this crate's job per the no-I/O/no-text-rendering design rule).

Walks every node (not just leaves, unlike M2.2's `walk`/M2.3's `strip_comments`, since `MISSING`
nodes can be unnamed leaves like `")"` and `ERROR` nodes can wrap a whole subtree):

- `node.is_missing()` → one `Diagnostic` whose `message` is the node's own `kind()` verbatim
  (`docs/plan.md`: "a `MISSING` node's kind *is* the message" — no synthesis, the missing node
  already names the expected token). Zero-width span at the point of insertion. Does not descend
  further (missing nodes carry no real children).
- `node.is_error()` → one `Diagnostic` whose `message` comes from `error_message`, a small match
  on the node's *immediate parent's* kind (the "context-based upgrading" the plan names but never
  defines — this shape was confirmed with the user before coding, see round 28 below). One
  grounded entry today: `parent.kind() == "assignment"` → a specific message, everything else
  (including no parent, i.e. the whole tree is one root-level `ERROR`) → a generic fallback.
  Does not descend into the error subtree — the one diagnostic already covers that byte range;
  reporting partial nodes recovery managed to parse underneath it would be noise, not signal.

Both real-parser shapes (which construct becomes `MISSING` vs. `ERROR`, and where) were probed
against the actual grammar first — via a throwaway `tests/_scratch_probe.rs` deleted before
committing, not guessed or hand-derived from memory (checklist: "hand-wrote an expected
S-expression from memory, got the node shape wrong"). Two surprises the probe caught before they
became wrong test fixtures:

- A missing statement separator (`;`) does **not** produce a `MISSING ";"` node — it produces an
  `ERROR` node (the next statement's value misparses as part of the current one), whose immediate
  parent is `"assignment"`. This is the table's one grounded entry.
- An unbalanced `(` **does** produce a genuine zero-width `MISSING ")"` node, but `tree-sitter
  parse`'s default S-expression pretty-printer doesn't render it inline (it only appears in the
  CLI's one-line error summary) — the CLI output alone would have looked like there was no
  `MISSING` node in the tree at all. The Rust API's `Node::is_missing()` still sees it correctly;
  cross-checked via the scratch probe rather than trusted from the CLI text.

Tested in `crates/xoft-core/tests/diagnostic.rs`, all written before the implementation:

- `clean_source_has_no_diagnostics` — sanity check, zero diagnostics on a valid parse.
- `missing_node_reports_its_own_kind_as_the_message` — the unbalanced-`(` case; asserts
  `message == ")"`, zero-width span, and that the span sits exactly where the `)` was expected
  (right before the newline).
- `error_node_gets_a_context_upgraded_message_from_its_parent_kind` — the missing-`;` case;
  asserts the message is *not* the generic fallback and mentions `"assignment"`.
- `error_node_without_a_table_entry_falls_back_to_a_generic_message` — an `IF` with no `ELSE`
  swallows the module's own `END`, producing one root-level `ERROR` with no parent; asserts the
  generic fallback message.

The `error_message` lookup table is deliberately small — one entry, grounded in what was actually
observed, not a speculative catalog of every possible parent kind. `docs/plan.md`'s M3.3
("~8 hand-written broken files" + `insta` snapshots) is expected to surface more real `ERROR`
contexts to add table entries for; this round doesn't invent them ahead of that evidence.

## M3.2 — `xoft transpile` / `xoft check` + `codespan-reporting` ✅ (round 30, 2026-08-11)

Two new `xoft-cli` library modules, both I/O-facing per `CLAUDE.md`'s no-I/O-in-core rule —
`xoft-core` is only ever a consumer here, untouched by this milestone.

`crates/xoft-cli/src/check.rs`: `check_source(filename, text) -> CheckResult` (parses, runs
`xoft_core::diagnostic::diagnostics` + an empty `RuleRegistry::run` and merges both `Vec`s, then
renders each via `codespan-reporting`'s `SimpleFiles`/`term::emit` into an in-memory `Buffer`) and
`check_file(path) -> Result<CheckResult>` (reads bytes, `Document::from_bytes`, delegates).
`CheckResult { diagnostics: Vec<Diagnostic>, rendered: String }` — both the structured list and
the rendered text are exposed, so tests (and `transpile`) don't have to re-parse rendered output
to check facts.

`crates/xoft-cli/src/transpile.rs`: `transpile_file(path) -> Result<TranspileResult>`. Phase 1
scope was genuinely ambiguous — M5's dialect-mapping rules don't exist yet — so asked the user
before coding rather than guessing (`docs/plan.md` only says "charset applied at render time" for
this milestone, nothing about `transpile`'s Phase-1 behavior). Confirmed: `check` plus a lossless
round-trip through M2's serializer (`serialize::walk` + `serialize::reconstruct`), exercising the
codec/serializer end-to-end from the CLI for the first time, rather than stubbing the command out.
`TranspileResult { check: CheckResult, output_bytes: Vec<u8> }`.

`main.rs` gained two new top-level `Command` variants (`Check { file }`, `Transpile { file, out:
Option<PathBuf> }`), siblings of `Corpus`, matching the existing shape. `check` exits 1 when
diagnostics are non-empty (prints rendered diagnostics either way, plus a `<file>: OK` line when
clean); `transpile` writes `output_bytes` to `--out` or stdout (raw bytes via `io::Write`, not
`println!`, since `Document::to_bytes` can produce non-UTF-8 output) and also exits 1 on
diagnostics.

`codespan-reporting = "0.12"` added to `xoft-cli/Cargo.toml` (workspace's `tree-sitter` dep also
added directly to `xoft-cli`, needed to parse in `check.rs`/`transpile.rs`) — first use of this
dependency anywhere in the workspace, confined to the CLI crate as planned.

Tested in `crates/xoft-cli/tests/check.rs` (2 tests: a clean file has empty diagnostics and empty
rendering; a broken file's rendered output contains the diagnostic message, a codespan-reporting
location marker, and the source filename) and `tests/transpile.rs` (2 tests: a clean file and a
broken file both round-trip byte-identical to their original bytes, the broken one still reporting
its one diagnostic). All four written before the implementation (TDD, confirmed red via the
missing-module compile error before writing `check.rs`/`transpile.rs`). One test-writing correction
caught before it mattered: assumed codespan-reporting's location-marker glyph was `-->`
(rustc-style); the real rendered output uses `┌─` — caught by running the test red/green rather
than trusting the assumption, fixed before commit.

`cargo test --workspace` green, 8 tests in `xoft-cli` (4 → 8) + 31 unchanged in `xoft-core`.

## M3.3 — Broken-source fixtures + `insta` snapshots — not started
