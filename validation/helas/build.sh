#!/usr/bin/env bash
# Build the HELAS Fortran77 sources into a Python extension module via f2py.
#
# Usage: bash build.sh   (or: pixi run -e helas-validation build-helas)
# Output: helas_f.cpython-<ver>-<arch>.so  (importable as "import helas_f")
set -euo pipefail

echo "Building HELAS f2py extension..."

# -fallow-argument-mismatch: suppress errors from mixed real/complex arg types
# in old Fortran77 code (required for gfortran >= 10).
# The .F extension triggers C-preprocessor (handles #ifdef HELAS_CHECK blocks).
python -m numpy.f2py \
    -c \
    --f77flags="-fallow-argument-mismatch" \
    ixxxxx.F oxxxxx.F jioxxx.F iovxxx.F vxxxxx.F \
    -m helas_f

echo "Done. Module: helas_f"
