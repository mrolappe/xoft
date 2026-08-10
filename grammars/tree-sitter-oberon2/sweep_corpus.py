#!/usr/bin/env python3
"""Parse-only corpus sweep: runs `tree-sitter parse` over every corpus/manifest.json
file and reports the ERROR/MISSING-free percentage plus the list of failures.

Run from this directory (grammars/tree-sitter-oberon2) so tree-sitter finds the
local grammar. Throwaway tool per NEXT.md M1.4 — not a substitute for the real
M4 corpus runner.
"""
import json
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = REPO_ROOT / "corpus" / "manifest.json"
ROOTS_TOML = REPO_ROOT / "corpus" / "roots.toml"


def load_roots() -> dict[str, Path]:
    data = tomllib.loads(ROOTS_TOML.read_text())
    return {r["alias"]: Path(r["path"]) for r in data["root"]}


def main() -> int:
    roots = load_roots()
    manifest = json.loads(MANIFEST.read_text())
    files = manifest["files"]

    # tree-sitter parse always reads its input as UTF-8. ~42% of the corpus
    # (manifest encoding "high-bytes") is Latin-1/high-byte source (Amiga/Atari
    # era tools), so feed those through a UTF-8-transcoded temp copy — otherwise
    # a single non-ASCII byte (e.g. "\xa9" in a copyright-banner comment)
    # collapses the whole file into one ERROR node and masks the real signal.
    failures = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp) / "sweep.tmp"
        for entry in files:
            root_path = roots[entry["root"]]
            abs_path = root_path / entry["path"]
            if entry["encoding"] == "high-bytes":
                text = abs_path.read_text(encoding="latin-1")
                tmp_path.write_text(text, encoding="utf-8")
                parse_path = tmp_path
            else:
                parse_path = abs_path
            result = subprocess.run(
                ["tree-sitter", "parse", "--quiet", str(parse_path)],
                capture_output=True,
                text=True,
            )
            if result.returncode != 0:
                failures.append((entry["root"], entry["path"], result.stdout.strip()))

    total = len(files)
    ok = total - len(failures)
    pct = 100.0 * ok / total if total else 0.0
    print(f"Total: {total}  OK: {ok}  Failed: {len(failures)}  Success: {pct:.2f}%")

    if failures:
        print("\nFailures:")
        for root, path, detail in failures:
            print(f"  {root}/{path}")
            if len(sys.argv) > 1 and sys.argv[1] == "-v":
                print(f"    {detail}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
