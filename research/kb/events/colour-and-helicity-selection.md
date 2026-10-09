---
type: Algorithm
title: Per-event helicity and colour-flow selection
description: "Helicities summed and colours contracted while integrating; each accepted event draws a helicity, then a configuration (AMP2 or channel-cut weight), then a flow ∝ JAMP2 in its ICOLAMP row."
status: draft
tags: [events, colour, helicity, icolup, madevent-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n15-jamp2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/15-eval-optimization-plan.md#L365-L465", title: "Note 15 §2.2, the JAMP2 diagonal as SELECT_COLOR's input"}
  - {id: n21-helcol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L512-L544", title: "Note 21, helicity and colour handling"}
  - {id: n23-e1a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L49-L65", title: "Note 23 E1a, the JAMP2 diagonal requirements"}
  - {id: n23-e1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L130-L195", title: "Note 23 E1 outcome (eval_jamp2, flow tags)"}
  - {id: n23-e1c, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/23-event-output-lhef-plan.md#L196-L257", title: "Note 23 E1c, NCOLOR=6 JAMP comparison"}
  - {id: n27-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L298-L481", title: "Note 27 B3, MadEvent's colour selection as read"}
  - {id: n27-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L912-L1038", title: "Note 27 B6, the per-configuration AMP2 accumulator"}
  - {id: n27-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/27-v3-backlog-plan.md#L1158-L1181", title: "Note 27 §6 decisions D1/D4"}
  - {id: n36-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L355-L412", title: "Note 36 B4, ud_to_epemud_qcd0 ICOLUP diagnosis"}
  - {id: n36-b3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/36-banked-open-ends-plan.md#L567-L650", title: "Note 36 B3, MadGraph's channel set and the channel-cut weight"}
  - {id: mg-select-color, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L1087", title: "MadGraph 3.7.1 SELECT_COLOR"}
  - {id: mg-icolamp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064/madgraph/iolibs/export_v4.py#L1295", title: "MadGraph get_icolamp_lines"}
---

# Per-event helicity and colour-flow selection

## Summed while integrating, selected per event

The integrand never samples helicity or colour. `|M|²(p) = Σ_hel |M_hel(p)|²`
runs over the helicity-expanded arena with zero-helicity pruning (MadGraph's
`GOODHEL`), and the colour sum is the CF contraction inside it. The momentum
channels are driven by that full sum; there is no per-helicity channel[^n21-helcol].

Once a point is accepted (see [events/unweighting](unweighting.md)), the event
record still needs a helicity tuple (`SPINUP`) and a colour flow (`ICOLUP`).
Both are **selections**: categorical draws off diagonal accumulators of the
same evaluation, with no effect on σ or on the integrand. A caller may skip
them and integrate the same number.

MadEvent's other option, helicity Monte Carlo (`nhel = 1`: one adapted
helicity per point, `|M|²` divided by its pick probability), is not
implemented and is refused at parse; it is in scope as
[feature/nhel1-run-cards-refused](../backlog/feature/nhel1-run-cards-refused.md).
Sherpa-style Monte Carlo over colour is not planned.

## The three draws

`FixedBeamIntegrand::select_event` (`vibegraph-lib/src/hadronic.rs`) and
`ProtonIntegrand::select_event` (`vibegraph-lib/src/proton.rs`) take one
uniform per label. On the proton path a subprocess, a flavour member and a beam
ordering are drawn first ([events/generate](generate.md)); then, on both paths:

1. **Helicity** `∝ |M_c(p)|²` over the surviving combinations
   (`AmplitudeEvaluator::select_helicity`, `helas/eval/compile.rs:419`, MadGraph's
   `SELECT_HEL`). The weights come from `BoundAmplitude::eval_hel_m2`.
2. **Configuration** `c`, drawn from a per-configuration weight (below).
3. **Flow** `i ∝ JAMP2(i)` restricted to the flows configuration `c` reaches at
   leading colour (its `ICOLAMP` row), with a fallback to every flow when the
   restricted weights vanish (`select_flow_reached_by`,
   `helas/color/flow_tags.rs:307`).

Steps 2 and 3 are `AmplitudeEvaluator::select_config_and_flow`
(`helas/eval/compile.rs:467` is `select_color_flow`, which wraps it). It
returns the flow, the configuration, and whether the flow is leading in that
configuration; the last two decide the event's resonance records
([events/resonance-records](resonance-records.md)).

Steps 2–3 reproduce MadGraph 3.7.1's `SELECT_COLOR`[^mg-select-color]:

```fortran
cconfig = iconfig
if (ickkw.gt.0) then ... cconfig = igraphs(1) ...
do i=1,nc
  if(icolamp(i,cconfig,iproc))then
    targetamp(i) = targetamp(i-1) + jamp2(i)
  else
    targetamp(i) = targetamp(i-1)
  endif
enddo
if (targetamp(nc).eq.0)then      ! no admitted flow carries weight
  is_LC = .false.                ! re-accumulate over every flow
  ...
xtarget=rcol*targetamp(nc)
```

`ICOLAMP(flow, config, iproc)` is written by `get_icolamp_lines`: flow `f` is
admitted for a configuration's diagram exactly when that diagram contributes
to `f` at the basis's largest power of `Nc`[^mg-icolamp]. vibegraph's table is
`LeadingColorFlows`, pinned row for row against MadGraph's generated
`coloramps.inc` (`vibegraph-lib/tests/color_cf.rs::leading_color_flows_match_madgraphs_coloramps`),
which also fixes that diagram and flow orders match MadGraph's[^n27-b3].

### The accumulators

- `JAMP2(i) = Σ_hel |JAMP_i|²`, `BoundAmplitude::eval_jamp2`
  (`helas/eval/run.rs:414`). It walks the same per-(combination, flow) slots the
  CF contraction reads and is a separate entry point, so `eval_m2` and the
  integration path pay nothing. The CF diagonal constant is left off, since it
  cancels in the selection probability. Off-diagonal flow interference plays no
  role: it is `1/N²`-suppressed and not sign-definite, so it cannot be a
  probability[^n15-jamp2][^n23-e1a][^n23-e1].
- `AMP2(c)`, `BoundAmplitude::eval_amp2` (`helas/eval/run.rs:477`), helicity-summed
  per configuration and **coherent within a configuration**: the member diagrams'
  amplitudes are summed, then squared, as `export_v4.py` writes MadGraph's
  merged accumulator. Configurations are MadGraph's (`config_groups`); diagrams
  with a vertex wider than the narrowest diagram's widest (the four-gluon
  contact of `g g > g g`) carry none, per `get_amp2_lines`[^n27-b6][^n36-b3].
  Ownership of the accumulator itself is [amplitudes/per-diagram-amp2](../amplitudes/per-diagram-amp2.md);
  multi-flow JAMP evaluation is [amplitudes/colour-flow-evaluator](../amplitudes/colour-flow-evaluator.md).

### The configuration weight

MadEvent's `SELECT_COLOR` takes the `ICONFIG` the point was generated in, so a
written flow's marginal follows the multichannel weight share. At a fixed point
that share is the single-diagram-enhancement weight, which depends on the run
card ([phase-space/madevent-single-diagram-enhancement](../phase-space/madevent-single-diagram-enhancement.md)):

| run card | per-configuration weight | where it is formed |
|---|---|---|
| `SDE_strategy = 1`, `tmin_for_channel = -1` (MadGraph's default) | `AMP2(c)` | `eval_amp2` |
| `SDE_strategy = 2` | `GET_CHANNEL_CUT`: product over the configuration's propagators of `1/(t − m²)²` (spacelike) or `1/((t − m²)² + m²Γ²)` (timelike), no amplitude, no coupling | `ChannelSet::channel_cuts` (`coupling/cluster/graph.rs:221`) |

The switch is `EventScaleSource::weights_configurations_by_amp2`
(`hadronic.rs:383`), the same condition the clustering-scale configuration draw
reads ([scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md)),
so scale and colour follow one rule. `tmin_for_channel ≠ -1` is refused at
parse, so the `AMP2 × CC` product MadGraph forms at `SDE_strategy = 1` with a
non-default `tmin_for_channel` is never needed.

On a decay-chain card the weights of configurations whose forced lines are
outside their windows are zeroed first (`SubprocessResonances::mask_unadmitted`),
which picks the pairing on identical decays. Under matching (`ickkw = 1`, proton
beams only) the configuration is not drawn at all: it is the clustering's
`igraphs(1)` ([events/mlm-matched-event-record](mlm-matched-event-record.md)).

## Why the configuration is drawn, not taken from our sampler

Our multichannel labels a point with a *density* share, `α_j g_j(x)/g(x)`.
MadEvent's configuration label has the *amplitude* share
`P(c | x) = AMP2_c(x)/Σ AMP2(x)` at its default strategy: the sampling density
cancels against the enhancement factor, so the label is independent of how each
channel samples. Both partition σ identically and label events completely
differently[^n27-b3].

The difference is measured, not argued. On `uux_to_uux` and `g g > g g` all
propagators are massless, the per-diagram channel maps are bit-identical, α
stays uniform, and the channel index carries no information about which diagram
produced the point. Conditioning the flow on our sampled channel moved
`uux_to_uux`'s `ICOLUP` χ² from 1015 to 7268 on one dof (flow-1 share 51.0%
against MadGraph's 99.96%)[^n27-b3][^n27-dec]. Drawing the configuration from
the point reproduces MadEvent's conditional and keeps the label independent of
our channel technology. The configuration index also feeds
[phase-space/channel-set](../phase-space/channel-set.md)'s partition, which is
MadGraph's merged one.

## Measured agreement

Three seeds of 20k events against MadGraph's banked samples[^n27-b6][^n36-b3]:

| row | ICOLUP | note |
|---|---|---|
| `uux_to_uux` | 99.960% against 99.960%, χ² p 0.39–1.00 | s-channel config admits only flow 2, t-channel only flow 1 |
| `pp_to_bb_fixed` | sub-percent flows 0.060/0.070% against 0.070/0.080% | the sharper test: a mask that is merely on cannot fake per-configuration weights |
| `gg_to_ttx` | χ² p 0.46–0.71 | pruning moves `AMP2` here (below) |
| `gg_to_gg` | χ² 14.0–18.1 / 5, p 2.8e-3 to 1.6e-2 | gates, but the row where a further colour subtlety would show first |
| `ud_to_epemud_qcd0` | χ² 0.0/0.1/0.2 (was 590–671 under the `AMP2` weight) | the only banked run with both `SDE_strategy = 2` and NCOLOR > 1; needs both the channel-cut weight and MadGraph's merged configurations[^n36-b4] |

The `ud_to_epemud_qcd0` diagnosis is the worked case for the second ingredient:
MadGraph's written flow-2 fraction is 0.8185; `GET_CHANNEL_CUT` over its 21
merged configurations predicts 0.8218, over 35 per-diagram configurations
0.782, and the per-diagram `AMP2` share 0.917. Neither ingredient alone
reproduces it[^n36-b4].

The full `samples` gate is [validation/samples-gate](../validation/samples-gate.md).

## Caveats and blind spots

- **Helicity pruning moves `AMP2` but not `|M|²`.** Dropped combinations
  vanish coherently (J_z conservation about the beam axis) while their diagram
  amplitudes do not: pruning moves the incoherent per-configuration sum by 39.5%
  on `gg_to_ttx` and 3.2% on `gg_to_gg`. The production draw uses the pruned
  evaluator, the analogue of MadEvent's `GOODHEL`-filtered accumulation
  (`LIMHEL = 1e-8` there, 1e-24 here); the measured `ICOLUP` frequencies support
  it, and the amplitude gate measures the gap every run[^n27-b6].
- **|M|² cannot see colour labels.** A flow permutation, a per-flow phase or a
  per-flow rescale leaves |M|² unchanged. `color_jamp_oracle` pins JAMPs
  element-wise against MadGraph up to one fitted global phase, with `|g| = 1`
  asserted separately so a uniform rescale cannot hide in the fit[^n23-e1c].
- **Trace-reversal partners in `g g > g g`** carry identical JAMPs
  (`J₁ = J₆`, `J₂ = J₄`, `J₃ = J₅`), so swapping one is invisible to JAMP2 and
  |M|²; only the `leshouche.inc` connectivity comparison sees it
  ([validation/colour-oracles](../validation/colour-oracles.md))[^n23-e1c].
- **`Σ JAMP2 ≠ |M|²`** on non-orthogonal bases; a test asserts the inequality so
  a "simplification" returning the contraction fails[^n23-e1].
- **Helicity selection on a group member** uses the representative's
  per-helicity `|M|²`; the `SPINUP` χ² is evidence, not proof, of that
  correspondence ([events/per-member-colour-flow-tables](per-member-colour-flow-tables.md)).
- A single-flow (or colourless) process reduces to a no-op by construction: its
  one `ICOLAMP` row admits every configuration's flow.

[^n15-jamp2]: Note 15 §2.2, the JAMP2 diagonal and its consumer.
[^n21-helcol]: Note 21, helicity and colour are not sampling channels.
[^n23-e1]: Note 23 E1 outcome.
[^n23-e1c]: Note 23 E1c, NCOLOR=6 JAMP agreement and the reversal blind spot.
[^n27-b3]: Note 27 B3.1–B3.2, MadEvent's rule as read and the falsified channel-conditioned draw.
[^n27-dec]: Note 27 §6, D1 superseded by D4.
[^n27-b6]: Note 27 B6 outcome.
[^n36-b3]: Note 36 B3, merged configurations and the channel-cut weight.
[^n36-b4]: Note 36 B4, the `ud_to_epemud_qcd0` diagnosis.
[^n23-e1a]: Note 23 E1a, the accumulator must not move σ and must stay off the integration path.
[^mg-select-color]: MadGraph 3.7.1 `super_auto_dsig_group_v4.inc:1087`, `select_color`.
[^mg-icolamp]: MadGraph `export_v4.py:1295`, `get_icolamp_lines`.
