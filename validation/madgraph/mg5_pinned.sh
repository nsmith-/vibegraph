#!/usr/bin/env bash
# Run the *pinned* MadGraph — `research/refs/mg5amcnlo`, the same checkout the
# source-level reading gates parse — rather than whatever `mg5_aMC` the
# environment puts on PATH.
#
# Why the pin rather than the packaged build: the packaged 3.5.x line computes a
# propagator's off-shellness in `get_channel_cut` as `(t-Mass)*(t+Mass)` with `t`
# already `p²`, so the `sde_strategy = 2` multichannel weight never peaks on a
# resonance. A run whose events feed a reference must not inherit that, and the
# 3.5.x line never took the fix.
#
# What the pinned checkout needs: a Python 3 with `six`, and — for the Fortran a
# generated directory later builds — the compiler toolchain the packaged
# environment already carries. Both come from `pixi run -e madgraph`, which is
# how every caller here invokes it. Generation itself compiles nothing.
#
# Three environment details this handles so callers do not repeat them:
#
#   * MadGraph drops a scratch `py.py` into its working directory, so it runs in
#     a temporary one and the repository stays clean.
#   * LHAPDF finds the validation layer's own fetched sets, so a `pdlabel =
#     lhapdf` run reads the set this repository pins instead of downloading one.
#   * A generation launches no browser and posts no desktop notification. The
#     pinned checkout carries no site configuration, so both default to on and a
#     batch run would open a page per process directory; the two `set` lines
#     below turn them off for this invocation. `--no_save` keeps them out of the
#     submodule — and out of the generated `Cards/me5_configuration.txt` too, so
#     a caller that goes on to run `generate_events` must silence that file
#     separately (`silence_madgraph_ui` in `gen_hadronic_sigma.sh`).
#
# Usage: bash validation/madgraph/mg5_pinned.sh <script.mg5>
#        (paths inside the script must be absolute — the cwd is temporary)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
MG5_ROOT="$ROOT/research/refs/mg5amcnlo"

[ $# -eq 1 ] || {
  echo "usage: bash ${BASH_SOURCE[0]} <script.mg5>" >&2
  exit 2
}
[ -f "$1" ] || {
  echo "!!! no such MadGraph script: $1" >&2
  exit 1
}
[ -f "$MG5_ROOT/bin/mg5_aMC" ] || {
  echo "!!! no pinned MadGraph at $MG5_ROOT (git submodule update --init)" >&2
  exit 1
}

LHAPDF_DATA_PATH="$ROOT/validation/pdf"
if command -v lhapdf-config >/dev/null 2>&1; then
  LHAPDF_DATA_PATH="$LHAPDF_DATA_PATH:$(lhapdf-config --datadir)"
fi
export LHAPDF_DATA_PATH

SCRIPT="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/vg-mg5-XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

{
  printf 'set automatic_html_opening False --no_save\n'
  printf 'set notification_center False --no_save\n'
  cat "$SCRIPT"
} > "$WORK/driver.mg5"

cd "$WORK"

python "$MG5_ROOT/bin/mg5_aMC" "$WORK/driver.mg5"
