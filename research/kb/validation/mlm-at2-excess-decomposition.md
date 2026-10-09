---
type: Measurement
title: The pp_to_ll_0j2j_mlm @2 excess decomposed
description: "The split of a +1.94 pb @2 excess into reference, H1, composite and a generic 2->4 offset, what each probe eliminated, and where @2 stands against the independent-directory reference."
status: draft
tags: [mlm, sigma, madevent-reference, diagnosis, 2to4]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
measured:
  - {command: "vibegraph integrate per subprocess directory against fresh and patched MadEvent directories (note 41 D2)"}
  - {commit: 590f87a, host: "Apple M3 Max (16 cores), macOS 15.7", command: "pixi run -e madgraph --skip-deps validate-mlm-sigma"}
sources:
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3, the budget ladder, D2 diagnosis, decisions and R1"}
  - {id: n41-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L3403-L3489", title: "Note 41 Z2, the mixed rows' sigma gate"}
---
# The `pp_to_ll_0j2j_mlm` `@2` excess decomposed

## The question

On the matched `p p > e+ e- @0 + j @1 + j j @2` card (13 TeV, `ickkw = 1`,
`xqcut = 20`), the first composite integration read `@2` = 132.38 ± 0.31 pb
against a MadEvent reference of 130.44 ± 0.20: +1.49%, 5.2σ, while `@0` and `@1`
agreed. That reference was ten seeds, nine of which shared one directory. The
diagnosis changed no production code; its probes were throwaway and rebuilt in a
private target to reproduce[^n41-m3].

## Answer: four parts, one of them not MLM

| part | pb | how it was measured |
|---|---|---|
| the reference sat low against independent MadEvent runs | +0.65 | 7 fresh directories against the ten shared-directory seeds |
| H1, MadEvent's first `setclscales` call on `PP` | +0.33 ± 0.11 | MadEvent patched to hand the first call `P1`, 4 runs against 5 |
| the composite against vibegraph run per directory | +0.38 ± 0.33 | not significant |
| a generic 2 → 4 offset, present without matching | ≈ +0.6 | fixed scale, no `xqcut`; also at fixed beams |

The first three are specific to this row. H1 is
[madgraph-permuted-first-call](madgraph-permuted-first-call.md). The fourth is
open and outside MLM:
[generic-2to4-sigma-offset-vs-madevent](../backlog/validation/generic-2to4-sigma-offset-vs-madevent.md).

## Localisation by subprocess directory

MLM card, pb; errors are the spread over seeds over √n.

| directory | MG reference | MG fresh | MG patched | vibegraph | vg / fresh | vg / patched |
|---|---|---|---|---|---|---|
| `P2_gg_llqq` | 11.353 ± 0.042 | 11.378 ± 0.040 | 11.322 ± 0.006 | 11.369 ± 0.007 | −0.1% | +0.4% |
| `P2_gq_llgq` | 91.418 ± 0.178 | 91.934 ± 0.126 | 91.817 ± 0.116 | 92.320 ± 0.069 | +0.4% | +0.6% |
| `P2_qq_llgg` | 9.670 ± 0.030 | 9.710 ± 0.022 | 9.659 ± 0.014 | 9.733 ± 0.007 | +0.2% | +0.8% |
| `P2_qq_llqq` | 17.999 ± 0.073 | 18.063 ± 0.062 | 18.394 ± 0.077 | 18.579 ± 0.063 | +2.9% | +1.0% |
| sum | 130.44 | 131.09 | 131.19 | 132.00 | +0.7% | +0.6% |

- MG fresh: `p p > e+ e- j j` alone, one freshly generated directory per seed,
  five seeds of 10000 events plus two of 50000. More events do not raise `@2`
  (130.54 and 131.00 at 50000).
- MG patched: the same fresh directories with the first call handed `P1`.
- vibegraph: each directory's subprocesses integrated alone.
- Every MadEvent column scatters more than it quotes (`gq`: 0.33 pb across fresh
  directories against 0.22 quoted per run).

## What was eliminated, and by what

- **Scales, `rewgt`, PDFs, `αs`, per event.** On MadEvent's own events, the
  weight factor `f₁f₂(μF) · αs^n(μR) · rewgt`, each side at its own scales, has
  ratio 1 at 1e-9 on all 9859 non-permuted events, and each factor is 1
  separately; absolute densities agree with the dumped LHAPDF values on all 4089
  `RWPDF` points, worst 5e-16.
- **The matrix element and symmetry factors.** |M|² against a MadGraph standalone
  build on MadEvent's own event momenta (`u u~ > e+ e- g g`, `g u > e+ e- g u`,
  `u d > e+ e- u d`, `u u > e+ e- u u`; 941 points, 31 above √ŝ = 600 GeV) agree
  at 6e-8, the precision of the LHE records.
- **The flavour sum.** Every one of the 1214 `@2` events maps to a vibegraph
  member; no group is missing or doubled.
- **The clustering configuration draw.** Under matching the weight depends on the
  configuration through the jet memo's restricted re-cluster, by up to a factor 2.
  vibegraph's expected factor over MadEvent's at MadEvent's own channel is
  0.994 ± 0.005 on `@2`.
- **The jet memo.** M1 proved vibegraph's memo equals MadEvent's on every event
  of every row ([mlm-dump-oracle](mlm-dump-oracle.md)), so the memo, once the
  next suspect, is not a cause.
- **vibegraph's sampler.** Ten times the budget moves no directory (`qq_llgg`
  9.732 ± 0.022 against 9.733 ± 0.007; `gg` 11.381 ± 0.008 against
  11.369 ± 0.007; `gq` 92.13 ± 0.24 against 92.32 ± 0.07). A budget ladder of
  `@2` alone on the code path before the composite showed the excess already
  there and not shrinking with iterations.

## The generic offset

It survives every piece of MLM switched off:
- fixed scales, `ickkw = 0`, `xqcut = 0`, `ptj = mmjj = 20`: vibegraph 102.36
  against MadEvent's 101.65 ± 0.14 (six runs), +0.7%;
- fixed beams, √s = 500 GeV, `u u~ > e+ e- g g`, the same cuts: 0.52894 ± 0.00024
  against 0.52743 ± 0.00040, +0.29%, 3.2σ, with no PDFs, no running scales and
  |M|² identical point by point. This is the cheapest reproducer.

On `qq_llgg` the unweighted samples put vibegraph's surplus in the high-ŝ tail
(m_ll > 150 +14%, ŝ > 600 GeV +9.5%). Which side is right is not settled;
MadEvent's dedicated tail slice recovering most of the tail, and its runs
scattering beyond their quotes, point at MadEvent's tail coverage. An acceptance
difference in regions MadEvent never populates is the one class the per-event
oracle cannot see, so the decisive test is σ in sliced regions on both sides.

## Where `@2` stands

The recommendations were adopted: the `@2` reference was regenerated from
independent directories, H1 was registered, and the generic offset was filed.
Against the 21-directory reference, ten seeds through the composite read
([mlm-sigma-gate](mlm-sigma-gate.md))[^n41-z2]:

| | vibegraph (pb) | MadEvent (pb) | pull | rel |
|---|---|---|---|---|
| `@2` | 131.275 ± 0.663 | 130.908 ± 0.195 | +0.53 | +0.28% |

The registered deviations (H1 +0.33 pb, the generic offset ≈ +0.6 pb) are 0.7% of
`@2`. They fit inside the 3σ tolerance only while this side's `@2` error stays
near its measured 0.66 pb; a budget that brings it below about 0.3 pb needs them
as an explicit allowance, or the gate fails on known causes. Ten seeds at this
budget cannot tell "no difference" from that +0.7% budget.

[^n41-m3]: Note 41, D2 diagnosis and its decisions; the earlier M3 budget ladder.
[^n41-z2]: Note 41, Z2 "The mixed rows' σ gate", tree `590f87a` on the bank host.
