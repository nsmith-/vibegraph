---
type: Algorithm
title: MadEvent's single-diagram enhancement and get_channel_cut
description: "SMATRIX weights channel c by AMP2_c·CC_c at sde_strategy 1 and by CC_c alone at 2; CC ≡ 1 when tmin_for_channel = −1; the induced P(c|p) is partition-free."
status: draft
tags: [madevent, multichannel, sde-strategy, configuration, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n29-d0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L2821-L2884", title: "Note 29 §D.0 (get_channel_cut cannot reach ee_to_mumua)"}
  - {id: n29-dm0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3253-L3323", title: "Note 29 §D.M0 (premise re-verified in 3.5.7 and 3.7.1)"}
  - {id: n29-dm2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L3341-L3360", title: "Note 29 §D.M2 (3.5.7 vs 3.7.1: 0.074%)"}
  - {id: n29-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L4758-L4825", title: "Note 29 §B.1 (the rule is conditional)"}
  - {id: n29-b12, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5323-L5352", title: "Note 29 §B.12 (errors in the brief)"}
  - {id: n29-bres, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/29-v01-validation-sprint-plan.md#L5523-L5756", title: "Note 29, chain B results (P(c|p) derivation)"}
  - {id: mg-matrix, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/matrix_madevent_group_v4.inc#L214-L228", title: "MadGraph matrix template, multi-channel block"}
  - {id: mg-gcc, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/genps.f#L1817-L1951", title: "MadGraph genps.f get_channel_cut"}
  - {id: mg-banner, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/various/banner.py#L4990-L5059", title: "MadGraph banner.py sde_strategy auto-selection"}
---

# MadEvent's single-diagram enhancement and `get_channel_cut`

## The rule, as the generated Fortran has it

MadEvent integrates each configuration (channel) `c` separately and multiplies
the matrix element by an enhancement factor. The block in the matrix template
(`matrix_madevent_group_v4.inc:214-228`, generated as `matrix1_orig.f`):[^mg-matrix]

```fortran
      IF (MULTI_CHANNEL) THEN
        XTOT=0D0
        DO I=1,LMAXCONFIGS
          J = CONFSUB(1, I)
          IF (J.NE.0) THEN
            IF(SDE_STRAT.EQ.1) THEN
              AMP2(J) = AMP2(J) * GET_CHANNEL_CUT(P, I)
              XTOT=XTOT+AMP2(J)
            ELSE
              AMP2(J) = GET_CHANNEL_CUT(P, I)
              XTOT=XTOT+AMP2(J)
            ENDIF
          ENDIF
        ENDDO
        IF (XTOT.NE.0D0) THEN
          ANS=ANS*AMP2(CHANNEL)/XTOT
```

So the per-point weight of configuration `c` is

| `sde_strategy` | weight of `c` |
|---|---|
| 1 | `AMP2_c · CC_c` |
| 2 | `CC_c` alone; the squared amplitude is discarded |

where `CC_c = get_channel_cut(p, c)` (`genps.f:1817`). At strategy 2 it is a
product over the configuration's first `nexternal − 3` forest lines of
`1/(t − m²)²` (spacelike, with an `s_tot·1e-10` offset) or
`1/((t − m²)² + m²Γ²)` (timelike). Its first statement
(`genps.f:1878-1881`) short-circuits it:[^mg-gcc]

```fortran
      if(sde_strat.eq.1.and.tmin_for_channel.eq.-1)then
         get_channel_cut = 1d0
         return
      endif
```

**The familiar "channel `c` is weighted by `AMP2_c/Σ AMP2`" is therefore
conditional:** it holds only when `sde_strategy = 1` *and*
`tmin_for_channel = −1` (its default). Statements of the rule without that
condition are wrong.[^n29-b1][^n29-b12] Off its default, `tmin_for_channel`
multiplies `CC` by an exponential suppression of spacelike lines below
`t/s_tot = tmin`; at strategy 1 that factor is all `CC` is, and there
`genps.f` reads `t` uninitialised (the propagator invariant is formed only
under `sde_strat.eq.2`), a MadGraph defect found in note 36 B3. At strategy 1
`CC` also returns 1 when the configuration has fewer than two t-channel lines.

### Which strategy a card gets

`banner.py` auto-selects (`:4990-5059`):[^mg-banner] `sde_strategy = 2` when the
process has a single colour flow (and `proc_characteristic['gauge'] != 'FD'`),
then back to 1 for a pure-lepton/photon final state from partonic beams, or
unless every process fixes `QCD = 0` explicitly; 2 for interference runs; 1 for a `1 → n` decay; and 1
whenever the `$` (forbidden on-shell s-channel) syntax is used. A user may
override. So most QCD rows run at 1, while e.g. the banked
`ee_to_mumu_tata_qcd0` card runs at 2.

## The conditional distribution it induces

MadEvent does not draw a configuration per event: it clusters in the channel that
sampled the point (`genps.f` sets `this_config = iconfig`; `cluster.f` roots the
clustering on it). The configuration nevertheless has a well-defined
distribution. A point from configuration `c` carries map density `g_c` and
integrand weight `|M|² · w_c/Σ w`, so the density of events generated in `c` at
`p` is `∝ |M(p)|² · (w_c(p)/Σ_i w_i(p)) · g_c(p)/g_c(p)`. The map density
cancels:[^n29-bres]

```
P(c | p) = w_c(p) / Σ_i w_i(p)        (w = AMP2·CC at strategy 1, CC at 2)
```

independent of `g_c` and of how either side partitions phase space. vibegraph's
α partition (Kleiss–Pittau over its own channels,
[phase-space/multichannel](multichannel.md)) differs from MadEvent's, so it does
**not** imitate MadEvent's channel; it samples the conditional MadEvent's channel
induces, which is the same distribution. Where the configurations agree on what
is read (e.g. the scale), the draw changes nothing, which is why rows whose
configurations share a scale came out bit-identical when the draw was
introduced.

## What vibegraph does with it

- **The weights.** `EventScaleSource::weights_configurations_by_amp2`
  (`hadronic.rs:383`) is true exactly when `SDE_strategy == 1 &&
  tmin_for_channel == −1` (`hadronic.rs:348`). Then the configuration weights
  are `AMP2_c` from the evaluator, formed at the coupling the amplitudes were
  bound at, so the draw is a function of the momenta alone. Otherwise they are
  `ChannelSet::channel_cuts` (`coupling/cluster/graph.rs:221`), a port of
  `get_channel_cut`'s strategy-2 branch: walking the forest's first
  `nexternal − 3` lines, without the `tmin` factor.
- **Who reads them.** The configuration a point's cluster scale is taken in
  ([scales-pdf/clustering-configuration-draw](../scales-pdf/clustering-configuration-draw.md))
  and the configuration its colour flow is drawn in (`SELECT_COLOR`'s `ICONFIG`;
  [events/colour-and-helicity-selection](../events/colour-and-helicity-selection.md)).
  `draws_configuration` (`hadronic.rs:362`) says whether a per-event
  prescription draws at all; constant and closed-form scales do not.
- **Which channels exist** is the same configuration list, each MadGraph
  configuration group being one integration channel, with channel cuts
  available per configuration ([phase-space/channel-set](channel-set.md)).
- **Run-card classes** (`runcard/classes.rs`): `SDE_strategy` is `Consumed`
  (`:420`), since it decides what the configuration weight is.
  `tmin_for_channel` is `IgnoredPhysics` (`:410`): it is read beside
  `SDE_strategy`, but no `tmin` suppression is implemented, so a card setting it
  is refused rather than integrated under a rule that does not describe it.
  `the_configuration_draw_needs_both_run_card_fields` (`hadronic.rs:3892`) pins
  both guards; `the_amp2_configuration_order_matches_the_forest_order`
  (`hadronic.rs:3820`) pins that `AMP2`'s configuration order is the forest's,
  on `g g → g g`, whose diagrams outnumber its three configurations (the
  contact diagram carries none), so an off-by-one cannot hide.
- A decay card with `sde_strategy = 2` is refused, because channel forests are
  built for `2 → n` only
  ([backlog](../backlog/feature/decay-card-sde-strategy-2-refused.md)).

The frequency law itself (`P(c|p) ∝ w_c`) is not measured from outside the
crate; the probe that exists asserts independence from the sampling channel and
replay stability
([backlog](../backlog/validation/drawn-scale-config-frequency-law-ungated.md)).

## A worked refutation: `get_channel_cut` and `ee_to_mumua`

The `ee_to_mumua` σ moved between MadGraph 3.5.7 and 3.7.1 reference banks, and
3.7.1 changed `get_channel_cut`. The change cannot be the cause: the banked
cards carry `sde_strategy = 1`, `Source/run_card.inc` sets
`TMIN_FOR_CHANNEL = −1`, so `CC ≡ 1` and the function returns before either
expression the fix touched (both are under `sde_strat.eq.2` anyway). The
auto-selection rule gives 1 in both versions (its only change between them is
the `gauge != 'FD'` guard), and both versions build the same six-channel
decomposition (`configs.inc` differs only by a `FAKE_ID` line). Re-measured:
3.5.7 and 3.7.1 agree to 0.074%.[^n29-d0][^n29-dm0][^n29-dm2] The lesson generalises: check
the run-card conditions a MadGraph code path is guarded by before attributing
a difference to it. A single MadEvent run's quoted error is also not evidence of
a drift; a seed sweep of the reference is
([validation/madevent-reference-seed-policy](../validation/madevent-reference-seed-policy.md)).

MadEvent's phase-space maps themselves are surveyed in
[references/codebases/madevent-phase-space-maps](../references/codebases/madevent-phase-space-maps.md).

[^mg-matrix]: The multi-channel block of MadGraph's matrix template at `b7687064`.
[^mg-gcc]: `get_channel_cut` at `b7687064`, with its early return.
[^n29-b1]: Note 29 §B.1: the rule is conditional on two run-card fields.
[^n29-b12]: Note 29 §B.12: the unconditional statement was an error in the brief.
[^mg-banner]: `banner.py` `sde_strategy` auto-selection at `b7687064`.
[^n29-bres]: Note 29 chain B results: why drawing `P(c|p)` reproduces MadEvent.
[^n29-d0]: Note 29 §D.0: the mechanism refuted from the cards.
[^n29-dm0]: Note 29 §D.M0: re-verified in both MadGraph lines.
[^n29-dm2]: Note 29 §D.M2: the two MadGraph lines agree to 0.074%.
