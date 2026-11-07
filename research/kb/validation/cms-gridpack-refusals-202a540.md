---
type: Measurement
title: CMS gridpack refusals at 202a540, by reason, with the work that would lift them
description: "199 CMS gridpacks at 202a540: 66 ok, 93 refused, 37 timeouts, 3 panics. Each refusal reason is mapped to its backlog item and ranked for the support work after the v0.2 cut."
status: draft
tags: [validation, cms, gridpacks, refusals, backlog, survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
measured:
  commit: 202a540f
  host: "4-core Linux host (EL10), shared with other jobs"
  command: "pixi run -e gridpacks gpdb run strat-v2-200, campaign main-202a540-200-pc: vibegraph integrate --neval 10000 --niter 3 --fixed-budget --seed 1, 600 s per attempt, gpdb card patches between attempts"
  data: "validation/cms_gridpacks/gridpacks.sqlite on the CERN host (vg_run, vg_run_reason, vg_reason); each gridpack's last attempt, keyed by gridpack_id"
---
The `main-202a540-200-pc` campaign ran the 199 gridpacks of the `strat-v2-200`
sample (196 labels; three labels name two gridpacks each) through vibegraph at
202a540f. **Measured**: every count below is read from the campaign database,
taking each gridpack's last attempt and its first refusal reason (23 refused
runs give more than one reason). The **rank** is a judgement: gridpacks
unlocked, how common the feature is in production, and how small the change is.
Support work waits until after the v0.2 release cut (user, 2026-11-07).

| outcome | gridpacks |
|---|--:|
| ok | 66 |
| refused | 93 |
| timeout (600 s) | 37 |
| panic | 3 |

## Refusals, ranked

Items marked *Sprint H branch* are on `sprinth-kb` and not yet on `main`.

| rank | reason | n | item | priority |
|--:|---|--:|---|---|
| 1 | squark–squark–gluon–gluon colour `T(1,-1,3)*T(2,4,-1)` | 3 | [coloured-scalar-seagull-colour-unsupported](../backlog/feature/coloured-scalar-seagull-colour-unsupported.md) | high |
| 2 | `import model --modelname`, plus gpdb's rewrite of it | 2 | [proc-card-import-model-options-refused](../backlog/feature/proc-card-import-model-options-refused.md) | high |
| 3 | model without `coupling_orders.py` (`QAD`) | 2 | [coupling-orders-not-inferred-without-coupling-orders-py](../backlog/feature/coupling-orders-not-inferred-without-coupling-orders-py.md) | high |
| 4 | `ptj4min = -1` | 2 | `ptj-n-min-nonpositive-refused` (*Sprint H branch*) | medium |
| 5 | negative coupling order (onia_sm) | 2 | `negative-ufo-coupling-orders-refused` (*Sprint H branch*) | medium |
| 6 | coupling order as a Python expression (dim6top) | 2 | [ufo-coupling-order-expressions-refused](../backlog/feature/ufo-coupling-order-expressions-refused.md) | medium |
| 7 | particle charge referencing a parameter | 2 | [ufo-particle-attribute-parameter-references-refused](../backlog/feature/ufo-particle-attribute-parameter-references-refused.md) | medium |
| 8 | subprocesses with different outgoing masses | 2 | [mixed-outgoing-mass-subprocesses-not-split](../backlog/feature/mixed-outgoing-mass-subprocesses-not-split.md) | medium |
| 9 | decay chain without nested parentheses | 2 | [decay-chain-without-nested-parentheses-refused](../backlog/feature/decay-chain-without-nested-parentheses-refused.md) | medium |
| 10 | squared-order constraint | 4 | [squared-order-constraints-refused](../backlog/feature/squared-order-constraints-refused.md) | medium |
| 11 | polarised leg in a non-default `me_frame` | 2 | [me-frame-other-than-partonic-cm-refused](../backlog/feature/me-frame-other-than-partonic-cm-refused.md) | medium |
| 12 | polarisation on a decayed leg | 1 | [polarized-intermediate-resonances-refused](../backlog/feature/polarized-intermediate-resonances-refused.md) | medium |
| 13 | σ carried by an effective 1.2 points (TTZ2b) | 1 | [ttz2b-sigma-carried-by-one-point](../backlog/performance/ttz2b-sigma-carried-by-one-point.md) | medium |
| 14 | Majorana fermion, or a fermion-number-violating vertex | 21 | [majorana-fermions-unsupported](../backlog/feature/majorana-fermions-unsupported.md) | high (planned after v0.2, user, 2026-11-07) |
| 15 | `xqcut` with a resolved `ptj` below it | 3 | [mlm-ptj-below-xqcut-refused](../backlog/feature/mlm-ptj-below-xqcut-refused.md) | low (user, 2026-10-09) |
| 16 | spin-2 propagator (bulk graviton) | 2 | [spin-2-and-spin-3-2-particles-unsupported](../backlog/feature/spin-2-and-spin-3-2-particles-unsupported.md) | low (descoped by user decision) |
| 17 | clustering scale with a four-line vertex | 1 | [clustering-scale-contact-vertex-refused](../backlog/feature/clustering-scale-contact-vertex-refused.md) | low |
| 18 | custom `propagators.py` form | 1 | [custom-ufo-propagators-unevaluated](../backlog/feature/custom-ufo-propagators-unevaluated.md) | low |
| 19 | zero weight on every point (TpTp) | 1 | [tptp-integrand-zero-on-every-point](../backlog/validation/tptp-integrand-zero-on-every-point.md) | low |
| — | NLO or split-order proc card | 36 | [nlo-generation-missing](../backlog/feature/nlo-generation-missing.md) | low; outside the LO charter |
| — | μF below the PDF floor (QCD-4Jets_HT-100to200) | 1 | Sprint H fix in progress | — |

Total 93. The Majorana and fermion-number-violating refusals are the largest
in-charter group (21: HNL, type-III seesaw, dark Higgs, higgsinos, gluinos,
RPV stops, diquarks, leptoquarks); the user lifted their earlier descoping and
planned them after the v0.2 cut. **Proposed for the user's decision, not
applied:** the `mlm-ptj-below-xqcut` cards (DY2Jets, WJets HT) are common SM
backgrounds, which argues for raising that item to medium.

## Timeouts and panics

All 37 timeouts are at the 600 s limit, triaged by
[multijet-mlm-cards-time-out-in-the-gridpack-campaign](../backlog/performance/multijet-mlm-cards-time-out-in-the-gridpack-campaign.md)
(high): DYBto2LB-4Jets ×2, DYJets_HT ×2, DYto2L-4Jets ×2, GJ-4Jets ×4,
GJets-4Jets ×2, GJets_HT-100to200, QCD-5Jets ×2, TT-3Jets ×2, TTto2L2Nu-3Jets,
TTto4Q-3Jets, VBS_WPMZTo4Q ×2, WBtoLNuB-4Jets ×2, WWjj_SS_ewk, Wto2Q-3Jets ×2,
WtoLNu-4Jets ×2, ZJetsToNuNu_HT ×2, Zto2Nu-4Jets ×2, Zto2Q-4Jets ×2,
aQGC ×2, idm_dilepton_mH110_mA202.

The 3 panics (DarkPhoton-monojet ×2 at RAMBO with one final-state momentum,
idm_dilepton_mH80_mA160 at a NaN clustering measure) are being turned into
refusals or fixed by Sprint H.
