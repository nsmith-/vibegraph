#!/usr/bin/env bash
# Compile MadGraph's own alfas_functions.f together with driver.f and write
# validation/alphas/reference.csv.
#
# Usage: pixi run -e madgraph generate-alphas-reference
#
# -ffp-contract=off keeps the compiler from fusing a*b+c into an FMA. Rust emits
# no such contraction, so without this flag the two sides would differ in the low
# bits of every Newton iterate for reasons that have nothing to do with the
# algorithm.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MG_SOURCE="${CONDA_PREFIX:?run under the madgraph pixi environment}/MG5_aMC/Template/LO/Source"

if [ ! -f "$MG_SOURCE/alfas_functions.f" ]; then
  echo "MadGraph LO template source not found at $MG_SOURCE" >&2
  exit 1
fi

cd "$SCRIPT_DIR"

gfortran -O2 -ffp-contract=off -std=legacy \
  -I"$MG_SOURCE" \
  "$MG_SOURCE/alfas_functions.f" driver.f \
  -o alphas_reference

./alphas_reference
rm -f alphas_reference

echo "reference.csv: $(wc -l < reference.csv) lines"
