# xoft — working agreement

A workbench for designing Oberon-flavored dialects. Read `docs/plan.md` first — it holds the
decisions (D1–D8), the milestone breakdown and the corpus facts. `docs/language-baseline.md`
holds the normative Oberon-2 EBNF.

## Method

**Test-first, always.** Write the failing test, run it, see it red, then implement. No
production code without a test that failed first. Integration tests live in `tests/`; the
`.rs` test file names the milestone it belongs to in its header comment.

**Design rule:** `xoft-core` performs no I/O and renders no text. Walking the filesystem,
reading files and rendering diagnostics belong in `xoft-cli`.

**Ambiguous syntax, ask.** When a grammar case, corpus interpretation, or dialect quirk is
genuinely ambiguous, stop and ask for a check/clarification rather than guessing and moving on.

**Start of every round:** read `docs/checklist.md` — a condensed error → mitigation list — before
writing any code, so a past mistake doesn't repeat.

## End of every round

Before stopping, in this order:

1. Update `PROGRESS.md` and the per-phase file under `docs/progress/`.
2. Append anything learned to `docs/insights.md`.
3. Append any mistake made, with its mitigation, to `docs/errors.md`, and add a matching
   one-line bullet to `docs/checklist.md`.
4. Rewrite `NEXT.md`: the single next task, plus whatever a fresh session needs to start it
   cold.
5. `cargo test --workspace`, then commit and push.

Then stop, so the next round starts in a fresh session.

## Commands

```sh
cargo test --workspace
cargo run -p xoft-cli -- corpus manifest    # rebuild corpus/manifest.json (792 files)
```

The corpus lives outside this repo; absolute paths exist only in `corpus/roots.toml`.
