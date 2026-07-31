#!/usr/bin/env bash
# Build the MadGraph |M|^2 probe module and (re)generate the pointwise Drell-Yan
# integrand oracle (dy_integrand_oracle.json + dy13_param_card.dat) consumed by
# validate_hadronic's pointwise_integrand_oracle test.
#
# Requires the dy13_default MadGraph output (from generate-hadronic-sigma), which
# supplies the grouped subprocess matrix elements (MATRIX1 = u u~, MATRIX2 =
# d d~) and the built libmodel/libdhelas.
#
# Usage: pixi run -e madgraph generate-dy-oracle
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PDIR="$HERE/output/dy13_default/SubProcesses/P1_qq_ll"
LIBDIR="$HERE/output/dy13_default/lib"

if [ ! -f "$PDIR/matrix1_optim.f" ] || [ ! -f "$LIBDIR/libmodel.a" ]; then
  echo "!!! missing $PDIR / libmodel.a — run 'pixi run -e madgraph generate-hadronic-sigma' first" >&2
  exit 1
fi

if ! ls "$HERE"/output/f2py/mg_dy_probe*.so >/dev/null 2>&1; then
  echo ">>> building mg_dy_probe (MATRIX1 = u u~, MATRIX2 = d d~) ..."
  pushd "$PDIR" >/dev/null
  python -m numpy.f2py -c \
    --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
    matrix1_optim.f matrix2_optim.f "$HERE/wrappers/dy_probe.f" \
    -L"$LIBDIR" -lmodel -ldhelas \
    -m mg_dy_probe
  mkdir -p "$HERE/output/f2py"
  mv mg_dy_probe*.so "$HERE/output/f2py/"
  popd >/dev/null
fi

python "$HERE/gen_dy_oracle.py"
