---
type: Validation Methodology
title: Auditing an integral with windowed re-integrations
description: "A partition of one integration cannot audit it; independently re-surveyed windows on both sides plus a closure test can. Window rules, estimators, statistics, blind spots."
status: draft
tags: [validation, cross-section, madgraph, windows, attribution]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n27-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L44-L211", title: "Note 27 B1 (the h → ττ pole bin: windowed σ on both sides)"}
  - {id: n29-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2921-L2974", title: "Note 29 D.2 (the windows and the rule that fixes them)"}
  - {id: n29-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2975-L3040", title: "Note 29 D.3 (the four estimators)"}
  - {id: n29-d4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3041-L3057", title: "Note 29 D.4 (seed and budget protocol)"}
  - {id: n29-d7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3139-L3231", title: "Note 29 D.7–D.8 (gates afterwards; risks and blind spots)"}
  - {id: n29-addenda, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3771-L3844", title: "Note 29 chain D addenda (error conventions, provenance, ruling)"}
  - {id: vsigma, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_sigma.rs#L158-L199", title: "validate_sigma.rs PULL_REPORTED_NOT_ASSERTED (the adjudicated ee_to_mumua offset)"}
  - {id: mg-genps, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1949-L1951", title: "MadGraph genps.f get_channel_cut (3.7.1 form)"}
---

When our σ and MadGraph's disagree by a fixed amount, the question is which side
mis-covers which region. The tool is a windowed re-integration on both sides,
with a closure test. It was used twice, and both times the reference was the
side in error: the `h → ττ` pole in `ee_to_mumu_tata_qcd0` and the `pt(a)`
offset of `ee_to_mumua`.[^n27-b1][^vsigma]

## The core asymmetry

**A partition of one integration cannot audit that integration**: its windows
sum to the total by construction. Only an *independently re-surveyed* windowed
integral can, so each side gets both kinds, and the closure test (sum of
independent windows against the unwindowed control) is the oracle.[^n29-d3]

## The four estimators

| estimator | how | closure carries information? | error |
|---|---|---|---|
| `MG-part(w)` | window imposed through MadGraph's `dummy_cuts` (`SubProcesses/dummy_fct.f`), which `passcuts` applies after every other cut and which leaves the phase-space generator untouched, so the run integrates the same integrand restricted to `w` | **yes**: `Σ_w MG-part(w)` vs the unwindowed control | MadEvent's quoted error per run |
| `MG-cut(w)` | window as run-card cuts (`pta = lo`, `ptamax = hi`); `setcuts.f` feeds `ptamax` to the generator, which re-optimises for the window | no: a better estimate of the true windowed σ, which is what disqualifies it from closure | quoted |
| `VG-part(w)` | indicator accumulators on our production sampler at the gate's own configuration (`w·1[x ∈ w]` and `w²` per window over the same draws) | no: trivially exact; it gives our *shape* at the gated configuration | same-draw variance |
| `VG-cut(w)` | an independent integration per window with a run card carrying the window; channel maps and VEGAS grids re-adapt | **yes**: `Σ_w VG-cut(w)` vs our unwindowed total | quadrature |

Our closure is **structurally weaker** than MadGraph's: `VG-cut` re-adapts grids
but reuses the same channel construction and map code as the unwindowed run, so
a defect in that shared code can survive in both. MadGraph's windowed runs
re-survey with a genuinely different channel allocation. The asymmetry favours
the reference, which is the safe direction, but it must be stated in any
verdict.[^n29-d7] How the `dummy_cuts` window is patched in is
[MadGraph custom cuts](../tooling/madgraph-custom-cuts.md); the patch must be
asserted to land on the pinned version's body and the windowed leg asserted
against `leshouche.inc` (e.g. the photon as external leg 5 from
`DATA (IDUP(I,1,1),I=1,5)/-11,11,-13,13,22/`), never assumed.[^n29-d3][^n29-d7]
MadEvent events have been seen outside a `dummy_cuts` window in another study
([dummy-cuts window leakage](../backlog/validation/madevent-dummy-cuts-window-leakage.md)),
which matters only for event-by-event use of a windowed sample; σ comes from
`results.dat`.

A third witness, when two MadGraph versions are runnable, turns "the reference
moved" into a measurement: two seed clouds that overlap mean it fluctuated
rather than moved.[^n29-d3]

## Fixing the windows before measuring

The edges are frozen in the design, before any measurement, by a rule:
**physics-derived outer and boundary edges, equal population in
between**.[^n29-d2] For `ee_to_mumua` in `pt(a)`:

1. Outer edges from the run card and kinematics: `pta = 10` GeV and `√s/2 = 250`.
2. A cut-boundary edge at `2 × pta = 20` GeV, isolating where the cut, not the
   dynamics, sets the density.
3. A kinematic threshold at `pt_RR^min = p_RR / cosh(etaa) = 241.685 / cosh(2.5)
   = 39.4` GeV: below it no on-shell-Z event survives `etaa = 2.5`. The banked
   sample confirms it (0% of events below 20 GeV and 2.1% in 20–40 GeV sit in
   the Z mass window, against 46% just above).
4. Interior edges: equal-population tertiles of the reference sample *above* the
   threshold (77 and 144 GeV), so the two regimes are not smeared together.

Frozen edges `10, 20, 39.4, 77, 144, 250`; no window under 8% of σ, so no
window's error blows up relative to the total. A secondary axis in `m(μμ)`
(`0, 60, 86, 96, 200, 500`) is reported without a verdict of its own: `pt(a)` is
a smeared image of the Breit–Wigner that `m(μμ)` resolves directly, so if `pt(a)`
localises nothing, `m(μμ)` says whether there was nothing to localise.[^n29-d2]

## Statistics, defined in advance

With `Δ_w = σ_VG(w)/σ_MG(w) − 1` and `ε_w` its combined error:[^n29-d3]

- **Localised** ⟺ `χ²_flat = Σ_w (Δ_w − Δ̄)²/ε_w² > 13.28` on 4 dof (p < 0.01),
  `Δ̄` the inverse-variance mean.
- **Closure fails** ⟺ `|C| > 3` combined σ, with
  `C_MG = Σ_w MG-part(w)/MG-control − 1` and `C_VG` likewise.
- **Seed-consistent** ⟺ five-seed χ²/dof ≤ 2 on 4 dof.
- **Budget-stable** ⟺ under 4× budget the error shrinks ≥ 1.7× and
  `|Δ_w(4×) − Δ_w(1×)| ≤ 2 ε_w(1×)`.
- **Version-separated** ⟺ two versions' five-seed clouds differ by more than 3×
  the combined *seed spread*, not the quoted per-run errors.

The verdict table keyed on these is pre-registered too
([pre-registered verdicts](pre-registered-verdicts.md)). The seed and budget
protocol on both sides (five seeds each, gate budget and 4×, MadGraph's own
cards with only `nevents` and `iseed` changed, spread and χ²/dof reported, never
a headline pull) is in
[seed sweeps and budget ladders](seed-sweeps-and-budget-ladders.md#protocol-for-an-attribution-measurement).[^n29-d4]
A window that rejects most draws (`VG-cut(W1)` rejected about 92%) needs extra
budget, or its error silently dominates `C_VG` and makes the closure
vacuous.[^n29-d7]

Two error conventions coexist and give slightly different digits for the same
quantity: the ratio form (`rel = a/b − 1`, errors propagated through the ratio)
and plain quadrature on the absolute difference. Name which one a number uses.
A headline significance should name its error basis too; a 19.9σ figure on a
five-seed SEM read 15.8σ on the most conservative basis.[^n29-addenda]

## Worked cases

**`h → ττ` pole (`ee_to_mumu_tata_qcd0`).** No run-card cut could express an
`m(ττ)` window (the `mmll` family applies to every same-flavour opposite-sign
pair, and `banner.py` refuses per-PDG lepton cuts), so it went into
`dummy_cuts`.[^n27-b1] MadGraph's window plus complement exceeded its own
unwindowed integral by 3.1e-5 pb, 7.2σ on its own errors, and the excess was the
pole; our σ sat 0.35σ from MadGraph's sum. Three fresh MadEvent seeds agreed with
each other and with the bank, all confidently wrong. Our side measured the window
three independent ways (a single Breit–Wigner channel, the 25-channel combiner,
flat RAMBO). The root cause was MadGraph 3.5.7's `sde_strategy = 2` channel
weight: `get_channel_cut` computed `tmp = (t-Mass)*(t+Mass)` with `t` already
`p²`, so the Higgs configuration got α ≈ 1.9e-3 of its own pole (the windowed
run's realised split was 0.198%). The 3.7.1 form,
`tmp = (t-Mass**2)` with a plain Breit–Wigner, gives α ≈ 1; an `sde_strategy = 1`
rerun agreed within 1.1σ. The fix is upstream `286feb8e` (first released in
3.6.2, never backported to 3.5.x).[^n27-b1][^mg-genps] See
[MadGraph defects](madgraph-defects.md). The windowed agreement is now a live
measurement, `validate_samples::the_higgs_pole_window_is_measured_against_madgraph`,
against the committed `higgs_window_reference.json`.

**`ee_to_mumua` radiative-return offset.** Our σ sits a fixed +1.04% above the
bank. MadGraph's `pt(a)` closure failed (`C_MG` +3.07σ) while ours held
(`C_VG` −1.53σ), and MadGraph's `pt(a)` and `m(μμ)` partitions of its own run
disagree with each other at 16.7σ, both above its unwindowed control; its
`m(μμ)` partition recovers our total to 0.16σ.[^vsigma][^n29-addenda] The row's
gating is in [sigma-row gating exceptions](sigma-row-gating-exceptions.md) and
its open item is
[ee-mumua radiative return](../backlog/validation/ee-mumua-radiative-return-sigma-high.md).
The probe is `probe_pta_windows_against_madgraph` (`#[ignore]`d), its driver
`validation/madgraph/gen_pta_windows.sh`.

**Keep the committed JSON as the record.** `pta_window_reference.json` was
verified row by row (58 of 58) against the MadGraph run directories that
produced it, but those live under the gitignored `validation/madgraph/output/`,
which may be pruned; anything later readers need is read from the JSON.[^n29-addenda]
A windowed study that changes no production code should move no report cell;
check with `pixi run --skip-deps validate` (never bare, which can launch a
multi-hour MadGraph regeneration) and an empty report diff.[^n29-d7]

## What it cannot decide

- **Anything both sides get wrong the same way inside a window.** They share
  the matrix element and the window definition; this is a coverage statement,
  not a matrix-element one.
- **A window whose lower edge is a cut** cannot separate "the cut boundary is
  implemented differently" from "the region is mis-covered".
- **The σ verdict does not own a `samples` KS cell**: KS is normalisation-free,
  so a flat `Δ_w` leaves a KS failure unexplained; the per-window shape is the
  evidence for that.
- **A compensating error that cancels in the projection** (`η(a)` is integrated
  over inside each `pt(a)` window); the second axis covers part of this, not a
  defect orthogonal to both.
- **Inert physics**: on a fixed-beam electroweak row, scale prescription, PDFs
  and polarisation are not tested.[^n29-d7]

[^n27-b1]: Note 27 B1 brief and outcome, including the upstream provenance paragraph.
[^n29-d2]: Note 29 D.2.
[^n29-d3]: Note 29 D.3.
[^n29-d4]: Note 29 D.4.
[^n29-d7]: Note 29 D.7–D.8.
[^n29-addenda]: Note 29 chain D addenda A2–A5.
[^vsigma]: `vibegraph-lib/tests/validate_sigma.rs`, `PULL_REPORTED_NOT_ASSERTED` doc comment.
[^mg-genps]: MadGraph `genps.f` at `b7687064`, the `sde_strategy = 2` resonant branch of `get_channel_cut`:
    ```fortran
    tmp = (t-Mass**2)
    get_channel_cut = get_channel_cut/(tmp**2 + tmp2**2)
    ```
