#!/usr/bin/env bash
# Compile the LHAPDF C++ oracle generator and dump the committed reference values
# for both grid shapes: oracle.json (one rectangular Q² subgrid) and
# oracle_multigrid.json (two bands joined at one seam).
#
# LHAPDF is the library MadGraph evaluates PDFs through, so its own values on
# these probe points are the reference our interpolation has to reproduce.
#
# Usage: pixi run -e madgraph generate-pdf-oracle
#
# The rpath makes the built generator self-contained; the system c++ from
# /usr/bin is used when the environment ships no compiler of its own.
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/fetch_common.sh"

PDF_DIR="$VG_VALIDATION_DIR/pdf"
GEN="$PDF_DIR/gen_oracle"

c++ -std=c++17 -O2 "$PDF_DIR/gen_oracle.cpp" \
  $(lhapdf-config --cflags --ldflags) -lLHAPDF \
  -Wl,-rpath,"$(lhapdf-config --libdir)" -o "$GEN"

"$GEN" "$PDF_DIR" NNPDF23_lo_as_0130_qed 0 "$PDF_DIR/oracle.json"
"$GEN" "$PDF_DIR" NNPDF31_lo_as_0130 0 "$PDF_DIR/oracle_multigrid.json"
