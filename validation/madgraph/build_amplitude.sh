#!/usr/bin/env bash
# Compile MadGraph Fortran matrix elements into f2py Python extension modules.
#
# Each compiled module mg_PROCESS.so lands in the work area under output/f2py/,
# which the generators put on sys.path — a compiled artifact of the generated
# Fortran, so it belongs with the rest of the work area rather than beside the
# scripts that import it.
#
# Compiling is the expensive part, so a module that is already there is left
# alone; VG_FORCE=1 rebuilds everything.
#
# Usage:
#   bash validation/madgraph/build_amplitude.sh
#   bash validation/madgraph/build_amplitude.sh ee_to_mumu gg_to_gg   # only these
#   pixi run -e madgraph build-amplitude
#
# Prerequisites: pixi run -e madgraph build-diagrams  (generates MG output dirs)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MG_OUTPUT="$REPO_ROOT/validation/madgraph/output"
WRAPPERS="$REPO_ROOT/validation/madgraph/wrappers"
OUTDIR="$MG_OUTPUT/f2py"
FORCE="${VG_FORCE:-0}"
mkdir -p "$OUTDIR"

# already_built MODULE — true when the work area holds this extension module.
already_built() {
    [ "$FORCE" != 1 ] && compgen -G "$OUTDIR/$1*.so" > /dev/null
}

# All amplitude-validation processes build against the shared wrappers/generic.f
# (SETPARA reads couplings from param_card.dat; libmodel.a supplies the model
# routines), so registering a new process is one .mg5 script plus one
# `mg_amplitude` table in validation/manifest.toml — the same table
# gen_amplitude.py reads, listed here through its own resolved registry so the
# two can never name different sets.
# Positional arguments narrow the set to those rows; with none, every row builds.
mapfile -t GENERIC_PROCESSES < <(
    python "$REPO_ROOT/validation/madgraph/gen_amplitude.py" --dump-processes |
        python -c 'import json,sys; [print(r["name"]) for r in json.load(sys.stdin)]' |
        { if [ $# -gt 0 ]; then grep -Fx -f <(printf '%s\n' "$@"); else cat; fi; }
)
[ ${#GENERIC_PROCESSES[@]} -gt 0 ] || {
    echo "ERROR: the manifest resolved to no mg_amplitude rows" >&2
    exit 1
}

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
    if already_built "mg_${name}"; then
        echo "⊘ mg_${name} already built"
        return 0
    fi
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
    # `launch` writes matrix1_optim.f as the helicity-recycled matrix element,
    # whose `MATRIX1` is a subroutine summing over helicities into an array --
    # the signature wrappers/generic.f is written against. Where recycling did
    # not run, MadGraph copies matrix1_orig.f into place instead, and that file's
    # `MATRIX1` is a per-helicity function with a different argument list; the
    # two link but disagree, so the module would be built and then crash. A 2 -> 1
    # process is the case that reaches here: it has no phase space to integrate,
    # so `launch` never gets as far as recycling. Its |M|^2 comes from the
    # per-diagram probe below, which calls the per-helicity form directly.
    if grep -qi 'USE DISCRETESAMPLER' "$pdir/matrix1_optim.f"; then
        echo "SKIP $name: matrix1_optim.f is the un-recycled matrix element; \
its |M|^2 comes from mg_amp_probe_${name} instead"
        return 0
    fi

    echo "Building mg_${name} (generic, $(basename "$pdir"))..."

    # f2py scans only matrix1_optim.f and the wrapper; the model coupling
    # routines (SETPARA/COUP/lha_read) are linked from the precompiled libmodel.a
    # that `launch` builds, so f2py never has to parse their complex PARAMETER
    # decls. Run from the subprocess directory so Fortran INCLUDE statements
    # resolve, with `-I../../Source` for the generated code's Fortran modules --
    # the search path MadGraph's own makefileP uses for the subprocess.
    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I. -I../../Source" \
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
    if already_built "mg_ee_amp_probe"; then
        echo "⊘ mg_ee_amp_probe already built"
        return 0
    fi
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
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I. -I../../Source" \
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
    if already_built "mg_amp_probe_${name}"; then
        echo "⊘ mg_amp_probe_${name} already built"
        return 0
    fi
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
    #
    # A source that already carries one of the two blocks keeps it: declaring
    # the same COMMON member twice makes f2py emit a setup function with
    # duplicate parameters, which does not compile.
    local has_amp=0 has_jamp=0
    grep -q 'COMMON/DBG_AMP/' "$pdir/matrix1_orig.f" && has_amp=1
    grep -q 'COMMON/DBG_JAMP/' "$pdir/matrix1_orig.f" && has_jamp=1
    sed -n '/FUNCTION MATRIX1(/,$p' "$pdir/matrix1_orig.f" |
        awk -v has_amp="$has_amp" -v has_jamp="$has_jamp" '
        /^      JAMP\(:,:\) = \(0D0,0D0\)$/ && has_amp == 0 {
            print "      DO I = 1, NGRAPHS"
            print "        AMP_DBG(I) = AMP(I)"
            print "      ENDDO"
        }
        /^      MATRIX1 = 0.D0$/ && has_jamp == 0 {
            print "      DO I = 1, NCOLOR"
            print "        JAMP_DBG(I) = JAMP(I,1)"
            print "      ENDDO"
        }
        { print }
        /^      COMPLEX\*16 AMP\(NGRAPHS\), JAMP\(NCOLOR,NAMPSO\)$/ {
            if (has_amp == 0) {
                print "      COMPLEX*16 AMP_DBG(NGRAPHS)"
                print "      COMMON/DBG_AMP/AMP_DBG"
            }
            if (has_jamp == 0) {
                print "      COMPLEX*16 JAMP_DBG(NCOLOR)"
                print "      COMMON/DBG_JAMP/JAMP_DBG"
            }
        }
    ' > "$pdir/matrix1_ampdbg.f"
    sed -e "s/@NGRAPHS@/$ngraphs/" -e "s/@NCOLOR@/$ncolor/" \
        "$WRAPPERS/amp_probe.f.in" > "$pdir/amp_probe_gen.f"

    pushd "$pdir" > /dev/null
    python -m numpy.f2py \
        -c \
        --f77flags="-fallow-argument-mismatch -ffixed-line-length-132 -I. -I../../Source" \
        "matrix1_ampdbg.f" \
        "amp_probe_gen.f" \
        -L"$libdir" -lmodel -ldhelas \
        -m "mg_amp_probe_${name}"
    mv mg_amp_probe_${name}*.so "$OUTDIR/"
    popd > /dev/null
    echo "  -> $OUTDIR/mg_amp_probe_${name}*.so"
}

# Per-diagram / per-flow oracle for every validated process: the committed
# amplitude tables carry AMP() and JAMP() per helicity, which is the finest
# linear level MadGraph exposes and the level the gate compares at.
AMP_PROBE_PROCESSES=("${GENERIC_PROCESSES[@]}")
for name in "${AMP_PROBE_PROCESSES[@]}"; do
    build_amp_dump_probe "$name"
done

echo "Done building amplitude modules."
