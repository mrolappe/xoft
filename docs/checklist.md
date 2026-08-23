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
- **A 13-round-old scoping decision (`STRUCT` "bigger than lexical-superset scope") was restated
  every round without being re-sampled** → treat a user re-asking about an old scoping call as a
  trigger to re-derive from the actual corpus, not just cite the prior answer.
- **Modeled `BPOINTER` as a modifier keyword like its sibling `UNTRACED`, without checking its
  own corpus line** → a second dialect keyword found alongside one just confirmed can still have
  an unrelated grammar shape; verify each one against real source individually.
- **Added NUL (byte 0) to `extras`/`is_space()` to tolerate a stray trailing byte, hung the whole
  parser on every input** → tree-sitter uses lookahead value 0 as its own EOF sentinel; never
  add byte 0 to whitespace/extras tolerance. If a `tree-sitter test`/`parse` run doesn't return
  in seconds after a lexer change, suspect the change first — check on a trivial one-line file.
- **Always wrap `tree-sitter parse`/`tree-sitter test`/any tree-sitter CLI call in `timeout N`**
  (a few seconds for one file, more for the full suite) — a scanner/grammar bug can hang it
  indefinitely (round 25 ran 20+ min unguarded before being noticed), and an unguarded call
  gives no fast signal that the just-made change is the cause.
- **Nearly baked an absolute checkout path into a committed `insta` snapshot** → before accepting
  a snapshot that renders a filesystem path, check whether the path is machine-/checkout-relative;
  use an API that takes an explicit logical name (e.g. `check_source(name, text)`) instead of one
  that derives it from `env!`/an absolute `Path`.
- **`build.rs` compiles `grammars/tree-sitter-oberon2/gen-src/parser.c`, gitignored, but CI never
  ran `tree-sitter generate` to produce it** → CI failed on the very first push that added it,
  since a fresh checkout has no `gen-src/` at all. When a build step depends on a gitignored
  generated artifact, either commit the artifact or make CI regenerate it — verify by simulating
  a clean checkout (`rm -rf` the generated dir) locally, not just by having it work on a machine
  where it was already generated once.
- **Bare `tree-sitter generate` collided with copied `src/` symlinks on a freshly forked grammar
  dir** → on a fork, `mkdir -p gen-src` and always pass `-o gen-src` explicitly until the first
  generate has populated it and the copied symlinks resolve.
- **Renamed a grammar's `name` field without renaming `src/scanner.c`'s
  `tree_sitter_<name>_external_scanner_*` symbols to match** → link failure, only caught at
  `tree-sitter test`, not at `generate`. Rename both in the same step; a clean `generate` isn't
  proof the rename is complete.
- **Assumed a Tauri command's JSON casing without checking** → `#[tauri::command]` *argument*
  names auto-convert to camelCase, but a command's *return type* keeps whatever its own
  `#[derive(Serialize)]` says (no auto-rename) — confirm each command's actual JSON with a
  throwaway serialization test before writing frontend code or docs against it, don't assume
  the same casing rule applies to both sides of the boundary.
- **Guessed `tauri.conf.json`'s object-form hook command field as `command` instead of `script`**
  → don't guess Tauri config field names from what reads naturally; confirm against the schema or
  a `cargo build` before trusting a JSON shape that merely parses.
- **A new IPC command mirrored an existing command's parameter shape and inherited its
  already-acknowledged trust-boundary gap** → when a new command's design is justified by
  "mirrors an existing pattern," check whether that pattern has an open security caveat before
  copying its parameter shape.
- **Nearly built a symmetric reverse for a mapping rule whose forward direction was many-to-one
  (`BEGIN`/`DO` synonyms), chasing a byte-identical round-trip that cannot exist** → before
  implementing any mapping rule, check injectivity first: if two distinct inputs collapse to one
  output, no reverse rule is correct and the round-trip invariant must be scoped to where it
  holds, not engineered around. A round-trip test whose fixtures all use one spelling will pass
  anyway — pick fixtures that use *both* sides of any alias.
- **Guessed an ERROR node's line/column from an old round's "lands in parent kind X" note instead
  of the real parse** → run any new assertion about a diagnostic's byte span/position against the
  real parser first (red-first), the same as an S-expression tree shape; a note about *which
  parent* an ERROR lands in says nothing about *where* it starts.
- **A repo-wide `*.wasm` `.gitignore` rule silently dropped newly checked-in grammar artifacts,
  invisible to plain `git status`** → after deciding to check in a new binary artifact, confirm
  with `git status --porcelain --ignored` or `git check-ignore -v <path>` that it isn't caught by
  an existing broad ignore rule, before trusting `git add -A` to have picked it up.
