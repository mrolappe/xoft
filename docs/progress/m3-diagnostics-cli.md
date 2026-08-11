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

## M3.3 — Broken-source fixtures + `insta` snapshots ✅ (round 31, 2026-08-11)

8 hand-written broken `.mod` files in `crates/xoft-cli/tests/fixtures/broken/`, snapshotted via
`insta` over `check_source`'s rendered `codespan-reporting` output in
`crates/xoft-cli/tests/broken_fixtures.rs`. Each fixture's real parse shape (which node ends up
`MISSING` vs. `ERROR`, and its parent kind) was probed first against the actual grammar — a
throwaway `crates/xoft-core/tests/_scratch_probe.rs`, deleted before committing, per the
checklist's "hand-wrote an expected tree from memory" mitigation — several first-draft sources
didn't reproduce the intended failure once actually parsed (see below) and were revised before
becoming fixtures.

Fixture location: `crates/xoft-cli/tests/fixtures/broken/*.mod`, committed source (not
tempdir-generated, unlike `manifest.rs`'s tests) — decided without asking the user, the layout
insta itself expects (`.snap` files next to the test) settled the natural counterpart location for
committed inputs. One parametrized `#[test]` iterates a `CASES` table (file, snapshot name,
expected diagnostic count, expected message substrings) rather than 8 separate `#[test]` fns —
per round 30's insight, structural facts (count, message content) are asserted against
`CheckResult::diagnostics` directly, the snapshot only covers the rendered text.

One design correction made before the first snapshot was accepted: `check_file` (which renders the
absolute checkout path into the codespan-reporting output) would have baked a
machine-specific/CI-specific absolute path into every committed `.snap` file. Switched to
`check_source(case.file, &text)` with the fixture's bare filename instead — caught by actually
looking at the first `stored new snapshot` diff before accepting, not assumed.

The 8 fixtures (final sources, after probing ruled out several non-reproducing first drafts):

- `unbalanced_parens.mod` — genuine `MISSING ")"`, same category as `xoft-core`'s own
  `diagnostic.rs` test but a distinct file/message text, per `NEXT.md`'s explicit instruction not
  to reuse that exact source.
- `unbalanced_begin_end.mod` — a stray extra `END;` before the module's real `END` surfaces as an
  `ERROR` node whose immediate parent is `"module"` — genuinely new, not covered by M3.1's table.
  First draft (`BEGIN` nested inside an `IF`'s `THEN` arm, mimicking a block-structured language)
  didn't reproduce a clean unbalanced-BEGIN/END failure at all — Oberon's `IF` has no nested
  `BEGIN...END`, so probing caught the wrong mental model before it shipped as a fixture.
- `bad_case_label.mod` — a `+` where a `CASE` label was expected misparses as a unary-operator
  factor with a `MISSING ident` operand.
- `if_no_matching_end.mod` — an `IF...ELSIF` chain with no closing `END` pushes the whole file into
  one root-level `ERROR` (same fallback path as `diagnostic.rs`'s own IF-no-`END` test, but a
  different source shape — adds an `ELSIF` arm — so the fixture isn't a byte-identical duplicate).
- `malformed_procedure_heading.mod` — a formal parameter with no type after `:` gets a
  `MISSING ident` inside `formal_type`'s `qualident`.
- `stray_token_in_declaration.mod` — a bare `RETURN` keyword where a declaration was expected
  (outside any procedure) is also an `ERROR` node parented by `"module"` — same new table entry as
  `unbalanced_begin_end.mod`; probing confirmed a corpus-plausible mistake (a misplaced keyword)
  hits the identical grounded context as a structurally unrelated mistake (an extra `END`), which
  is why the new table entry's message ("unexpected token in module body") is deliberately generic
  enough to cover both truthfully rather than describing only the case it was first noticed on.
- `missing_semicolon.mod` — the table's pre-existing `"assignment"`-parent entry, exercised by a
  fresh file (different variable names/values than `diagnostic.rs`'s fixture, per `NEXT.md`).
  Probing initially got a *different*, non-matching shape (parent `"module"`, generic fallback)
  from a source using a binary-expression right-hand side (`a := b + 1`); switching to a bare
  numeric literal (`a := 10`), matching `diagnostic.rs`'s original fixture's shape more closely,
  reproduced the `"assignment"`-parent case — confirms round 28's insight that recovery shape
  depends on the surrounding grammar path, not just "a `;` is missing" in the abstract.
- `two_diagnostics.mod` — two independent `PROCEDURE` headings, each with `malformed_procedure_heading.mod`'s
  same mistake, produce two separate `MISSING ident` diagnostics in one file (not yet exercised by
  anything before this). Getting a genuine *second* diagnostic took several failed attempts:
  anything that made the parser fall into `ERROR`-node recovery (as opposed to a clean, localized
  `MISSING`-token insertion) caused it to swallow everything up to the file's final `END` into one
  giant `ERROR` span, regardless of how many independent mistakes the source actually contained —
  confirmed empirically via the scratch probe across five source variants before finding a shape
  (two independent, self-contained `MISSING`-producing procedure headings) that actually yields two
  diagnostics. Worth remembering: this grammar's `ERROR`-node recovery is not scoped per-mistake,
  only `MISSING`-node insertion is.

`error_message` (`crates/xoft-core/src/diagnostic.rs`) gained one new grounded entry from this
round's probing: `parent.kind() == "module"` → `"unexpected token in module body"`, covering both
new contexts found (stray keyword, extra `END`) rather than one message per fixture — table stays
keyed on parent kind only, consistent with M3.1's original design, not widened to also inspect the
`ERROR` node's own child kind just to get a more specific-sounding message for one fixture.

`insta = "1.48.0"` added as a dev-dependency to `xoft-cli` only (`cargo add --dev insta`, no
feature flags needed — plain-text `assert_snapshot!` is the default). Snapshots accepted via
`INSTA_UPDATE=always`, not hand-typed, confirmed by first running the suite red (missing
`"module"` table entry) before the table fix, then green in one pass after.

`cargo test --workspace` green: `xoft-cli` 8 → 9 (one new parametrized test covering all 8
fixtures + their structural assertions); `xoft-core` unchanged (28) except the `error_message`
table's `"module"` arm, which doesn't add a test count on its own — covered by the CLI-side
fixture test rather than a new `xoft-core`-level unit test, since the table entry's own
correctness is only meaningful in terms of what actually gets rendered.

**M3 is done** — all three sub-milestones (M3.1, M3.2, M3.3) complete.
