---
type: Measurement
title: "σ shifts from the AMP2 configuration draw and the per-group fix"
description: "Before/after σ of the clustered rows for the AMP2_c configuration draw (2026-08-03, branch val4-b) and for per-group, per-ordering scales (2026-09-26, 20 paired seeds against 14029ce)."
status: draft
tags: [scales, sigma, measurement, kt-clustering, hadronic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n29-b0out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5353-L5522", title: "Note 29 B-0 output (baseline and μ-spread census)"}
  - {id: n29-bres, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5523-L5756", title: "Note 29 Chain B results (val4-b)"}
  - {id: n40-5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L117-L144", title: "Note 40 §5 (σ rows, paired 20-seed sweep)"}
measured:
  - {commit: cd011a1, command: "probe_channel_partition_moves_sigma, probe_llj_parton_seed_stability, probe_llj_dyn_budget_ladder (extended-validation, --ignored); baseline 949ef6b on branch val4-b"}
  - {pr: 12, landed_in: 1539abc, command: "probe_dynamic_rows_seed_sweep with VG_SWEEP_SEEDS=20, on the per-group fix (branch pg-z1s) and on baseline 14029ce with the same seeds"}
---
# σ shifts from the AMP2 configuration draw and the per-group fix

Two changes to how the `dynamical_scale_choice = -1` scale picks its
clustering configuration, each measured before and after on the same seeds.
Together they are the evidence for
[clustering-configuration-draw](clustering-configuration-draw.md): the first
moved the scale off the sampler's channel onto an `AMP2_c` draw, the second made
the draw per flavour group and per beam ordering. The two sets were taken at
different commits and read against references of their time; they are kept side
by side because the comparison is the point. **The host is not recorded for
either set.** The gates they feed are [sigma-gate](../validation/sigma-gate.md);
how seed sweeps are read is
[seed-sweeps-and-budget-ladders](../validation/seed-sweeps-and-budget-ladders.md).

## 1. The `AMP2_c` draw (2026-08-03)

Measured on branch `val4-b`: the draw is commit `cd011a1`, the tolerance change
`02c6915`, against a baseline taken at `949ef6b` before any production line
changed. These are branch commits; the squash commit that carried them to
`main` is not recorded.[^bres]

**Pre-registered movement census.** Before the change, a probe
(`probe_cluster_scale_spread_over_configurations`, `validate_sigma.rs` and
`validate_hadronic.rs`) measured the worst relative spread of `μR` and both
`μF` over every configuration at 64 cut-passing points per clustered row.
Zero-spread rows were pre-registered bit-identical, the rest "may move":[^b0]

| row | `μR` spread over configurations |
|---|---|
| `gg_to_gg`, `gg_to_ttx`, `uux_to_uux`, `uux_to_epemg`, `ddx_to_epemg` | `0.000e0` |
| `pp_to_jj` | `0` within each of 8 groups; `5.0e-7` across groups |
| `gu_to_epemu`, `gux_to_epemux` | `9.961e-1` (4 configurations) |
| `pp_to_llj_dyn` | `8.2744` within group, equal across groups; four `g q` groups carry it, the two `q q̄` none |

At this commit nine further clustered fixed-beam rows (`ee_*`, `uux_to_mumu`)
compiled no per-event prescription because their matrix element carries no
`αs`. That is no longer so: every fixed-beam 2 → n clustered card now compiles
one. The may-move set was exactly `gu_to_epemu`, `gux_to_epemux`,
`pp_to_llj_dyn`.

**Result.** With the uniform drawn but unread, the validation report was
byte-identical over all 23 `integrals` and 22 `samples` cells. With the draw
live, exactly three `integrals` cells and two `samples` cells changed, the
pre-registered set; `pp_to_jj` (a live draw with zero within-group spread) came
out byte-identical.

Channel-partition gap (converged against uniform `αⱼ`, one seed;
`probe_channel_partition_moves_sigma`):

| row | gap before | gap after | Monte Carlo |
|---|---|---|---|
| `uux_to_epemg` | `+1.047e-3` | `+1.047e-3` (bit-identical) | `1.6e-3` |
| `ddx_to_epemg` | `+1.857e-3` | `+1.857e-3` (bit-identical) | `1.5e-3` |
| `gu_to_epemu` | `−1.484e-2` | `+1.867e-3` | `1.6e-3` |
| `gux_to_epemux` | `−1.528e-2` | `+1.493e-3` | `1.6e-3` |

Against MadGraph:

| row | rel before (pull) | rel after (pull) | reference's error |
|---|---|---|---|
| `gu_to_epemu` | `+1.076e-2` (`+5.21`) | `+3.98e-5` (`+0.02`) | `0.18 %` |
| `gux_to_epemux` | `+9.75e-3` (`+4.32`) | `−1.10e-3` (`−0.49`) | `0.20 %` |
| `pp_to_llj_dyn` | `−6.82e-3` (`−2.05`) | `−7.08e-5` (`−0.02`) | `0.33 %` |

Five seeds at the gate budget and at four times it
(`probe_llj_parton_seed_stability`), after:

| row | 1× mean / worst \|rel\| | 4× mean / worst | pulls | χ²/dof |
|---|---|---|---|---|
| `gu_to_epemu` | `+1.57e-4` / `1.35e-3` | `+7.53e-4` / `1.53e-3` | `≤ 0.65` | `0.58`–`1.25` |
| `gux_to_epemux` | `−8.69e-4` / `1.51e-3` | `−2.19e-4` / `1.20e-3` | `≤ 0.67` | `0.71`–`1.74` |

`pp_to_llj_dyn` budget ladder, five seeds a rung
(`probe_llj_dyn_budget_ladder`), read against MadGraph's `415.42 ± 1.36` pb
**of that time**; do not mix these numbers with the current reference quoted in
`validate_hadronic.rs`:

| `neval` | σ after (pb) | rel | pull | χ²/dof | σ before (pb) |
|---|---|---|---|---|---|
| 75 000 | `412.5969 ± 0.3617` | `−0.68 %` | `−2.00` | `6.38` | `409.55` |
| 150 000 | `414.2659 ± 0.2494` | `−0.28 %` | `−0.83` | `0.82` | `411.39` |
| 300 000 | `415.2694 ± 0.1733` | `−0.04 %` | `−0.11` | `0.65` | `412.53` |
| 600 000 | `415.7450 ± 0.1223` | `+0.08 %` | `+0.24` | `0.30` | `412.95` |

Before the draw the ladder asymptoted 0.6 % low; after, increments halve and
the row crosses the reference between the last two rungs. The χ²/dof of 6.38 at
75k is the draw's extra per-point variance at an under-budget rung.

**What followed from it.** All three residuals are Monte Carlo, so the
partition exemption was retired: `gu_to_epemu`/`gux_to_epemux` gate at
`rel_tol 0.005` (`validate_sigma.rs`), `LLJ_DYN_MAX_REL = 0.005`
(`validate_hadronic.rs`) with its pull asserted, and
`PULL_REPORTED_NOT_ASSERTED` now holds only `ee_to_mumua`, for an unrelated
reason. Each bound was set as the larger of the reference's own error with
headroom and the measured five-seed spread, not fitted to the central value.

**What this set could not see.** The group axis: at this commit all groups were
evaluated at one scale drawn in the sampled group, and the census showed the
group axis reaching no scale the within-group axis did not. That range statement
held; the conclusion drawn from it, that per-group scales were unnecessary, did
not (section 2). The mirror ordering was also untested here.

## 2. Per flavour group and per beam ordering (2026-09-26)

The fix of note 40: each group draws its own configuration, and a mirrored term
draws from `AMP2` at the rotated point and clusters the rotated event. Twenty
seeds per row at each row's enforced budget, run on the fixed code and on
`14029ce` with the same seeds; "paired shift" is new − old seed by seed. Rel
and pull are against MadGraph's banked σ of that time. Landed in PR #12
(`1539abc`).[^n40]

| row | before: rel, pull, χ²/dof | after: rel, pull, χ²/dof | paired shift |
|---|---|---|---|
| `pp_to_llj_dyn` | `+0.182 %`, `+0.55`, `1.06` | `+0.005 %`, `+0.01`, `1.25` | `−0.176 % ± 0.008 %` |
| `pp_to_llj` | `+0.087 %`, `+0.26`, `0.80` | `+0.033 %`, `+0.10`, `0.57` | `−0.054 % ± 0.029 %` |
| `pp_to_bb_qcd2` | `+0.007 %`, `+0.10`, `0.65` | `+0.009 %`, `+0.13`, `0.65` | `+0.002 % ± 0.001 %` |
| `pp_to_jj` | `+0.154 %`, `+0.70`, `1.34` | unchanged | 0 |
| `pp_to_bb` | `−0.005 %`, `−0.07`, `0.94` | unchanged | 0 |
| `pp_to_ll_scalefact2` | `−0.052 %`, `−0.26`, `0.46` | unchanged | 0 |

On the gates' own seeds `pp_to_llj_dyn` reads `−0.026 %` (pull `−0.08`, five
seeds) and `pp_to_llj` `−0.047 %` (pull `−0.13`, three seeds). No `rel_tol` or
gate mode changed. The paired shift is the latest measurement for
`pp_to_llj_dyn`; its earlier `+0.12 %` to `+0.18 %` residual, once attributed
to the reference's own 0.33 % error, was mostly the two defects. Rows whose
groups all cluster to one scale (every 2 → 2, and `pp_to_ll_scalefact2`) do not
move. The full banked layer on the merged tree had the same cells as before the
fix.

The per-event evidence for the same change (events replaying in their own
group, MadEvent's configuration frequencies, `SCALUP` per initial-state class)
is in [clustering-configuration-draw](clustering-configuration-draw.md); this
concept holds only the σ side.

[^bres]: Note 29 "Chain B results".
[^b0]: Note 29 "B-0 output", §B-0.1 and §B-0.2.
[^n40]: Note 40 §5.
