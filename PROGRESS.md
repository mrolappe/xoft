# Progress

Overall state of the xoft MVP. One line per milestone; details in the per-phase files.

| Milestone | State | Detail |
|---|---|---|
| M0 — Foundations | ✅ done | [docs/progress/m0-foundations.md](docs/progress/m0-foundations.md) |
| M1 — Grammar | 🟨 in progress (M1.1, M1.2a, M1.2b, M1.2c, M1.3, M1.5 done; M1.4 remains) | [docs/progress/m1-grammar.md](docs/progress/m1-grammar.md) |
| M2 — Core: lossless parse/serialize | ⬜ not started | — |
| M3 — Diagnostics and CLI | ⬜ not started | — |
| M4 — Corpus runner | ⬜ not started | — |
| M5 — Toy dialect Oberon-X | ⬜ not started | — |
| M6 — Testbed | ⬜ not started | — |
| M7 — Phase 2 plan | ⬜ not started | — |

**Next task:** see [NEXT.md](NEXT.md).

Cross-cutting records: [insights](docs/insights.md) · [errors and mitigations](docs/errors.md) ·
[plan and decisions](docs/plan.md) · [language baseline](docs/language-baseline.md)

## Rounds

| # | Date | Covered |
|---|---|---|
| 1 | 2026-08-10 | Project bootstrap, M0 complete |
| 2 | 2026-08-10 | M1.1 base grammar vendored and building; M1.5 highlights.scm |
| 3 | 2026-08-10 | M1.2a — receivers, forward declarations, DEFINITION module header |
| 4 | 2026-08-10 | M1.2b — WITH, LOOP, EXIT, RETURN statements, empty statements, CASE label ranges confirmed |
| 5 | 2026-08-10 | M1.2c — IS, SET, open arrays confirmed as already working; procedure types fixed in `formal_type`; incidental fix for `field_list_seq` trailing-semicolon gap |
| 6 | 2026-08-10 | M1.3 — external C scanner for nested comments + `(*$…*)` pragma node; `.gitignore` fix so `scanner.c` is tracked |
