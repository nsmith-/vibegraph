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

# compile_process_generic NAME SUBPROCESS_DIR
#   Generic build: links matrix1_optim.f + the shared wrappers/generic.f +
#   the Source/MODEL coupling routines (so SETPARA resolves).  No per-process
#   Fortran or hand-coded couplings.  Use for any process whose external legs
#   are all 2-helicity (massless fermions/vectors).
compile_process_generic() {
    local name="$1"
    local subproc="$2"
    local pdir="$MG_OUTPUT/$name/SubProcesses/$subproc"
    local libdir="$MG_OUTPUT/$name/lib"
    local model="$MG_OUTPUT/$name/Source/MODEL"
    local wrapper="$WRAPPERS/generic.f"

    if [ ! -f "$pdir/matrix1_optim.f" ]; then
        echo "SKIP $name: matrix1_optim.f not found (did the .mg5 script 'launch'?)"
        return 0
    fi
    if [ ! -f "$libdir/libmodel.a" ]; then
        echo "SKIP $name: libmodel.a not found (did the .mg5 script 'launch'?)"
        return 0
    fi

    echo "Building mg_${name} (generic, subprocess $subproc)..."

    # f2py scans only matrix1_optim.f + the wrapper; the model coupling routines
    # (SETPARA/COUP/lha_read) are linked from the precompiled libmodel.a that
    # `launch` builds, so f2py never has to parse their complex PARAMETER decls.
    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
        "matrix1_optim.f" \
        "$wrapper" \
        -L"$libdir" -lmodel -ldhelas \
        -m "mg_${name}"

    mv mg_${name}*.so "$OUTDIR/"
    popd > /dev/null
    : "$model"  # (model dir no longer needed; kept for clarity)

    echo "  -> $OUTDIR/mg_${name}*.so"
}

# uux_to_ccx_emmm_qcd0: u u~ > c c~ e+ e- mu+ mu- (QCD=0).  Different-flavor
# quarks => single color flow (NCOLOR=1); built with the generic wrapper.
# MATRIX1 includes the CF(1,1)=9 color factor; vibegraph applies it on its side.
compile_process_generic uux_to_ccx_emmm_qcd0 P1_qq_qqllll

# ee_to_mumu: pure QED/EW process, no color — expected to match Rust eval_m2
compile_process ee_to_mumu P1_ll_ll

# pp_to_ll_qcd0: u u~ > l+ l- via gamma/Z (QCD=0).  MATRIX1 includes CF=3 quark
# color factor; Rust omits color, so agreement is informational until color is
# implemented in vibegraph.
compile_process pp_to_ll_qcd0 P1_qq_ll

echo "Done building amplitude modules."
