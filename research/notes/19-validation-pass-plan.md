# 19 — `validation-2` Sprint Plan

**Date**: 2026-07-19. **Position in the loop**: validation pass following the
`hadronic-xsec` feature sprint (note 18). **Goal**: clear the unblocked
validation backlog accumulated across the eval performance program (note 15),
`hadronic-xsec`, and the `validation-sprint` leftovers — plus new σ-level
integration coverage for every `MG_VALIDATED_PROCESSES` entry.

## 1. Scope

**In**: pruned-frame contract guard, interned-SM CI check, NHEL-table pinning
14/14, `vibegraph integrate` generalization (absorbs `cli-proc-card`),
14-process σ gate through the CLI, multi-subgrid PDF seam coverage,
`rooting-soundness` spike, branch-level rooted-tree pattern assertions,
per-flavor diagram matching.

**Out (blocked, stays in `TODO.md`)**: flow→LHEF color-string dictionary and
`mg-single-helicity-bench` (both ride with `event-output-lhef`); `IdentityAmp`
process coverage (needs a non-SM model, rides with `non-sm-ufo`);
`Coeff(f64)`→`CoeffRat` rationalization (optional cleanup, no consumer).

## 2. Key survey findings (2026-07-19)

These reshaped the plan and are worth recording:

- **Every `validation/madgraph/scripts/*.mg5` already carries a `launch`
  block**, and all 19 `output/` dirs contain completed runs (`Events/run_01`
  banner, `results.dat`, σ ± err in `build.log`). The σ references for the
  σ-gate session are **already banked on disk** — no new MG generation rig is
  needed (`gen_hadronic_sigma.sh` stays specific to the dy13 PDF-convolved
  runs). The exact run cards MG used are in `output/*/Cards/run_card.dat`.
- **Beam conventions of the existing runs**: the partonic/colored/2→6 scripts
  launch with `lpp=0` fixed-energy beams (√ŝ = 500 for the 250+250 ones,
  Z-pole for `ee_to_mumu`), so their σ is the **partonic σ̂** — directly
  comparable to a vibegraph integral with no PDF. Only the `pp_*` scripts use
  proton beams.
- **Cross-check**: `ee_to_mumu`'s run σ = 2025 ± 1.116 pb matches the
  `MG5_SIGMA_PB` already pinned in `validate_vegas.rs`.
- **Coverage gap**: `u u~ > mu+ mu-` has no dedicated `lpp=0` script — its
  only σ reference is the PDF-convolved `pp_to_ll`, which ran with MG's
  *internal* `nn23lo1` pdlabel (not an LHAPDF grid, so not comparable to our
  `pdf/` loader without a re-launch). V3b adds a trivial `uux_to_mumu.mg5`
  (`lpp=0`); the PDF-convolved path is already gated end-to-end by
  `validate_hadronic` (dy13, 0.14 % / 0.07 %, note 18).
- **Profiling substrate**: `Cargo.toml` already defines
  `[profile.profiling]` (`inherits = "release"`, `debug = 1`).

### Banked MG run σ values (from `build.log`, for V3b's extractor to formalize)

| process | σ ± err (pb) | beams |
|---|---|---|
| `ee_to_mumu` | 2025 ± 1.116 | e⁺e⁻ 45.6+45.6 |
| `ee_to_ee` | 155.8 ± 0.2096 | lpp=0, 250+250 |
| `ee_to_mumua` | 0.1006 ± 3.865e-4 | lpp=0, 250+250 |
| `ee_to_ttx` | 0.5486 ± 3.292e-4 | lpp=0, 250+250 |
| `ee_to_wpwm` | 7.155 ± 0.02169 | lpp=0, 250+250 |
| `ee_to_zh` | 0.05723 ± 2.068e-5 | lpp=0, 250+250 |
| `ee_to_tatah` | 0.001884 ± 2.405e-6 | lpp=0, 250+250 |
| `ee_to_mumu_tata_qcd0` | 0.001337 ± 2.804e-6 | lpp=0, 250+250 |
| `uux_to_ccx_emmm_qcd0` | 6.556e-7 ± 1.611e-9 | lpp=0, 250+250 |
| `bbx_to_ccx_emmm_qcd0` | 1.605e-6 ± 4.84e-9 | lpp=0, 250+250 |
| `uux_to_uux` | 3.343e+4 ± 52.11 | lpp=0, 250+250 |
| `gg_to_ttx` | 15.95 ± 0.03742 | lpp=0, 250+250 |
| `gg_to_gg` | 1.688e+5 ± 252.4 | lpp=0, 250+250 |
| `uux_to_mumu` | *(to add, V3b)* | lpp=0 |

## 3. Sessions

### V1 — Quick guards (light; Sonnet-able)

- **Pruned-frame contract guard**: nothing enforces the partonic-CM
  ±z-beam requirement of a pruned evaluator — a boosted input silently
  revives J_z-forbidden combinations (up to 3e-3 of the sum, note 15 §2.3).
  Add a debug-build frame assertion (or an explicit boost-to-CM seam) plus a
  test that a boosted point on a pruned evaluator is caught.
- **Interned-SM CI check**: `gen_sm_blob` + `git diff --exit-code` job to
  catch a stale interned SM blob vs the pinned submodule.
- The `validate_helas_mg` timing print is **not** touched here: it currently
  doubles as the samply profiling target, so its retirement rides with V3b,
  which lands the replacement.

### V2 — NHEL-table pinning 14/14

Extend `prune_zero_helicities_matches_madgraph_filter_bitwise`
(`helas/eval/run.rs`) from 7 pinned processes to all 14
`MG_VALIDATED_PROCESSES`, including in-test pinning of the 2→6 survivor
counts (16/256 uux, 32/256 bbx — previously only eyeball-checked against MG
reports). Reference `NHEL` tables come from the existing per-process MG
output dirs.

### V3a — Generalize `vibegraph integrate` (feature-shaped; feature-dev agent)

Absorbs the `cli-proc-card` backlog item. Three hard-codings to remove:

1. **Process assembly**: `hadronic.rs` hard-codes the `p p > e+ e-`
   flavor-class assembly → proc-card-driven subprocess/PDF assembly through
   the existing `GlobalConfig::load_ufo` seam.
2. **Beams**: `runcard.rs` rejects `lpp ≠ (1,1)` → add `lpp=(0,0)`
   fixed-energy no-PDF beams (the mode 13 of the 14 reference runs use).
3. **Phase space**: the (τ,y)×2-body map is 2→2-only → add a
   flat-RAMBO-uniforms-under-VEGAS path for 2→3/2→4/2→6 final states (H3
   substrate: massive RAMBO + splittable substreams). Channel mappings stay
   `lips-nbody` scope; expect slow convergence on peaked integrands — the
   V3b gate is statistical, with precision set by what flat sampling affords.

### V3b — 14-process σ gate through the CLI

- **Extractor** (à la `extract_diagrams.py`): parse σ ± err per process from
  the existing run output (`results.dat` / run banner) into a banked JSON.
- **Run card as single source of truth**: drive `vibegraph integrate` with
  the exact `output/<proc>/Cards/run_card.dat` MG used, so beams *and* cuts
  are pinned identically on both sides by construction. The default cuts in
  those cards screen the divergent processes (Bhabha t-channel, ee→μμγ
  soft/collinear photon, the massless colored 2→2s, the 2→6 γ*→ℓℓ mass
  singularity); `cuts.rs` hard-errors on any active unimplemented cut, so
  gaps surface loudly.
- **Statistical gate**: pull = (σ_vg − σ_MG)/√(err_vg² + err_MG²), |pull| ≲ 3
  plus a relative-tolerance backstop. σ-agreement is a weak oracle (blind to
  mis-sampled regions of small measure) — the bit-exact `validate_helas_mg`
  net remains the fine instrument; this gate covers what that net *cannot*
  see: flux, spin/color averaging, symmetry factors, cuts, beam/PDF handling.
- **Close the uux→μμ gap**: add `uux_to_mumu.mg5` (`lpp=0`) and run it once.
- **Retire (or relabel) the `validate_helas_mg` timing print** and document
  the profiling recipe that replaces its samply role: build the σ-gate test
  under `--profile profiling` and point `samply record` at it. Per-process
  time is then weighted by how hard each process is to *integrate*, so
  hotspots rank in a tackle-worthy order (unlike the old fixed-N loop).

### V4 — Multi-subgrid PDF seam (free-floating)

The LHAPDF oracle only covers the single-subgrid NNPDF23_lo_as_0130_qed set;
the subgrid-walk and two-Q²-knot bilinear fallback are pinned only by
synthetic fixtures. Pick a real multi-Q²-subgrid LHAPDF set, extend
`validation/pdf/gen_oracle.cpp` to describe LHAPDF's seam-derivative-
flattening behavior instead of hard-erroring on repeated Q² knots, and pin
the walk + fallback against it.

### V5 — `rooting-soundness` spike

Per note 15 §3 + `rooting-study-results.md`: the amplitude is correct only
for feyngraph's `VtxIdx(0)` edge orientation — every node-reducing rooting
silently corrupts multi-boson/≥6-point amplitudes (max_rel up to 1.7e+3).
Make momentum routing, Lorentz-output rooting, and fermion-spine sign
root-invariant. **First deliverable is the failing test**: all V rootings of
every `MG_VALIDATED_PROCESSES` diagram pass the gate, via the
`set_root_override` hook already on `explore/rooting`. This blocks any
production rooting change and the Track-3 re-rooting rule family; the perf
prize (−21 % nodes / −34 % slot traffic) is secondary to the correctness fix.

### V6 — Branch-level coverage (after V5; same code territory)

Rooted-tree pattern assertions per MG-pinned convention: every "pinned by
process X" claim gets a test that fails if X stops exercising that branch —
an unexercised branch silently drifting out of sync with its exercised
sibling is exactly the failure mode that produced the `gg_to_gg` VVVV bug
(note 16 §6).

### V7 — Per-flavor diagram matching (optional tail)

`madgraph-diagram-cmp-per-flavor`, design moved here from `TODO.md`. An
independent, verification-heavy refactor (Python extractor + Rust matching +
JSON regen). Background: the `validate_madgraph_diagrams` reference count
uses the representative subprocess's true `NGRAPHS` from `matrix1_orig.f`,
not `MAPCONFIG(0)` (which counts the integration-channel *union* across a
P-class — e.g. 2672 vs the actual 2316 for `u u~ > u u~ l+ l- l+ l-`).
**Remaining gap**: `count_mg_style_topologies`
(`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) collapses vibegraph
subprocesses into coarse particle-type classes and compares one
representative per class against the summed `total_diagrams` — fragile,
since it assumes vibegraph's first-enumerated subprocess matches MadGraph's
`matrix1` representative.

Design for the refinement (validates *all* variants, incl. the 40 of the
qq4l class):

- **Robust flavor source — the matrix-file header, not `IDUP`.** Each
  `SubProcesses/P*/matrix<N>_orig.f` carries
  `C     Process: u u~ > u u~ e+ e- e+ e- QCD=0 @1` comment lines — one per
  concrete flavor process sharing that variant's `NGRAPHS` (u/c and e/mu are
  grouped). Parse these directly: it avoids reverse-engineering MG's fragile
  `matrix<N> ↔ IDUP(I,J,K)` 3-index mapping in `leshouche.inc`.
  `extract_diagrams.py` grows a per-concrete-process
  `{in:[pdg…], out:[pdg…], ngraphs}` list (name→PDG via a bounded SM dict:
  the full token set is `a b b~ c c~ d d~ e± g h mu± s s~ t t~ ta± u u~ w± z`).
- **Rust side**: key each MG entry and each vibegraph subprocess by
  `(sorted initial PDGs, sorted final PDGs)`; compare per-subprocess
  (`set.diagrams.len()` vs `ngraphs`).
- **Known risk to resolve first**: this exposes whether vibegraph enumerates
  the *same set* of concrete subprocesses as MG's `C Process:` union — i.e.
  whether the multiparticle `p`/`l` definitions and flavor-symmetry pruning
  align. Validate on a small process (`pp_to_ll`) before the qq4l class; a
  set mismatch is a real finding, not a test bug, and needs physics judgment
  (note-12 territory: MG-convention reconciliation is a bug magnet).

## 4. Sequencing & mechanics

- **Order**: V1 → V2 → V3a → V3b; V4 free-floating (independent of
  everything); V5 → V6; V7 last, dropped to the next pass if the sprint runs
  long. V5 is cleanly separable into its own spike if needed.
- **Branch**: `validation-2`, sessions land as one commit(-set) each, merged
  to `main` at close-out.
- **Agents**: one session per agent — `validation-dev` for V1/V2/V3b/V4/V5/
  V6/V7, `feature-dev` for V3a; light sessions (V1) may run Sonnet.
  Worktrees managed manually with cd-verification.
- **Gate**: everything stays behind the 14-process `validate_helas_mg` net;
  V3b adds the σ-level net on top.
