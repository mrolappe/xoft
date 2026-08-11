# Checklist — read this before starting a round

One line per mistake in `docs/errors.md`: what went wrong → what stops it recurring. Full
writeups (with the diagnostic trail) live in `docs/errors.md`; broader lessons live in
`docs/insights.md`. Keep this in sync each round — see `CLAUDE.md`.

- **Workspace member listed before its manifest existed** → write a member's `Cargo.toml` in the
  same step that adds it to `workspace.members`; a manifest-load error means fix that first, don't
  read the rest of the `cargo` output as a real result.
- **Trusted the handoff doc's corpus description instead of measuring it** → measure the corpus
  (`corpus manifest`) before writing a grammar milestone or its exit criterion.
- **Nearly shipped a vacuous acceptance test** (`serialize(parse(s)) == s`) → before adopting an
  acceptance criterion, check what the laziest passing implementation would look like.
- **Widened every `statement_seq` element to `optional`, broke `tree-sitter generate`** → after
  any change that wraps every alternative/element of a rule in `optional()`, run
  `tree-sitter generate` immediately, before writing tests.
- **External scanner returned `false` on first char instead of skipping whitespace, so it never
  fired past the file's first byte** → an external scanner sharing an `/\s/` extra must skip its
  own leading whitespace (`advance(lexer, true)`) before checking for its target token.
- **Hand-wrote an expected S-expression from memory, got the node shape wrong** → generate
  expected trees via `tree-sitter test --update` against real input; if hand-writing is
  unavoidable, copy the shape from a structurally similar existing case in the same test file.
- **Declared a `conflicts` entry while the ambiguous rules were still in different parent rules —
  wasted a cycle re-testing a no-op** → when `tree-sitter generate` calls a conflict
  "unnecessary," believe it immediately; fix the grammar *shape* (make the rules true siblings at
  one choice point) instead of re-wording the conflict list.
- **`Edit`'s `old_string` silently failed to match a line containing a literal NBSP** → when a
  byte is a non-obvious/invisible Unicode character, don't hand-retype it in a later `Edit` —
  use a `\uXXXX` escape or a small script keyed on the codepoint, and confirm with `od -c` /
  `repr()`, not by eyeballing.
- **A hand-typed heredoc repro used an ASCII space where the bug needed an actual NBSP** → when a
  repro's minimality depends on a specific non-ASCII byte, construct the file programmatically
  (explicit `\xa0`) from the first attempt, and confirm with `repr()` before trusting a
  "doesn't reproduce" result.
- **A corpus grep stopped at the first `;`, which was an inner formal-parameter separator, not the
  heading terminator** → when scanning multi-line constructs for a trailing marker, don't assume
  the first occurrence of the terminator character is the real one; widen the window or match
  structurally.
- **Forgot an already-documented lesson (`grep -r` skips Latin-1 files without `-a`) and had to
  rediscover it mid-round** → default to `-a` on every corpus-wide `grep -r`/`grep -rl` against
  these roots; treat a suspiciously low/zero hit count as a signal to re-check against this
  pitfall before trusting the number.
- **Declared a `tree-sitter generate` conflict against the containing rules instead of the exact
  symbols the error named** → when the generator suggests "add a conflict for these rules: `X`,
  `Y`," pair exactly `X`/`Y` first, not a higher-level rule that seems to capture the same idea.
- **A hand-written (not corpus-copied) test source for a nested-procedure construct parsed with
  0 errors but wasn't actually testing nesting** → after `--update` succeeds on a hand-written
  source, read the generated tree before trusting it, especially near an existing
  bodiless/optional-body alternative; prefer copying the real corpus file's unambiguous
  structural shape over a minimal invented skeleton.
