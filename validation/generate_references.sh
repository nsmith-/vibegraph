#!/usr/bin/env bash
# The oracle layer's single entry point: everything under `validation/` that is
# derived from MadGraph, LHAPDF or the HELAS Fortran is (re)produced from here.
#
#     pixi run generate-references              # every stage
#     pixi run generate-references refs bundle  # named stages only
#
# Stages, in order:
#
#   deps      the inputs the generators themselves need — the pinned model
#             submodule and the two LHAPDF sets — through validation/fetch_common.sh
#   madgraph  the MadGraph runs. **The work area is the cache**: a process
#             directory that already exists is never rebuilt, and neither is a
#             cross-section run whose banked answer is already written. This is
#             the expensive stage and the one a rerun is meant to skip.
#   refs      every committed reference, recomputed from the work area. These
#             are cheap and pure functions of it, so they always rerun: that is
#             what makes a reference that changed show up as a diff.
#   bundle    the banked-reference archive, for the machines that fetch instead
#             of generating (validation/madgraph/assemble_bundle.sh).
#
# The process list comes from `validation/manifest.toml` by way of the `.mg5`
# scripts it names; nothing here carries a second copy of it.
#
# Not reached from here: the Fortran77 HELAS grid, whose generator needs the
# `helas-validation` environment (gfortran + f2py) rather than this one. The
# `refs` stage says so and names the command when the committed grid is missing.
set -euo pipefail

. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/fetch_common.sh"

MG="$VG_VALIDATION_DIR/madgraph"

stage_banner() {
  vg_say ""
  vg_say "━━━ $1 ━━━"
}

# ── deps ─────────────────────────────────────────────────────────────────────

stage_deps() {
  stage_banner "deps"
  vg_ensure_submodule
  vg_ensure_pdf_set NNPDF23_lo_as_0130_qed ||
    vg_die "the reference generators evaluate PDFs; NNPDF23_lo_as_0130_qed is required"
  vg_ensure_pdf_set NNPDF31_lo_as_0130 ||
    vg_die "the PDF grid oracle needs a multi-subgrid set; NNPDF31_lo_as_0130 is required"
}

# ── madgraph ─────────────────────────────────────────────────────────────────

stage_madgraph() {
  stage_banner "madgraph (cached: existing process directories are never rebuilt)"
  bash "$MG/build.sh"

  # Cached on the work area the same way the process directories are: the script
  # reads a Drell-Yan run that is already on disk instead of re-running madevent,
  # and rewrites the reference from it either way, so the banked cross section
  # always belongs to the banked events. VG_FORCE=1 re-runs both.
  bash "$MG/gen_hadronic_sigma.sh"
}

# ── refs ─────────────────────────────────────────────────────────────────────

stage_refs() {
  stage_banner "refs"

  vg_say ">>> diagrams.json — per-process diagram counts"
  python "$MG/extract_diagrams.py"

  vg_say ">>> f2py matrix-element modules (cached per process)"
  bash "$MG/build_amplitude.sh"

  vg_say ">>> fixed-grid amplitude CSVs"
  python "$MG/gen_amplitude.py"

  vg_say ">>> amplitudes/<process>.json — |M|^2, AMP() and JAMP() at events + grid"
  python "$MG/gen_amplitude_tables.py"

  vg_say ">>> sigma_reference.json — banked fixed-energy cross sections"
  python "$MG/extract_sigma.py"

  vg_say ">>> runcard_defaults.json — MadGraph's own RunCardLO defaults"
  python "$MG/dump_runcard_defaults.py"

  vg_say ">>> alphas/reference.csv — MadGraph's alfas_functions.f on a grid"
  bash "$VG_VALIDATION_DIR/alphas/gen_reference.sh"

  vg_say ">>> pdf/oracle*.json — LHAPDF's own values on both grid shapes"
  bash "$VG_VALIDATION_DIR/pdf/gen_oracle.sh"

  if [ -f "$MG/output/dy13_default/SubProcesses/P1_qq_ll/matrix1_optim.f" ]; then
    vg_say ">>> dy_integrand_oracle.json — pointwise Drell-Yan integrand"
    bash "$MG/gen_dy_oracle.sh"
  else
    vg_say "⊘ dy_integrand_oracle.json: no dy13_default work area (VG_FORCE=1 in the"
    vg_say "  madgraph stage rebuilds it), keeping the committed oracle"
  fi

  if [ -f "$VG_VALIDATION_DIR/helas/reference.csv" ]; then
    vg_say "⊘ helas/reference.{csv,npz}: committed; regenerate with"
    vg_say "  pixi run -e helas-validation generate-helas"
  else
    vg_die "helas/reference.csv is missing: pixi run -e helas-validation generate-helas"
  fi
}

# ── bundle ───────────────────────────────────────────────────────────────────

stage_bundle() {
  stage_banner "bundle"
  bash "$MG/assemble_bundle.sh"
}

# ─────────────────────────────────────────────────────────────────────────────

STAGES=("$@")
if [ ${#STAGES[@]} -eq 0 ]; then
  STAGES=(deps madgraph refs bundle)
fi

for stage in "${STAGES[@]}"; do
  case "$stage" in
    deps) stage_deps ;;
    madgraph) stage_madgraph ;;
    refs) stage_refs ;;
    bundle) stage_bundle ;;
    *) vg_die "unknown stage '$stage' (deps, madgraph, refs, bundle)" ;;
  esac
done

vg_say ""
vg_say "✓ reference generation complete: ${STAGES[*]}"
