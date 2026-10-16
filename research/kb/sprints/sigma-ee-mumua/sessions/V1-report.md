---
type: Session Report
title: "V1 report: the ee → μμγ offset is MadEvent's, in its Z radiative-return channels"
description: "An adaptation-free estimator agrees with vibegraph in every window and cell. The bank reads 1.00 % low. MadEvent's Z-ISR channels (G6, G8) have rare, very large weights; its mean²/σ² iteration combination discounts them, and more budget does not shrink the bias. Recombining MadEvent's own iterations with equal weights at 100k events closes it."
status: draft
tags: [validation, sigma, madevent-reference, ee-to-mumua, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: base, resource: "https://github.com/nsmith-/vibegraph/commit/162891d", title: "Measured at 162891d (sigma/ee-mumua base), no source changes"}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
Diagnosis only. No source change and no committed probe. All MadEvent runs
used the **pinned MadGraph 3.7.1** (`mg5_pinned.sh` under `pixi run -e
madgraph`, `MGMEVersion.txt` = `3.7.1`). The template was generated with `set
gauge unitary` / `generate e+ e- > mu+ mu- a`. Its `matrix1_orig.f` and
`leshouche.inc` are byte-identical to the banked run's, with `IDUP =
-11,11,-13,13,22`. Each seed ran in a fresh `cp -a` of the never-run template
with `nb_core = 2`, using the banked `Cards/run_card.dat` and
`Cards/param_card.dat` verbatim. Only `iseed` changed, plus whatever override a
row names. Windows were imposed through `dummy_cuts`, using the
`gen_pta_windows.sh` body for `pt(γ)` and its `m(μμ)` body, combined for the
2D cells. vibegraph ran from `target/release-debug` with two threads. The
scratch work, scripts and logs are in the scratchpad under `ee-work/`. The
per-run `results.dat`, `G*/results.dat` and banners are in
`ee-work/ee-runs-results.tgz`. The throwaway probe code is
`ee-work/ee-probe.patch`, applied to `vibegraph-lib/tests/validate_samples.rs`
and then reverted.

## Verdict

- **Side:** MadEvent. The bank (`0.099801 ± 0.000234` pb) reads **−1.00 %**
  against an adaptation-free estimator. vibegraph agrees with that estimator:
  - the inclusive σ and the 1D `pt(γ)` windows to ≤ 0.11 %;
  - every 2D cell within 1.6σ of its 10-seed error;
  - the worst 1D `m(μμ)` window (`[96,200)`) at +0.45 %, +2.2σ, out of
    15 window comparisons.
- **Region:** MadEvent channels **G6 and G8**. These are configurations 6 and
  8: the photon from the e+ or e− line through the t-channel, and Z → μ+μ− in
  the s-channel, so the Z radiative-return channels. The deficit lives in the
  off-peak part of those channels. A run whose integrand holds both the Z
  peak and its off-peak continuum under-integrates them. A run restricted to
  one `m(μμ)` cell does not.
- **Mechanism:** rare, very large weights in G6 and G8, combined with
  MadEvent's iteration weighting, which discounts the iterations that catch
  them. For example, one `[39.4, 77)` run at 100k events has a G6 iteration
  reading `0.02284 ± 0.0151` against a typical `0.0077 ± 0.0001`. MadEvent's
  weighted combination gives that iteration almost no weight. The resulting
  bias **does not shrink with budget**:

  | MadEvent budget | reported σ | χ²/dof |
  |---|---|---|
  | 10k events | −0.95 % | 0.62 |
  | 100k events | −0.94 % | 3.6 (confidently wrong) |

  Equal-weight recombination of the same runs' iterations reads −0.55 % at
  10k and **−0.19 % ± 0.34 %** at 100k. In `[39.4, 77)` at 100k it reads
  **−0.06 % ± 0.23 %**.
- **Nothing on this side is implicated.** The adaptation-free estimator shares
  only |M|² and the cut code with vibegraph:
  - |M|² is gated against MadGraph to 1e-11.
  - The cuts were checked against an independent re-implementation in the
    probe: 0 disagreements in 4×10⁸ points.
  - MadEvent's own 190k events all pass the card cuts and reach every cut
    boundary (min `pt` 10.0000, `2.5 − |η|` 6e-9, `ΔR` 0.40005). MadEvent's
    cuts are therefore no tighter than ours.
- **What would falsify it:**
  - An unbiased, honestly-errored estimator that agrees with MadEvent's
    reported σ.
  - A third fixed-density parametrisation that disagrees with the two here by
    much more than ~0.02 %.
  - MadEvent's equal-weight iteration mean staying low as budget grows. It
    moved −0.55 % → −0.19 % from 10k to 100k.
  - MadEvent's `m(μμ)`-cell runs failing to converge at higher budget. The
    named cell reads −0.23 % at 10k and −0.04 % at 100k.

## Step 0: the paired iteration test (watch item)

The script is `mg_table.py`, an extension of the `2to4-work` `iter_seed.py`
logic. Its columns:
- "reported" is the top line of `SubProcesses/results.dat`;
- "equal-iter" is the sum over channels of the unweighted mean of each
  `G*/results.dat` iteration row;
- "paired" is the per-seed (equal − reported), as a mean ± SEM;
- "vs FD" is measured against the adaptation-free estimate below.

`err = max(√Σq²/n, sd/√n)`.

| run | n | reported (pb) | χ²/dof | equal-iter (pb) | paired | reported vs FD | equal vs FD |
|---|---|---|---|---|---|---|---|
| inclusive, 10k | 19 | 9.98449e-2 ± 6.3e-5 | 0.62 | 1.00248e-1 ± 1.0e-4 | **+0.404 % ± 0.089 %** | −0.953 % | −0.553 % |
| `[39.4,77)`, 10k | 18 | 2.75216e-2 ± 3.2e-5 | 1.83 | 2.77228e-2 ± 8.8e-5 | +0.731 % ± 0.277 % | −1.430 % | −0.709 % |
| `[77,144)`, 10k | 19 | 2.70371e-2 ± 1.8e-5 | 0.63 | 2.72512e-2 ± 3.5e-5 | +0.792 % ± 0.120 % | −0.712 % | +0.074 % |
| inclusive, 100k | 9 | 9.98606e-2 ± 5.3e-5 | **3.60** | 1.00617e-1 ± 3.4e-4 | +0.757 % ± 0.339 % | −0.937 % | **−0.187 %** |
| `[39.4,77)`, 100k | 10 | 2.75743e-2 ± 2.8e-5 | **9.31** | 2.79038e-2 ± 6.4e-5 | +1.195 % ± 0.214 % | −1.241 % | **−0.061 %** |

So the paired test closes the offset at 100k events, but only about 40 % of
it at the bank's 10k. At 10k the equal-weight mean is itself heavy-tailed:
the rare hits are too seldom sampled. Unlike the 2 → 4 case, the reported
value does not improve with budget.

Per channel, G6 and G8 carry the whole paired shift. In `[39.4, 77)` at 100k:
- G6 reports 7.7291e-3 against an equal-iter 7.8888e-3;
- G8 reports 7.7185e-3 against an equal-iter 7.8843e-3;
- G1, G2, G5 and G7 move by < 0.1 %.

The 1D `[39.4, 77)` run against the sum of its five `m(μμ)` cells (10k)
agrees channel by channel within errors except G6 and G8. Their combined G6 +
G8 is 1.5384e-2 in the window run against 1.5739e-2 summed over the cells
(−2.3 %). In `[77, 144)` the same comparison gives −1.0 %.

**The brief's "+0.68 % above the reported value" was mis-stated.** The bundle
keeps only survey logs for the banked run: three iterations per channel, and
none for G2. Re-reading them, the equal-weight survey sum is +0.69 % above the
**survey's own** combination (`Cross sec`), 0.09791 against 0.09724 over five
channels. It is not +0.68 % above the reported σ. With G2 omitted, that sum
cannot be compared with the bank.

## Step 2: adaptation-free estimate

The estimator is a fixed-density importance sampler, never adapted. It draws:
- `Q² = m(μμ)²` from a fixed mixture: a Breit–Wigner at M_Z, `1/Q²`,
  `1/(s − Q²)`, and flat on `[1, s − 2√s·9.9]`;
- the photon direction from a fixed mixture of flat `cos θ` and flat `η` on
  `|η| < 2.6`;
- the muons isotropic in their rest frame.

The weight is `f/p` in the standard dΦ₃, using vibegraph's |M|² and
`Cuts::pass`. The spin average 1/4 is asserted equal to the library's. Two
parametrisations (A and B: different mixture weights) each ran 40 independent
batches × 5M points. The quoted error is the batch spread. Max/median weight
over batches is ~1.2, so the weights are not heavy-tailed.

| | A (pb) | B (pb) | used |
|---|---|---|---|
| total | 1.008163e-1 ± 7.8e-6 (χ²/dof 0.80) | 1.007948e-1 ± 7.9e-6 (0.87) | **1.00806e-1 ± 1.1e-5** |
| `[39.4,77)` | 2.79201e-2 ± 6.8e-6 | 2.79215e-2 ± 6.3e-6 | 2.79208e-2 |
| `[77,144)` | 2.72332e-2 ± 5.2e-6 | 2.72286e-2 ± 5.2e-6 | 2.72309e-2 |
| `[39.4,77) × [86,96)` | 1.28337e-2 ± 3.9e-6 | 1.28366e-2 ± 5.0e-6 | 1.28351e-2 |

A and B differ by 1.9σ on the total. That scatter is ~0.02 %, fifty times
below the question being asked, and the "used" error is inflated to cover it.

## Step 1: the named cell and its neighbours

The rows below use these estimators:
- **FD:** the adaptation-free estimate above (A/B mean).
- **VG-part:** a frozen-grid pass at the gate configuration (80k × 8), 10
  seeds.
- **VG-cut:** an independent vibegraph integration with the cell's
  `pta/ptamax/mmll/mmllmax` in the run card, 5 seeds.
- **MG-part:** a `dummy_cuts` cell, reported σ.
- **MG-cut:** run-card cuts.

| cell | estimator | σ (pb) | n | χ²/dof | vs FD |
|---|---|---|---|---|---|
| **`[39.4,77) × [86,96)`** | FD | 1.28351e-2 ± 4e-6 | 80 batches | 0.83/1.19 | — |
| | VG-part | 1.28233e-2 ± 2.4e-5 | 10 | 1.32 | −0.09 % (−0.5σ) |
| | VG-cut | 1.28263e-2 ± 1.2e-5 | 5 | 0.81 | −0.07 % (−0.7σ) |
| | MG-part 10k | 1.28062e-2 ± 1.1e-5 | 6 | 0.91 | −0.23 % (−2.7σ) |
| | MG-part 100k | 1.28304e-2 ± 3.8e-6 | 5 | 0.85 | −0.04 % (−0.9σ) |
| | MG-cut 10k | 1.28188e-2 ± 1.0e-5 | 6 | 0.32 | −0.13 % (−1.6σ) |
| `[39.4,77) × [0,60)` | VG-part / MG-part 10k | 1.8634e-3 / 1.8736e-3 | 10 / 6 | 0.40 / 1.38 | −0.88 % (−1.5σ) / −0.34 % |
| `[39.4,77) × [60,86)` | VG-part / MG-part 10k | 1.2608e-3 / 1.2592e-3 | 10 / 6 | 1.35 / 0.47 | −0.01 % / −0.14 % |
| `[39.4,77) × [96,200)` | VG-part / MG-part 10k | 3.4769e-3 / 3.4608e-3 | 10 / 6 | 1.51 / 0.87 | +0.24 % / −0.23 % |
| `[39.4,77) × [200,500)` | VG-part / MG-part 10k | 8.4748e-3 / 8.4580e-3 | 10 / 6 | 1.01 / 0.28 | −0.02 % / −0.22 % |
| `[77,144) × [0,60)…[200,500)` | MG-part 10k | −0.31, −0.09, −0.06, −0.06, +0.11 % vs FD | 5–6 | 0.16–0.95 | — |
| `[77,144) × …` | VG-part | −0.49, −0.40, −0.20, +0.58, −0.13 % vs FD | 10 | 0.44–1.11 | −1.0, −0.6, −1.4, +1.6, −0.7σ |
| `[39.4,77)` 1D | VG-part / VG-cut | 2.78991e-2 / 2.78978e-2 | 10 / 5 | 1.54 / 2.25 | −0.08 % / −0.08 % |
| `[39.4,77)` 1D | MG-part 10k / 100k | see step 0 | | | −1.43 % / −1.24 % |
| inclusive | VG gate / VG-part total | 1.00723e-1 ± 5.6e-5 / 1.00788e-1 ± 5.6e-5 | 10 | 0.40 / 1.15 | −0.08 % / −0.02 % |

There are two readings of step 1, and the measurements decide between them:
1. **"Our η(γ) at fixed m(μμ) differs":** falsified. At fixed `m(μμ)` on the
   Z peak, `pt(γ)` is a function of η(γ) alone. Every VG estimate of the named
   cell sits within 0.1 % of FD, and so does MadGraph's own cell run once its
   budget is adequate.
2. **"MadGraph's pt(γ)-restricted runs under-recover":** confirmed, and it is
   not specific to `pt(γ)`. Any MadEvent run whose integrand contains both the
   Z peak and its off-peak part under-integrates G6 and G8. That covers the
   inclusive run, a `pt(γ)` window and MG-cut `[39.4,77)`.

This is why chain D's `m(μμ)` partition recovered our total while its
`pt(γ)` partition did not. The `m(μμ)` partition at 100k is measured below.

### The m(μμ) partition at 100k (5 fresh seeds per cell)

| cell | reported vs FD | χ²/dof | equal-iter vs FD |
|---|---|---|---|
| `[0,60)` | −0.15 % | 0.63 | −0.18 % |
| `[60,86)` | −0.04 % | 1.92 | +0.01 % |
| `[86,96)` | −0.05 % | 0.24 | −0.04 % |
| `[96,200)` | −0.19 % | **4.92** | −0.10 % |
| `[200,500)` | −0.40 % | **3.29** | +0.14 % |
| **sum** | **−0.193 % (−5.8σ, 1.006111e-1 ± 3.4e-5)** | | **+0.011 % ± 0.078 % (1.008165e-1)** |

Heavy tails persist in the off-peak cells: χ²/dof 4.9 and 3.3, and the
paired shift in `[200,500)` is +0.53 % ± 0.22 %. Even MadEvent's `m(μμ)`
partition reads 0.19 % low on its reported σ, which matches the bundle's
3-seed figure (`Σ MG-part(m)` 1.006551e-1, −0.15 %). Only its equal-weight
recombination is unbiased here.

### Ruled out

| test | result | what it rules out |
|---|---|---|
| `sde_strategy = 2` (10 seeds, 10k) | reported −2.16 % with χ²/dof 3.4 inclusive; −3.67 % with χ²/dof 30.7 in `[39.4,77)`; G6 equal-iter +4.6 % ± 2.0 % | That channel weighting as a fix. It makes things worse, through the same G6 tail. |
| `bwcutoff = 1000` (10 seeds, 10k) | reported −0.85 % inclusive, −1.50 % in `[39.4,77)` | The sub-hypothesis that the tail is points outside the ±15Γ Breit–Wigner mapping window. |

Where in G6 and G8 the large weights sit, beyond "off the Z peak", is not
pinned. The `[39.4,77) × [96,200)` and `[77,144) × [200,500)` cells show the
largest paired shifts (+1.1 % ± 0.9 % and +0.39 % ± 0.18 % at 10k).

## Which of the row's existing numbers are event fractions

These numbers come from event fractions of MadEvent's unweighted sample
(suspect):

- **D1's** "8.732 % of σ in `pt(γ) ∈ [10, 20)`". Confirmed, and it is the
  sample's own defect. My 19 fresh 10k samples put 8.691 % ± 0.054 % there,
  against FD 9.391 %, which is −7.5 % relative. The sample also
  over-populates higher windows:

  | window | sample fraction | FD |
  |---|---|---|
  | `[39.4,77)` | 28.00 % | 27.70 % |
  | `[77,144)` | 27.33 % | 27.01 % |
  | `[144,250)` | 26.49 % | 26.12 % |

  Sample fraction × reported σ happens to land within 0.1 % of FD in
  `[39.4,77)`. An event-fraction comparison there would have hidden the σ
  deficit.
- **The `samples` KS cell** (`pt(a)` p-values) compares against that sample,
  so it inherits the shape error.
- **The tertile edges** 77 and 144, and the "0 % / 2.1 % / 46 % in the Z
  window" statement, are design inputs only. They carry no verdict.

These numbers come from sliced σ (`results.dat`) and are trustworthy as
measurements of MadEvent's output, though biased low as above:
- the bank;
- every `pta_window_reference.json` row;
- `Δ_3` and `Δ_4` (VG-part against MG-part).

## Brief corrections

- **The +0.68 % re-combination** is survey-against-survey; see step 0.
- **The +1.04 % offset is not what the current tip measures.** The
  `validate_samples` integrand (`with_mumua_integrand`, the gate's own
  configuration) gives +0.90 % over the five `PTA_SEEDS` and +0.92 % over ten
  seeds (gate value 1.00723e-1). The +1.04 % comes from `validate_sigma`'s
  `integrate()`, which I did not re-run. Either way, the bank is −1.00 %
  against FD.
- **The paired test does not close the offset at the bank's budget.** It
  closes it only at 100k, so steps 1–2 did not shrink.

## Banking request (user, M3 host)

**Re-bank `sigma_reference.json#ee_to_mumua`.** The current value is low by
1.00 % ± 0.23 % (bank error). MadEvent's reported σ at the banked card does
not converge with `nevents` (10k: −0.95 %, 100k: −0.94 %). The recipe below
is the only one measured to land on the adaptation-free value.

Recipe:
- **Stage:** `validation/madgraph/gen_pta_windows.sh --stage mll-371`, with
  the five `m(μμ)` `dummy_cuts` cells on the banked card. Pinned 3.7.1, a
  fresh directory per seed.
- **Parameters:** `NEVENTS=100000`,
  `SEEDS="20261001 20261002 20261003 20261004 20261005 20261006 20261007 20261008"`.
- **Extra outputs:** keep every run's `SubProcesses/P1_ll_lla/G*/results.dat`
  in the bundle. The script currently keeps only the top-level value.

Expected outputs:
- Per seed, the sum over cells of
  - the reported σ, expected about `1.0061e-1` (−0.19 %);
  - the equal-weight iteration σ, expected `1.0082e-1 ± 0.0001` (±0.08 %).
- The banked reference is the **equal-weight sum**: the seed mean, with
  `err = max(quoted, seed spread)`. The reported sum is recorded beside it
  with the measured −0.19 % recorded as MadEvent's.

The equal-weight recombination needs a small script reading `G*/results.dat`
rows. The `mg_table.py` logic in the scratchpad is ~30 lines. If the manager
prefers a reference that is "MadGraph's reported number", the
`m(μμ)`-partition reported sum is the least-biased one (−0.19 %, 5.8σ on
seed spread). The row's gate would then need a documented tolerance above
0.2 %. The pull cannot be asserted against it.

On re-banking, the gate can re-arm:
- the σ pull asserted (vibegraph gate −0.08 % ± 0.06 % against FD);
- the `samples` cell, only against a reference sample that is not MadEvent's
  own unweighted sample, which is mis-shaped by 7.5 % in `[10, 20)`. That is a
  separate decision.

## Found

1. **The `vg_part` helper in `validate_samples.rs` panics on this tip.**
   Evidence: `assertion left == right failed: a point is 5 map coordinates and
   1 scale-draw uniforms` (`hadronic.rs:2827`). It builds `x` with
   `channel_grid_ndim()` (5), but `event_in_channel` now requires
   `point_ndim()` (6), so `probe_pta_windows_against_madgraph` cannot run. The
   fix is to size `x` by `point_ndim()` and fill the trailing scale-draw
   uniforms, which is what the scratch probe did. That code sits in the
   boundary-free test file, but it is a production-probe change I did not
   make.
2. **The VG χ²/dof report blows up on a narrow 2D cut cell.** Evidence:
   `VegasResult::chi2_per_dof` read 1e256–1e260 on 4 of 5 seeds of VG-cut
   `pta 39.4–77, mmll 86–96`. σ and its error were sane and agreed with FD.
   The likely cause is an iteration with zero variance (no point surviving the
   cuts) entering the χ² sum. It was not diagnosed further. The fix is to
   exclude zero-variance iterations from χ², or report it as undefined, with a
   test on a cut that empties early iterations.
3. **MadEvent's equal-weight iteration recombination should become the
   reference statistic for heavy-tailed rows.** On this row the reported σ is
   confidently wrong (χ²/dof 3.6–9.3 at 100k) and does not converge with
   budget. Evidence: step 0 and the `m(μμ)` table above. What would close it:
   - an extension of `validation/madevent-iteration-weighting-bias` to say the
     bias need not shrink with budget when a channel has rare large weights;
   - a recombination script beside `madevent_seeds.sh`;
   - an audit of the other banked rows with resonant ISR channels
     (`ee_to_tatah`, `ee_to_zh`) for χ²/dof > 2 across fresh seeds.
4. **MadEvent's unweighted sample for this row is mis-shaped by 7.5 % in
   `pt(γ) ∈ [10, 20)` relative to the true σ.** This is a sample defect, not a
   σ defect. Any `samples` gate against it measures MadEvent. What would close
   it: decide the row's sample reference, either a non-MadEvent reference or
   MadEvent at a budget and setting shown to fix the shape.
