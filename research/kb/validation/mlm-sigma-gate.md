---
type: Validation Gate
title: MLM cross-section gate
description: "The five MLM rows: per-@N sigma through the composite integrand against seeded MadEvent references read under the seed policy, the gating rule, and the measured table at refdata-9."
status: draft
tags: [mlm, sigma, madevent-reference, seed-policy, long-tier]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
measured:
  - {commit: 590f87a, host: "Apple M3 Max (16 cores), macOS 15.7", command: "pixi run -e madgraph --skip-deps validate-mlm-sigma"}
sources:
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L303-L530", title: "Note 41 M0, the five rows and their references"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1, the pure-cut sigma"}
  - {id: n41-m2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L758-L1013", title: "Note 41 M2, seeded llj sigma and the negative control"}
  - {id: n41-m3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L1014-L1414", title: "Note 41 M3 and D2"}
  - {id: n41-fb, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2006-L2701", title: "Note 41 F-B, P12 and C"}
  - {id: n41-z, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/41-mlm-feature-sprint-plan.md#L2916-L3489", title: "Note 41 Z, the independent-directory reference and Z2's gate"}
---
# MLM cross-section gate

## The rows

| row | process | card | what it isolates |
|---|---|---|---|
| `pp_to_llj_xqcut_only` | `p p > e+ e- j` | `ickkw = 0`, `xqcut = 20` | the pure cut: the jet-cut rewrite and the clustering's `xqcut` rejection, `pdfwgt = F` |
| `pp_to_llj_mlm` | `p p > e+ e- j` | `ickkw = 1`, `xqcut = 20` | `rewgt`'s `αs` and PDF ratios (+26% over the pure cut) |
| `pp_to_llj_mlm_alps2` | as above | `alpsfact = 2`, `use_syst = F` | where `alpsfact` enters (10% of σ) |
| `pp_to_ll_0j2j_mlm` | `@0` + `j @1` + `j j @2` | as `pp_to_llj_mlm` | the canonical mixed-multiplicity sample |
| `pp_to_ttx_0j1j_mlm` | `t t~ @0` + `j @1` | `xqcut = 30` | the massive core (`mt2last`, the massless–massive `dj`) |

All at 13 TeV, `pdlabel = lhapdf` at `lhaid 247000`, `vector_size = 1`. The
`_alps2` card must switch systematics off: MadGraph forces `alpsfact = 1` under
`use_syst` (`banner.py:4551-4555`, `setrun.f:151-159`, the latter whether or not
matching is on), and this crate applies the same override, so a card with
`use_syst = T` and `alpsfact ≠ 1` silently measures `alpsfact = 1`[^n41-m0].
`xqcut = 30` for `t t̄` is MadGraph's own default for a mixed jet card
(`banner.py:4958`).

## The gate

`pixi run -e madgraph --skip-deps validate-mlm-sigma` runs, in
`validate_hadronic.rs` (long tier, `#[ignore]`), `sigma_llj_xqcut_only_vs_madevent`,
`sigma_llj_mlm_vs_madevent` and `sigma_llj_mlm_alps2_vs_madevent` (through
`mlm_sigma_row`), `sigma_ll_0j2j_mlm_vs_madevent` (gate) and
`sigma_ttx_0j1j_mlm_vs_madevent` (info). It reads only the committed
`mlm_sigma_reference.json`; no MadGraph runs.

- **This side.** The mixed rows build the composite exactly as `vibegraph
  integrate` does — process lines from the row's script, `split_by_multiplicity`,
  union-shape maps, `MultiplicitySum`, the α survey — then integrate at
  `--fixed-budget --allocate neyman --neval 200000 --niter 8`, ten seeds
  (20260928–37), one collator cell per `@N` plus the total; the first seed
  reproduces the CLI to every printed digit. The integrand is
  [phase-space/mixed-multiplicity-integrand](../phase-space/mixed-multiplicity-integrand.md).
  The llj rows run 150000 × 10 per seed (ten seeds for `_mlm`, five for the
  others).
- **The rule.** Both sides read mean ± max(quoted, spread/√n): MadEvent's mean is
  inverse-variance, this side's unweighted. A gating cell needs |pull| < 3 and
  this side's seed χ²/dof inside its 0.1–99.9% band[^n41-z].
- **The references** follow [the seed policy](madevent-reference-seed-policy.md).
  `pp_to_ll_0j2j_mlm`'s is 21 independent directories (`independent_directories:
  true`): the samples-grade `run_01`, itself the first run of a fresh directory,
  and twenty fresh directories `run_s20261101`…`20`. Its error is their spread,
  which puts MadEvent's measured inter-directory spread into the tolerance
  (`@2` per run: sd 0.93 pb, 0.71%, against 0.84 pb quoted). The other rows'
  references are `run_01` plus nine seeds run in one shared directory; those
  seeds inherit each other's grids and scatter at χ²/dof 0.3–0.8, too little to be
  independent draws, and on the mixed row the shared directory read `@1` 0.43%
  high and `@2` 0.34% low against the independent ones.

## Measured (refdata-9)

| row | vibegraph (pb) | MadEvent (pb) | pull | rel | this side χ²/dof |
|---|---|---|---|---|---|
| `pp_to_llj_xqcut_only` | 212.608 ± 0.126 (5) | 212.549 ± 0.229 | +0.23 | +0.03% | 0.82 |
| `pp_to_llj_mlm` | 268.433 ± 0.119 (10) | 268.169 ± 0.284 | +0.86 | +0.10% | 0.86 |
| `pp_to_llj_mlm_alps2` | 241.255 ± 0.154 (5) | 240.710 ± 0.252 | +1.85 | +0.23% | 1.04 |
| `pp_to_ll_0j2j_mlm` `@0` | 665.258 ± 0.217 | 665.001 ± 0.348 (21) | +0.63 | +0.04% | 0.66 |
| `pp_to_ll_0j2j_mlm` `@1` | 268.594 ± 0.451 | 267.874 ± 0.370 | +1.23 | +0.27% | 0.95 |
| `pp_to_ll_0j2j_mlm` `@2` | 131.275 ± 0.663 | 130.908 ± 0.195 | +0.53 | +0.28% | 1.98 |
| `pp_to_ll_0j2j_mlm` total | 1065.126 ± 0.684 | 1063.646 ± 0.546 | +1.69 | +0.14% | 1.06 |
| `pp_to_ttx_0j1j_mlm` `@0` (info) | 513.222 ± 0.177 | 512.898 ± 0.181 | +1.28 | +0.06% | 2.01 |
| `pp_to_ttx_0j1j_mlm` `@1` (info) | 583.192 ± 0.297 | 575.836 ± 0.777 | **+8.85** | **+1.28%** | 0.99 |
| `pp_to_ttx_0j1j_mlm` total (info) | 1096.413 ± 0.381 | 1088.640 ± 0.797 | +8.80 | +0.71% | 1.42 |

The whole task takes 9.5 min on that host and is bit-identical on a second run; a
mixed-row seed is about 35 s there[^n41-z]. `@2`'s error is its seed spread
(quoted errors run 1.2–1.8 pb a seed), because its heavy-tailed channels
understate their own errors.

## Registered deviations and the budget

`@2` carries two known causes with no allowance: H1, the grouped first call
(+0.33 pb, [madgraph-permuted-first-call](madgraph-permuted-first-call.md)), and
the generic 2 → 4 offset (≈ +0.6 pb), 0.7% of `@2` together
([mlm-at2-excess-decomposition](mlm-at2-excess-decomposition.md)). They fit inside
3σ only while this side's `@2` error stays near 0.66 pb. A budget that brings it
below about 0.3 pb needs them as an explicit allowance, or the gate fails on known
causes.

## Standing residuals

- **`t t̄` `@1` is +1.28%, undiagnosed**: all ten seeds (581.9–584.5 pb) sit above
  MadEvent's whole range (572.5–579.7) while `@0` agrees. Not yet separated: the
  shared-directory reference and H1's first-call rejections on this row. The cell
  stays `info`, and a gate needs a five-seed agreement on record first
  ([ttx-mlm-at1-sigma-high](../backlog/validation/ttx-mlm-at1-sigma-high.md)).
- **The three llj rows all sit high**, +0.03% to +0.23%, a shift common to them
  and not following the reweighting; their references come from one shared
  directory each
  ([llj-mlm-rows-sit-high-together](../backlog/validation/llj-mlm-rows-sit-high-together.md)).
- A full banked run on a heavily loaded host once aborted in `validate_hadronic`
  with no panic message, and passed alone on the same data
  ([malloc-abort-under-proton-sample-load](../backlog/validation/malloc-abort-under-proton-sample-load.md)).

## What the gate rests on

- **Per-event checks below it.** Scales, `rewgt` and the record are gated event
  by event in [mlm-dump-oracle](mlm-dump-oracle.md), which also asserts the
  negative control: dropping the `αs` factor moves `pp_to_llj_mlm`'s σ by −14.5%,
  about 140 reference errors.
- **The budget.** Channels that share a phase-space map share one channel (43 on
  the mixed row: 1 / 6 / 36 for `@0` / `@1` / `@2`), and a Neyman re-split is
  handed what the α split spends, floors included; without that, floor-bound
  two-jet channels starved `@0`
  ([phase-space/channel-budget-allocation](../phase-space/channel-budget-allocation.md)).
  The cost of `@2`'s heavy tail is not priced in the allocation, so at this
  `--neval` `@2` is the weak part
  ([mlm-at2-tail-unpriced-in-allocation](../backlog/performance/mlm-at2-tail-unpriced-in-allocation.md)).
- **Samples against their integration.** The writer normalises each `@N` to its
  integrated σ, so a file's `XSECUP` equals the integration by construction; the
  sample check reads the header's pre-normalisation estimate as a pull instead
  ([events/multi-process-normalisation](../events/multi-process-normalisation.md)).
- **The shower's reading** of the matched sample is a separate, informational
  comparison: [mlm-pythia-matched-comparison](mlm-pythia-matched-comparison.md).
  The whole matching design is [scales-pdf/mlm-matching](../scales-pdf/mlm-matching.md).

[^n41-m0]: Note 41 M0 cards and the `use_syst` override; M1 for this crate's port of it.
[^n41-z]: Note 41 Z.4 (the flip rule) and Z2 ("The mixed rows' σ gate", "Corrections to Z.4").
