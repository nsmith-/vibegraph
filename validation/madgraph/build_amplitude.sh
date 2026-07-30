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

# All amplitude-validation processes build against the shared wrappers/generic.f
# (SETPARA reads couplings from param_card.dat; libmodel.a supplies the model
# routines), so registering a new process is one line here + one .mg5 script +
# one gen_amplitude.py registry entry.
GENERIC_PROCESSES=(
    ee_to_mumu
    pp_to_ll_qcd0
    ee_to_ee
    ee_to_mumua
    ee_to_ttx
    ee_to_wpwm
    ee_to_zh
    ee_to_tatah
    ee_to_mumu_tata_qcd0
    uux_to_ccx_emmm_qcd0
    bbx_to_ccx_emmm_qcd0
    uux_to_uux
    gg_to_ttx
    gg_to_gg
    uux_to_epemg
    gu_to_epemu
    ddx_to_epemg
    gux_to_epemux
)

# subprocess_dir NAME — the single SubProcesses/P1_* directory of a process.
# All registered processes are single-subprocess by construction (one concrete
# flavor assignment per .mg5 script), so a glob with a uniqueness check avoids
# hardcoding MadGraph's P1 naming per process.
subprocess_dir() {
    local name="$1"
    local matches=("$MG_OUTPUT/$name/SubProcesses/"P1_*/)
    if [ ${#matches[@]} -ne 1 ]; then
        echo "ERROR $name: expected exactly one SubProcesses/P1_* dir, found: ${matches[*]}" >&2
        return 1
    fi
    echo "${matches[0]%/}"
}

# compile_process_generic NAME
#   Generic build: links matrix1_optim.f + the shared wrappers/generic.f +
#   the launch-built libmodel.a (so SETPARA resolves).  No per-process Fortran
#   or hand-coded couplings.
compile_process_generic() {
    local name="$1"
    local pdir libdir
    if ! pdir="$(subprocess_dir "$name")"; then
        echo "SKIP $name: no unique subprocess dir (did build-diagrams run?)"
        return 0
    fi
    libdir="$MG_OUTPUT/$name/lib"

    if [ ! -f "$pdir/matrix1_optim.f" ]; then
        echo "SKIP $name: matrix1_optim.f not found (did the .mg5 script 'launch'?)"
        return 0
    fi
    if [ ! -f "$libdir/libmodel.a" ]; then
        echo "SKIP $name: libmodel.a not found (did the .mg5 script 'launch'?)"
        return 0
    fi

    echo "Building mg_${name} (generic, $(basename "$pdir"))..."

    # f2py scans only matrix1_optim.f + the wrapper; the model coupling routines
    # (SETPARA/COUP/lha_read) are linked from the precompiled libmodel.a that
    # `launch` builds, so f2py never has to parse their complex PARAMETER decls.
    # Run from the subprocess directory so Fortran INCLUDE statements resolve.
    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
        "matrix1_optim.f" \
        "$WRAPPERS/generic.f" \
        -L"$libdir" -lmodel -ldhelas \
        -m "mg_${name}"

    mv mg_${name}*.so "$OUTDIR/"
    popd > /dev/null

    echo "  -> $OUTDIR/mg_${name}*.so"
}

for name in "${GENERIC_PROCESSES[@]}"; do
    compile_process_generic "$name"
done

# ee_amp_probe: per-diagram amplitude + intermediate wavefunction probe for the
# e+ e- > mu+ mu- ta+ ta- continuum debugging.  Uses wrappers/matrix1_func.f (a
# patched MATRIX1 with DBG_AMP + DBG_WFUNCS COMMON blocks) instead of the
# generated matrix1_optim.f, plus wrappers/ee_amp_probe.f (f2py entry points).
build_amp_probe() {
    local name="ee_to_mumu_tata_qcd0"
    local subproc="P1_ll_lltaptam"
    local pdir="$MG_OUTPUT/$name/SubProcesses/$subproc"
    local libdir="$MG_OUTPUT/$name/lib"

    if [ ! -f "$pdir/matrix1_optim.f" ]; then
        echo "SKIP ee_amp_probe: $pdir/matrix1_optim.f not found"
        return 0
    fi

    echo "Building mg_ee_amp_probe (wavefunction + amplitude probe)..."
    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
        "$WRAPPERS/matrix1_func.f" \
        "$WRAPPERS/ee_amp_probe.f" \
        -L"$libdir" -lmodel -ldhelas \
        -m "mg_ee_amp_probe"
    mv mg_ee_amp_probe*.so "$OUTDIR/"
    popd > /dev/null
    echo "  -> $OUTDIR/mg_ee_amp_probe*.so"
}

build_amp_probe

# build_amp_dump_probe NAME: per-diagram amplitude + per-flow JAMP probe for any
# process.  matrix1_orig.f (one HELAS call sequence per diagram, AMP(i) ==
# diagram i, then JAMP(i,1) = Σ coeff·AMP per color flow) is patched at build
# time with COMMON/DBG_AMP/ and COMMON/DBG_JAMP/ blocks exposing AMP and JAMP,
# then wrapped by wrappers/amp_probe.f.in (f2py entry point; NGRAPHS and NCOLOR
# substituted from the process's matrix1_orig.f).  Consumed by
# validation/madgraph/compare_amps.py.
build_amp_dump_probe() {
    local name="$1"
    local pdir libdir ngraphs ncolor
    if ! pdir="$(subprocess_dir "$name")"; then
        echo "SKIP amp_probe $name: no unique subprocess dir"
        return 0
    fi
    libdir="$MG_OUTPUT/$name/lib"

    if [ ! -f "$pdir/matrix1_orig.f" ]; then
        echo "SKIP amp_probe $name: matrix1_orig.f not found"
        return 0
    fi

    ngraphs=$(grep -m1 -o 'NGRAPHS=[0-9]*' "$pdir/matrix1_orig.f" | cut -d= -f2)
    ncolor=$(grep -m1 -o 'NCOLOR=[0-9]*' "$pdir/matrix1_orig.f" | cut -d= -f2)
    echo "Building mg_amp_probe_${name} (per-diagram AMP + per-flow JAMP probe, NGRAPHS=$ngraphs NCOLOR=$ncolor)..."

    # Keep only MATRIX1 (drop SMATRIX1, whose MadEvent deps — DSIG, unwgt,
    # DiscreteSampler — the probe neither needs nor links), then inject the
    # COMMON/DBG_AMP/ block copying the per-diagram AMP() and the
    # COMMON/DBG_JAMP/ block copying the per-flow JAMP(:,1) out (NAMPSO=1 for
    # every registered process, so the single split-order slot is the JAMP).
    sed -n '/FUNCTION MATRIX1(/,$p' "$pdir/matrix1_orig.f" | awk '
        /^      JAMP\(:,:\) = \(0D0,0D0\)$/ {
            print "      DO I = 1, NGRAPHS"
            print "        AMP_DBG(I) = AMP(I)"
            print "      ENDDO"
        }
        /^      MATRIX1 = 0.D0$/ {
            print "      DO I = 1, NCOLOR"
            print "        JAMP_DBG(I) = JAMP(I,1)"
            print "      ENDDO"
        }
        { print }
        /^      COMPLEX\*16 AMP\(NGRAPHS\), JAMP\(NCOLOR,NAMPSO\)$/ {
            print "      COMPLEX*16 AMP_DBG(NGRAPHS)"
            print "      COMMON/DBG_AMP/AMP_DBG"
            print "      COMPLEX*16 JAMP_DBG(NCOLOR)"
            print "      COMMON/DBG_JAMP/JAMP_DBG"
        }
    ' > "$pdir/matrix1_ampdbg.f"
    sed -e "s/@NGRAPHS@/$ngraphs/" -e "s/@NCOLOR@/$ncolor/" \
        "$WRAPPERS/amp_probe.f.in" > "$pdir/amp_probe_gen.f"

    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I." \
        "matrix1_ampdbg.f" \
        "amp_probe_gen.f" \
        -L"$libdir" -lmodel -ldhelas \
        -m "mg_amp_probe_${name}"
    mv mg_amp_probe_${name}*.so "$OUTDIR/"
    popd > /dev/null
    echo "  -> $OUTDIR/mg_amp_probe_${name}*.so"
}

# Per-diagram / per-flow oracles for processes under active convention debugging.
AMP_PROBE_PROCESSES=(
    uux_to_ccx_emmm_qcd0
    ee_to_ee
    ee_to_wpwm
    ee_to_zh
    ee_to_tatah
    bbx_to_ccx_emmm_qcd0
    uux_to_uux
    gg_to_ttx
    gg_to_gg
    uux_to_epemg
    gu_to_epemu
    ddx_to_epemg
    gux_to_epemux
)
for name in "${AMP_PROBE_PROCESSES[@]}"; do
    build_amp_dump_probe "$name"
done

echo "Done building amplitude modules."
