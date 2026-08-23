#!/usr/bin/env bash
# Regenerates testbed-ui/src/grammars/{oberon2,oberon-x}.wasm + highlights.scm from the
# checked-in grammar sources (grammars/tree-sitter-oberon2, grammars/tree-sitter-oberon-x).
# Run after either grammar changes. Requires `tree-sitter` (tree-sitter-cli) on PATH; its
# first `--wasm` build downloads and caches a wasi-sdk toolchain.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
out="$repo_root/testbed-ui/src/grammars"
mkdir -p "$out"

tree-sitter build --wasm -o "$out/oberon2.wasm" "$repo_root/grammars/tree-sitter-oberon2"
tree-sitter build --wasm -o "$out/oberon-x.wasm" "$repo_root/grammars/tree-sitter-oberon-x"
cp "$repo_root/grammars/tree-sitter-oberon2/queries/highlights.scm" "$out/highlights.scm"
