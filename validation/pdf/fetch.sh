#!/usr/bin/env bash
# Fetch the pinned LHAPDF6 set (NNPDF23_lo_as_0130_qed, MG5's LO default
# `nn23lo1`) from the LHAPDF data server and extract it into this directory.
#
# Usage: bash fetch.sh   (or: pixi run -e madgraph fetch-pdf)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SET_NAME="NNPDF23_lo_as_0130_qed"
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
