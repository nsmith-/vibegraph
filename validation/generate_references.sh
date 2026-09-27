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
#   seeds     the MadEvent references committed as one run per seed (widths,
#             decay-chain and `$` cross sections, the decay-chain and `@N` event
#             summaries, the `>`/`$$`/polarized cross sections). Their process
#             directories and runs live in validation/madgraph/work/, outside the
#             bundle, cached the same way: an existing directory is never
#             regenerated and a finished seed is read back
#             (madevent_seeds.sh); the committed JSON is rewritten from them.
#   refs      every committed reference, recomputed from the work area. These
#             are cheap and pure functions of it, so they always rerun: that is
#             what makes a reference that changed show up as a diff. The
#             census dumps and the standalone tables run MadGraph's Python or
#             its standalone output, which is minutes, not the hours of a run.
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

# ── seeds ────────────────────────────────────────────────────────────────────

stage_seeds() {
  stage_banner "seeds (cached: existing process directories and finished seeds are reused)"
  # Each generator takes RESULT_JSON as an override of its own output; one set
  # for the whole stage would send every table to the same file.
  unset RESULT_JSON

  vg_say ">>> decay_width_reference.json — 1 -> n partial widths, ten seeds per row"
  bash "$MG/gen_decay_widths.sh"

  vg_say ">>> decay_chain_sigma_reference.json — decay-chain cross sections"
  bash "$MG/gen_decay_chain_sigma.sh"

  vg_say ">>> decay_chain_events_reference.json — decay-chain and @N event summaries"
  bash "$MG/gen_decay_chain_events.sh"

  vg_say ">>> onshell_veto_reference.json — \$ A cross sections (one row on the patched tree)"
  bash "$MG/gen_onshell_veto.sh"

  vg_say ">>> grammar_sigma_reference.json — > A, \$\$ A and polarized cross sections"
  bash "$MG/gen_grammar_sigma.sh"
}

# ── refs ─────────────────────────────────────────────────────────────────────

stage_refs() {
  stage_banner "refs"

  vg_say ">>> diagrams.json — per-process diagram counts"
  python "$MG/extract_diagrams.py"

  vg_say ">>> interactions.json — MadGraph's post-restriction interaction counts"
  python "$MG/extract_interactions.py"

  vg_say ">>> configs.json — MadGraph's colour-flow and integration-configuration counts"
  python "$MG/extract_configs.py"

  vg_say ">>> f2py matrix-element modules (cached per process)"
  bash "$MG/build_amplitude.sh"

  vg_say ">>> fixed-grid amplitude CSVs"
  python "$MG/gen_amplitude.py"

  vg_say ">>> amplitudes/<process>.json — |M|^2, AMP() and JAMP() at events + grid"
  python "$MG/gen_amplitude_tables.py"

  vg_say ">>> couplings/<process>.json — the model couplings MadGraph's Python and its Fortran hold"
  python "$MG/gen_couplings.py"

  vg_say ">>> sigma_reference.json — banked fixed-energy cross sections"
  python "$MG/extract_sigma.py"

  vg_say ">>> runcard_defaults.json — MadGraph's own RunCardLO defaults"
  python "$MG/dump_runcard_defaults.py"

  vg_say ">>> proc_grammar.json — MadGraph's own parser over the proc-card corpus"
  python "$MG/dump_proc_grammar.py"

  vg_say ">>> schannel_census.json — MadGraph's generation over the >, \$\$ and five-flavour cards"
  python "$MG/dump_schannel_census.py"

  vg_say ">>> decay_chain_census.json — MadGraph's combined decay-chain matrix elements"
  python "$MG/dump_decay_chain_census.py"

  vg_say ">>> polarization_census.json — MadGraph's NHEL tables and IDEN for polarized legs"
  python "$MG/dump_polarization_census.py"

  vg_say ">>> sm_decay_widths.json — the SM UFO's analytic two-body widths"
  python "$MG/dump_sm_decay_widths.py"

  vg_say ">>> decay_width_exact.json — h > e+ e- mu+ mu- by quadrature"
  python "$MG/decay_semianalytic.py"

  vg_say ">>> decay_amplitudes.json — standalone |M|^2 of the 1 -> n decays"
  python "$MG/gen_decay_amplitudes.py"

  # A SMEFTsim row imports the model build.sh stages under output/models.
  if [ -d "$MG/output/models" ]; then
    vg_say ">>> standalone/*.json — standalone per-helicity, per-flow JAMPs (cached per key)"
    python "$MG/gen_standalone_jamps.py" all
  else
    vg_die "standalone/*.json needs the models the madgraph stage stages under output/models"
  fi

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
  STAGES=(deps madgraph seeds refs bundle)
fi

for stage in "${STAGES[@]}"; do
  case "$stage" in
    deps) stage_deps ;;
    madgraph) stage_madgraph ;;
    seeds) stage_seeds ;;
    refs) stage_refs ;;
    bundle) stage_bundle ;;
    *) vg_die "unknown stage '$stage' (deps, madgraph, seeds, refs, bundle)" ;;
  esac
done

vg_say ""
vg_say "✓ reference generation complete: ${STAGES[*]}"
