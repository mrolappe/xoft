# Fixture corpus provenance

The full corpus (`corpus/roots.toml`) lives outside this repository at machine-local
paths and cannot be vendored (see that file's header comment) -- so CI (M4.2) has no
access to it. This directory is a small, deliberately vendored subset that stands in for
the real corpus in CI. It is not a replacement for the full local `xoft corpus run`.

## `voc/` -- vishap oberon compiler standard library

Six files copied unmodified from `github.com/vishaps/voc`, `src/library/`, under that
project's GPL-3.0 license (see `voc/LICENSE` in the upstream repo). Picked as small,
diverse (five subdirectories), non-allowlisted (i.e. known clean-parsing) examples.

## `oberon-a/` -- Oberon-A 1.6 (Frank Copeland)

Six files extracted unmodified from `Obrn-A_1.6_src.lha`, downloaded from
`https://aminet.net/package/dev/obero/Obrn-A_1.6`. That package's own Aminet readme
states: "Distribution: Freeware, under the GNU General Public Licence." Picked as small,
diverse (four subdirectories), non-allowlisted examples.

## Roots left out

- `amiga-oberon-31` (AmigaOberon 3.1, A+L AG): commercial software, no confirmed freely
  redistributable source found. Left out per user decision ("when in doubt, leave it
  out").
- `stj` (STJ-Oberon, Atari ST): no public source URL at hand. Left out.

Both stay covered by the full local corpus run (`corpus/roots.toml`, not automated in
CI) only.
