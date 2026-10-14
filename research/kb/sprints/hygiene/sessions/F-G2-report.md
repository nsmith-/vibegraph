---
type: Session Report
title: "F-G2 report: sampling, scale, PDF and event test fixes"
description: "All 14 R-G2 findings and five claimed items fixed in 8 commits: σ bounds tightened to recorded readings, a cluster-scale test that sees the draw, the Z-pole and RAMBO tests bound by their own errors, fallback counters read everywhere, the jj η components fixed; σ calibrations found drifted at the tip."
status: draft
tags: [hygiene, fix, tests, sampling, manifest, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-10}]
sources:
  - {id: 1139b39, resource: "https://github.com/nsmith-/vibegraph/commit/1139b39", title: "stale comments, scales module doc, jj η components"}
  - {id: 8836842, resource: "https://github.com/nsmith-/vibegraph/commit/8836842", title: "Drell–Yan σ gate: pull AND rel"}
  - {id: ff08484, resource: "https://github.com/nsmith-/vibegraph/commit/ff08484", title: "nine rel_tol rows; the configuration draw reaches the cluster scale"}
  - {id: 585017c, resource: "https://github.com/nsmith-/vibegraph/commit/585017c", title: "Z-pole, RAMBO and flat-MC tests read the bank; fallback counters"}
  - {id: ac34222, resource: "https://github.com/nsmith-/vibegraph/commit/ac34222", title: "α_s grid relative bound; every committed run card; private mlm items"}
  - {id: 2f61c60, resource: "https://github.com/nsmith-/vibegraph/commit/2f61c60", title: "manifest notes"}
  - {id: 5a0d378, resource: "https://github.com/nsmith-/vibegraph/commit/5a0d378", title: "manifest notes"}
---
The dev agent's report, condensed by the manager. Logs are `fg2-*` in the
scratchpad. Each tightened test fails on its mutation, and the pre-fix test
passes the same mutation.

## Fixed

| finding or item | commit | change | mutation: new / pre-fix |
|---|---|---|---|
| R-G2.2 | 8836842 | The Drell–Yan gate is `pull < 3 && rel < 0.005` (6.9× the census worst); census readings quoted | σ ×1.008: fails (pull 10.7) / passes |
| R-G2.5 | ff08484 | Nine `rel_tol` rows tightened from the larger of the census and a tip re-run of the same probe, both quoted (0.005, 0.008 or 0.01) | σ ×1.007–1.012 with inflated error: all nine fail / pass |
| R-G2.3 | ff08484 | `the_configuration_draw_reaches_the_cluster_scale` sweeps the draw through `event_scales_at` | Draw pinned to config 0: fails / the old test passes |
| R-G2.4, R-G2.14 | 585017c | Duplicate Z-pole test removed; `sigma_z_pole` reads the bank's σ and √s (91.2), pull bound 3; two-body tests bounded by the Z effect plus 4× own error | ×1.02: both fail / both pass |
| R-G2.15 | 585017c | `flat_mc_partonic_sigma` reads the bank (it was 8.2% stale) and requires the run's card | — |
| R-G2.19 | 585017c | `assert_no_scale_draw_fallbacks` in the samples, mumua, unweighting, LHEF and ladder builders | Forced fallback: LHEF and unweighting fail / pass |
| R-G2.20 | ac34222 | α_s grid bound is relative 1e-12, and every mismatch is counted | A ×10 Newton tolerance moves 275 points by up to 3.9e-7: fails |
| R-G2.21 | ac34222 | Reads all 19 committed run cards | `nhel` appended to a decay card: fails / passes |
| R-G2.22 | ac34222 | `validate_mlm_dumps` items private | — |
| R-G2.6, .9 | 1139b39 | llj report strings say five seeds; four rationales stop crediting inverse-variance combination | — |
| R-G2.7, .8, manifest-notes-describe-superseded-state | 2f61c60, 5a0d378 | `pp_to_ll`, `uux_to_uux`, `gu`/`gux`, `ud_to_epemud_qcd0`, `qqx_to_o8o8`, `kt-cluster`, `couplings-mg` notes corrected; `qqx_to_o8o8`'s quoted G and residuals were also wrong | — |
| validate-hadronic-calibration-comments-superseded | 1139b39 | jj, recarded-llj and MLM rationale quote the current recorded readings | — |
| validate-scales-module-doc-stale | 1139b39 | `unwgt.f:752`, `reweight.f:1275`; `<mgrwt>` on the ten proton runs; `scalefact` pinned; three fixed-scale runs | — |
| validation-test-comments-stale | 1139b39 | All nine sites fixed | — |
| jj-banked-orderings-eta-uses-wrong-components | 1139b39 | `[E, px, py, pz]` read correctly; counts re-recorded 1765/1857 | An independent Python parse gives the same counts |

## Gate (agent)

- **Banked, at 2f61c60:**
  - `validate_vegas` 2 (one test removed)
  - `rambo_flat_mc` 1, plus `flat_mc_partonic_sigma` with `--ignored`
  - `validate_sigma` 7
  - `validate_hadronic` 14
  - `validate_scales` 10
  - `validate_unweighting` 1
  - `validate_lhef` 3
  - `validate_samples` 6
  - `validate_pdf_grid` 20
  - `validate_mlm_dumps` compile only
- **Hermetic, at 5a0d378:**
  - **fmt:** passes.
  - **clippy:** passes in both configurations.
  - **`cargo test --workspace`:** 1391 passed, 16 ignored.
  - **The manifest-reading tests:** pass after the last notes commit.

## Found

1. **σ calibrations have drifted at `3f401f0` against the census.**
   `ee_to_ee` worst \|rel\| 1.44e-3 → 3.24e-3 (χ²/dof 2.31);
   `gg_to_gg` 1.97e-3 → 2.66e-3; `ll_to_qqx_toy_tensor` 1.40e-3 → 2.23e-3;
   `ee_to_mumu_4f` 1.33e-3 → 6.4e-4. Re-record `plan_for` and the census page.
   **The sprint did not cause this** (see the manager check below): the drift
   predates the sprint.
2. **Hadronic manifest gate readings no longer reproduce:** `pp_to_jj`,
   `pp_to_llj` and `llj_dyn`.
3. **The `pp_to_jj` integrals note's "tightest cell" claim is stale.**
4. **Stale kb:**
   - `validation/sigma-gate.md:94-99`
   - `pipeline/overview.md:86-92`
   - `phase-space/rambo.md:84-92`
   - `scales-pdf/alpha-s-sources.md:51-53`
   - the census page
5. **The `validate_scales` "geometric-mean unpinned" bullet** is probably stale.
   Not verified.
6. **A stray 4-line `run_card.dat` at the repository root** (from `5331e45`).
7. **`flat_mc_partonic_sigma` compares a cut-free integral with a cut σ.** It
   is ignored, and no task runs it. Fix it or delete it.
8. **The `kt-cluster` "75 MB" inputs size** is unverified.
9. **Inline fallback checks remain in `validate_sigma` and `validate_hadronic`.**
   They belong with the σ-gate scaffold item.
10. **Test-target intra-doc links can't be checked by `cargo rustdoc`**
    (E0433).

## Brief corrections

- **The census no longer describes the tip for every row.** Bounds use the
  larger of the census and a tip re-run of the same probe.
- **R-G2.5's ratios are 9.9–29.9×.**
- **The banked gates ran at `2f61c60`;** only whitespace and note text came
  after.

## Manager check (2026-10-10)

- **Gate re-run at 5a0d378:** fmt passes; clippy exits 0 in both
  configurations; `cargo test --workspace` gives 37 suites, 1391 passed,
  0 failed, 16 ignored.
- **σ-drift attribution:** `probe_gate_row_seed_headroom` was run at the
  pre-sprint commit `669f3fa`
  (`cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma probe_gate_row_seed_headroom -- --ignored --nocapture`).
  Its 68 `HEADROOM` lines are identical, digit for digit, to F-G2's run at
  the sprint head `3f401f0`: every worst \|rel\|, pull, mean and χ²/dof
  matches. The sprint moved no σ on any gated row. The drift from the 2026-09
  census predates the sprint, and is filed for re-calibration.
