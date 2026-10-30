---
type: Session Report
title: "F1 report: the either-order H → ZZ* split brings bbx's ladder to 0.78, and with an α floor to 0.71"
description: "Splits with two Breit–Wigner daughters now draw either daughter first with probability ½ (the pair-order map choice, symmetric by default; artifact format 15). An α floor after the survey is selectable and off. On bbx_to_ccx_emmm_qcd0, 20 paired seeds at 600k and 1.2M, the per-seed sd ratio goes from 1.227 (control) to 0.784 (split) and 0.708 (split + floor 1e-4), with χ²/dof at 1.2M of 1.41, 1.18 and 0.66. T1's arithmetic is reproduced through the code on 8 of 8 points. Falsifier (b) and (c) hold on both arms. (a) holds for the on-shell-mirror population T1 described, but fails literally on a third one, both Z pairs off shell on the H peak. (d) misses by about 1σ of the difference, with no consistent sign. The validate pair moves 2 cells for the split (ee_to_mumu_tata_qcd0, pass → pass) and 47 for the forced-on floor (all pass → pass, 41 at the ulp)."
status: draft
tags: [two-to-six, multichannel, heavy-tail, budget-ladder, higgs, alpha-survey, phase-space, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-30}
measured:
  host: "cloud container, 4 vCPU, linux-64, 2026-10-29/30, one long job at a time (exceptions noted in Method)"
  commit: "change 91c3e6d (tree 6726f450); base 4479e6a (tree 993b2938). Ladder arms: binary sha256 1fdffd4e… (91c3e6d + T1's capture patch). Control: T1's binary sha256 f6180b60… (537f623 + capture patch; validate_sigma sources identical to 4479e6a). Probe: sha256 ee8341b1… (91c3e6d lib tests)."
  command: "see Method; TSVs, traces, records and scripts under the session scratchpad, prefixed ast-F1-"
sources:
  - {id: brief, resource: "F1.md", title: "F1 brief"}
  - {id: t1, resource: "T1-report.md", title: "T1 report"}
  - {id: dc, resource: "../../../../../vibegraph-lib/src/phasespace/diagram_channel.rs", title: "with_symmetric_pairs, either_order_measure"}
  - {id: maps, resource: "../../../../../vibegraph-lib/src/phasespace/maps.rs", title: "PairOrder"}
---
## Verdict

- **The split.** A split whose two daughters both carry a finite-width resonance
  draws either daughter first, each with probability ½, inside its one channel.
  - The split's first coordinate picks the order, and the density is the mean of
    the two orders' densities, so the weight stays exact.
  - It is a banked map choice, `PairOrder`: `symmetric` by the rule, `ordered`
    in `LEGACY`, and `--map-pair-order` on the CLI. The artifact format moves to
    15.
  - It changes 39 of bbx's 615 channels (all 15 H → ZZ* channels, plus 24
    `b b̄ → H* → Z Z` trees) and 3 of uux's 579 (the ZH → ZZ* channels). It also
    changes channel 16 of `ee_to_mumu_tata_qcd0` and its variants, of
    `var_sde1`, and of `z_to_uuxddx_decay`. No other banked SM row has such a
    split.
- **The α floor.** `set_alpha_floor` on both integrands; off by default.
  `VG_2TO6_ALPHA_FLOOR` and `VG_TAIL_ALPHA_FLOOR` in the probes.
- **The arithmetic agrees.** Through the implemented code, T1's eight Step 5
  points land on its counterfactuals. Seed 55's point in channel 613 falls
  ×1/203 (T1: ×1/197): 1.18e-8 → 5.8e-11 pb. The Z4 points fall with the floor
  ×1/3.6 … ×1/125, as T1 projected.
- **The ladder** (bbx, 20 paired seeds):

  | arm | sd 600k → 1.2M | ratio | χ²/dof 600k / 1.2M |
  |---|---|---|---|
  | control | 2.714e-9 → 3.330e-9 | 1.227 | 0.80 / 1.41 |
  | split | 2.409e-9 → 1.888e-9 | 0.784 | 1.09 / 1.18 |
  | split + floor 1e-4 | 1.696e-9 → 1.201e-9 | 0.708 | 0.69 / 0.66 |

  T1 projected 0.91 for both fixes together. That projection was an upper bound,
  and the measured ratio sits below it.
- **T1's falsifier.**
  - **(a)** Literally, it fails on both arms: 3–7 H-mirror points per rung
    remain above 1e-3, the largest at 0.353 (floor arm, 1.2M, seed 165,
    channel 604). Every one of them has *both* Z pairs off shell
    (m = 39–76 GeV) on the H peak, a population T1's class lumped in. The
    points T1 described, the second-drawn pair on shell, are gone: none above
    1e-3 in either arm at either rung, against 101 and 54 in the control.
  - **(b)** Holds: 613 ×0.522 / ×0.480, 614 ×0.524 / ×0.506, log-ratio sd
    0.10–0.40.
  - **(c)** Holds on both arms (split by 0.02 in χ²).
  - **(d)** Misses as written. Split − control is +9.5e-10 at 600k and
    −1.03e-9 at 1.2M, t ±1.1 of the difference's error, opposite signs.
    Floor − control is +1.49e-9 (t +1.80) at 600k and −5.5e-10 at 1.2M.
  - So the mechanism is right and the next step is not the heavy-tail error
    model. The remaining tail is the H peak with both Z's off shell
    (Found 1). The floor arm's χ²/dof of 0.66–0.69 means its quoted errors
    overstate the seed spread by about 20%.
- **uux** (5 seeds per arm). σ does not move: split − control t −0.29 and
  +0.35. The control's 1.2M rung has one low seed (seed 11, 3.6e-9 below the
  others' mean), which gives χ²/dof 4.09 and sd ratio 1.44. The split rung
  reads χ²/dof 1.79 and ratio 1.08. Five seeds cannot calibrate a rung ratio,
  so this is not a finding about uux's ladder.
- **The validate pair.**
  - Split: 223 cells identical, 2 moved, both `ee_to_mumu_tata_qcd0`, both
    pass → pass. The integrals pull goes +1.065 → +0.968 over its 5 seeds. The
    samples min p goes 2.2e-2 → 1.2e-2; five seeds read 1.17e-2, 117× the
    floor.
  - The forced-on α floor: 47 moved, all pass → pass. 41 of them moved at
    1e-14 or less (renormalisation ulps). The real moves are `gg_to_gg`,
    `gg_to_gg_cg` and `uux_to_uux`, at 1e-5 to 4e-4.

## Step 1: the split

**Design: either order inside one channel.** A split whose two daughters both
carry a finite-width resonance now draws either daughter first with probability
½ (`DiagramChannel::with_symmetric_pairs`, `diagram_channel.rs`):

- The split's first coordinate picks the order: below ½ the left daughter is
  drawn first, from `2x`; otherwise the right one, from `2x − 1`. The second
  coordinate draws the other daughter over the range the first leaves. The
  dimension is unchanged.
- The density is the mean of the two orders' densities
  (`either_order_measure`): `1/m = ½ (1/m_LR + 1/m_RL)`, with each order's
  measure the product of its two Breit–Wigner measures over that order's
  ranges, and a forced window read per order. The sampler's weight is computed
  from the drawn invariants by the same function, so weight and density stay
  exact reciprocals.
- It is a map choice. `PairOrder { Ordered, Symmetric }` sits in `MapChoices`
  beside the angle, `τ` and rung-order choices: `symmetric` by the rule,
  `ordered` in `MapChoices::LEGACY`, `--map-pair-order auto|ordered|symmetric` on
  `vibegraph integrate`. The artifact banks the choice, so its format moves
  14 → 15 (a v14 artifact is refused by version, as v13 was before).
- `ProcessShape` counts the splits (`resonant_pair_splits`) and the
  phase-space-maps log line prints it.

**Why not mirror channels.** Mirror channels would add one channel per H → ZZ*
channel (15 on bbx, 3 on uux) and give each half its own α and its own grid.
That is MadEvent's design (below). Against it:

- every added channel costs `MIN_CHANNEL_NEVAL` per iteration under the floor
  and one more density per point in every channel's mixture sum;
- the α survey would have to discover the mirror channels from the same
  30k × 6 survey that already starved the Z → 4f channels (T1 Found 2);
- channel indices would shift, so T1's record and the control would no longer
  line up channel by channel.

Inside one channel the cost per point is two more `invariant_measure`
evaluations per resonant pair in the density sweep, the channel count and the
α survey stay as they are, and the VEGAS grid on the first coordinate adapts the
two halves independently (each is half of that axis).

**Against MadEvent's coverage** (the coordinator's reading of MG 3.7.1 and this
row's output, `mg-hzz-coverage.md` in the scratchpad; not re-derived here):

| | MadEvent | this change |
|---|---|---|
| H → ZZ* halves | one integration channel per on/off assignment (`G122.203`, `G122.205`, plus both-off) | one channel, either order with probability ½ |
| separation | hard cut, \|m − M\| < 5Γ required / ≥ 5Γ excluded (`cut_bw`) | none; the mixture density is exact everywhere |
| off-shell pair | log-spaced (1/s) grid | its own Breit–Wigner over the range the on-shell pair leaves |
| draw order | both Z masses first, H last | parent first (fixed by the tree), then the pair |
| weight per half | separate channel, own share | one α for both halves |

Two consequences of the difference, both untested here:
- A Z* far below its pole has `|M|²` set by its own propagator, which the
  Breit–Wigner tail follows exactly. A 1/s map would follow a γ* component.
  The γ*(cc̄) and γ*(ll) diagrams have channels of their own here, so the
  Breit–Wigner is the matching shape for the H → ZZ* channels.
- With one α for both halves, a half the survey under-weights cannot be raised
  alone. MadEvent's per-half channels can, and its 1000-point floor per
  sub-channel guarantees each half points.

**Which splits.** The rule takes every split with two finite-width Breit–Wigner
daughters, not only H → ZZ*. Where both daughters can reach their poles the two
orders' densities nearly coincide and the change costs nothing. Only a final
state of four or more legs can have such a split. The census
(`census_resonant_pairs`, a throwaway test run once on the change's lib and not
committed; `ast-F1-census.txt`) builds every banked row's channels the way
production does, with the spine regulated:

```
$ ast-F1-census-bin --ignored --nocapture --exact hadronic::tail_probe::census_resonant_pairs
CENSUS bbx_to_ccx_emmm_qcd0 channels 615 with-pair 39 splits 42 | [288, 289, 290, 291, 292, 293, 294, 295, 296, 313, 314, 315, 316, 317, 318, 319, 320, 321, 322, 323, 324, 325, 342, 343, 344, 345, 346, 395, 396, 461, 462, 527, 528, 561, 562, 563, 564, 565, 566]
CENSUS ee_to_mumu_tata_qcd0 channels 25 with-pair 1 splits 1 | [16]
  (and its control_2026080{1,2,3}, hanti, hwindow variants: the same)
CENSUS uux_to_ccx_emmm_qcd0 channels 579 with-pair 3 splits 6 | [288, 305, 322]
CENSUS var_sde1 channels 25 with-pair 1 splits 1 | [16]
CENSUS z_to_uuxddx_decay channels 17 with-pair 1 splits 1 | [16]
every other SM row: with-pair 0; the NP/toy rows (ee_to_mumu_4f, *_smeft, *_dipole, gg_to_h_*, the toys) are 2 → ≤ 3 and cannot have one
census-exit 0
```

On bbx all 15 H → ZZ* channels change (325, 461, 462, 563, 564, 296, 395, 396,
561, 562, 346, 527, 528, 565, 566); 296, 325 and 346 change twice, at the root
Z H split as well. The other 24 (288–295, 313–324, 342–345) are `b b̄ → H*`
diagrams at `√ŝ` = 500 GeV whose root splits into two Z lines, one of them
decaying to four fermions; both Z's can sit on their poles there, so the two
orders nearly coincide (diagram content from the probe's `C` lines). The spine
channels T1 named (613, 614, 598, 604, 606, 582, 574, 590) do not change: their
pairs hang off the spine as separate blobs. On uux the three ZH → ZZ* channels change, each at the
root Z H split and at H → ZZ*. A first census pass that built the channels
unregulated (no spine, so every diagram a timelike tree) counted 99 bbx and 63
uux channels; that is not the production decomposition
(`ast-F1-census-unregulated.txt`).

**The density test.** `an_either_order_split_has_the_mean_density_of_the_two_orders`
builds H → Z Z* at rest (125 GeV, two Z pairs) and, at 20 000 points the
either-order map draws, compares its density with the mean of the *ordered*
map's density at the point and at the point with the pairs exchanged. That is
the brute-force mixture of the two orders, each from the existing code. It
fails if either order is left out: the ordered map alone differs from the mean
by more than ×100 on the half it leaves in a tail, which the test also
asserts. The two agree to 1e-9 relative, not to the bit, because the exchanged
configuration re-forms every subsystem sum in another order (a first version
at 1e-12 failed on 8e-12). The other tests:
- `an_either_order_split_keeps_the_phase_space_volume`: `V_4` at 125 and
  500 GeV, 1M points, within 6σ;
- `an_either_order_split_respects_forced_windows`;
- `only_a_resonant_pair_is_drawn_in_either_order`: a channel with one
  resonance is bit-identical with and without the installer; map identity and
  key separate the installed pair.

## Step 2: the α floor

`floored_alphas(alphas, floor)` (`phasespace/channel.rs`) raises every α to at
least the floor and renormalises; a weight that sat on the floor ends a little
below it, as in T1's counterfactual. Both integrands take it through
`set_alpha_floor(Option<f64>)`, applied after the survey in
`FixedBeamIntegrand::use_multichannel` and `ProtonIntegrand::adapt_alphas`.
The floored vector is pushed onto the survey's trajectory, so the report's last
α is the one installed. The default, `None`, installs the survey's weights
unchanged. Weights installed from a completed run
(`use_multichannel_with_alphas`, `set_channel_alphas`) are taken as given. The
probes:
- `VG_2TO6_ALPHA_FLOOR` in `probe_2to6_budget_ladder`, threaded through
  `with_integrand_parts`;
- `VG_TAIL_ALPHA_FLOOR` in `probe_2to6_tail_points`, on a survey's weights and
  on a trace's weights alike, so a trace from a floored run is replayed without
  it.

The multiplicity sum (`MultiplicitySum`) has no setter of its own: its parts are
`ProtonIntegrand`s, and nothing yet asks for the floor there.

## Step 3: the arithmetic, through the implemented code

The probe built from the change (`ast-F1-lib-probe`, sha256 `ee8341b1…`) installs
the control's α vector from T1's trace and re-evaluates T1's recorded points at
the same coordinates (`R` lines: weight as recorded by the control binary, and
now). The "both" runs add `VG_TAIL_ALPHA_FLOOR=1e-4`, which floors the traced α
with the production function (`floored_alphas`).

```
$ T="VG_TAIL_TRACE=ast-T1-ladder-trace.txt VG_TAIL_RECORDED=ast-T1-ladder-trace.txt.points"
$ ast-F1-probe.sh chk-split-12 55,22,176,11,20260719 1200000 613,614,604,606,425,574 $T
$ ast-F1-probe.sh chk-both-12  55,22,176,11,20260719 1200000 613,614,604,606,425,574 $T VG_TAIL_ALPHA_FLOOR=1e-4
$ ast-F1-probe.sh chk-split-6 66,33 600000 598,586 $T ;  ast-F1-probe.sh chk-both-6 … VG_TAIL_ALPHA_FLOOR=1e-4
   (all four: exit 0)
$ python3 -I ast-F1-scripts/check.py ast-F1-probe-chk-split-12.txt 55:1200000:613:3:1.180e-8 …
```

The rank-0 point of each named iteration. "momenta identical" compares the
point's outgoing momenta with T1's decomposition of it, digit for digit: the
recorded coordinates land on the same configuration, because none of these
channels' own maps changed.

| seed, ch, it | class | T1 now | T1 projected: split / floor / both | measured split | measured both | weight ratio split (both) | momenta identical |
|---|---|---|---|---|---|---|---|
| 55, 613, 3 | H-mirror | 1.180e-8 | 6.0e-11 / 1.18e-8 / 6.0e-11 | 5.81e-11 | 5.81e-11 | 1/203.2 (1/203.3) | yes |
| 22, 614, 6 | H-mirror | 4.682e-9 | 1.6e-11 / 4.68e-9 / 1.6e-11 | 1.53e-11 | 1.53e-11 | 1/306.9 | yes |
| 176, 604, 4 | H-mirror | 3.574e-9 | 2.7e-11 / 3.55e-9 / 2.7e-11 | 2.63e-11 | 2.63e-11 | 1/136.0 | yes |
| 66, 598, 4 (600k) | H-mirror | 2.912e-9 | 1.7e-11 / 2.91e-9 / 1.7e-11 | 1.65e-11 | 1.65e-11 | 1/176.1 | yes |
| 11, 606, 7 | Z4 | 2.497e-9 | 2.50e-9 / 5.8e-10 / 5.8e-10 | 2.50e-9 | 5.80e-10 | 1.000 (1/4.3) | yes |
| 20260719, 425, 3 | Z4 | 1.524e-9 | 1.52e-9 / 1.2e-11 / 1.2e-11 | 1.52e-9 | 1.22e-11 | 1.000 (1/125.4) | yes |
| 20260719, 574, 6 | Z4 | 1.130e-9 | 1.13e-9 / 3.1e-10 / 3.1e-10 | 1.13e-9 | 3.15e-10 | 1.000 (1/3.6) | yes |
| 33, 586, 7 (600k) | Z4 | 2.961e-9 | 2.96e-9 / 4.6e-11 / 4.6e-11 | 2.96e-9 | 4.62e-11 | 1.000 (1/64.1) | yes |

Measured contributions are T1's contribution times the measured weight ratio.
Seed 55's point: the mixture density rises 2.357e-17 → 4.791e-15 (×203; T1
projected ×197) and channel 613's own share falls 5.3e-3 → 2.6e-5. The 3% beyond
T1's projection is consistent with the two things T1's counterfactual left out:
it took the invariants' lower edges as 0, and it symmetrised only the H → ZZ*
splits, not the root Z H splits of 296, 325 and 346 that the rule also takes.
**The arithmetic agrees with T1's counterfactuals on all eight points**, so the
ladder went ahead.

## Step 4: the ladder

`bbx_to_ccx_emmm_qcd0`, multi sampler, ByAlpha, 8 iterations, B1's 20 seeds
(`55, 20260719, 11, 22 … 209`), 600k and 1.2M, one seed per process, detached
(`ast-F1-run.sh`, `ast-F1-chain.sh`), one job at a time on the 4-core container.
Each arm's binary carries T1's capture patch (uncommitted), so the arms record
the same trace and top points as the control.

- **control**: T1's ladder, from T1's binary (`ast-T1-validate_sigma-instr`,
  sha256 `f6180b60…`, built from 537f623 + capture patch). The
  `validate_sigma` binary's sources at 537f623 and at this session's base 4479e6a
  are identical (the commits between add the `#[cfg(test)]` tail probe and kb
  pages only), and T1's runs were made with this kept binary, so it is the
  control "built once from the base and kept". It was not re-run: rerunning the
  same binary reproduces it (T1 Step 1), and 12 h of container time went to
  the arms instead.
- **split**: 91c3e6d + capture patch, `ast-F1-validate_sigma-split`, sha256
  `1fdffd4e…`, 2026-10-29T15:53Z – 10-30T02:53Z.
- **split + floor**: the same binary with `VG_2TO6_ALPHA_FLOOR=1e-4`,
  10-30T02:53Z – 14:10Z.

```
$ python3 -I ast-F1-scripts/ladder.py bbx 600000 1200000 control=ast-T1-ladder.tsv split=ast-F1-split.tsv floor=ast-F1-floor.tsv
```

| arm | rung | mean σ (pb) | error of the mean | RMS quoted | per-seed sd | χ²/dof | points / iteration | wall / seed |
|---|---|---|---|---|---|---|---|---|
| control | 600k | 1.690011e-6 | 6.95e-10 | 3.109e-9 | 2.714e-9 | 0.80 | 1 630 578 | 802 s |
| control | 1.2M | 1.691689e-6 | 8.19e-10 | 3.662e-9 | 3.330e-9 | 1.41 | 2 217 845 | 1433 s |
| split | 600k | 1.690961e-6 | 5.39e-10 | 2.391e-9 | 2.409e-9 | 1.09 | 1 627 874 | 761 s |
| split | 1.2M | 1.690657e-6 | 4.83e-10 | 2.160e-9 | 1.888e-9 | 1.18 | 2 214 793 | 1218 s |
| split + floor | 600k | 1.691503e-6 | 4.48e-10 | 2.004e-9 | 1.696e-9 | 0.69 | 1 575 597 | 829 s |
| split + floor | 1.2M | 1.691141e-6 | 3.31e-10 | 1.480e-9 | 1.201e-9 | 0.66 | 2 133 482 | 1200 s |

The error of the mean is max(RMS quoted, sd)/√20. χ²/dof is the seeds about
their own mean with their quoted errors.

| arm | sd ratio 600k → 1.2M | paired Δ (1.2M − 600k) | sd of Δ | t | corr |
|---|---|---|---|---|---|
| control | 1.227 | +1.678e-9 | 4.502e-9 | +1.67 | −0.10 |
| split | 0.784 | −3.04e-10 | 3.062e-9 | −0.44 | −0.00 |
| split + floor | 0.708 | −3.62e-10 | 1.400e-9 | −1.16 | +0.58 |

| difference | 600k | 1.2M |
|---|---|---|
| split − control | +9.50e-10 (t +1.08) | −1.03e-9 (t −1.09) |
| floor − control | +1.49e-9 (t +1.80) | −5.48e-10 (t −0.62) |
| floor − split | +5.42e-10 (t +0.77) | +4.84e-10 (t +0.83) |

(t: the difference over the two means' errors in quadrature.)

Walls: the control's were measured by T1 with probe runs overlapping, so they
are not comparable to the arms'. The uux pair below ran both arms in this
session under the same load: 831 s against 829 s a seed at 1.2M, 555 s against
525 s at 600k. The either-order density costs nothing measurable. The floor arm
draws 4% fewer points. Renormalising after the floor takes α from the
share-allocated channels. Most of it goes to channels whose share stays under
their per-channel point floor, which draw the floor either way. Seed 11 at 1.2M (split arm)
overlapped a 3.5-minute compile (the census build); its wall, 1281 s, is in the
mean.

**Which channels carry the variance now** (`rank.py`, T1's script, on each
arm's trace):

```
split, 1.2M: 582 holds 0.516 of the quoted variance (one excursion, 6.4e-9 pb on seed 20260719); then 606 0.095, 590 0.054, 598 0.051, 574 0.050, 614 0.044
split, 600k: 606 0.158, 582 0.128, 590 0.122, 598 0.107, 574 0.086, 551 0.067, 614 0.065, 346 0.061
floor, 1.2M: 574 0.115, 346 0.110, 582 0.108, 606 0.096, 590 0.093, 614 0.088 (no channel above 0.12)
floor, 600k: 574 0.124, 582 0.119, 590 0.116, 606 0.115, 598 0.097, 614 0.094, 346 0.055, 325 0.031
```

Channel 613, 52% of the control's quoted variance at 1.2M, is out of every
top list.

### T1's falsifier, per arm

**(a) The probe's record of 613, 614, 598 and 604: no H-mirror point with a
single-point share above 1e-3 of its iteration.** The arms' recorded top points
(8 per iteration, the four channels, 20 seeds, both rungs) were decomposed by
the change's probe against each arm's own trace and classified with T1's
`classify.py`. Every one of the 4 × 5 120 recorded points re-evaluates to its
recorded weight within 1e-9 (`R` lines). The share is the point's weight over
its iteration's summed weight, `fval/(I·n)` from the trace (`falsa.py`; on
the control it reproduces T1's 0.92 for seed 55's point).

```
$ python3 -I ast-F1-scripts/falsa.py <class> <trace> 613,614,598,604              (T1's H-mirror class)
$ python3 -I ast-F1-scripts/falsa.py <class> <trace> 613,614,598,604 1e-3 5       (second-drawn pair within 5 Γ_Z of its pole)
```

| arm | rung | H-mirror points above 1e-3 (largest share) | of them with the second pair within 5 Γ_Z |
|---|---|---|---|
| control | 1.2M | 105 (0.919, 613 seed 55) | 101 (0.919) |
| control | 600k | 56 (0.082, 598 seed 66) | 54 (0.082) |
| split | 1.2M | 3 (0.0106, 604 seed 154) | 0 (largest below 1e-3) |
| split | 600k | 3 (0.0071, 604 seed 33) | 0 (largest 6.6e-4) |
| split + floor | 1.2M | 7 (0.353, 604 seed 165) | 0 |
| split + floor | 600k | 3 (0.0075, 604 seed 33) | 0 (largest 3.5e-4) |

Read literally, **(a) fails on both arms**. What fails is not the population T1
described. T1's class marks a point H-mirror when the H channels' second-drawn
pair is *nearer* M_Z than the first. Every point still above 1e-3 has both
pairs far off shell, on the H peak: m(cc̄) = 39–49 GeV with m(ee) = 57–76 GeV
(604, 614), or m(cc̄) = 59, m(μμ) = 62 (598). For example, the floor arm's
0.353 point (seed 165, channel 604, iteration 3) has H at +6.5 Γ_H,
m(cc̄) = 43.1 (−29 Γ_Z) and m(ee) = 64.5 (−18.7 Γ_Z). Neither draw order puts
either pair near its pole there, so the either-order map does not cover it.
This is MadEvent's third sub-channel, "both off" (the coordinator's notes).
The H-mirror points with the second pair on shell, which carried the control's
tail, are gone: none above 1e-3 in either arm at either rung, against 101 and
54 in the control.

**(b) The geometric-mean ratio of 613's and 614's quoted variance ≤ 0.6, with
the log-ratio sd under 1.0** (`geo.py`, T1's script; points ×2.00 in every
channel):

| channel | control | split | split + floor |
|---|---|---|---|
| 613 | ×1.001 (2.41) | ×0.522 (0.40) | ×0.480 (0.35) |
| 614 | ×0.757 (1.43) | ×0.524 (0.26) | ×0.506 (0.10) |
| 604 | ×0.737 (1.83) | ×0.392 (0.91) | ×0.613 (1.16) |
| 598 | ×0.635 (0.71) | ×0.483 (0.32) | ×0.484 (0.07) |

**(b) holds on both arms.** 613 and 614 now scale as their points do (×0.5).
604 in the floor arm keeps a log-ratio sd above 1, consistent with the 0.353 both-off
point.

**(c) The per-seed sd ratio ≤ 0.85 and χ²/dof at 1.2M ≤ 1.2.**
- split: 0.784 and 1.18. **Holds**, χ² by 0.02.
- split + floor: 0.708 and 0.66. **Holds**. That is the 1/√2 the backlog item
  asks for. Its χ²/dof is 0.66 at 1.2M and 0.69 at 600k: the quoted errors
  overstate the seed spread by about 20% (RMS quoted 1.48e-9 against sd
  1.20e-9). AGENTS.md reads a χ²/dof far below 1 as a warning; here it is the
  quoted error still carrying the rare-hit variance estimate of the rows the
  floor now covers. Twenty seeds cannot calibrate it further.

**(d) σ moves by less than its 20-seed error at each rung.**
- split − control: +9.5e-10 at 600k, −1.03e-9 at 1.2M, opposite signs. Each is
  1.08–1.09 of the difference's error, and twice the split arm's own error of
  the mean. **Fails as written**, by 8–9%.
- floor − control: +1.49e-9 at 600k (t +1.80), −5.5e-10 at 1.2M (t −0.62).
  **Fails as written at 600k.**
- Against MadEvent's seed record (1.689940e-6 ± 2.565e-9): control 1.2M +0.66σ,
  split +0.26σ, floor +0.45σ.
- The control is the arm whose per-seed values carry the rare hits (seed 55's
  +1.18e-8 alone moves its 1.2M mean by 5.9e-10), and the sign flips between
  rungs. That reads as the control's tail moving its mean, not as a bias the fix
  introduced. A paired test does not apply: the arms' α surveys differ per seed,
  so the per-seed differences scatter at 3–4e-9.

**Reading.** (a) fails, but only on a third population, H on peak with both Z
pairs off shell, that T1's class lumped in. The population T1 named, the
second pair on shell, is gone. (b) holds. (c) holds, with the floor arm at
0.708. (d) misses by about 1σ of the difference, with no consistent sign. So
the mechanism was right, the fix works, and there is no case for the heavy-tail
error model on this row. The remaining tail is the both-off-shell H peak: one
point at 0.35 of its iteration in 40 floor-arm seed-runs (Found 1).

## Step 5: uux

`uux_to_ccx_emmm_qcd0`, same settings, seeds 20260719, 11, 22, 33, 44. Control:
T1's binary (`f6180b60…`). Split: `1fdffd4e…`. Both arms ran in this session,
10-30T14:10Z – 17:59Z. Watched channels 288, 305, 322, the three ZH → ZZ*
channels. In uux the census finds only these three changed, each at its root
Z H split and at H → ZZ*.

```
$ python3 -I ast-F1-scripts/ladder.py uux 600000 1200000 control=ast-F1-uux-control.tsv split=ast-F1-uux-split.tsv
control  600000 n= 5 mean 7.193570e-07 ± 7.415e-10 | RMS quoted 1.658e-09 | seed sd 1.384e-09 | chi2/dof 0.63 | points/iter 1558290 | wall 555 s
control 1200000 n= 5 mean 7.181705e-07 ± 8.939e-10 | RMS quoted 1.089e-09 | seed sd 1.999e-09 | chi2/dof 4.09 | points/iter 2144605 | wall 831 s
control 600000->1200000 over 5 paired seeds: sd ratio 1.444 | paired diff -1.186e-09 (sd 1.651e-09, t -1.61) | corr +0.58
split    600000 n= 5 mean 7.190768e-07 ± 6.206e-10 | RMS quoted 1.388e-09 | seed sd 9.853e-10 | chi2/dof 0.54 | points/iter 1559787 | wall 525 s
split   1200000 n= 5 mean 7.185219e-07 ± 4.735e-10 | RMS quoted 8.778e-10 | seed sd 1.059e-09 | chi2/dof 1.79 | points/iter 2146382 | wall 829 s
split   600000->1200000 over 5 paired seeds: sd ratio 1.075 | paired diff -5.549e-10 (sd 1.042e-09, t -1.19) | corr +0.48
split - control @ 600000: -2.801e-10 pb (-3.89e-04 rel), unpaired t -0.29
split - control @ 1200000: +3.514e-10 pb (+4.89e-04 rel), unpaired t +0.35
```

- **σ does not move** (t −0.29 and +0.35).
- **The ladder, read with five seeds, looks better under the split, and five
  seeds cannot say so.** The control's 1.2M rung has one low seed: seed 11 reads
  7.153e-7 ± 9.0e-10, 3.6e-9 below the mean of the other four (7.168–7.198e-7).
  That gives χ²/dof 4.09 and sd ratio 1.44. Under the split, seed 11 reads
  7.180e-7 ± 9.4e-10 and the rung's χ²/dof is 1.79. A low σ with an ordinary
  quoted error is the missed-region signature, but no top-point record was
  decomposed for uux, so it is not attributed. AGENTS.md's rule applies: five
  seeds detect a missed region, they do not calibrate a rung ratio.
- uux's MadEvent seed record is 7.1771e-7 ± 1.08e-9. Split 1.2M reads
  7.1852e-7 ± 4.7e-10, +0.68σ.

## Step 6: what the split moves

`pixi run --skip-deps validate`, detached, one at a time, on three trees. Each
report was copied out (`ast-F1-report-{base,split,floor}`) and compared cell by
cell with durations and host stripped (`cells.py`).

```
base:  4479e6a exported (git archive, reference data symlinked), 2026-10-29T14:26Z – 15:51Z
       VALIDATE EXIT 0 | 73 results, 2134 passed, 0 failed, 99 ignored | 74 rows × 4 = 296 cells: 216 measured (211 ✅, 5 ⚠️, 13 ⏳, 67 — / uncovered)
split: 91c3e6d, 2026-10-30T17:59Z – 19:06Z
       VALIDATE EXIT 0 | 73 results, 2138 passed, 0 failed, 99 ignored | same cell counts
floor: 91c3e6d + ast-F1-floor-forced.patch (the floor forced to 1e-4 in both integrands; sha256 590ed15d…), 19:06Z – 20:10Z, patch reverted after
       VALIDATE EXIT 0 | 73 results, 2138 passed, 0 failed, 99 ignored | same cell counts
$ python3 -I ast-F1-scripts/cells.py ast-F1-report-base ast-F1-report-split
integrals/ee_to_mumu_tata_qcd0: gate pass -> pass | pull +1.0652 -> +0.9678 | sigma 1.376268e-03 -> 1.376051e-03 (-1.58e-04) | chi2/dof 1.65 -> 1.52 | seeds moved 5/5, max |dsigma/sigma| 3.1e-04
samples/ee_to_mumu_tata_qcd0: gate pass -> pass | seeds moved 3/3, max |dsigma/sigma| 7.9e-03 | min_ks_p 2.203e-2 -> 1.170e-2, min_chi2_p 1.261e-1 -> 1.762e-1, min_scale_ks_p 2.80e-1 -> 1.19e-1
cells identical 223, moved 2
```

The first attempt at the base run used `CARGO_PROFILE_RELEASE_DEBUG_DEBUG=0`,
which cargo reads as a key of the `release` profile and refuses. It stopped
before building anything (`ast-F1-validate-base-attempt1.log`), and the run was
repeated without it. Every run used the profile's own `debug = 1`.

Both moved cells are the row whose channel 16 the census names. 223 identical
cells across two separate builds of the base and change sources show no
build-to-build drift here (T1 Found 4 did not recur).

- **integrals/ee_to_mumu_tata_qcd0** runs five seeds in the gate itself. All
  five moved, pull +1.07 → +0.97, pass → pass.
- **samples/ee_to_mumu_tata_qcd0** runs three generation seeds. Five, from the
  headroom probe on 91c3e6d:

  ```
  $ pixi run cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_samples -- --ignored --nocapture --exact probe_samples_p_floor_headroom
    ROW ee_to_mumu_tata_qcd0  mode gate | 3-seed min p 1.170e-2 (117.0x floor) | 5-seed min p 1.170e-2 (117.0x floor) on seed 0x5a4d0003 KS cos(ta-)
  HEADROOM samples P_FLOOR 1e-4 | 3 seeds over 42 gating rows: min p 1.573e-4 (1.57x) on ee_to_wpwm seed 0x5a4d0003 KS pt(w+) | 5 seeds: min p 1.573e-4 (1.57x) …
  test result: ok. 1 passed; … finished in 346.05s
  ```

  The suite-wide minimum, `ee_to_wpwm` at 1.57× the floor, is a cell the split
  does not move (identical in base and split; no resonant pair). It is recorded
  as Found 4, not attributed here.
- No moved cell flips, so no deciding pair needs 20 seeds.

**Long-tier gates that run an X → VV* split:**
- The 2 → 6 integrals cells (`validate-sigma-2to6`, bbx and uux) were not run
  at their plan budget. Their rows are what Steps 4 and 5 measured, at 20 and 5
  seeds.
- The MLM gates (`validate-mlm-sigma`, `validate-mlm-samples`) were not run:
  `pp_to_ll_0j2j_mlm`, `pp_to_llj_mlm*` and `pp_to_ttx_0j1j_mlm` have no
  resonant-pair split (census), so their maps are unchanged.
- The Higgs-window comparison (`the_higgs_pole_window_is_measured_against_madgraph`,
  whose `hwindow`/`hanti` rows carry channel 16) is in `validate` and passed
  on all three trees.
- `var_sde1` and `z_to_uuxddx_decay` change channel 16 too. Their integrals
  cells are evidence and uncovered respectively, and no gate integrates them.

**The forced-on α floor (for D1).** 47 cells move, all pass → pass:

```
$ python3 -I ast-F1-scripts/cells.py ast-F1-report-split ast-F1-report-floor
integrals/gg_to_gg:   pull +1.7642 -> +1.7578 | sigma -2.86e-06 | seeds moved 5/5, max 1.2e-05
integrals/gg_to_gg_cg: pull +0.6784 -> +0.6790 | sigma +4.44e-06 | seeds moved 5/5, max 3.4e-05
integrals/uux_to_uux: pull +0.4782 -> +0.4841 | sigma +3.07e-06 | seeds moved 5/5, max 1.8e-05
samples/gg_to_gg:     min_ks_p 1.111e-1 -> 1.042e-1, min_chi2_p 1.401e-1 -> 1.294e-1 (max per-seed sigma 3.9e-04)
samples/gg_to_gg_cg:  min_ks_p 1.609e-2 -> 1.570e-2, min_chi2_p 3.804e-2 -> 3.884e-2 (3.2e-04)
samples/uux_to_uux:   min_ks_p 2.986e-1 -> 3.041e-1, min_chi2_p 3.655e-1 -> 3.712e-1 (1.8e-04)
41 more cells (integrals and samples of ddx_to_epemg, ee_to_ee*, ee_to_mumu_tata_qcd0, ee_to_mumua, ee_to_wp0wm*, ee_to_wpwm*, gg_to_ttx*, gu_to_epemu,
   p3r3 toys, pp_to_bb_*, pp_to_jj*, pp_to_ll_*, pp_to_llj*, qqx toy, ud_to_epemud_qcd0, uux_to_epemg, uux_to_ztg): |dsigma/sigma| <= 2.1e-14, gate statistics equal to 1e-13
cells identical 178, moved 47
```

The 41 ulp moves are the renormalisation: where no α sits below the floor, the
raised set sums to 1 within an ulp, and dividing by it moves every α by about
1e-16. A floor that skipped the renormalisation when nothing was raised would
leave those cells bit-identical. That is a choice for D1, not made here. The
three real moves are the rows whose survey drives a channel under 1e-4.

## Method

- **Worktree** `/home/user/wt/alpha-share-tails`, branch `alpha-share-tails`,
  base 4479e6a; change 91c3e6d; this report and the rule's documentation in
  the commit after it.
- **Builds** (`ast-F1-build.sh`): the change with T1's capture patch applied,
  built (`validate_sigma`, release-debug, debug = 0, as T1's control) and copied
  to `ast-F1-validate_sigma-split` (sha256 `1fdffd4e42d6fed95071d5f063121f2933a782291cb1f7bdf3fd2a1ba1a19117`).
  The patch was then reverted (the tree's diff checked equal before and after)
  and the lib tests built for the probe, `ast-F1-lib-probe` (sha256
  `ee8341b1fdac12ba0f46fc3abcd853bc6b77bc672383862d65b618f7d03ea24d`). The
  binaries were built before `docs/src/cli/reference.md` was regenerated, from
  sources identical to 91c3e6d.
- **Control**: `ast-T1-validate_sigma-instr`, sha256
  `f6180b602ac87792e352e446bbfaed97a8b06915adc10bff1f279c11a9a054f0`. Its bbx
  data are T1's (`ast-T1-ladder.tsv`, `-trace.txt`); its uux runs are this
  session's.
- **Ladder** (`ast-F1-chain.sh` → `ast-F1-run.sh`): one process per seed, the
  arm's TSV skipping finished seeds, `nohup setsid`, 10-29T15:53Z – 10-30T17:59Z
  with no restart. Exceptions to one job at a time:
  - the census build (16:35–16:39Z on 10-29) overlapped split seed 11 at 1.2M;
  - the arms' record decomposition (4 probe runs, 14:10–14:13Z on 10-30)
    overlapped uux-control seed 20260719 at 1.2M.
- **Scripts** in `ast-F1-scripts/`:
  - `ladder.py`: the arm tables;
  - `check.py`: Step 3;
  - `falsa.py`: falsifier (a), whose share reproduces T1's 0.919 for seed 55;
  - `cells.py`: the validate comparison.

  T1's `rank.py`, `robust.py`, `geo.py` and `classify.py` were used unchanged.
- **Census**: a throwaway `#[ignore]`d test appended to the tail probe, built
  once into `ast-F1-census-bin`, run, and removed (`git diff` empty after);
  output `ast-F1-census.txt`.

## Gates

On 91c3e6d's code (the gates ran before the CLI chapter was regenerated and
again after), detached, with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0` (`ast-F1-gates.log`, `ast-F1-tests.log`):

```
$ pixi run cargo fmt --all --check                                   --> exit 0
$ pixi run cargo clippy --workspace --all-targets -- -D warnings     --> exit 0
$ pixi run cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings   --> exit 0
$ pixi run cargo test --workspace                                    --> exit 101: cli_reference_docs, the committed CLI chapter lacked --map-pair-order
$ pixi run bash scripts/gen-cli-docs.sh                              (docs/src/cli/reference.md +12 lines)
$ pixi run cargo test --workspace --no-fail-fast                     --> exit 0; 48 results, 1708 passed, 0 failed, 20 ignored
$ pixi run --skip-deps kb-lint                                       --> exit 0
$ pixi run --skip-deps validate                                      --> base, split and forced floor all exit 0 (Step 6)
```

The report commit (documentation of the rule plus this file) reran
`fmt --check`, both clippy configurations, `cargo test -p vibegraph-lib --lib
phasespace::maps` and `kb-lint`; see the commit's log `ast-F1-gates2.log`.

## Brief corrections

- **"control (no change, built once from the base and kept)".** The control is
  T1's kept binary and T1's bbx ladder, not a rebuild. The `validate_sigma`
  sources are identical between T1's 537f623 and this base, and a rebuild could
  give a different α vector (T1 Found 4). Within one binary a seed reproduces to
  every printed digit (T1 Step 1), so a rerun would have spent 12 h for no new
  information.
- **"Apply it to every X → VV* split where both daughters are drawn as
  Breit–Wigners."** In production that is 39 bbx channels, not 15. Most
  `X → VV*` structures on the spine channels are not splits: their pairs are
  separate blobs. A census of the unregulated (timelike-tree) decomposition
  over-counts 99; the regulated one is the production map.
- **Falsifier (a) as worded** counts a third population, the H peak with both
  pairs off shell, that neither T1's mechanism nor this fix addresses. Read
  with the second pair within 5 Γ_Z of its pole, (a) holds with zero points.
- **Falsifier (d) "less than its 20-seed error"** is ambiguous between the
  arm's error of the mean and the difference's. Both are reported. The
  difference's error is the one a comparison of two independent 20-seed means
  needs, and against it the moves are 1.1σ (split) and 1.8σ (floor, 600k).

## Found

1. **The H peak with both Z pairs off shell has no map.**
   - What: neither draw order of an H → ZZ* channel puts a Z near its pole
     there, so the either-order map leaves it on two Breit–Wigner tails.
   - Evidence: every H-peak point still above 1e-3 of its iteration in the arms'
     records has m = 39–76 GeV for both pairs. The largest is 0.353 (floor arm,
     1.2M, seed 165, channel 604, iteration 3: H +6.5 Γ_H, m(cc̄) 43.1,
     m(ee) 64.5). Channel 604's log-ratio sd in the floor arm is 1.16.
   - To close: MadEvent's "both off" sub-channel (the coordinator's reading of
     `cut_bw`/`write_bash`), or a third component in the either-order mixture
     drawing both pairs on 1/s or flat maps. Measure it with (a) unrestricted on
     the same 20-seed ladder.
2. **The floor arm's quoted errors overstate its seed spread.** χ²/dof 0.69 and
   0.66 at 600k and 1.2M over 20 seeds; RMS quoted 1.48e-9 against sd 1.20e-9
   at 1.2M. To close: read the quoted error's per-channel terms against their
   seed spread in the floor arm's trace (`rank.py`'s quoted share against the
   across-seed share) before the floor's default is decided (D1).
3. **uux's control at 1.2M has a low seed with an ordinary error.** Seed 11 at
   7.153e-7 ± 9.0e-10, 3.6e-9 below the other four's mean (χ²/dof 4.09 over 5
   seeds), not low under the split (7.180e-7). To close: decompose its recorded
   top points (`ast-F1-uux-control-trace.txt.points`) with the probe
   (`VG_TAIL_DIR=uux_to_ccx_emmm_qcd0`), and run uux at 20 seeds if the split
   is to be credited with it.
4. **`ee_to_wpwm`'s samples gate sits at 1.57× its p floor** (KS pt(w+), seed
   0x5a4d0003, three and five seeds alike; `ee_to_wpwm_cw` 2.4×). This is
   unchanged by this session and identical in the base run. It was seen in the
   headroom probe, not investigated.
5. **The α floor renormalises when nothing is raised.** Forced on, it moves 41
   cells by an ulp each. To close, if D1 turns it on: skip the renormalisation
   when no α is below the floor, so rows the floor does not touch stay
   bit-identical.
