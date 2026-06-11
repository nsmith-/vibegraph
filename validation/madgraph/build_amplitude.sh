#!/usr/bin/env bash
# Compile MadGraph Fortran matrix elements into f2py Python extension modules.
#
# Each compiled module mg_PROCESS.so is placed in validation/madgraph/ so that
# gen_amplitude.py can import it via sys.path.
#
# Usage:
#   bash validation/madgraph/build_amplitude.sh
#   pixi run -e madgraph build-amplitude
#
# Prerequisites: pixi run -e madgraph build-diagrams  (generates MG output dirs)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MG_OUTPUT="$REPO_ROOT/validation/madgraph/output"
WRAPPERS="$REPO_ROOT/validation/madgraph/wrappers"
OUTDIR="$REPO_ROOT/validation/madgraph"

# compile_process NAME SUBPROCESS_DIR
#   NAME       — process name, e.g. ee_to_mumu
#   SUBPROCESS — subprocess subdirectory, e.g. P1_ll_ll
compile_process() {
    local name="$1"
    local subproc="$2"
    local pdir="$MG_OUTPUT/$name/SubProcesses/$subproc"
    local libdir="$MG_OUTPUT/$name/lib"
    local wrapper="$WRAPPERS/${name}.f"

    if [ ! -f "$pdir/matrix1_optim.f" ]; then
        echo "SKIP $name: matrix1_optim.f not found in $pdir"
        return 0
    fi
    if [ ! -f "$wrapper" ]; then
        echo "SKIP $name: wrapper $wrapper not found"
        return 0
    fi

    echo "Building mg_${name} (subprocess $subproc)..."

    # Run from the subprocess directory so Fortran INCLUDE statements resolve.
    # -I. makes the compiler find coupl.inc, maxamps.inc, nexternal.inc, genps.inc, etc.
    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
        "matrix1_optim.f" \
        "$wrapper" \
        -L"$libdir" -ldhelas \
        -m "mg_${name}"

    # Move the compiled .so to validation/madgraph/ for import by gen_amplitude.py
    mv mg_${name}*.so "$OUTDIR/"
    popd > /dev/null

    echo "  -> $OUTDIR/mg_${name}*.so"
}

# ee_to_mumu: pure QED/EW process, no color — expected to match Rust eval_m2
compile_process ee_to_mumu P1_ll_ll

# Future (colored processes — scaffold only, not expected to agree until color is
# implemented in vibegraph):
# compile_process pp_to_ll  P1_ll_ll
# compile_process pp_to_bb  P1_bb_gg

echo "Done building amplitude modules."
