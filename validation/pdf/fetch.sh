#!/usr/bin/env bash
# Fetch an LHAPDF6 set from the LHAPDF data server and unpack it into this
# directory.
#
# The default set is NNPDF23_lo_as_0130_qed (MG5's LO default `nn23lo1`), a
# single-Q²-subgrid set. Pass a set name to fetch another; the grid oracle also
# needs a genuinely multi-Q²-subgrid set (its Q² axis is split into bands joined
# at seam knots) to exercise the subgrid walk and seam interpolation.
#
# Usage:
#   bash fetch.sh                       # NNPDF23_lo_as_0130_qed
#   bash fetch.sh MSHT20lo_as130        # a chosen set
#   pixi run fetch-pdf                  # NNPDF23 via pixi
#   pixi run fetch-pdf-multigrid        # the multi-subgrid set
#
# The download and the consent it needs live in ../fetch_common.sh, shared with
# the submodule checkout and the banked-reference bundle.
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/fetch_common.sh"

SET_NAME="${1:-NNPDF23_lo_as_0130_qed}"
vg_ensure_pdf_set "$SET_NAME" ||
  vg_die "$SET_NAME is a declared input of the banked layer and was not acquired"
