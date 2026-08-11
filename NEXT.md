# Next task

**M2.4 — rule registry (empty shape, Phase 1).** This was deferred at the start of round 28
specifically to wait for a real `Diagnostic` type to exist — it now does
(`crates/xoft-core/src/diagnostic.rs`, M3.1, done round 28). Per `docs/plan.md` line 106: "Rule
registry: query-driven traversal, empty in Phase 1 | shape only; filled in M5."

## What's confirmed (do not re-derive, just verify before coding)

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments,diagnostic}.rs`: all green, see
  `docs/progress/m2-core.md` and `docs/progress/m3-diagnostics-cli.md` for what each does.
- `Diagnostic { start_byte: usize, end_byte: usize, message: String }` in `diagnostic.rs` is the
  real, no-longer-placeholder type M2.4's registry can now be typed against.
- "Empty in Phase 1, filled in M5" means M5 (toy dialect Oberon-X) is what actually populates
  this registry with rules; M2.4's job is only the plumbing/shape.
- "Query-driven traversal" strongly suggests `tree_sitter::Query`/`QueryCursor` — already used
  nowhere yet in this codebase, but the tree-sitter crate is already a dependency throughout
  `xoft-core` — as the mechanism a rule uses to find the nodes it cares about, rather than a
  hand-rolled visitor per rule.

## Suggested shape (confirm before coding, this is a proposal not a decision)

A `Rule` trait plus a registry that's empty by construction in Phase 1:

```rust
pub trait Rule {
    fn check(&self, tree: &Tree, text: &str) -> Vec<Diagnostic>;
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new() -> Self { RuleRegistry { rules: Vec::new() } }
    pub fn run(&self, tree: &Tree, text: &str) -> Vec<Diagnostic> {
        self.rules.iter().flat_map(|r| r.check(tree, text)).collect()
    }
}
```

This is now typed against the real `Diagnostic`, so option 1 from round 28's M2.4 discussion no
longer forces a placeholder type into the tree ahead of schedule — that objection is resolved.
Still worth a quick sanity check with the user before coding (not a full re-ask): does `Rule`
need `text: &str` at all, or is `tree` alone enough for every rule this registry will ever run
(M5's rules aren't designed yet, so this is a real unknown, not a formality)?

## Definition of done

- A failing-then-passing test, TDD per `CLAUDE.md`. Given the registry is empty in Phase 1, the
  test is necessarily about the *mechanism* (an empty registry runs zero rules and returns an
  empty `Vec`; registering a trivial always-returns-one-diagnostic rule and running it proves the
  wiring), not about any real dialect rule — there are none yet.
- Update `docs/progress/m2-core.md`'s M2.4 section (currently "not started") — once done, M2's
  row in `PROGRESS.md` moves from 🟨 to ✅ (its only remaining sub-item besides the full-corpus
  exit measurement, which explicitly waits on M4's corpus runner regardless).
- Usual end-of-round ritual: `PROGRESS.md` round table, `docs/insights.md`/`docs/errors.md`/
  `docs/checklist.md` only if something genuinely mistake-worthy came up.

## State of the tree

- `crates/xoft-core/src/{codec,grammar,serialize,strip_comments,diagnostic}.rs` + `build.rs`:
  all green.
- `grammar.js`/`src/scanner.c`: unchanged since round 24, M1 is frozen unless a new corpus gap
  surfaces.
- `cargo test --workspace`: green, 29 tests in `xoft-core` (25 → 29 this round) + `xoft-cli`
  unchanged.
