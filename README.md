# xoft

A workbench for designing Oberon-flavored dialects.

xoft parses Oberon-2 and experimental dialects derived from it, transpiles between them, and
puts the source back together **byte for byte** — comments, indentation, line endings and all.
Lossless round-tripping is the point: a dialect experiment is only interesting if you can run
your real code through it and get your real code back.

Status: early. Milestone M0 (foundations) is complete; see [PROGRESS.md](PROGRESS.md).

## Why

Oberon-2 has a small, clean grammar and a family of real dialects that diverged in the wild —
Oberon-A on the Amiga, STJ-Oberon on the Atari ST, AmigaOberon 3.1. That makes it an unusually
good substrate for asking *what if the syntax were different?* and answering with running code
over a real 792-file corpus rather than a toy example.

## Design in one paragraph

A [tree-sitter](https://tree-sitter.github.io/) grammar produces a lossless CST with byte spans.
Dialects are grammar overlays that inherit from the Oberon-2 base, and mappings between them are
tree-sitter queries plus small transform functions — so adding a dialect is a query file, not a
compiler refactor. Encodings are handled by a byte↔codepoint bijection rather than charset
tables, which makes byte-identity a property of the design instead of a thing to test for.
The reasoning behind each of these is written down as decisions D1–D8 in [docs/plan.md](docs/plan.md).

## Layout

| Path | What |
|---|---|
| `crates/xoft-core` | parsing, lossless serialization, mapping — no I/O, renders no text |
| `crates/xoft-cli` | the `xoft` binary |
| `grammars/` | tree-sitter grammars: Oberon-2 base, plus dialect overlays |
| `corpus/` | inventory of the test corpus (the sources themselves live outside the repo) |
| `docs/` | [plan and decisions](docs/plan.md), [language baseline](docs/language-baseline.md), [insights](docs/insights.md) |

## Build

```sh
cargo test --workspace
cargo run -p xoft-cli -- corpus manifest
```

## License

MIT. The tree-sitter grammar is forked from
[`viegasfh/tree-sitter-oberon-2`](https://github.com/viegasfh/tree-sitter-oberon-2) (MIT).
Corpus sources are third-party and are not redistributed here.
