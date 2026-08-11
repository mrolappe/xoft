# Next task

**M2.4 — rule registry (empty shape, Phase 1).** M2.1–M2.3 are done (codec, grammar linkage,
token-walk serializer, `strip_comments`), all TDD, all green — see `docs/progress/m2-core.md`.
Per `docs/plan.md` line 106: "Rule registry: query-driven traversal, empty in Phase 1 | shape
only; filled in M5." This is the last M2 item.

## What's confirmed (do not re-derive, just verify before coding)

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments}.rs`: all green, see
  `docs/progress/m2-core.md` for what each does.
- `docs/plan.md`'s Layout section (line 48) lists "rule registry" and "Diagnostic" as separate
  `xoft-core` responsibilities. **`Diagnostic` doesn't exist yet — it's M3.1** (`docs/plan.md`
  line 114: "`Diagnostic` + `ERROR`/`MISSING` walk with context-based message upgrading"), which
  hasn't started. So M2.4's registry cannot be typed against a real `Diagnostic` yet.
- "Empty in Phase 1, filled in M5" means M5 (toy dialect Oberon-X) is what actually populates
  this registry with rules; M2.4's job is only the plumbing/shape.
- "Query-driven traversal" strongly suggests `tree_sitter::Query`/`QueryCursor` — the tree-sitter
  crate already used throughout `xoft-core` — as the mechanism a rule uses to find the nodes it
  cares about, rather than a hand-rolled visitor per rule.

## Open design question — ask before coding

M2.4 sits in a genuine gap: the plan names its two collaborators (`Diagnostic` from M3, real
rules from M5) but neither exists yet, so there's no concrete signature to write a test against.
Before writing the first failing test, get the shape confirmed rather than guessing — options
worth putting to the user:

1. A `Rule` trait (e.g. `fn check(&self, tree: &Tree, text: &str) -> Vec<Diagnostic>`) plus a
   `RuleRegistry`/`Vec<Box<dyn Rule>>` that's empty by construction in Phase 1 — but this forces
   a placeholder `Diagnostic` type into `xoft-core` ahead of M3, which may be exactly the kind of
   speculative shape `CLAUDE.md`'s method warns against (a trait with zero implementations and a
   type invented early to satisfy it).
2. Defer M2.4 until M3.1 lands `Diagnostic` for real, so the registry can be typed against the
   actual thing instead of a stand-in — re-order to M3 first, M2.4 right after.
3. A minimal placeholder now (e.g. a registry keyed on `&'static str` rule names, no `Diagnostic`
   dependency at all, just enough shape to prove the registration mechanism) — smallest diff, but
   worth confirming it's not throwaway work M3 will just delete.

Don't pick one and proceed silently — this is an architectural ordering call, not a grammar/
corpus ambiguity, but it's the same kind of "stop and ask" situation `CLAUDE.md` calls out.

## Definition of done (once the shape is confirmed)

- A failing-then-passing test, TDD per `CLAUDE.md`.
- Update `docs/progress/m2-core.md`'s M2.4 section (currently "not started") — and if M2.4 is
  fully done, M2's row in `PROGRESS.md` moves from 🟨 to ✅ (its only remaining sub-item besides
  the full-corpus exit measurement, which explicitly waits on M4's corpus runner regardless).
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` only if something genuinely mistake-worthy came up.

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments}.rs` + `build.rs`: all green.
- `grammar.js`/`src/scanner.c`: unchanged since round 24, M1 is frozen unless a new corpus gap
  surfaces.
- `cargo test --workspace`: green, 25 tests in `xoft-core` (17 → 25 this round) + `xoft-cli`
  unchanged.
