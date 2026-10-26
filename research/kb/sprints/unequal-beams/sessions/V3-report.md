---
type: Session Report
title: "V3 report: the 6500 x 4000 Drell-Yan disagreement traced to MadEvent's mirrored-ordering cut"
description: "vibegraph's unequal-beam kinematics check out; MadEvent cuts its mirrored beam ordering at the direct ordering's cm_rap, which at E1 != E2 is the lab frame with the beam energies exchanged. Emulating that in vibegraph on the real PDF set gives 717.575 +- 0.309 pb against MadGraph's 717.25 +- 0.145 (pull +0.95). No source fix: the defect policy's outcome is a user decision. One hermetic beam-exchange test added, three mutations caught."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-22}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
---
Worktree `/home/user/wt/S`, branch `sprintd-unequal-beams`, from `41672eb`.
Nothing in `vegas.rs`, `budget.rs` or the phase-space maps was touched; no
backlog item was edited.

The diagnosis began hermetically:
- MadGraph read at `b7687064` through raw.githubusercontent.com;
- vibegraph probed with toy PDFs.

Mid-session the manager placed `NNPDF23_lo_as_0130_qed` in
`validation/pdf/` (with the user's consent). The hypothesis was then measured on
the real set; see "Real-PDF measurement" below. The MadGraph output bundle is
still absent, so MadGraph's own events were not read.

## Correction to the brief

The brief puts the bug in this PR's unequal-beam kinematics. The evidence says
the kinematics are right and **MadEvent's mirrored beam ordering is cut in the
wrong frame when `ebeam1 ≠ ebeam2`**. This is a MadGraph defect, so under the
[defect policy](../../../validation/madgraph-defect-policy.md) it is not "fixed"
on our side. Whether to reproduce it, refuse the card, or register a deviation
is a user decision, so no production change was made. The one committed source
change is a test that pins vibegraph's frame.

## The MadEvent defect

Line numbers are from the pinned commit `b7687064`.

1. `genps.f:345` sets `cm_rap = ½ ln(xbk(1)·ebeam(1) / (xbk(2)·ebeam(2)))` for
   the sampled point.
2. `dsample.f:181` evaluates `passcuts(p)` **once per phase-space point**,
   before `select_grouping` chooses the beam ordering `IMIRROR`
   (`dsample.f:222`). `rap()` adds that `cm_rap` (`kin_functions.f:131`), and
   `cuts.f:426` cuts on `|rap|`. The grouped `DSIG` has no second
   `PASSCUTS`: `export_v4.py:5103` sets `passcuts_begin = ""`, and the call at
   `super_auto_dsig_group_v4.inc:835` is commented out.
3. For `IMIRROR = 2`, `super_auto_dsig_group_v4.inc:815-829` rotates the
   momenta by π about x, sets `IB = (2,1)`, swaps `XBK` and sets
   `CM_RAP = −CM_RAP`. The PDF lines read `XBK(IB(i))`
   (`export_v4.py:2004-2005`). So the mirrored event has parton `b` on
   beam 1 at `x₂` and parton `a` on beam 2 at `x₁`, with outgoing `R·p`. The
   vectorised path does the same (`:984-990`).
4. `unwgt.f:623-629` flips legs 1 and 2 back so that beam 1 is along +z.
   `unwgt.f:648-660` then boosts with `ebi = p(0,1)/xbk(1)`, where `xbk(1)`
   is now `x₂`, to `ebeam(1)`. The written mirrored event therefore sits at its
   physical boost `Y_m = ½ ln(x₂E₁/(x₁E₂))`.

**What follows.** Write `L = ln(E₁/E₂)` and `y*` for a lepton's rapidity in
the mirrored event's own CM frame. The physical lab rapidity is
`y = −y*_p − ½ ln(x₁/x₂) + ½L`, where `y*_p` is the rapidity in the sampled
frame. The cut sees `y*_p + ½ ln(x₁/x₂) + ½L = −y + L`. MadEvent therefore
accepts mirrored events whose written lab rapidity lies in
`[L − 2.5, L + 2.5] = [−2.014, 2.986]` at 6500 × 4000 (`L = ln 1.625 = 0.4855`).
The direct ordering is cut correctly. At equal beams `L = 0`, and the two
frames coincide exactly. That is why every equal-beam row agrees.

Put another way, MadEvent cuts the mirrored term at `cm_rap` with the beam
energies exchanged, `½ ln(x₁E₂/(x₂E₁))`.

## What vibegraph does, candidate by candidate

| candidate | finding |
|---|---|
| boost sign or direction | `proton.rs:2063`: `β = (x₁ − r x₂)/(x₁ + r x₂)` with `r = E₂/E₁`. F1's `unequal_beams_place_and_boost_the_partons_as_madgraph_does` pins `β = 5/21` at `x₁ = x₂` and a rapidity shift of `+½ ln(13/8)` toward beam 1 (+z). Correct. |
| x₁/x₂ against the beams | `lab[0] = x₁E₁` along +z (`:2065`) and `beam_rows(pdf, x₁, x₂)` (beam 1 reads x₁, `:304`). They agree. |
| flux, ŝ Jacobian, s | `s_had = (2√(E₁E₂))²` (F1 test: 1.04e8). `√ŝ = √(τ s)` (`:2043`). The flux `1/(2ŝ)` (`:2110`) and the `(τ, y)` Jacobian are the equal-beam code unchanged. |
| cut frame | lab, for both orderings. The mirrored term's physical event (b from beam 1 at x₁, outgoing q) has the same lab momenta as the direct one, so one `cuts.pass(&lab)` (`:2090`) is that event's own cut. The new test pins this pointwise. |
| σ path against event path | `event_in_channel` (`:2628`) calls the same `shape` as `value_in_channel`. |
| flavour + ICOLUP failing together | Expected from the defect: the direct-to-mirror split is exactly "which beam carries the quark". MadEvent's mirrored acceptance window is displaced by `L`, so its share of events and their rapidities differ. |

## Hermetic evidence

**Toy-PDF σ probe** (scratch, uncommitted). The PDF is valence `u, d` plus a
sea, on 141 log-x knots. The card is DY: `ptl 10`, `etal 2.5`, `drll 0.4`,
`mmll 0`, fixed scales at 91.188. Each run used 100k × 8 iterations at one
seed. The second run of each pair emulates MadEvent's mirrored cut: the
mirrored term is cut on the lab frame at `r = E₁/E₂`. Log:
`…/scratchpad/ubS-diag-probe2.log`.

```
UBS beams 6500 4000 mg_mirror_emulated=false: sigma = 675.5811231571402 +- 0.9785511355314025
UBS beams 6500 4000 mg_mirror_emulated=true: sigma = 660.2097572270669 +- 0.8897312274746482
UBS beams 4000 6500 mg_mirror_emulated=false: sigma = 675.7482797913731 +- 0.9787566691891761
UBS beams 4000 6500 mg_mirror_emulated=true: sigma = 692.0232524322439 +- 0.9466086345185717
UBS beams 6500 6500 mg_mirror_emulated=false: sigma = 758.4775754420813 +- 1.0929003063002127
UBS beams 6500 6500 mg_mirror_emulated=true: sigma = 758.4775754420813 +- 1.0929003063002127
```

- vibegraph is invariant under exchanging the beams: 675.58 against 675.75.
- The emulated defect breaks that invariance. It lowers σ by **−2.3%** at
  6500 × 4000 and raises it by +2.4% at 4000 × 6500.
- At equal beams the emulation is bit-identical, which is the control.

The measured offset is MG/vibegraph = 717.25/754.96 = 0.950, in the same
direction. Its size depends on how strongly the real PDFs separate the direct
and mirrored rapidity distributions, which a toy cannot calibrate. So the toy
establishes the sign and the order of magnitude, not the 5.0%.

The defect lowers σ in this orientation because of where the mirrored events
sit. They put the valence quark on the 4000 GeV beam, so they lean toward −z,
while their acceptance window is pushed to +z, by `+L`.

An earlier, smaller probe (40k × 8) compared the real 6500 × 4000 against
equal beams at the same √s (5099.02 × 2). It gave 677.06 ± 1.57 against
678.88 ± 1.58 (log `…/scratchpad/ubS-diag-probe1.log`). The physical boost
costs well under 1% of acceptance, so a correct boost cannot move σ by 5%.

**Committed test**:
`proton::tests::exchanging_unequal_beam_energies_reflects_the_integrand`. It
draws 20 000 production points of `p p > e+ e-` at 6500 × 4000 with the
`beam_asymmetric_pdf` probe. At each point it requires
`shape(x₁, x₂, q)` at 6500 × 4000 to equal `shape(x₂, x₁, R q)` at 4000 × 6500,
to `1e-12` relative. It also requires that at least 500 points have cuts in the
physical frame and in MadEvent's mirrored frame that disagree, so the check
cannot go vacuous. Mutation driver: `…/scratchpad/ubS-diag-mut.sh`; log:
`ubS-diag-mut.log`. Each mutant was restored, and the restore was checked
with `cmp`.

| mutation | result |
|---|---|
| M1: MadEvent's defect, the mirrored term cut on the lab frame at `r = E₁/E₂` | **FAILED** at trial 6: `3.16e-6` against `4.70e-7` |
| M2: mirrored luminosity read as `f₁[a]·f₂[b]` (beams not exchanged) | **FAILED** at trial 0 |
| M3: `mirror_into` without the rotation | **FAILED** at trial 0 |
| M4: cuts on the partonic-CM frame for both orderings | survives. It is symmetric under the reflection, so it lies in the documented blind spot; the banked equal-beam σ rows see it |

The test is also blind to a boost of the wrong sign in both orderings, and to
partons placed on the wrong beam in both. F1's test pins the boost's
direction.

The test passes on the unmodified integrand. It is not a fix test: there is
nothing in vibegraph to fix. It is the pin that a reproduction (option 1 below)
would have to change on purpose.

## Real-PDF measurement

These runs use NNPDF23_lo_as_0130_qed member 0 and `validate_hadronic`'s own
`run_seed` at the gate's budget and its five seeds (`DY_SEEDS`). They are
combined with the gate's `combine` (unweighted mean). The driver is a scratch
test (`UBS_CASE`, uncommitted) in `validate_hadronic.rs`; the MadEvent
emulation is the M1 patch behind an environment variable, also uncommitted.
Log: `…/scratchpad/ubS-diag-real.log`.

```
UBS default emu=false ebeams=(6500, 6500) MEAN 933.921 +- 0.434 chi2/dof 0.80
UBS beams emu=false ebeams=(6500, 4000) MEAN 754.956 +- 0.352 chi2/dof 0.51
UBS beams emu=true ebeams=(6500, 4000) MEAN 717.575 +- 0.309 chi2/dof 0.85
UBS swapped emu=false ebeams=(4000, 6500) MEAN 755.156 +- 0.363 chi2/dof 1.06
UBS swapped emu=true ebeams=(4000, 6500) MEAN 792.280 +- 0.418 chi2/dof 1.39
UBS even emu=false ebeams=(5099.019513592785, 5099.019513592785) MEAN 755.375 +- 0.390 chi2/dof 1.19
```

| case | vibegraph σ (pb) | reading |
|---|---|---|
| equal 6500 × 6500 (`dy13_default`), control | 933.921 ± 0.434 | MG 933.23 ± 0.48: rel +0.07%, the passing row |
| 6500 × 4000 | 754.956 ± 0.352 | the M3's number, reproduced to the digit |
| 6500 × 4000, MadEvent's mirrored cut emulated | **717.575 ± 0.309** | MG 717.25 ± 0.145: pull **+0.95**, rel **+0.045%**, inside the gate's \|pull\| < 3 and \|rel\| < 0.005 |
| 4000 × 6500 | 755.156 ± 0.363 | beam exchange leaves σ unchanged (Δ = +0.20 ± 0.51) |
| 4000 × 6500, emulated | 792.280 ± 0.418 | prediction for MadGraph at 4000 × 6500: +4.9% above vibegraph, mirror-symmetric to today's −5.0% |
| 5099.02 × 5099.02 (same √s, no boost) | 755.375 ± 0.390 | the physical boost costs (0.06 ± 0.07)%, so a 5% σ shift cannot come from a boost |

The single-line emulation reproduces MadGraph's σ to 0.045%. This closes the σ
side. The samples side (KS on the rapidities and beam E/pz; flavour and ICOLUP
χ²) is predicted by the same mechanism but was not run here, because it needs
MadGraph's banked events.

## Measurements still open (for the M3, with the banked events)

1. **MadGraph's own LHE violates its own cut.** In `run_beams_6500_4000`,
   count events with a lepton at `|η_lab| > 2.5`.
   - Prediction: a few per cent of events. All of them have a lepton at
     `2.5 < η ≤ 2.986`, and none has a lepton below −2.5.
   - Every such event has the antiquark on beam 1 (the mirrored ordering; leg
     1's ICOLUP is an anticolour only).
   - Among events with the quark on beam 1, the count is exactly zero.
   - vibegraph's events have zero.
   - A count of zero with a clean `η` range falsifies this diagnosis.
2. *(done above: 717.575 ± 0.309 against 717.25 ± 0.145.)*
3. **Control:** MadGraph at 4000 × 6500 should give about 792.3 pb, the
   emulated value, where vibegraph gives 755.2.

## Prediction for the M3's two tests

No production code changed, so both tests fail exactly as before:
- `sigma_unequal_beams_vs_mg`: vibegraph 754.96 against MG 717.25 pb.
- `validate_samples_proton … unequal_beam`: the same KS failures, and the
  same flavour and ICOLUP χ² failures.

They pass only once the user picks a policy outcome:
- **Reproduce (1):** the mirrored term's cut at `r = E₁/E₂`, in `shape`,
  `per_group_sum`/`select_event` and generation.
- **Deviation (3):** the samples gate's mirrored events reported `info` and
  the σ gate re-based.
- **Refuse (2):** proton cards with `ebeam1 ≠ ebeam2`, a mirrored group and a
  rapidity-type cut (`eta*`, `etaonium`, …) are refused.

Reproducing writes events that violate the run card's own cut, as MadEvent's
do.

## Gate

See the session's final message for the tails; the commands are
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
(with and without `--features vibegraph/extended-validation`), and
`cargo test --workspace`. `pixi run --skip-deps validate` was not run: no
amplitude, eval or coverage-list code changed, and the container has no
reference data.
