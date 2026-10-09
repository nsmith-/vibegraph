---
type: Caveat
title: "MadGraph's clustering scale depends on the integration channel"
description: "The -1 scale reads the integration channel (nqcd filter, checkbw, igraphs collapse, jet memo, stale isbw); what that means for replay and σ, and the jet-memo rule with its proof."
status: draft
tags: [kt-clustering, scales, channels, madevent, caveat]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n28-k111, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1268-L1298", title: "Note 28 §K1.11 (findings: the scale is not a pure function of momenta and process)"}
  - {id: n28-k32, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2065-L2137", title: "Note 28 §K3.2–K3.3 (consumed state; stale isbw)"}
  - {id: n28-k43, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2479-L2506", title: "Note 28 §K4.3 (replay channel search, per-run counts)"}
  - {id: n28-k5b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3090-L3174", title: "Note 28 §K5b.2–K5b.3 (the channel met in production)"}
  - {id: n28-k6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3331-L3524", title: "Note 28 §K6.3–K6.8 (per group; μR spread; the channel partition and its negative control)"}
  - {id: n28-c3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L3625-L3662", title: "Note 28 §C.3 (pp_to_jj: no partition residual on a 2 → 2)"}
  - {id: n41-12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L69-L104", title: "Note 41 §1.2 (setclscales under matching; the jet memo)"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 §4 M0 (jet-memo census)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 §4 M1 (jet-memo rule and proof)"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2701", title: "Note 41 §4 F-B (MadEvent's per-directory configurations)"}
  - {id: mg-cluster, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f", title: "MadGraph 3.7.1 cluster.f"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f", title: "MadGraph 3.7.1 reweight.f"}
---
# MadGraph's clustering scale depends on the integration channel

MadEvent computes the `dynamical_scale_choice = -1` scale inside the
integration channel `iconfig` that sampled the point. The same momenta can give
a different `μR` and `μF` in a different channel. An LHE record names no
channel, so a replay of banked events cannot always reproduce the scale from the
record alone. A cross section whose scale reads the channel also depends on how
the integrand is split into channels. vibegraph avoids that second problem by
drawing the configuration from the squared amplitudes
([clustering-configuration-draw](clustering-configuration-draw.md)). The
algorithm itself is [kt-clustering-algorithm](kt-clustering-algorithm.md); our
engine is [kt-clustering-engine](kt-clustering-engine.md).[^n28-k111]

## The routes by which the channel enters

| route | where | what it changes |
|---|---|---|
| coupling-order filter | [`cluster.f:359-366`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L359-L366) | configurations with `nqcd ≠ nqcd(this_config)` leave the merge graph |
| resonance tagging | `checkbw`, [`cluster.f:419-423`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L419-L423) | only `this_config`'s lines can be tagged on-shell, switching their measure to the invariant mass |
| graph collapse | [`cluster.f:809-817`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L809-L817) | `igraphs(1) = this_config` when it survives; every line PDG the walk reads comes from `igraphs(1)` |
| jet memo | [`reweight.f:662-679`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L662-L679), `:985-1030` | an event whose jet count disagrees with the channel's stored count is re-clustered restricted to the channel |
| `chcluster` | `cluster.f:466-470` | restricts the clustering to `iconfig` itself (forced on by the memo) |

All of them are live in the bank. `pp_to_bb_qcd2`'s `this_config = 3` sees only
the two `nqcd = 0` channels; `igraphs(1) ≠ iconfig` on 7 to 7877 events per
dumped run; the memo re-clustered 1873 dumped `pp_to_llj` events restricted.
The engine reproduces every one against the
[kt-cluster-dump-oracle](../validation/kt-cluster-dump-oracle.md).

The size of the effect is uneven. On the gluon-beam `2 → 3` rows
(`gu_to_epemu`, `gux_to_epemux`) `μR` at one drawn point differs by up to a
factor of two between channels (spread `9.93e-1`); on the annihilation rows
(`uux_to_epemg`, `ddx_to_epemg`) it is identically the same in every channel
(spread `0.000e0`). On `gu_to_epemu` the four configurations share one `NQCD`,
so there the spread comes from the different forests, not from the coupling-order
filter.

## Stale `isbw` across events

`isbw` is a common block. `checkbw` clears it only on the current channel's own
timelike lines, so a leg set tagged on-shell under one channel stays tagged when
the next event runs under another. Signature: a final-state pair measured by
invariant mass whose mask is not in that event's own `ibwlist`. Example,
`bbx_to_ccx_emmm_qcd0` event 442 at `this_config = 487`: mask 204 = `12 + 192`,
the `h → ZZ` line of a different channel, is measured as a resonance.

It fires on 81 events of `bbx_to_ccx_emmm_qcd0` and 163 of
`uux_to_ccx_emmm_qcd0` (0.8 % and 1.6 %), both `2 → 6` with three Z and an h;
no `2 → 2`, `2 → 3` or `2 → 4` run is affected. With the reference's own extra
flags passed in (`carried_on_shell` in `coupling/cluster/kt.rs`) all 244
reproduce completely: every candidate, merge, frame change and scale. So this
is a diagnosis, not a tolerance. In production the generator owns its state and
the pure-function reading is correct; only a replay of those two runs needs the
flags.[^n28-k32]

## Consequence for replaying banked events

Because the record carries no channel, the replay in
[scale-replay-gate](../validation/scale-replay-gate.md) searches: it adopts the
first channel whose `μF` lands inside `SCALUP`'s printing budget and reads every
other field, and the independent `AQCDUP` oracle, off that same channel, so a
wrong clustering cannot be repaired field by field. How often the first channel
is not that channel (out of 10 000 events per run):[^n28-k43]

| run | events needing another channel |
|---|---|
| `gu_to_epemu` / `gux_to_epemux` | 7204 / 7231 |
| `pp_to_llj_dyn` / `pp_to_llj` | 5768 / 5572 |
| `ee_to_mumua` | 370 |
| `ee_to_mumu_tata_qcd0` | 262 |
| `pp_to_bb_qcd2` | 141 |
| every other replayed run, `pp_to_jj` included | 0 |

The search accepts any channel that reproduces the record, so it is blind to
*which* channel MadEvent used. It cannot test a configuration-choice rule; that
is what
`madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`
(`tests/validate_hadronic.rs`) is for.

## Consequence for a cross section: the channel partition

With a channel-dependent scale, the multichannel estimator
`σ = Σⱼ ∫ dΦ f(p, j)·αⱼgⱼ(p)/g(p)` is no longer independent of the weights
`αⱼ`: they decide which scale a region is evaluated at, not just how often it is
visited. σ is then defined only up to the channel partition. Measured by
`probe_channel_partition_moves_sigma` (`tests/validate_sigma.rs`), integrating
the same row at converged and at uniform `αⱼ` while the scale read the sampled
channel:[^n28-k6]

| row | partition gap | Monte Carlo |
|---|---|---|
| `uux_to_epemg` | `+1.05e-3` | `1.6e-3` |
| `ddx_to_epemg` | `+1.86e-3` | `1.5e-3` |
| `gu_to_epemu` | `−1.48e-2` | `1.6e-3` |
| `gux_to_epemux` | `−1.53e-2` | `1.6e-3` |

The two rows whose scale is channel-independent are the negative control: their
gap sits at their own Monte Carlo error, the other two at 9σ. MadGraph's own σ
lay inside the interval our two partitions spanned, and MadEvent's partition is
a third one: single-diagram enhancement weights channel `c` by
`AMP2_c/Σ AMP2`, a function of the point that no constant `αⱼ` reproduces. That
is the reason the scale configuration is now drawn `∝ AMP2_c`: drawing the
conditional MadEvent's channel induces removes the partition from σ (the gap on
the two gluon rows fell to `+1.9e-3` and `+1.5e-3`). The partition argument is
the reason for the draw, not a tolerance; see
[configuration-draw-sigma-shifts](configuration-draw-sigma-shifts.md).

**A 2 → 2 has no merge to choose.** The terminal core is the event itself, so
`jlast`/`jcentral`, `μR` and both `μF` are functions of the momenta alone.
`probe_jj_channel_partition` (`tests/validate_hadronic.rs`) measures `pp_to_jj`'s
gap at its own Monte Carlo error, the smallest of any clustered row, against a
fixed-scale control on the same path:[^n28-c3]

| arm | partition gap | Monte Carlo |
|---|---|---|
| `j j`, as enumerated | `+1.77e-4` | `9.5e-4` |
| `j j`, permutations collapsed | `+1.03e-3` | `9.6e-4` |
| `pp_to_llj_fixed` (control) | `−1.32e-4` | `1.4e-3` |

So `pp_to_jj`'s tolerance is set by its reference error and seed spread.

## The jet memo: MadEvent's rule and ours

MadEvent's rule, per process (job) directory and configuration `iconfig`, with
`njetstore(iconfig)` starting at `−1`:

1. While the entry is `−1`, the clustering is restricted to `iconfig`
   (`reweight.f:662-665` forces `chcluster`). `:985-998` stores the number of
   final-state legs with `iqjets > 0` and re-clusters unrestricted.
2. Every later point compares its unrestricted jet count with the stored one;
   on a mismatch it re-clusters restricted to the channel (`stop 4` if that
   fails too, which never fired on any reference row).
3. The memo is consulted on every call that clusters: under `ickkw > 0`,
   `xqcut > 0`, or any dynamic scale. Only the `:643` early return skips it.
   One entry per directory configuration is shared by every subprocess of the
   directory (`reweight.f:588-592`).[^n41-fb]

The memo is **not MLM-specific**: the banked `ickkw = 0` `pp_to_llj` already
re-clusters 1857 of 10 000 events.

vibegraph starts the memo empty on every event and stores *this event's*
channel-restricted jet count (`ScaleChoice::cluster_scales`'s doc in
`coupling/scales.rs`). That equals MadEvent's value on every event exactly when
the restricted count is a property of the channel alone. It is plausible (a
restricted clustering follows the channel's forest) but the jet tagging reads
kinematics, so it was measured:
`the_jet_memo_is_the_channel_s_restricted_jet_count`
(`tests/validate_mlm_dumps.rs`) clusters every event of a directory restricted
to **every** channel of the directory on five reference rows. Each of the 135
channels (8 + 8 + 8 + 29 + 82) gives a single count over every event, every
census channel agrees, and per event the stored count equals the dumped
`njetstore` on entry and the restricted-re-cluster branch equals MadEvent's. No
code changed; `ickkw = 0` behaviour is the same rule.[^n41-m1] A harness must
fill `confsub` from the directory's own `config_subproc_map.inc`; filling it with
every channel for every subprocess produced spurious two-valued counts.

Census of how often the restricted branch fires (from the reference rows' M0
dumps, refdata-9 era):[^n41-m0]

| row | written events re-clustered | where |
|---|---|---|
| `pp_to_llj_mlm` | 1123 of 10 000 (11.2 %), both calls | the two `P1_gq_llq` channels whose memo holds 0 |
| `pp_to_llj_xqcut_only` | 1407 of 10 000 | same channels |
| `pp_to_ll_0j2j_mlm` | 661 of 10 000 | 270 `P1_gq_llq`; 391 two-jet events in channels whose memo holds 1 (2 for five) |
| `pp_to_ttx_0j1j_mlm` | 0 | every `P0` job stored 0 jets, every `P1` job 1 |

No job directory held two values for one channel. Under matching the memo moves
the matched weight by up to a factor of two on the affected events, so its rule
is load-bearing for [mlm-scales](mlm-scales.md); the per-event check is the
[mlm-dump-oracle](../validation/mlm-dump-oracle.md).

[^n28-k111]: Note 28 §K1.11 finding 2, confirmed in §K3.2 and §K3.7.
[^n28-k32]: Note 28 §K3.3.
[^n28-k43]: Note 28 §K4.3.
[^n28-k6]: Note 28 §K6.4–K6.5; the post-draw gaps are note 29 "Chain B results".
[^n28-c3]: Note 28 §C.3.
[^n41-fb]: Note 41 §4 F-B, "MadEvent's rule".
[^n41-m1]: Note 41 §4 M1, "The jet memo (M0's finding), rule and proof".
[^n41-m0]: Note 41 §4 M0, Census 1.
