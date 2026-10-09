---
type: Physics Convention
title: "Where α_s comes from: MadGraph's RGE or the PDF set's grid"
description: "MadGraph's own ALPHAS/NEWTON1 running with asmz from the PDF label, or at pdlabel = lhapdf LHAPDF's AlphaS_Ipol on the set's knots; AlphaSSource picks; α_EW never runs."
status: draft
tags: [alpha-s, pdf, lhapdf, coupling, madgraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n22-11, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L36-L59", title: "Note 22 §1.1 (αs is MadGraph's own RGE)"}
  - {id: n22-14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L127-L156", title: "Note 22 §1.4 (AQCDUP as an oracle; α_EW constant)"}
  - {id: n22-4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L217-L255", title: "Note 22 §4 (the aS(M_Z) override risk)"}
  - {id: n22-close, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/22-dynamical-scales-plan.md#L283-L338", title: "Note 22 close-out (RunningAlphaS bit-exact; gated on AQCDUP)"}
  - {id: n36-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L458-L524", title: "Note 36 §4 B6 item 3 (RunningAlphaS below 0.5 GeV)"}
  - {id: n24-pc1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L469-L517", title: "Note 24 plan correction 1 (pdlabel = lhapdf puts αs outside the coupling layer)"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L943-L1010", title: "Note 24 P2 decisions (αs from the grid)"}
  - {id: n24-p2b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L1093-L1156", title: "Note 24 P2b §2 (GridAlphaS and AlphaSSource)"}
  - {id: n28-k5a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2765-L2878", title: "Note 28 §K5a.1–K5a.4 (AlphaS_Ipol, the 20 000 events, the ceiling, refusals)"}
  - {id: mg-alfas, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/alfas_functions.f#L74-L210", title: "MadGraph 3.7.1 alfas_functions.f (ALPHAS, NEWTON1)"}
  - {id: mg-alfas-lhapdf, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/alfas_functions_lhapdf.f#L74-L83", title: "MadGraph 3.7.1 alfas_functions_lhapdf.f (ALPHAS = alphasPDF)"}
  - {id: mg-setrun, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/setrun.f#L130-L145", title: "MadGraph 3.7.1 setrun.f (asmz source)"}
---
# Where α_s comes from: MadGraph's RGE or the PDF set's grid

MadGraph evaluates the strong coupling at `μR` per event and recomputes every
`aS`-dependent coupling from it (`G = √(4π·αs(μR))`, `update_as_param`). Which
routine computes `αs(μR)` is decided at link time by `pdlabel`, and vibegraph's
`coupling::alphas::AlphaSSource` makes the same decision from the same field:

| `AlphaSSource` arm | MadGraph links | when |
|---|---|---|
| `Running(RunningAlphaS)` | `Source/alfas_functions.f`: `ALPHAS`/`NEWTON1` | any `pdlabel` other than `lhapdf`, and every `lpp = 0` run |
| `Grid(GridAlphaS)` | `Source/alfas_functions_lhapdf.f`: `ALPHAS = alphasPDF(Q)` | `pdlabel = lhapdf` |

`α_EW` does not run in MadGraph's LO path (`AQEDUP = 7.5467710e-3 = 1/132.507`
on every event of every run), so only QCD couplings move per event. How the
per-event coupling reaches the amplitude pools is
[per-event-coupling-rescale](per-event-coupling-rescale.md); where `μR` comes
from is [madgraph-scale-choice](madgraph-scale-choice.md).

## MadGraph's own running (`RunningAlphaS`)

`ALPHAS(Q)` evolves `asmz` from `M_Z` by inverting the integrated `nloop`-order
β function with Newton's method (`NEWTON1`), stopping on a **relative** step
below `TOL = 5d-4`
([`alfas_functions.f:172`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/alfas_functions.f#L172)).
The result is a specific iterate, not the exact root, so reproducing MadGraph
means reproducing the iteration. Thresholds are fixed constants, not the
model's masses: `CMASS = 1.42`, `BMASS = 4.7`, `ZMASS = 91.188`
(`:98-103`), with `nf = 5 → 4 → 3`. The β coefficients are the Fortran `DATA`
literals, not recomputed. `coupling/alphas.rs` ports it bit-exactly against
MadGraph's own Fortran on a 792-point grid at `nloop` 1–3
(`alphas_reference_grid.rs`).[^n22-close]

**Where `asmz` and `nloop` come from**
([`setrun.f:130-145`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/setrun.f#L130-L145)):

```fortran
      if(lpp(1).ne.0.or.lpp(2).ne.0) then
          write(*,*) 'A PDF is used, so alpha_s(MZ) is going to be modified'
          call setpara(param_card_name)
          asmz=G**2/(16d0*atan(1d0))
          write(*,*) 'Old value of alpha_s from param_card: ',asmz
          call pdfwrap
          write(*,*) 'New value of alpha_s from PDF ',pdlabel,':',asmz
      else
          ...
          nloop=2
          pdlabel='none'
```

| beams | `asmz` | `nloop` |
|---|---|---|
| `lpp = 0` | parameter card `aS` (via `G`) | 2 |
| any `lpp ≠ 0` | **the PDF label's tabulated value** (`pdfwrap.f`), e.g. `nn23lo1 → 0.130` | 2 unless the label overrides |

With a PDF, MadGraph **overrides the parameter card's `aS(M_Z)`**. A hadronic
QCD comparison that used the card's `0.118` would be wrong by ~10 % per power of
`αs` in σ before any running. A label `pdfwrap.f` does not know falls back to `0.118` in MadGraph;
vibegraph refuses it (`AlphaSError::UnknownPdLabel`).[^n22-11]

**Below the perturbative solve.** Once `Q` approaches the `nf = 3` divergence the
Newton seed stops being positive: below about `0.40` GeV at two loops and `0.51`
at three for `αs(M_Z) = 0.118` (`0.66` and `0.83` at `0.130`), the iteration
returns NaN or a negative coupling. MadGraph marks part of that region with a
`9d98` sentinel that nothing downstream tests, and otherwise runs on.
`RunningAlphaS::eval` refuses a non-positive or non-finite result instead.[^n36-b6]

## The grid's own running (`GridAlphaS`)

At `pdlabel = lhapdf` MadGraph's `ALPHAS` is a one-line forward:

```fortran
      ALPHAS=alphasPDF(Q)
```

([`alfas_functions_lhapdf.f:83`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/alfas_functions_lhapdf.f#L83)),
which LHAPDF resolves as `PDF::alphasQ(Q)` → `alphasQ2(q*q)` on the set's
`AlphaS` object; for `AlphaS_Type: ipol` that is `AlphaS_Ipol`. A PDF set is
fitted at its own coupling, so its densities and that coupling are one object.
`RunningAlphaS::from_run_card` refuses such a card (`AlphaSError::LhapdfRunning`)
and `AlphaSSource::from_run_card` routes exactly that refusal to the grid, so
the source rule is stated once. Without the set's `AlphaS_*` metadata the result
is `GridUnavailable`, not a fall back to the β-function solve, which would run
the set's densities against a coupling it was not fitted with. The integrand
therefore needs the whole `PdfSet` (the tabulation lives in `set.info`), not
just a member.[^n24-p2b]

`pdf/alphas.rs` reproduces `AlphaS_Ipol::alphasQ2`, read from LHAPDF 6.5.3's
sources; MadGraph's environment ships 6.5.6, and the probe agreement below is
what says they are the same routine:[^n28-k5a]

| element | behaviour | LHAPDF source |
|---|---|---|
| variable | `ln Q²`, natural log | `AlphaSArray::_syncq2s` |
| interpolant | cubic Hermite, `2t³ − 3t² + 1` basis | `AlphaS_Ipol::_interpolateCubic` |
| slopes | central inside a subgrid, forward at its first knot, backward at its last | `AlphaSArray::ddlogq_*` |
| subgrids | cut at every repeated `Q²`, keyed by first `Q²` so a later piece with the same key replaces an earlier one | `AlphaS_Ipol::_setup_grids` |
| above the last knot | **frozen** at the last value: `if (q2 > _q2s.back()) return _as.back();` | `alphasQ2` |
| below the first knot | power law in `Q²`, exponent = the first interval's slope in `log₁₀`–`log₁₀` | `alphasQ2` |
| reading with \|αs\| ≥ 2 | replaced by `DBL_MAX` (`f64::MAX`) | `_interpolateCubic` |

Refused, because LHAPDF's own reading is undefined there: a scale that is not
positive and finite; a subgrid a query can select with fewer than three knots
(the central slope reads one knot beyond the interval); a non-`ipol`
`AlphaS_Type` (the knots are not the source); a non-positive tabulated value
(the below-table power law takes its `log₁₀`). A one-knot piece from a repeated
*first* knot is not refused: it is shadowed by the next piece and never
interpolated on (`a_leading_repeated_scale_is_shadowed_rather_than_refused`). A
scale outside the table is **not** refused: both continuations are part of the
algorithm.

## What pins the grid reading

| oracle | measurement | test |
|---|---|---|
| run log's 17-digit `New value of alpha_s from PDF lhapdf : 0.13000271085472234`, against the grid read at `ZMASS = 91.188` | residual `0`; `GRID_ALPHA_S_TOL = 1e-14` (two orders above one `ln` call's noise, room for a different system `libm`) | `banked_run_logs_pin_the_alpha_s_source_rule` (`tests/validate_alphas.rs`) |
| per-event `AQCDUP` on the dynamical grid runs | `pp_to_llj_dyn`: 0 events outside budget, worst 0.999; `pp_to_jj`: 0 outside, worst 0.996. A straight line through the same knots puts 9976 and 9993 of 10 000 events outside, worst 1777× and 1076× | `banked_events_reproduce_aqcdup` |
| LHAPDF's own `alphasQ`, through `validation/pdf/gen_oracle.cpp` | 414 probes over NNPDF23 and NNPDF31 (every knot, `t = ¼, ½, ¾` of every interval, NNPDF31's repeated `Q = 4.92`, four above `q_max`, four below `q_min`): `0.00e0` relative in every category | `alpha_s_matches_lhapdf_across_the_table_and_past_both_ends` (`tests/validate_pdf_grid.rs`) |
| LHAPDF up to `Q = 10⁷` | the last tabulated value to the bit, both sets | `above_the_alpha_s_table_lhapdf_freezes_rather_than_extrapolates` |
| hermetic | a quadratic in `ln Q²` on the knots is exact only where both slopes are central, which separates a linear reading, a reading in `Q`, and central slopes carried into the edges | `a_quadratic_in_log_q2_is_exact_inside_and_only_inside` (`pdf/alphas.rs`) |

The run log is the only oracle at 17 digits and only at `M_Z`, where `91.188`
sits `2.4e-5` into its knot interval, so a straight line would also land within
`1e-8` there: the log pins the **source**, and the dynamical-scale events pin
the **shape**. The log also asserts the grid value differs from the parameter
card's by more than half a printed `AQCDUP` digit (`0.1300027` against
`0.1300028`), so a silent revert to the card fails every grid-sourced event.
For a non-`lhapdf` label the events cannot see the override at all: MadGraph's
tooling has already written the PDF's `aS` into the parameter card, so only the
log's "Old value"/"New value" lines pin it. `GRID_ALPHA_S_RUNS` in
`validate_alphas.rs` lists the grid-sourced runs and is asserted both ways, so a
run changing source fails rather than being reclassified.[^n24-pc1]

**Neither end of the table is reachable from the bank.** The largest banked
`SCALUP` is 845.5 GeV (`pp_to_llj_dyn`) and 167.2 (`pp_to_jj`) against a table
running to 10 000 GeV, and the smallest `AQCDUP` is 0.0976 against the top
knot's 0.07695485. The integrator does sample above the table (an integration
reached `Q = 10 647` GeV), so the continuations are pinned only by the LHAPDF
probes. NNPDF23 freezes at 0.07695485 from 10 TeV up, which is what makes a
13 TeV dynamical scale evaluable:
`a_dynamical_scale_resolves_where_the_table_stops_below_the_collider`
(`proton.rs`) compiles a dynamical card over a 10 TeV table on a 13 TeV collider
and checks the frozen value is returned.

## Which integrand builds which source

- **Fixed beams** (`lpp = 0`): `setrun.f` overwrites `pdlabel` with `none`, so
  the source is always the parameter card's `aS` run at two loops. It is built
  whenever the model declares `aS`, even when the matrix element carries no
  strong coupling, because the event record reports `αs(μR)` on every run that
  has a strong coupling to report.
- **Hadron beams**: the set's tabulation wherever `pdlabel = lhapdf`, otherwise
  the running solve with the label's `asmz`. The pinned set and its label are
  [pinned-pdf-set](pinned-pdf-set.md); the grid interpolator for the densities
  themselves is [lhapdf-interpolation](lhapdf-interpolation.md).

The per-event gates are described in
[scale-replay-gate](../validation/scale-replay-gate.md).

[^n22-11]: Note 22 §1.1; the trap is note 22 §4's first risk.
[^n22-close]: Note 22 close-out, D1.
[^n36-b6]: Note 36 §4 B6, landed item (3): `alfas_functions.f` neither clamps nor stops.
[^n24-p2b]: Note 24 P2b §2; the refusal list and continuations are as revised in note 28 §K5a.4.
[^n28-k5a]: Note 28 §K5a.1–K5a.3.
[^n24-pc1]: Note 24 plan correction 1 and P2b §2, with the tolerance as tightened in note 28 §K5a.2.
