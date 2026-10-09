---
type: Session Report
title: "R-G2 report: sampling, scale, PDF and event test targets"
description: "22 findings: contradictory seed-combination rules across σ gates, a Drell–Yan OR that makes its bound 1%, a cluster-scale test blind to its defect, duplicated Z-pole tests at the wrong √s, slack rel_tol rows, and seven copies of the σ-gate scaffold."
status: draft
tags: [hygiene, review, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/f7efda6", title: "Reviewed at f7efda6, read-only"}
---
The reviewer's report, condensed by the manager. Mutations were reasoned
from the code, not run.

**Backlog check:** 34 hits. Two filed items miss sites:
`llj-gate-comments-quote-pre-floor-ladders` (R-G2.6) and
`manifest-notes-describe-superseded-state` (R-G2.7, R-G2.8).

## Findings

Paths are under `vibegraph-lib/tests/` unless stated.

| id | point | site | finding | proposed | confidence |
|---|---|---|---|---|---|
| R-G2.1 | 4, 2 | `validate_sigma.rs:4503-4532,1929`; `validate_hadronic.rs:412-433,4474` | "The seed policy" is implemented three ways. The grammar gate reads this side's seeds by 1/σ²; the MLM gates read them unweighted, and `combine_seeds` calls 1/σ² biased. The written policy says inverse-variance for both sides. There are 2 copies of `combine_seeds` and 6 inverse-variance means. | file (decide the rule, then one `common/` helper) | checked |
| R-G2.2 | 2 | `validate_hadronic.rs:507,157-164` | The Drell–Yan gate passes on `pull < 3` OR `rel < 1%`, so its effective bound is 1% (about 4× the pull). The stated reason (floating-point) is wrong. A +0.8% bias passes. | fix here (AND, rel 0.005) | checked |
| R-G2.3 | 2 | `validate_sigma.rs:2791-2850` | `the_sampled_channel_reaches_the_cluster_scale` evaluates every configuration via `sole(channel)`, so it cannot see the production draw; pinning the draw to config 0 passes. | fix here (rename, or assert through the draw) | checked |
| R-G2.4 | 2 | `validate_vegas.rs:113-184` | `sigma_z_pole` and `validate_vegas` are the same computation, both at √s = 91.188 where the bank used 91.2, against a hard-coded copy of the reference. | fix here (drop one; read the JSON and its ebeam) | checked; √s effect suspected |
| R-G2.5 | 2, 1 | `validate_sigma.rs:291-325` | Nine σ rows have `rel_tol` 10–28× their census-measured worst \|rel\|, with no rationale at the constant. | fix here (tighten from the census, quoted) | checked |
| R-G2.6 | 1 | `validate_hadronic.rs:913-917,1400-1410,3286-3291` | Report-cell notes say "three seeds" where the gates use five, and quote the pre-floor ladder. | fix here (missing sites of the llj item) | checked |
| R-G2.7 | 1 | `validate_hadronic.rs:157-168`; `manifest.toml:501` | The Drell–Yan rationale and manifest note quote readings the census re-measured differently. | fix here (with the claimed manifest item) | checked |
| R-G2.8 | 1 | `manifest.toml:300` | The `uux_to_uux` note describes a −0.30% bias the fiducial window removed. | fix here (with the claimed manifest item) | checked |
| R-G2.9 | 1 | `validate_unweighting.rs:130`; `validate_hadronic.rs:1732,3214`; `validate_sigma.rs:1951` | Rationales credit VEGAS's inverse-variance combination; the default is `Unweighted`. | fix here | checked |
| R-G2.10 | 2 | `validate_hadronic.rs:81,1509,3043` | Three-seed χ² gates are bounded by limits calibrated on five. | file (a gate-cost decision) | checked |
| R-G2.11 | 4 | seven `validate_hadronic.rs` gates plus two in `validate_sigma.rs` | One σ-gate scaffold hand-copied. Eleven bare `3.0` pull literals; `llj_dyn`'s status omits its (unreachable) pull assert. | file (`SeedGate`) | checked |
| R-G2.12 | 1, 4 | both 5k gate files | Probes make up about 2.6k and 1.8k lines, with copied helpers and colliding probe names. | file (probe targets, shared helpers) | checked |
| R-G2.13 | 1 | `validate_sigma.rs:289-741` | `plan_for` is 452 lines, and its rationale duplicates the manifest notes (the drift source). | file | checked |
| R-G2.14 | 2 | `rambo_flat_mc.rs:94-99`; `validate_vegas.rs:83-104` | The two-body normalisation is tested at 3% while a sibling claims it is pinned at 0.06%. A ×1.02 weight passes. | fix here | checked |
| R-G2.15 | 2 | `rambo_flat_mc.rs:104-130` | `BANKED_SIGMA_PB = 6.556e-7` against the JSON's 7.1428e-7 (−8.2%). Default-card fallback; no task runs it; window (0.02, 50). | fix here | checked |
| R-G2.16 | 4 | 7 `output_dir`, 3 `banked_runs`, 4 declared-absent variants | Banked-directory helpers are copied, with same-named functions behaving differently. | file (`common::banked`) | checked |
| R-G2.17 | 4 | `validate_kt_cluster.rs`, `validate_mlm_dumps.rs`, six gzip sites | The dump reader is copied, and six sites shell out to `gzip` where `flate2` is a dependency. | file | checked |
| R-G2.18 | 4, 2 | `validate_alphas.rs:102`; `validate_scales.rs:252` | `GRID_ALPHA_S_RUNS` is defined twice with different members; one is asserted. | file | checked |
| R-G2.19 | 2 | samples, unweighting, LHEF and ladder builders | Only the σ builders read `scale_draw_fallbacks`. | fix here | checked; mutation outcome suspected |
| R-G2.20 | 2 | `alphas_reference_grid.rs:108-133` | A cross-platform 4-ulp bound (against AGENTS.md), and the mismatch count is capped at 10 in its message. | fix here | checked |
| R-G2.21 | 2, 1 | `scales_run_cards.rs:17-86` | "Every committed run card" checks 3 of 20; a doc overclaims. | fix here (glob the tracked cards) | checked |
| R-G2.22 | 3 | `validate_mlm_dumps.rs` | `pub` items in a test-crate root with no outside user. | fix here | checked |

**Also seen:**
- hand-typed χ² band tables (`puruspe` could compute them);
- `require` given another test's name;
- a non-ignored asserting `probe_*`;
- long replay and compare functions;
- a duplicate `RefusedRunCard` list;
- `StdRng` is not stable across `rand` versions.

**Cross-cluster patterns:**
- AGENTS.md itself still says VEGAS combines by inverse variance;
- a kb contradiction between `seed-sweeps-and-budget-ladders.md` and
  `madevent-reference-seed-policy.md` (R-G2.1's root);
- a fifth declared-absent rule in `vibegraph-cli/tests/validate_samples_proton.rs`;
- status computed separately from asserts in row-writing tests.

## Method

| step | share | produced |
|---|---|---|
| backlog check | 15% | — |
| outlines and length ranking | 10% | — |
| σ gates, diffing the same statistic across files and against the kb policy | 40% | R-G2.1–2, R-G2.5–8, R-G2.10–11 |
| small files read whole | 15% | R-G2.4, R-G2.14–15, R-G2.20–21 |
| reuse greps | 15% | R-G2.16–19, R-G2.22 |

**Dead ends:**
- `plan_for`'s skip arm, which the collator catches;
- exception lists, all two-way;
- soft-skips (none; all go through `require`).

**Not deep-read:** `validate_samples`, `validate_kt_cluster`, `validate_pdf_grid`
and `decay_chain_ladder`.

## Found

- **The `event_scales` doc (`src/hadronic.rs:2229`) is stale** under the
  `AMP2` draw.
- **`validate_vegas` compares at the wrong √s** (R-G2.4).
- **The `flat_mc_partonic_sigma` reference is 8.2% stale** (R-G2.15); the 2→6
  bank was perhaps re-cut.

## Brief corrections

- **D3 defines *rejected*, not *leave*.** The protocol uses *leave*.
- **The cluster is about 28k lines.**
- **Two filed items miss sites** (above).
- **`git branch` is empty on a detached checkout.**

## Manager check (2026-10-09)

Worktree clean.
- **R-G2.2:** the OR at `validate_hadronic.rs:507` is as quoted.
- **R-G2.15:** `BANKED_SIGMA_PB = 6.556e-7` is at `rambo_flat_mc.rs:104`.
- **R-G2.20:** `MAX_ULPS = 4` is at `alphas_reference_grid.rs:108`.
