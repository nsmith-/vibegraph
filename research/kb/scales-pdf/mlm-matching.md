---
type: Overview
title: "MLM matching: what vibegraph does and what the shower does"
description: "Scope of MLM in vibegraph (xqcut, ickkw = 1 scales, rewgt, the event record) versus the shower's veto, MadEvent's per-point call flow, and the cards that are refused."
status: draft
tags: [mlm, matching, ickkw, xqcut, madevent]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n41-intro, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L11-L42", title: "Note 41 (goal and division of labour)"}
  - {id: n41-flow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L49-L68", title: "Note 41 §1.1 (call flow per point)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1 (xqcut and the ickkw = 1 scales)"}
  - {id: n41-d2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1357-L1374", title: "Note 41 D2 decisions (user, 2026-09-29)"}
  - {id: n41-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3490-L3512", title: "Note 41 §5 decisions (user, 2026-09-28)"}
  - {id: matching-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/runcard/matching.rs#L1-L80", title: "runcard/matching.rs"}
  - {id: generate-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/src/generate.rs#L436-L447", title: "generate.rs refuse_rounding_on_mixed_multiplicity"}
  - {id: mg-auto-dsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/auto_dsig_v4.inc#L124-L182", title: "MadGraph auto_dsig_v4.inc (DSIG)"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L1333-L1824", title: "MadGraph reweight.f rewgt"}
---

# MLM matching: what vibegraph does and what the shower does

MLM is split between two programs. **The matrix-element generator** applies the `xqcut`
cut, computes the clustering scales, reweights each event by `α_s` and PDF ratios, and
writes an event record a shower can match against. **The shower** does the jet matching
and the veto, which is where the Sudakov suppression comes from. vibegraph does the first
half only; nothing in it showers.[^n41-intro] The target is parity with MadEvent: a run
card with `ickkw = 1` and `xqcut > 0`, over a proc card whose `add process` lines differ in
jet multiplicity, gives MadEvent's cross sections and event record, and Pythia 8's kT-MLM
(`JetMatching:merge = on`) treats the file as it treats MadEvent's. MLM is inside the
release goal ([decisions: release-scope-lo-mlm](../decisions/release-scope-lo-mlm.md)).

## MadEvent's call flow per point

On the scalar path (`vector_size = 1`, the default, which every reference pins):[^n41-flow]

1. **`setclscales(p, keepq2bck = .false.)`**, reached from `update_scale_coupling_vec`
   (`reweight.f:1856,1907`), clusters, applies the `xqcut` cut and sets `μR`/`μF`. A
   failure sets the weight to 0. `banner.py:4966` forces `dynamical_scale_choice = -1`
   whenever MadGraph auto-enables matching.
2. **`DSIG<n>`** (`auto_dsig_v4.inc:124-182`) evaluates the PDFs at `√q2fact`, draws one
   flavour combination `IPSEL ∝ PD(IPSEL)`, computes `rwgt = REWGT(p)`, then `SMATRIX`,
   multiplies `DSIGUU *= rwgt` and calls `UNWGT`.[^mg-auto-dsig]
3. **`rewgt`** calls `setclscales(p, keepq2bck = .true.)` a **second time**
   (`reweight.f:1465`) and multiplies in the `α_s` and PDF ratios (`:1557-1795`).[^mg-reweight]
4. **Colour and mothers come from the clustered graph.** Under `ickkw > 0`,
   `select_color` draws from `igraphs(1)` instead of the integration channel
   (`super_auto_dsig_group_v4.inc:1120-1142`), and `addmothers` takes its configuration
   from the same graph (`addmothers.f:109-116`).

## What vibegraph implements, and where

| piece | concept | code |
|---|---|---|
| `ickkw`, `xqcut`, `alpsfact`, `asrwgtflavor`, `auto_ptj_mjj`, `use_syst`, and the `ptj`/`mmjj`/`drjj`/`drjl` rewrites | [run-card/matching-parameters](../run-card/matching-parameters.md) | `runcard/matching.rs` |
| the τ floor `setxqcuts` implies | [phase-space/cut-implied-timelike-floors](../phase-space/cut-implied-timelike-floors.md) | `cuts.rs` |
| both `setclscales` calls, the density and record `μF`, `ptclus` | [scales-pdf/mlm-scales](mlm-scales.md) | `coupling/scales.rs`, `coupling/cluster/setclscales.rs` |
| `rewgt`, per member and per beam ordering | [scales-pdf/mlm-rewgt](mlm-rewgt.md) | `coupling/cluster/rewgt.rs`, `proton.rs` |
| colour and resonances from the clustered configuration | [events/mlm-matched-event-record](../events/mlm-matched-event-record.md) | `EventScales::clustered_config`, `lhef/resonance.rs` |
| mixed multiplicities, one integrand per multiplicity | [phase-space/mixed-multiplicity-integrand](../phase-space/mixed-multiplicity-integrand.md) | `MultiplicitySum` |
| `<scales pt_clust_N>`, status-2 resonances, `<MGRunCard>` | [events/mlm-matched-event-record](../events/mlm-matched-event-record.md) | `lhef/build.rs`, `lhef/write.rs` |
| Pythia's matching settings and how they read our file | [events/pythia-interop](../events/pythia-interop.md) | `validation/pythia/` |

At `ickkw = 0` none of this changes a byte: every `grid.bin.zst` is byte-identical to the
unmatched code path, and every LHE file is identical except for the header line that names
the artifact path. That was measured binary against binary on six cases (`integrate` and
`generate` each) against `85e1459`, the build before matching existed, and is pinned by
`without_matching_the_record_scale_is_the_density_scale` and `scalup_reads_the_record_scale`
([validation/no-change-claims](../validation/no-change-claims.md)).[^n41-m1]

Two MadEvent switches do nothing under `ickkw = 1`: `hmult` and `highestmult`. MadEvent
treats every multiplicity alike at parton level; only the shower's `nJetMax` knows which is
highest. `ickkw = 2` is unreachable from the card (`banner.py:4284`, `allowed = [0, 1]`).

## Cards refused, and cards accepted with a warning

The run-card half is resolved once, when the card is read (`runcard/matching.rs`), so
the artifact records the card after MadGraph's own edits.[^matching-rs]

| card | behaviour | why |
|---|---|---|
| `ickkw ∉ {0, 1}` | refused (`UnsupportedIckkw`) | MadGraph allows only 0 and 1 |
| `ickkw = 1`, `maxjetflavor = 6` | refused (`MatchedTopJets`) | MadGraph refuses it (`banner.py:4556`) |
| `ickkw = 1` with exactly one fixed `μF` | refused (`MatchingWithOneFixedFactorisationScale`) | `reweight.f:1138`'s operator-precedence defect applies `scalefact` and `q2bck` to one beam only; refused rather than reproduced (decision c) |
| `xqcut > 0` with a resolved `ptj < xqcut` (`auto_ptj_mjj = F`, or `ptj < 0`) | refused (`XqcutAboveJetThreshold`) | MadEvent's τ floor becomes a cut that differs per channel; open, low priority ([backlog: mlm-ptj-below-xqcut-refused](../backlog/feature/mlm-ptj-below-xqcut-refused.md)) |
| matching or `xqcut` at fixed beams | refused (`FixedBeamMatching`) | the fixed-beam integrand cannot zero-weight a clustered-out point, and no reference exists; open, low priority ([backlog: mlm-at-fixed-beams-or-decays-refused](../backlog/feature/mlm-at-fixed-beams-or-decays-refused.md)) |
| matching or `xqcut` on a decay | refused permanently (today through the same `FixedBeamMatching`) | a decay has no incoming partons to match against; not a missing feature ([decisions: mlm-not-on-decays](../decisions/mlm-not-on-decays.md)) |
| CKKW-L (`ktdurham`, `ptlund`, `dparameter`) | refused as unimplemented cuts | a different merging scheme sharing the clustering; decision b ([backlog: ckkw-l-merging-refused](../backlog/feature/ckkw-l-merging-refused.md)) |
| `pdlabel1 ≠ pdlabel2` on proton beams | refused (`AsymmetricBeamPdf`) | MadGraph refuses it (`PDLabelBlock`) |
| `--strategy stochastic-rounding` on a mixed-multiplicity card | refused (`refuse_rounding_on_mixed_multiplicity`) | unit weights would leave each multiplicity's share of the file to the realised sample ([backlog: stochastic-rounding-refuses-mixed-multiplicity](../backlog/feature/stochastic-rounding-refuses-mixed-multiplicity.md))[^generate-rs] |
| `xqcut > 0` with `ickkw = 0` | accepted with a warning, applied as a pure cut | MadGraph's behaviour (an error log and a 5 s sleep, then it runs) |
| mixed multiplicities with `ickkw = 0`, `xqcut = 0` | accepted with a warning | MadGraph accepts and double counts; decision a, for parity |
| `use_syst = T` with `alpsfact ≠ 1` | `alpsfact` set to 1, with a warning | `setrun.f:151-159` does this whether or not matching is on; `banner.py` only under matching |

## Deviations from MadEvent

- **The permuted first call (H1).** Grouped MadEvent clusters its first `setclscales`
  call on the unpermuted momenta `PP` while the matrix element and the second call read
  the permuted `P1`. vibegraph clusters both calls on `P1`. **Decision (user,
  2026-09-29): documented, not reproduced or refused.** The affected rows' manifest notes
  name it, and per-event gates report those events as `info, permuted P1`
  ([validation/madgraph-permuted-first-call](../validation/madgraph-permuted-first-call.md)).[^n41-d2]
- **The stale `ipdgcl` table.** MadEvent carries the flavour table across events, and a
  non-jet final-state leg can keep an earlier event's code. vibegraph reads this event's
  walk. The difference needs an `IPROC` mixing jet and non-jet flavours on one leg; none
  of the reference rows has one ([scales-pdf/mlm-rewgt](mlm-rewgt.md)).
- MadGraph's matching defects (`reweight.f:1138` precedence, the duplicate `iforest(2)`
  test in `setcuts.f`, `addmothers.f:115`'s stale index, and others) follow the policy in
  [validation/madgraph-defect-policy](../validation/madgraph-defect-policy.md): reproduced
  bug-for-bug with a comment, or refused, never silently fixed. The register is
  [validation/madgraph-defects](../validation/madgraph-defects.md).

## Decisions behind the scope

Settled by the user on 2026-09-28, every recommendation accepted:[^n41-dec]

- **(a)** Mixed multiplicity without matching is accepted for parity, with a warning.
- **(b)** CKKW-L stays refused, with its own backlog item.
- **(c)** The `reweight.f:1138` defect's card (one fixed `μF` under `ickkw = 1`) is
  refused, not reproduced.
- **(d)** The `t t̄ + j` row is a reference: it is the only coverage of the massive-core
  branches (`mt2last`, the massless–massive `dj` case).
- **(e)** Matched end-to-end through a shower is an informational comparison, not a gate
  ([validation/mlm-pythia-matched-comparison](../validation/mlm-pythia-matched-comparison.md)).

## Validation

Cross sections against MadEvent: [validation/mlm-sigma-gate](../validation/mlm-sigma-gate.md).
Per-event scales, `rewgt` factors and record fields against an instrumented MadEvent:
[validation/mlm-dump-oracle](../validation/mlm-dump-oracle.md). Open items: the three
single-multiplicity `ℓℓj` rows read high together by 0.03–0.23%
([backlog: llj-mlm-rows-sit-high-together](../backlog/validation/llj-mlm-rows-sit-high-together.md)),
and `pp_to_ttx_0j1j_mlm`'s `@1` reads 1.28% high, undiagnosed
([backlog: ttx-mlm-at1-sigma-high](../backlog/validation/ttx-mlm-at1-sigma-high.md)).

[^n41-intro]: Note 41, goal and division of labour.
[^n41-flow]: Note 41 §1.1, MadEvent's call flow per point on the scalar path.
[^n41-m1]: Note 41 M1: the run-card rules as implemented, the refusals, and byte identity at `ickkw = 0`.
[^n41-d2]: Note 41, D2 decisions (user, 2026-09-29): H1 documented.
[^n41-dec]: Note 41 §5, decisions (user, 2026-09-28).
[^matching-rs]: `vibegraph-lib/src/runcard/matching.rs`.
[^generate-rs]: `vibegraph-cli/src/generate.rs`, `refuse_rounding_on_mixed_multiplicity`.
[^mg-auto-dsig]: MadGraph `auto_dsig_v4.inc`, `DSIG`.
[^mg-reweight]: MadGraph `reweight.f`, `rewgt`.
