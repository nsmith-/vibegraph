#!/usr/bin/env bash
# Fetch an LHAPDF6 set from the LHAPDF data server and extract it into this
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
#   pixi run -e madgraph fetch-pdf      # NNPDF23 via pixi
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SET_NAME="${1:-NNPDF23_lo_as_0130_qed}"
URL="https://lhapdfsets.web.cern.ch/current/${SET_NAME}.tar.gz"
SET_DIR="$SCRIPT_DIR/$SET_NAME"

if [ -d "$SET_DIR" ]; then
  echo "⊘ $SET_NAME already present at $SET_DIR — skipping fetch"
  exit 0
fi

TARBALL="$SCRIPT_DIR/${SET_NAME}.tar.gz"
echo "Downloading $URL ..."
curl -sSL --fail -o "$TARBALL" "$URL"

echo "Extracting into $SCRIPT_DIR ..."
tar xzf "$TARBALL" -C "$SCRIPT_DIR"
rm -f "$TARBALL"

echo "✓ $SET_NAME ready at $SET_DIR"
