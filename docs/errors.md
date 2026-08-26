# Errors and mitigations

Mistakes made while building, and what stopped them from recurring. Newest round last.

## Round 1 — 2026-08-10

### Workspace member listed before its manifest existed

`Cargo.toml` listed `crates/xoft-cli` as a member while only `crates/xoft-core` had been
written. Every `cargo` invocation then failed with `failed to load manifest for workspace
member`, including the ones meant to show the *core* tests failing — which briefly looked like
a broken core rather than a missing file.

**Mitigation:** write each member's `Cargo.toml` in the same step that adds it to
`workspace.members`, never before. If `cargo` reports a manifest error, fix that before reading
anything else in the output as a real result.

### Assumed the handoff document's corpus description

The handoff scoped Phase 1 against "real Standard Oberon sources". Measuring the actual files
on disk before planning showed a corpus that a Standard Oberon grammar cannot parse at all —
three encodings, CRLF, nested comments, `DEFINITION` modules, `INLINE` assembly. Had this been
taken on trust, M1 would have been declared complete against a grammar that fails on the
majority of real input.

**Mitigation:** measure the corpus before writing the grammar milestone, not after. The
`corpus manifest` command exists so the numbers stay checkable rather than remembered, and M1's
exit criterion is expressed as a percentage of the *real* corpus.

### Nearly shipped a vacuous acceptance test

`serialize(parse(s)) == s`, taken from the handoff, is satisfied by returning the input
unchanged — the test would have passed on day one and stayed green through every grammar bug.

**Mitigation:** decision D4 replaces it with a byte-coverage assertion (every byte belongs to
exactly one leaf or one trivia gap, zero `ERROR`/`MISSING`). When an acceptance criterion is
written down, check what the laziest passing implementation would be before adopting it.

## Round 4 — 2026-08-10

### Widening every `statement_seq` element to `optional` broke `tree-sitter generate`

Implementing the empty-statement fix literally as written in `NEXT.md` ("make each element of
the sequence `optional($.statement)`") produced `statement_seq: seq(optional($.statement),
repeat(seq(';', optional($.statement))))` — a rule that can match zero tokens, which
`tree-sitter generate` rejects with "The rule ... matches the empty string" rather than a
conflict or a silent wrong parse.

**Mitigation:** run `tree-sitter generate` immediately after any change that adds `optional()`
around every alternative/element of a rule, before writing tests against it — the failure mode
is a hard generate error, not a subtle parse bug, so it's cheap to catch immediately and
expensive to debug later if buried under other changes. See `docs/insights.md` round 4 for the
`choice`-of-two-non-empty-branches fix.

## Round 6 — 2026-08-10

### External scanner only got one chance per token, and leading whitespace burned it

First version of `scanner.c` checked `lexer->lookahead != '('` and returned `false` immediately
otherwise. This works for a comment that is the very first byte of the file, and fails for every
other comment in existence — confirmed by `tree-sitter test`, where even the pre-existing
two-comment corpus case (unchanged since M1.1) regressed to an `ERROR`, and by
`tree-sitter parse --debug`, which showed `lex_external` being called exactly once at the
position *before* the newline preceding a comment, declining (lookahead is `'\n'`, not `'('`),
and then `lex_internal` skipping that whitespace and committing to matching a *real* grammar
token (literal `(` used for expression grouping) instead of ever re-trying the external scanner.
Tree-sitter does not loop "skip one whitespace char, retry external scanner, repeat" — the
external scanner is consulted once per token boundary, before the internal DFA's own
whitespace-skipping runs.

**Mitigation:** an external scanner that has to coexist with a plain `/\s/` extra must skip its
own leading whitespace (`while (is_space(lexer->lookahead)) lexer->advance(lexer, true);`, the
`skip=true` argument marks the chars as not part of the token) before checking for the construct
it's actually looking for. Confirmed via `tree-sitter parse --debug`, not guessed — the debug
lex trace showing zero `consume character` lines for the failing `lex_external` call was the
proof that it never even reached the whitespace, not that the whitespace confused it.

## Round 8 — 2026-08-10

### Hand-wrote an expected S-expression from memory instead of generating it, got the shape wrong

Wrote the "Multi Digit Hex Literal" test's expected tree by hand (flat `(qualident (ident)
(ident))` for `S.INLINE`, and `actual_params` wrapping `expression` nodes directly) based on
guessing the shape from the rule names in `grammar.js`. `tree-sitter test` failed immediately —
the real shape is `(designator (qualident (ident)) (selector (ident)))` for a dotted qualified
name, and `actual_params` wraps its arguments in an `expression_list` node. Both were visible in
neighboring tests in the same file (`statements.txt` already had a `selector`/`expression_list`
example a few hundred lines up) but weren't checked before writing the new case by hand.

**Mitigation:** established practice in this repo (see round 4/5's insights) is to generate the
expected tree via `tree-sitter test --update` against real input and read it back, specifically
to avoid this. Reverted to that for the fix. When hand-writing is unavoidable, grep the same test
file for a structurally similar existing case (same rule combination) and copy its shape rather
than reconstructing it from `grammar.js` rule definitions alone — the generated node shape
depends on precedence/hiding choices in the grammar that aren't always obvious from the rule
text.

## Round 19 — 2026-08-11

### Declared a `conflicts` entry to fix an ambiguity, but the rules hadn't moved yet — wasted a cycle on a no-op

While diagnosing the `designator`/`actual_params` type-guard ambiguity (see `docs/insights.md`
round 19), the first attempt was `conflicts: $ => [[$.selector, $.actual_params]]` with the two
rules left in their *original* positions (`selector` inside `designator`'s `repeat`,
`actual_params` bolted onto `factor`/`procedure_call` afterward). `tree-sitter generate`
reported the declaration "unnecessary" and the minimal repro still failed identically —
because in that shape, the two rules are never actually offered as alternatives at the same
parser state; there was nothing for GLR to fork between. The declaration looked plausible
(both rules' first token is `(`) but conflict analysis operates on the automaton's actual
states, not on "these two things start with the same character."

**Mitigation:** when `tree-sitter generate` calls a declared conflict "unnecessary," believe it
immediately rather than re-testing the same declaration a second time hoping the warning was
stale — it means the automaton genuinely has no fork there, so the fix has to change what
states exist (here: moving `actual_params` into the same `repeat` as `selector`, so they're
truly siblings at one choice point), not add a bigger or differently-worded conflict list. This
was working correctly the first time it was tried (see the round-19 log above) — the mistake
was doubting the "unnecessary" warning enough to spend a second cycle confirming it rather than
moving straight to a grammar-shape change.

### Edit tool `old_string` silently failed to match text containing a literal NBSP character

Twice this round, an `Edit` call with `old_string` typed to *look* like the target line (e.g.
`extras: $ => [$.comment, $.pragma, $.bracket_pragma, /[\s ]/],`) failed with "String to
replace not found," even immediately after a successful edit had written that exact line. The
line actually contained a literal U+00A0 (non-breaking space) character inside the regex
(intentionally, to match the corpus's NBSP-as-whitespace bytes) — indistinguishable from a
plain space when read back visually, but a byte-exact mismatch against a hand-typed ASCII space
in the tool call.

**Mitigation:** when a byte a rule needs to match is a non-obvious/invisible Unicode character
(NBSP, zero-width chars, smart quotes), don't retype it by hand in a subsequent `Edit` call —
either use a `\uXXXX` escape in the replacement text (unambiguous, greppable) or drive the
substitution through a small Python/Bash script that references the character by codepoint,
as was eventually done here (`python3` one-liner replacing the exact old string read back from
the file). Confirm the result by reading the byte content back (`od -c` or `repr()` in Python),
not by eyeballing the file.

### Round 20: a hand-typed "matching" repro used a regular space where the bug needed an actual NBSP

While minimizing the NBSP+comment repro, several synthetic test files (`/tmp/x9.mod`,
`/tmp/x10.mod`, `/tmp/xC.mod`, …) were typed by hand via heredocs to match a failing file's
shape, including its trailing whitespace — but heredoc-typed whitespace is an ordinary ASCII
space, not the NBSP the real bug depended on. Every one of these hand-typed variants parsed
fine, which briefly looked like the failure required extra context (blank lines, comment
length, import blocks) beyond just "NBSP before comment" — a wrong lead chased through several
iterations before noticing (via `python3 -c "print(repr(...))"` on both files) that the two
"identical" files differed in exactly one invisible byte.

**Mitigation:** this is the same class of mistake `docs/errors.md`'s existing NBSP entry
already warns about (Edit `old_string` silently missing a literal NBSP), but on the *write*
side this time, not just the edit-match side: whenever a repro's minimality depends on a
specific non-ASCII byte, construct the repro file programmatically (Python string with an
explicit `\xa0`/` `) from the very first attempt, never by hand-typing a look-alike
character into a heredoc — and confirm with `repr()` before trusting a "doesn't reproduce"
result as signal rather than as a typo.

### Round 20: a corpus grep for "does a string appear before the first `;`" was truncated by inner `;`s in multi-line formal parameter lists

Checking whether every `PROCEDURE -ident` occurrence in `voc` had the expected trailing C
string before its heading's terminating `;` used a regex that grabbed text up to the *first*
`;` after the match. For single-line headings this is the real terminator; for multi-line
headings (formal parameters separated by `;`, e.g. `oocX11.Mod`'s `XCreateImage`) it stopped at
the first parameter separator instead, several hundred characters before the actual string —
producing 20 false "no string found" results that looked like exceptions to the pattern being
investigated, right before the fix was otherwise ready to write.

**Mitigation:** when scanning multi-line constructs for a trailing marker, don't assume the
first occurrence of the terminator character is the real one — either widen the search window
generously past what a single-line case would need (a few hundred extra characters cost
nothing) or match structurally (balance parens) rather than by first-occurrence of a character
that also appears inside the construct itself.

### Round 21: the already-documented "raw `grep -r` skips Latin-1 files" lesson was forgotten and had to be rediscovered mid-round

Checking underscore-identifier prevalence across the four corpus roots, the first `grep -rlE`
pass (no `-a`) reported only 1 matching file in `oberon-a` — implausibly low given the failing
file being investigated (`EAGUI.mod`) alone had 53 matches. The undercount was silently caused
by the exact issue already recorded in `NEXT.md`/`docs/insights.md` from round 18 (`grep -r`
over the raw corpus skips Latin-1 files unless given `-a`) — a lesson that was read at the start
of this round but not actively checked against before running a fresh grep, only noticed because
the result looked obviously wrong (one file, when direct inspection already showed 53 hits in a
single file) rather than from applying the rule proactively.

**Mitigation:** having a lesson recorded is not the same as applying it — when writing a *new*
corpus-wide `grep -r`/`grep -rl` command against these roots specifically, default to including
`-a` every time rather than adding it reactively after a suspiciously-low count; treat "surprise
zero or near-zero hit count for something known to be common" as itself a signal to re-check
the command against the known Latin-1 pitfall before trusting the number.

### Round 22: `tree-sitter generate`'s conflict resolution was declared against the wrong symbol pair

Adding STJ's `PROCEDURE-` trap-bound heading (a `kMinus` mark in `procedure_heading`) collided
with voc's pre-existing `external_proc_decl` rule, which spells the same leading `-` as an
inline anonymous literal rather than the named `kMinus` token. `tree-sitter generate` reported
an unresolved conflict and suggested (as resolution #4) "add a conflict for these rules:
`external_proc_decl`, `kMinus`." The first attempt instead declared `[$.external_proc_decl,
$.procedure_heading]` — the two *containing* rules whose expansions diverge, which looked like
the more meaningful pairing — and `tree-sitter generate` produced the exact same unresolved
error, byte-for-byte, on the next run. Only pairing the literal symbols the generator itself
named (`external_proc_decl`, `kMinus`) resolved it.

**Mitigation:** when `tree-sitter generate` names a specific symbol pair in its conflict
resolution suggestion, declare the conflict on exactly those symbols first — not a
higher-level rule that seems to capture the same idea — and only widen from there if that
doesn't resolve it. A conflict lives at the parse-table symbols that actually collide, which
isn't always the outermost rule a person would think to name.

### Round 22: a hand-written test source for a nested-procedure construct was accidentally ambiguous with an unrelated bodiless-heading rule

The first hand-written test for STJ's `PROCEDURE~` nested-procedure mark (`PROCEDURE Outer;
PROCEDURE~ Inner(...); BEGIN...END Inner; BEGIN...END Outer.`) passed `tree-sitter test
--update` with 0 errors, but the generated expected tree showed `Outer` had been parsed as a
*bodiless* heading (reusing round 20's AmigaOberon `definition_proc_decl`-at-module-level
precedent) with `Inner` promoted to an independent second module-level procedure — not nested
inside `Outer` at all, defeating the point of the test. The source's single trailing `END
Outer.` (no distinct `END Outer;` before a further module body) left the file genuinely
ambiguous: GLR found a fully valid alternate parse that didn't exercise nesting, and "0 errors"
gave no signal that anything was wrong.

**Mitigation:** after `--update` reports success on a *hand-written* (not corpus-derived) test
source, read the generated tree before trusting it, especially for any construct that shares a
grammar position with an existing bodiless/optional-body alternative — success only means *a*
valid parse was found, not the intended one. Prefer copying the real corpus file's structural
shape (receiver, enclosing `END name;` before further content) over a minimal invented
skeleton, since the corpus shape is unambiguous by construction and a minimal invented one may
not be.

### Round 23: modeled `BPOINTER` as a modifier keyword like its sibling `UNTRACED`, without checking its actual corpus shape first

Found `BPOINTER TO Type` (AmigaDOS's BCPL-relative pointer) in the same re-tally pass that
confirmed `UNTRACED POINTER TO Type`. Because both are AmigaOberon pointer-type keywords found
via the same grep, the first `pointer_type` draft added `kBPointer` as a second optional
modifier ahead of the mandatory `kPointer` — the same slot `kUntraced` occupies — which would
require `BPOINTER POINTER TO Type` to parse. Re-reading the actual corpus line (`Interfaces/
Dos.mod`: `FileLockPtr* = BPOINTER TO FileLock;`) before running `tree-sitter test --update`
showed `BPOINTER` fully replaces `POINTER`, never co-occurring with it.

**Mitigation:** when a second dialect keyword surfaces alongside one just confirmed (same grep,
same root, superficially similar semantics), read its own corpus line before assuming it shares
the first one's grammar shape — don't extrapolate from a sibling's already-confirmed shape.

## Round 25 — 2026-08-11

### Added tree-sitter's own EOF sentinel byte (NUL) to `extras`/`is_space()`, hung the entire parser

Two `oberon-a` files (`HelloWorld.mod`, `Skeleton.mod`) end with a single stray NUL byte after
the final `END Module.` and blank line. Modeled the fix on round 19/20's NBSP precedent: widen
`grammar.js`'s `extras` regex and `scanner.c`'s `is_space()` to also treat NUL as skippable
whitespace. `tree-sitter generate` initially rejected a literal backslash-zero escape in the
regex (the Rust regex parser reads it as a backreference), so the fix used a ` ` escape
instead — that part generated cleanly and looked correct on inspection.

Running `tree-sitter test` afterward never returned: it was still running after 20+ minutes at
about 99% CPU. Suspecting the two target files specifically, tried `tree-sitter parse` on one in
isolation — same hang. Then tried a trivial one-line file (`MODULE M; END M.`) — also hung, even
wrapped in `timeout 30`, which normally would have killed it (the `timeout` process itself sat
idle while its child kept spinning, meaning the child wasn't responding to the signal in any
useful timeframe). A hang on a trivial file with no NUL byte anywhere in it, immediately after
touching `is_space()`, was the signal that the change itself was broken, not the target files.

Root cause: tree-sitter uses lookahead value 0 as its internal EOF sentinel — both
`TSLexer.lookahead` in the external-scanner API and the generated internal lexer's DFA represent
"no more input" as byte 0. `is_space()` returning true for that value means the
leading-whitespace-skip loop (`while (is_space(lexer->lookahead)) advance(...)`) treats EOF
itself as another space character to skip past — but `advance()` at EOF is a no-op on
`lookahead` (it's already at the end), so the loop spins forever on every single parse, whether
or not the input actually contains a NUL byte.

**Mitigation:** killed the runaway `tree-sitter test`/`tree-sitter parse` processes, reverted
both the `grammar.js` extras regex and `scanner.c`'s `is_space()` to their exact prior text
(verified via `git diff` showing zero diff on either file), regenerated, and reran `tree-sitter
test` — clean 85/85 again, confirming the codebase was undamaged. Before adding any raw byte
value to a lexer's whitespace/extras tolerance, check first whether that value is reserved by
the lexer generator's own protocol (EOF/error sentinels), not just whether it's safe with
respect to the grammar being written — a byte that's inert at the grammar level can still be
load-bearing at the tool level. When a `tree-sitter test`/`parse` run doesn't return in a few
seconds on ordinary input, suspect the just-made change before suspecting a slow GLR blowup on a
specific file — confirm by testing the smallest possible input (a one-line trivial module) in
isolation.

## Round 31 — 2026-08-11

### Nearly committed `insta` snapshots with the checkout's absolute path baked in

M3.3's first fixture test called `check_file(&fixture_path)`, where `fixture_path` was built from
`env!("CARGO_MANIFEST_DIR")` — an absolute path specific to this checkout. `check_file` renders
that path straight into the `codespan-reporting` output (`┌─ /Users/.../tests/fixtures/broken/
unbalanced_parens.mod:4:14`), which `insta` would then have written verbatim into a committed
`.snap` file. The snapshot would have failed on every other machine (a different clone directory)
and in CI, not because the diagnostic logic changed but because the path did.

**Mitigation:** looked at the actual `stored new snapshot` diff before accepting the first one,
rather than trusting `INSTA_UPDATE=always` to always be safe to accept blind — the absolute path
was visible immediately. Switched the fixture test to call `check_source(case.file, &text)` with
the fixture's bare filename instead of `check_file` with its full path. General lesson: before
accepting any snapshot that renders a filesystem path, check whether the path is
machine-/checkout-relative; if so, use whichever API takes an explicit logical name instead of one
that derives the name from `env!`/`std::env::current_dir`/an absolute `Path`.

## Round 34 — 2026-08-11

### CI failed on the first push after `gen-src/` went gitignored — nothing ever regenerated it there

`crates/xoft-core/build.rs` (added round 26) compiles
`grammars/tree-sitter-oberon2/gen-src/parser.c`. Round 7 (2026-08-10) deliberately moved
generated tree-sitter output into `gen-src/` and gitignored it, with `src/` kept as symlinks
into it for `tree-sitter test`/`parse`. That decision was correct in isolation but nobody added
a step to CI to run `tree-sitter generate` — every round since then happened to run on a machine
that already had a locally-generated `gen-src/` from when the grammar was last touched, so the
gap was invisible locally. M4.2 (round 33) added `.github/workflows/ci.yml`, giving this a fresh
checkout with no `gen-src/` at all for the first time; `cargo test --workspace` failed immediately
with `cc1: fatal error: .../gen-src/parser.c: No such file or directory`.

**Mitigation:** `.github/workflows/ci.yml` now installs `tree-sitter-cli` via `npm` and runs
`tree-sitter generate` into `grammars/tree-sitter-oberon2/gen-src` before `cargo test`. Verified
by moving the local `gen-src/` aside (simulating a fresh checkout), regenerating with the exact
command CI now runs, and confirming `cargo test --workspace` passes — not just by reading the CI
log and guessing the fix. General lesson: when a build depends on a gitignored generated
artifact, a machine that generated it once will keep working silently while CI (or any other
fresh checkout) breaks on the very first run; test build-from-scratch locally whenever a build
step starts depending on something gitignored, don't wait for CI to be the first to notice.

## Round 35 — 2026-08-18

### Bare `tree-sitter generate` collided with the copied `src/` symlinks when forking a grammar dir

Round 7's `gen-src/`-as-real-target convention (`src/` holds only `scanner.c` plus symlinks
pointing into `gen-src/`) is set up once per grammar directory and assumed already in place —
`tree-sitter generate` (no `-o`) writes to `src/` by default, which is fine on an *existing*
grammar dir where `gen-src/` already has real files for the symlinks to resolve through. Forking
`tree-sitter-oberon2/` into a brand-new `tree-sitter-oberon-x/` via `rsync` copied the symlinks
themselves (tiny, real, tracked files, exactly as round 7 intended) but not `gen-src/`'s
generated *contents* (gitignored, correctly excluded from the copy) — so the symlinks were
dangling. A bare `tree-sitter generate` in the new directory then tried to create `src/tree_sitter`
and hit `File exists (os error 17)`, since `mkdir` fails on a path that's already a symlink node
regardless of whether the target exists.

**Mitigation:** on a freshly forked grammar directory, `mkdir -p gen-src` and always pass
`tree-sitter generate -o gen-src` explicitly, never a bare `tree-sitter generate`, until the
first successful generate has populated `gen-src/` and the copied symlinks resolve.

### Renamed a grammar's `name` field without renaming its external scanner's symbol prefix

`grammar.js`'s `name: 'oberon2'` → `'oberon_x'` (needed so the fork doesn't collide with the base
grammar's generated function name) silently broke the copied `src/scanner.c`: tree-sitter's
generated `parser.c` calls `tree_sitter_<name>_external_scanner_*`, but the copied `scanner.c`
still *defined* `tree_sitter_oberon2_external_scanner_*`. `tree-sitter generate` itself succeeded
(the scanner's C source isn't inspected at generate time); the mismatch only surfaced as a link
failure — `Undefined symbols for architecture arm64` — on the first `tree-sitter test`.

**Mitigation:** whenever a grammar's `name` field changes (including a fresh fork under a new
name), `grep tree_sitter_<old_name> src/scanner.c` and rename every match in the same step, before
running `tree-sitter test`. A clean `generate` is not evidence the rename is complete — only a
successful `test` (which actually links the scanner) is.

## Round 36 — 2026-08-18

### Nearly implemented a reverse mapping rule to satisfy a round-trip invariant that was impossible

The round's brief settled two things that turn out to contradict each other: (1) Rule A
(`BEGIN`/`DO`) is one-way, 2→X is a no-op; (2) `X→2→X` and `2→X→2` are both byte-identical. The
first draft of the design resolved the contradiction the obvious way — make Rule A symmetric by
having 2→X rewrite `BEGIN`→`DO` — and got as far as sketching the edit map before the first test
fixture killed it. `X_UNLESS`'s procedure body opens with `BEGIN`, which is perfectly legal
Oberon-X; under a symmetric Rule A that source round-trips to `DO`. The "fix" had not restored
invertibility, it had swapped which Oberon-X sources lose information.

The root cause is that `BEGIN` and `DO` are *synonyms*: the X→2 direction is many-to-one, so no
inverse exists, and no arrangement of the reverse rule can manufacture one. The brief's claim was
not a spec to satisfy but a premise to falsify. Had the symmetric version shipped, it would have
passed a round-trip test built only from `DO`-spelled fixtures and silently corrupted every
`BEGIN`-spelled one — a bug findable only by whoever wrote an M5.3 golden file the other way.

**Mitigation:** before implementing a mapping rule, ask whether the rewrite is injective — do two
distinct inputs produce the same output? If yes, no reverse rule can be correct, and a stated
byte-identical round-trip invariant is impossible rather than merely unimplemented. Say so and
scope the invariant to where it actually holds (here: unconditional for `2→X→2`, exact for the
additive Rule B, up-to-normalization for the alias Rule A) instead of building machinery that
makes the asymmetry harder to see. A given decision that asserts a property is *achievable* is
still a claim to check, not an instruction — especially when its supporting argument only walks
through one of the two rules it covers.

## Round 38 — 2026-08-23

### Assumed Tauri auto-camelCases command JSON uniformly, caught before it reached NEXT.md

While writing M6.2's handoff notes, the first draft described every command's return type —
`Manifest`, `RoundtripResult`, `TranspileResult`, `Diagnostic` — as camelCase, by analogy with
Tauri's well-known behavior of auto-converting `#[tauri::command]` *argument* names to camelCase.
That analogy doesn't hold: argument-name conversion is a `tauri-macros` feature of the command
wrapper itself, but a command's *return value* is serialized by plain `serde_json::to_value`
against whatever `#[derive(Serialize)]` the return type actually has — and none of these four
types carry `rename_all`, so the real JSON is snake_case throughout. Caught by writing a
throwaway test that serialized `Diagnostic`/`RoundtripResult` and printing the output, before the
docs were finalized — not by reasoning about the macro's behavior from memory.

**Mitigation:** added to `docs/checklist.md`. Don't assume the same casing convention holds on
both sides of a serialization boundary just because the framework auto-converts one side; check
the other side's actual JSON with a small serialization test before writing frontend code or
handoff docs against it.

## Round 39 — 2026-08-23

### `tauri.conf.json`'s object-form hook commands use `script`, not `command`

Adding `beforeDevCommand`/`beforeBuildCommand` as `{ "command": "npm run dev", "cwd":
"../../testbed-ui" }` (the field name that reads most naturally in English) built fine at the
JSON level but failed at `cargo build`'s build script with `data did not match any variant of
untagged enum BeforeDevCommand` — Tauri v2's schema names the field `script`, not `command`, for
the object form of a hook command. The plain-string form (`"beforeDevCommand": "npm run dev"`)
would have worked without this trap; the object form was only needed here to add `cwd` (the
frontend lives at `testbed-ui/`, not next to `tauri.conf.json`).

**Mitigation:** when Tauri config adds a nested object for a hook/command field, don't guess the
key name from what reads naturally — the build-script error names the exact expected variant
shape (`untagged enum BeforeDevCommand`) but not its field names; confirm against
`https://schema.tauri.app/config/2` or by testing the guess with `cargo build` immediately
rather than assuming it's right because the JSON parses.

## Round 40 — 2026-08-23

### Hand-guessed a diagnostic's line/column from an old round's note instead of the real parse

Writing `roundtrip_check_diagnostics_carry_a_line_column_position`'s first draft, guessed the
`missing_semicolon.mod` ERROR node would start at line 5 (`b := 20`) by analogy with
`docs/errors.md` round 31's "lands in `assignment`" note about *which parent kind* the ERROR node
gets, not *where* it starts. Running the test red-first (per TDD, before trusting the assertion)
showed the real position is line 4, columns 8–10 — the recovery actually swallows the trailing
`10` on the *previous* line, not the next line's `b`. Caught immediately by the red run itself,
no wasted round-trip.

**Mitigation:** already covered by an existing checklist entry ("hand-wrote an expected
S-expression from memory") — this is the same failure mode one layer up (a derived *fact about*
a parse, not the parse tree shape itself). Re-affirms: run the assertion against the real parser
first and copy the actual value, for any property of an `ERROR`/`MISSING` node, not just its tree
shape.

### A repo-wide `*.wasm` `.gitignore` rule silently swallowed the checked-in grammar artifacts

M6.3's compiled `.wasm` grammars were placed under `testbed-ui/src/grammars/` per the user's
"checked-in artifact" decision, but `git status` never showed them — the root `.gitignore` has a
blanket `*.wasm` rule from an earlier round, with no memory of what it was originally added to
exclude. `git add` on the directory silently added zero files; only `git status --porcelain
--ignored` (not plain `git status`, which omits ignored paths entirely) revealed they were being
dropped.

**Mitigation:** after deciding to check in a new binary artifact, confirm it actually shows as
untracked/addable (`git status --porcelain --ignored | grep <name>` or `git check-ignore -v
<path>`) before assuming an `Edit`/`Write` + a later `git add -A` will pick it up — a broad
existing ignore rule can make a file invisible to plain `git status` with no error at any step.

## Round 41 — 2026-08-26

### Assumed a Monaco diff editor's "original" pane was editable because the code intended it to be

`main.ts`'s own comment says the original model is "the live, editable source," and only
`getModifiedEditor().updateOptions({ readOnly: true })` is called — nothing sets the original
side's editability. Spent several failed click/type attempts (including a full app restart,
suspecting a stale build) assuming the clicks were landing on the wrong pane, before checking
Monaco's actual default: `createDiffEditor` defaults `originalEditable` to `false`, so the
*intended* editable pane was read-only from the start, by omission, not by a targeting mistake.

**Mitigation:** when a UI element rejects interaction ("Cannot edit," greyed out, no response),
check the widget library's own default for that exact option before re-attempting the click with
adjusted coordinates — a silently-wrong default is a common cause and re-clicking teaches
nothing new if the target was never interactive to begin with.

### Read an empty/transparent UI region as purely cosmetic before checking for a data error underneath

The corpus sidebar's empty appearance (an unrelated window showing through, see the M6.3 addendum
in `docs/progress/m6-testbed.md`) was initially treated as fully explained by a window-transparency
bug. It wasn't: the sidebar was also genuinely empty because `manifest::build` had aborted on one
missing machine-local corpus root, discarding every root's file list, not just the broken one's —
a real second bug, only found by reading the one line of real (non-bleed-through) text that was
present and tracing it to actual code.

**Mitigation:** when a UI region renders unexpectedly blank, check for a rendering/styling
explanation *and* an underlying data/error explanation before concluding it's cosmetic — a visible
error string (even one rendered oddly) is a lead to trace into the code, not something to explain
away by the more obvious-looking visual bug sitting next to it.
