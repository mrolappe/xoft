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

## M3.2 — `xoft transpile` / `xoft check` + `codespan-reporting` — not started

## M3.3 — Broken-source fixtures + `insta` snapshots — not started
