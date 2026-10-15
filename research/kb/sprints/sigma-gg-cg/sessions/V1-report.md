---
type: Session Report
title: "V1 report: the gg_to_gg_cg offset is the reference's draw"
description: "The banked σ is MadEvent's iseed 21, reproduced bit for bit here, and it sits +2.2e-3 above MadEvent's own converged value. This crate's integrand matches MadGraph point by point at every αs to 1e-15, and its exact quadrature σ agrees with 8 fresh 10×-budget MadEvent directories at rel −6e-6. The fix is to re-bank the row under the seed policy."
status: draft
tags: [smeftsim, sigma, dynamical-scale-choice-3, madevent, seeds, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
Diagnosis only. No source change was committed. The probes were an uncommitted
test file and a Fortran driver, both kept in the scratchpad. Every MadEvent run
used the pinned 3.7.1 (`validation/madgraph/mg5_pinned.sh` under
`pixi run -e madgraph`) and came from a process directory generated here.
That directory's `matrix1_orig.f` is byte-identical to the banked one, and its
`param_card.dat` is identical too.

## Verdict

**The offset belongs to the reference. Nothing on this crate's side contributes
to it.** The banked σ = 2.71240e5 ± 229.8 pb is a single MadEvent run at
`iseed = 21` (the banked banner says `21 = iseed`). Re-running seed 21 here
reproduces the banked `SubProcesses/results.dat` line bit for bit, all 13
columns. MadEvent's own value over many seeds and budgets is 2.7065e5. That puts
the bank at +2.17e-3, which is +2.5 of its quoted errors and +2.1 seed standard
deviations: a high draw, made conspicuous by selection. This crate computes
2.706509e5, which matches the converged MadEvent value at rel −6e-6.

## Evidence chain

1. **The reference moves; this side does not.** Same banked run card, unweighted
   seed means as in the seed policy:

   | MadEvent batch | n | mean (pb) | ± | seed sd | χ²/dof | bank/mean − 1 |
   |---|---|---|---|---|---|---|
   | banked budget, seeds 11–30, one directory with grids deleted between seeds | 20 | 270654 | 61 | 274 | 1.42 | +2.17e-3 |
   | banked budget, seeds 81–90, a fresh directory per seed | 10 | 270354 | 211 | 666 | 11.9 | +3.3e-3 |
   | the same without seed 84 (see Found) | 9 | 270559 | 55 | 164 | — | +2.5e-3 |
   | `nevents = 100000` (10×), seeds 41–48, a fresh directory per seed | 8 | **270652.5** | **24.6** | 65 | 0.80 | +2.17e-3 |

   Seed 21 is one of the 20 in the first batch. Its G3 iterations show no spike
   (max/mean 1.009); they sit uniformly high, at a mean of 271129.

2. **This side's σ, with sampling removed.** A deterministic quadrature of the
   production integrand (`FixedBeamIntegrand::event_in_channel`, summed over
   all 6 channels of the multichannel map) uses composite 5-point
   Gauss–Legendre in the polar coordinate and midpoints in the azimuth:

   | panels × φ points | σ (pb) |
   |---|---|
   | 4000 × 3 | 270650.4 |
   | 8000 × 1 | 270647.5 |
   | 16000 × 2 | **270650.9** |
   | 16000 × 4 | 270650.9 |
   | 16000 × 8 | 270650.9 |

   The value is φ-independent to 10 digits, and the panel convergence is about
   1e-5. Against the 10× MadEvent mean this gives rel −6e-6 ± 9e-5, pull −0.07.
   The recorded VEGAS seed means were rel −2.21e-3 and −1.58e-3 against the
   bank, which is 2.7064e5 and 2.7081e5: the same number.

3. **The integrand matches pointwise, at every coupling.** A Fortran driver
   linked against MadGraph's own standalone output for this process
   (`UPDATE_AS_PARAM2` followed by `SMATRIX`) was compared with this crate's
   production fallback path. That path is `ScaleAwareAmplitude`, here taking the
   `ComplexEntry(0)` reference path, which re-evaluates the model. The
   comparison covered 16 angles across the full cut range (|cos θ| up to 0.9968,
   which is pT = 20 GeV) and αs ∈ {0.08, 0.10, 0.1179, 0.13, 0.15, 0.17, 0.20}.
   The ratio is 512 at every point, MadGraph's `IDEN`, with a maximum deviation
   of 2.2e-15. αs(μ) at μ = pT is already pinned per event by `validate_scales`
   (AQCDUP on all 10000 events). With both pinned, the integrand is the same
   function, so the cut region contains nothing for an offset to live in.

   MadGraph treats no "independent" parameter of this model as depending on aS.
   Its per-event block recomputes exactly aS, sqrt(aS), gHgg1–5, G² and G³.

4. **Control at a fixed scale.** With `fixed_ren_scale = True` and
   `scale = 50`, this side's quadrature gives 228795.7 pb (8000 and 16000
   panels agree to 1e-5). MadEvent over 6 fresh directories gives
   228865 ± 74 pb (χ²/dof 0.57). That is rel −3.0e-4, pull −0.94: the two
   sides agree with and without running.

## Slice table (pT = μ_R, since choice 3 on a massless 2 → 2 is exactly pT)

On this side the slices come from the same quadrature (16000 × 2). On the
MadEvent side each slice is 5 fresh directories with `ptj = lo` and
`ptjmax = hi`.

| pT slice (GeV) | this side (pb) | MadEvent mean ± err | χ²/dof | rel | MadEvent per-seed values |
|---|---|---|---|---|---|
| [20, 25) | 111371 | 111042 ± 83 | 20.6 | +2.97e-3 | 110950 110940 111010 **111370** 110940 |
| [25, 30) | 55880 | 55647 ± 99 | 123 | +4.19e-3 | 55482 55488 **55897** **55880** 55487 |
| [30, 40) | 51092 | 50853 ± 71 | 39.1 | +4.69e-3 | 50778 50787 **51134** 50808 50760 |
| [40, 50) | 21521 | 21418 ± 34 | 49.6 | +4.82e-3 | 21350 **21506** 21369 21366 **21497** |
| [50, 75) | 19168.4 | 19170.8 ± 4.2 | 0.44 | −1.2e-4 | — |
| [75, 100) | 5913.0 | 5912.9 ± 1.6 | 0.83 | +3.0e-5 | — |
| [100, 150) | 3755.9 | 3755.5 ± 0.9 | 0.55 | +9.7e-5 | — |
| [150, ∞) | 1949.9 | 1949.6 ± 0.6 | 1.12 | +1.2e-4 | — |

Above 50 GeV the slices agree at ≤ 1.2e-4 with healthy χ². Below 50 GeV the
windowed MadEvent runs are **bimodal**. Most seeds land 0.4–0.7 % low with
quoted errors of about 30 pb, and the rest land on this side's value: 111370
against 111371, and 55897/55880 against 55880. Their sum is 0.33 % below
MadEvent's own unwindowed σ. This is a reference-side coverage failure under a
`ptjmax` window (see Found). It is not a signal, and it is why the brief's
slice test cannot isolate anything below 50 GeV on the MadEvent side. The
pointwise comparison (step 3) is the finer oracle there and leaves no room.

## Sibling items

- **(a) Iteration weighting** (`dsample.f` mean²/variance). Equal-weight
  per-channel iteration sum minus reported σ: −23.6 ± 29.3 pb (rel −0.9e-4 ±
  1.1e-4) over the 20 banked-budget seeds, and −14.5 ± 8.3 pb (−5.4e-5 ±
  3.1e-5) over the 8 seeds at 10×. Both are consistent with zero at this row's
  precision, which is negligible against 2.2e-3. The fresh batch reads +2586 pb
  only because of seed 84, where the mechanism shows in heavy-tailed form: one
  G3 iteration hit 1.2121e6 ± 9.4e5 (4.1× the mean). Weighting by mean²/variance
  suppressed it, and the seed reported 268510, which is 0.8 % low.
- **(b) Lab or CM frame.** Both beams are at 250 GeV and massless, so the lab
  frame *is* the partonic CM frame and the question cannot arise for this row.
  This crate evaluates the closed-form scale on the partonic-CM momenta
  (`FixedBeamIntegrand::matrix_element_at` → `apply_scale`). Choice 3 is built
  from m_T = √((E+p_z)(E−p_z)) (`coupling/scales.rs:96-114`), which is
  longitudinally boost-invariant, so on a hadronic row the frame would not
  matter either. αs is then evaluated at that μ_R, and the couplings follow it
  through `EvaluatedModel::set_alpha_s`.

## Banking request

The row's reference should be re-banked under the seed policy
(`madevent-reference-seed-policy`): at least 10 seeds, since any gate this row
earns is tighter than 0.3 %, stored per seed, one freshly generated directory
per seed.

- Task: `gg_to_gg_cg` from `validation/madgraph/scripts/gg_to_gg_cg.mg5` under
  the pinned 3.7.1. It needs the same banked run card, changing only `iseed`.
  `sigma_reference.json` holds one `(sigma_pb, sigma_err_pb)` per fixed-beam row
  today, so this needs either a seeded fixed-beam generator like
  `gen_grammar_sigma.sh`, with its reader, or the manager's choice of format.
- Seeds: 10 fresh directories at `nevents = 10000` (about 15 s each here), or
  for a tighter reference 8–10 at `nevents = 100000` (about 2 min each).
  Keep `run_01` at iseed 21 as the samples-cell event file. That cell is
  normalised and unaffected.
- Expected: an unweighted mean of 2.7065e5 pb, ±≈60 (10 seeds at 1×) or ±≈25
  (8 seeds at 10×). One seed in about ten may land about 0.8 % low the way seed
  84 did. The unweighted mean with `max(quoted, spread/√n)` absorbs it, but a
  reader should know it happens.
- Afterwards the `integrals` cell can be read against the new mean. Past
  measurements give this side's VEGAS budget ladder a −1.2e-3 to −2.9e-3
  scatter against the *old* bank, which is +1.0e-3 to −0.7e-3 against the new
  value. The gate's `rel_tol` and budget remain the manager's call, under
  `gate-thresholds`.

## What would falsify this

- A re-banked mean that differs from 2.7065e5 by much more than its error. That
  would mean this host's MadEvent and the M3 host's differ, which seed 21's
  bit-equal reproduction argues against.
- A point where the standalone |M|² ratio leaves 512. The tested set covers the
  cut edge to the centre at 7 couplings. A narrow feature between grid points is
  implausible for a smooth 2 → 2 amplitude, but it was not excluded by a dense
  scan.
- A quadrature panel-convergence failure larger than 1e-4. The measured spread
  is 1e-5.

## Brief corrections

- The brief's premise is wrong: the residual is not "the coupling this matrix
  element runs at across the cut region". The coupling's αs dependence matches
  to 1e-15, and the row's σ matches converged MadEvent at 1e-5. Earlier sessions
  measured this side well but compared it against one reference draw.
- Slicing σ in the scale variable on the MadEvent side through `ptj`/`ptjmax`
  windows is not a usable oracle below 50 GeV for this process (bimodal, see
  above).
- The manifest's `integrals` note and the backlog item quote different headline
  numbers: rel −1.22e-3 single seed and −1.58e-3 seed mean in the manifest,
  −2.21e-3 inverse-variance in the backlog. Both are against the same draw and
  both are superseded by this report.

## Found

- **Windowed MadEvent runs are bimodal at low pT.** Under `ptj`/`ptjmax`
  windows below 50 GeV, 3 or 4 of 5 seeds per slice land 0.4–0.7 % low with quoted
  errors 10–20× smaller than the gap (χ²/dof 20–123). Any banked or diagnostic
  MadEvent reference built from narrow pT windows (`pta_window_reference.json`,
  `higgs_window_reference.json` and similar) should be checked for the same mode
  structure. Closing it means fresh-directory seed sweeps of one windowed row,
  read for bimodality.
- **One fresh MadEvent seed in ten came out 0.8 % low** (iseed 84, 268510 ±
  196) after a single 4×-mean iteration spike was suppressed by the
  mean²/variance weighting. This is the heavy-tail version of sibling (a)'s
  bias. Closing it means a frequency estimate over more seeds, filed as a
  caution in the seed-policy concept.
- **Every single-run fixed-beam row in `sigma_reference.json` is one draw.**
  This row is the one whose draw was 2σ out. MadEvent's measured seed spread
  here is 1.2× its quoted error at 20 seeds (2.9× in the batch that contains
  seed 84). Closing it means extending the seeded banking to the other gated
  fixed-beam rows, or at least to those whose gate is tighter than about 3×
  their quoted error.
