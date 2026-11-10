---
type: Measurement
title: Configuration shares against MadEvent's per-channel cross sections
description: "Banked AMP2 shares (IntegrateArtifact::configurations) against MadEvent G-directory σ on four rows, five seeds a side: every share within 1.9σ, share χ²/n 0.08–0.80; the density share reads 3–2027σ off; integrate wall time unchanged within noise."
status: draft
tags: [validation, multichannel, configuration, amp2, madevent-parity, artifact]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
measured:
  - commit: 1d39fd6
    host: "cloud container, 4 vCPUs"
    command: "cargo test -p vibegraph --test cli_config_share --features extended-validation -- --nocapture --test-threads 1 (dev profile, opt-level 2)"
    data: "validation/madgraph/config_share_reference.json, written by WORK=<scratch> NB_CORE=2 pixi run -e madgraph --frozen bash validation/madgraph/gen_config_share.sh (MadGraph 3.7.1, seeds 20261110–20261114, one fresh directory per seed)"
  - commit: 1d39fd6
    host: "cloud container, 4 vCPUs"
    command: "release builds of a7c7061 (base) and this commit, alternating five times each: vibegraph integrate <card> --run-card validation/madgraph/config_share_<row>_run_card.dat --fixed-budget --neval 1000000 --niter 8 --seed 1, timed by bash `time` (all threads; host load average 4–6 from other sessions)"
---
# Configuration shares against MadEvent's per-channel cross sections

What is compared: σ_c, each integration configuration's share of σ as
[the integrate artifact](../pipeline/integrate-artifact.md#configuration-shares)
banks it, against the σ MadEvent writes in `SubProcesses/P*/G*/results.dat`.
MadEvent integrates configuration `c` in `G<c>` under the factor
`AMP2_c / Σ AMP2` (`sde_strategy = 1`, `tmin_for_channel = −1`;
[MadEvent's single-diagram enhancement](../phase-space/madevent-single-diagram-enhancement.md)),
summed over the `P` directory's subprocesses and beam orderings and over the
configurations `symfact.dat` folds into `G<c>`. The gate is
`vibegraph-cli/tests/cli_config_share.rs`; the three fixed-energy rows run in
`cargo test`, the proton row under `extended-validation` (it reads the fetched
NNPDF23 set).

## How the two sides are joined

- **Cards.** `validation/madgraph/config_share_<row>_run_card.dat`, MadGraph's
  generated default card with the beams (and, for `pp_eej`, the PDF and fixed
  scales) set, installed verbatim on both sides, `iseed` apart.
- **Channels by propagator content.** `gen_config_share.sh` banks each
  directory's `configs.inc` as per-channel propagator lists: the external legs
  on the far side of each line from leg 1, the particle's class (`a z w h g q
  l`; a grouped directory lists one t-channel code for all its subprocesses)
  and `s`/`t`. The test computes the same from each banked configuration's
  diagrams (in the enumerated set's leg order, beams exchanged where the set is
  a directory's `mirror` line) and maps a share onto the channel whose
  signature (or a symmetry-folded partner's) one of its diagrams carries. The
  map is asserted onto, and every share carrying weight lands on exactly one
  channel; no index is assumed.
- **Seeds.** Five on each side (here 11–55 at the budgets below; MadEvent one
  freshly generated directory per seed), each side read as its unweighted mean
  with error `max(quoted, spread/√5)` (`common/seeds.rs`).
- **What is gated.** Each channel's *fraction* of its own side's σ, pull at
  most 3.5, and `Σ pull²/n ≤ 3.3` per row; the absolute σ_c at pull 5. The
  totals of the two sides differ at MadEvent's campaign scatter (below), which
  moves every channel alike and belongs to the σ gates.

## Results

Budgets: fixed energy `--fixed-budget --neval 100000 --niter 6`; `pp_eej`
`--neval 200000 --niter 6`. "here" and "MadEvent" are seed means ± error in pb;
the share pull compares fractions of σ; χ²/dof is each side's seed scatter.
The density column is the known-wrong comparison: each sampling channel's own
term (`ChannelGrid::sigma_pb`, the density share), on the MadEvent channel its
configuration maps onto, pulled against MadEvent's (`-` where a merged channel
serves configurations of several MadEvent channels).

| row / channel | here | MadEvent | abs. pull | share here | share MadEvent | share pull | χ²/dof here/MG | density pull |
|---|---|---|--:|--:|--:|--:|--|--:|
| `e+ e- > mu+ mu-` G1 (γ) | 0.366231 ± 0.000073 | 0.366012 ± 0.000085 | 1.96 | 0.87243 | 0.87250 | −0.23 | 0.41/0.39 | −1490.7 |
| G2 (Z) | 0.053549 ± 0.000011 | 0.053488 ± 0.000032 | 1.81 | 0.12757 | 0.12751 | 0.75 | 0.42/0.48 | 2026.6 |
| `e+ e- > e+ e-` G1 (s γ) | 0.238317 ± 0.000689 | 0.237860 ± 0.000410 | 0.57 | 1.53e-3 | 1.53e-3 | 0.49 | 0.85/0.76 | −475.4 |
| G2 (s Z) | 0.034360 ± 0.000096 | 0.034340 ± 0.000050 | 0.19 | 2.20e-4 | 2.20e-4 | 0.10 | 0.82/0.69 | −36.7 |
| G3 (t γ/Z) | 155.562 ± 0.050 | 155.524 ± 0.056 | 0.51 | 0.99825 | 0.99827 | −0.04 | 0.57/1.31 | 3.4 |
| `e+ e- > u u~ g` G1 (γ; gluon off u, and off u~ folded) | 0.246916 ± 0.000165 | 0.246552 ± 0.000459 | 0.75 | 0.71390 | 0.71464 | −0.53 | 0.68/0.98 | −152.3 |
| G2 (Z; both) | 0.098953 ± 0.000068 | 0.098448 ± 0.000284 | 1.73 | 0.28610 | 0.28535 | 0.88 | 0.83/0.74 | 226.8 |
| `p p > e+ e- j` P1_gq_llq G1 | 19.4431 ± 0.0385 | 19.6436 ± 0.1174 | −1.62 | 0.07533 | 0.07622 | −1.88 | 1.50/0.77 | −51.1 |
| G2 | 30.4945 ± 0.0182 | 30.4266 ± 0.1179 | 0.57 | 0.11814 | 0.11807 | 0.16 | 0.29/0.70 | 100.3 |
| G3 | 27.5325 ± 0.0419 | 27.5394 ± 0.1599 | −0.04 | 0.10666 | 0.10686 | −0.31 | 1.08/0.18 | - |
| G4 | 115.6796 ± 0.0736 | 115.4500 ± 0.2642 | 0.84 | 0.44816 | 0.44799 | 0.16 | 1.69/0.01 | - |
| P1_qq_llg G1 | 3.8430 ± 0.0147 | 3.8202 ± 0.0247 | 0.80 | 0.01489 | 0.01482 | 0.58 | 0.63/0.11 | - |
| G2 | 30.7562 ± 0.0287 | 30.7146 ± 0.1093 | 0.37 | 0.11915 | 0.11918 | −0.07 | 0.91/0.29 | - |
| G3 | 3.5156 ± 0.0239 | 3.4525 ± 0.0350 | 1.49 | 0.01362 | 0.01340 | 1.27 | 2.32/1.34 | 11.0 |
| G4 | 26.8576 ± 0.0336 | 26.6622 ± 0.1972 | 0.98 | 0.10405 | 0.10346 | 0.86 | 0.42/2.16 | −13.0 |

Share χ²/n per row: `ee_mumu` 0.31 (2 channels), `ee_ee` 0.08 (3), `ee_uuxg`
0.53 (2), `pp_eej` 0.80 (8). Totals: `ee_mumu` 0.419781 here, 0.419500
MadEvent (spreads 0.000053, 0.000076); `ee_ee` 155.835 / 155.794; `ee_uuxg`
0.345870 / 0.345000; `pp_eej` 258.122 / 257.708 pb (spreads 0.082 / 0.340).
Every banked share with weight carries a configuration: no row records weight
on the unattributed entry, and every channel's members carry diagrams and a
share (asserted per seed). The sum rule holds per seed to 1e-12.

**MadEvent's own scatter.** `ee_mumu`'s MadEvent total here, 0.41950 ± 0.00008
(five fresh directories), sits 0.08% below
`grammar_sigma_reference.json`'s same process on the same card, 0.41982, and
this side's 0.41978 agrees with the latter. Both of `ee_mumu`'s absolute pulls
are +1.9 for that reason alone, while the fractions agree to 0.2σ and 0.8σ.
This is the effect [MadEvent's iteration weighting](madevent-iteration-weighting-bias.md)
and the [seed policy](madevent-reference-seed-policy.md) describe, and why the
gate reads fractions.

**The known-wrong comparison.** On `ee_mumu` the γ and Z channels' maps
coincide (a `2 → 2` at fixed `√s` draws angles only), α stays at 0.5 each and
the density share puts half of σ in each: 1490σ and 2027σ off MadEvent's
channels, where the AMP2 share is within 2σ. The test asserts that this
comparison reads more than 50σ off, so it fails if the gate ever compares the
density share. On `pp_eej` it is off by 11–100σ where it can be formed; on
Bhabha's t-channel, carrying 99.8% of σ, only 3.4σ.

## What the gate can and cannot see

- **Sees:** a configuration weighted by the wrong amplitude, dropped, double
  counted or attributed to the wrong set, configuration or channel, at
  0.02–1% of a channel (the share errors above). Swapping `ee_mumu`'s two
  shares would read thousands of σ off.
- **Cannot see:** a partition convention differing by less than that, for
  instance a `GET_CHANNEL_CUT` factor at per-mille level; and anything inside
  one MadEvent channel: `ee_uuxg`'s configurations with the gluon off the
  quark and off the antiquark are one `G` by MadEvent's symmetry, so only their
  sum is compared. The coupling the AMP2 ratio is read at matters only for a
  matrix element mixing αs orders at a running scale, which no row here has;
  `hadronic::tests::enhancement_weights_are_read_at_the_evaluated_coupling`
  pins it instead. `sde_strategy = 2` is not exercised by any row.
- **The sum rule** (Σ_c σ_c = σ) cannot see a share moved between two
  configurations; the per-channel comparison is what does.

## Cost of the accumulation

Release builds of the base (`a7c7061`) and of `1d39fd6`'s tree before a
clippy-driven refactor of `config_share::hadronic`'s bookkeeping (a tuple made
a struct; it runs once after the integration), same seed and
budget (`--fixed-budget --neval 1000000 --niter 8 --seed 1`), alternated five
times. The host is shared (load average 4–6), so wall times scatter by up to
2×; user CPU time is the steadier reading.

| card | base wall (median) | new wall (median) | base user (median) | new user (median) | user change |
|---|--:|--:|--:|--:|--:|
| `p p > e+ e- j` (6 groups, 30 shares) | 18.98 s | 21.22 s | 37.58 s | 41.59 s | +10.7% |
| `e+ e- > u u~ g` (1 set, 5 shares) | 15.42 s | 15.97 s | 54.55 s | 56.54 s | +3.6% |

Every pair printed the same σ, and on `p p > e+ e- j` the artifacts' σ, error
and every channel term are bit-identical between the builds (`inspect
--output-ldjson`): the split moves no value, and the chunk sizes rounded to
whole tiles move no grid. The cost is the `AMP2` read per group per point
(`filled_amp2` loops over every helicity row's configuration amplitudes) and
the per-point share bookkeeping; it is paid on every integration.
