---
type: Session Report
title: "I-Z2 report: validate-deep on the v0.2.0 release candidate (2b26c109)"
description: "The long tier on #113's approved head: the 2→6 σ rows, the bb and re-carded ladders and the CMS 880-member PDF check pass; kT-cluster cannot run on Linux; all 15 catch-all failures are expected, environmental or pre-sprint-I defunct probes."
status: draft
tags: [validation, long-tier, release, sprint-i]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-09}]
sources:
  - {id: head, resource: "https://github.com/nsmith-/vibegraph/commit/2b26c109f9973d1d294f48f998023ba6c4764951", title: "2b26c109, #113's head"}
---
I-Z2 ran steps 0–4 and the first catch-all run. I-Z2b resumed the catch-all
after a container restart and wrote this report.

Manager's spot checks:
- the `test result` lines of the 2→6, both ladder and both CMS logs;
- the five MLM σ gates' `ok` lines in the resumed catch-all;
- the cluster-scale probe's 167.34195398462134 value in both the 2b26c109 and
  the d87ad3ae logs;
- the two ignore attributes cited for the expected failures, in the source.

The rest is the sessions' account.


Every vibegraph run was at HEAD 2b26c109f9973d1d294f48f998023ba6c4764951, tree 12f87108dbb8c92b7a681e27a195e79e85bbc8ec, tracked-clean, refdata stamp 5a99624aa283d306814afb5a7bb42d3678487b198dba685f16b886626015eb5d. The only exception is the d87ad3ae control. Nothing was committed or pushed, and rc-deep is still tracked-clean at 2b26c109. /home/user/wt/pre-i has been removed (`git worktree remove --force`), and df is back to 15 GB free.

**Bottom line:**
- None of the 15 failures in the catch-all is a sprint I regression.
- They fall into three groups:
  - 9 are expected or environment failures: a known ignore reason, a missing environment variable or data, or doctests that can't compile.
  - 2 are the MLM dump oracles, which never ran because their dumps are absent.
  - 4 are probes whose own premise is wrong. All 4 fail identically at d87ad3ae (pre-sprint-I).
- **The per-event MLM scale oracle (`matched_scales_reproduce_madevents_event_by_event`) did not run here.** Nothing in this report validates the integrand's clustered scales against MadEvent event by event.

All logs are in /tmp/claude-0/-home-user-vibegraph/45497bfc-9152-5fee-929b-f6d31d701a40/scratchpad/:
- `iz2-*.log` are I-Z2's steps.
- `iz2-catchall-resume.log` holds the three resumed runs, with the kills and the reorder recorded inline.
- `iz2b-prei.log` is the d87ad3ae control.
- `iz2b-pre107.log` is the bisect, started and then cancelled.
- `iz2b-lastmod.tsv` gives the last-modified commit of every ignored test.

## 1. Steps and tasks

All times are UTC on 2026-11-08/09. The HEAD and tree are as above unless the row says otherwise.

| Step | Command | Start | End | Wall | Exit | Result |
|---|---|---|---|---|---|---|
| 0 build | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --no-run` | 23:28:26 | 23:47:46 | 19m20s | 0 | built |
| 1 sigma-2to6 | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_sigma -- --ignored --nocapture the_long_tier_sigma_rows_match_madgraph` | 23:48:04 | 00:07:09 | 1145 s | 0 | Info only, one seed (20260719) at 600000×8. **bbx_to_ccx_emmm_qcd0:** vg 1.690982e-6 ± 2.120e-9 pb against MG 1.689940e-6 ± 2.565e-9 pb, rel +6.17e-4, pull +0.31, integration χ²/dof 60.13. **uux_to_ccx_emmm_qcd0:** vg 7.211300e-7 ± 1.499e-9 against MG 7.177100e-7 ± 1.084e-9, rel +4.77e-3, pull +1.85, integration χ²/dof 559.83. |
| 1b collator | `cargo run --profile release-debug -p vibegraph-validation-report` | 00:07:09 | 00:07:19 | 10 s | 1 | Fails only because this worktree has no banked cells ("declared 'banked' and no gate wrote it — the cell is missing"). |
| 2 ladder-bb | `… --test validate_hadronic -- --ignored --nocapture probe_bb_budget_ladder` | 00:07:19 | 00:08:24 | 65 s | 0 | **pp_to_bb_fixed**, MG 2.144600e6 ± 1.291e3. Pulls by rung: 75k +1.88, 150k +0.68, 300k +0.29, 600k +0.96. Rel spans +0.0002 to +0.0016. |
| 2 ladder-recarded | `… probe_recarded_budget_ladder` | 00:08:24 | 00:21:44 | 800 s | 0 | Every rung is within pull 1.98. **pp_to_bb:** +0.06/+0.06/+0.07/+0.04%, pulls 1.35/1.59/1.98/1.08. **pp_to_bb_qcd2:** +0.10/+0.04/+0.02/+0.06%. **pp_to_llj:** +0.25/+0.26/+0.14/+0.19%, χ²/dof 0.45/0.23/1.12/1.07. **pp_to_ll_scalefact2:** +0.03/+0.01/+0.02/+0.04%, χ²/dof 0.12/0.47/0.24/0.68. |
| 3 fetch-cms | `VIBEGRAPH_FETCH_CONSENT=1 bash validation/pdf/fetch_cms.sh` | 00:21:44 | 00:25:10 | 206 s | 0 | fetched |
| 3 validate-cms-pdfs | `pdf::table_oracle` (lib), then, with `LHAPDF_DATA_PATH=$SYSTEMATICS_CMS_PDFS`, `validate_systematics cms_arguments_reproduce_madgraphs_weights the_pdfas_member_reads_its_own_coupling_and_clamp` | 00:25:10 | 00:26:18 | 68 s | 0 | **Members:** all 880 (43 sets) report "differ 0". **Weights:** each of pp_to_ll, pp_to_jj, pp_to_ll_0j2j_mlm and pp_to_ttx_0j1j_mlm has 9,260,000 of 9,260,000 exact, with the worst field ≤ 5.000e-8. **Controls:** the "MLM terms dropped" control is DETECTED only on the two MLM runs; the "set's coupling" control is DETECTED on three runs and blind on pp_to_ll. **pdfas test:** passed. |
| 4 kT probe | `VG_KT_STAGE=extract VG_NB_CORE=4 bash validation/madgraph/gen_kt_cluster_dumps.sh ee_to_ttx` | 00:26:21 | 00:27:08 | 47 s | 1 | The replay is not byte-identical, so validate-kt-cluster was **not run**; LHAPDF is also absent. The first differing event line is 55255: `-0.72372879882E+02` banked against `…881E+02` replayed, a last-digit difference of about 1.4e-11 relative. I-Z2 reports 2 of 70,001 lines differ in all. The banner also differs, because the replay MG prints "WARNING: UNKNOWN DEVELOPMENT VERSION" (372 against 375 lines). |
| catch-list | `cargo test --workspace --profile release-debug --features vibegraph/extended-validation --no-fail-fast -- --ignored --list` | 00:27:08 | 00:49:57 | 1369 s | 0 | 110 ignored tests listed |
| catch-all run 0 (I-Z2) | `cargo test --workspace --profile release-debug --features vibegraph/extended-validation --no-fail-fast -- --ignored --nocapture` plus the 12 substring `--skip`s, with `SYSTEMATICS_CMS_PDFS=$PWD/validation/pdf/cms` | 00:51:36 | killed 01:20:38 (container restart) | 29 min | none | 26 ok and 2 FAILED; the eemumu spine probe was running at the kill. |
| catch-all run 1 (I-Z2b) | Same, plus `--exact` and 40 exact `--skip`s (28 done + 12). `--list` matched the remaining 70 exactly. | 01:39:25 | killed 02:41:31 by me, on the manager's reorder | 62 min | none | 11 ok, 2 FAILED |
| catch-all run 2 | Same, plus run 1's finished tests and the 5 scatter-guard calibrations as `--exact --skip`. `--list` gave exactly 53. | 02:41:57 | killed 05:10:11, on the manager's call to drop the validate_sigma probes | 148 min | none | 15 ok, 9 FAILED |
| catch-all run 3 | `cargo test --workspace … --test validate_systematics --test weighted_search -- --ignored --nocapture`, then `… --doc -- --ignored --nocapture` | 05:10:35 / 05:12:33 | 05:12:33 / 05:12:37 | 118 s / 4 s | 101 / 101 | 1 ok, 5 FAILED |
| d87ad3ae control | `cargo test -p vibegraph-lib --profile release-debug --features extended-validation [--no-fail-fast] --test validate_hadronic/validate_samples/validate_scales -- --ignored --nocapture --exact <4 probes>`, at HEAD d87ad3ae692a2 (tree 71e56f53, clean, same refdata stamp) | 05:13:21 | 05:18:19 | 210 s + 69 s | 101 | All 4 probes fail identically (section 3). |
| 082298c1^ bisect | same 4 probes at 9d1e9e6f | 05:18:39 | killed 05:19:00 | none | none | Cancelled by the manager once d87ad3ae settled the question. |
| ladder-2to6 | none | none | none | none | none | **Not run**, by the user's decision for v0.2.0 (estimated 6–8 h here). |

A deviation from the brief: run 2 had to rerun validate_hadronic's `probe_cluster_scale_spread_over_configurations`. libtest's `--skip` matches on the name alone, and the same name also selects validate_sigma's probe, which hadn't run yet. The rerun reproduced the failure bit for bit.

## 2. Catch-all per-test results (98 selected; 12 skipped by design)

Duration is the time from the start of the test binary to the result line. Up to 4 tests run in parallel on this 4-core host, so the true duration is at most this.

**Run 0 (iz2-catchall.log)**
- **cli_generate_proton**
  - matched_sample_record_fractions_against_madevent: ok, 5 s
  - probe_sample_sigma_seed_headroom: ok, 42 s
- **cli_pdf_cache**
  - pinned_checksums_match_the_upstream_archives: ok, 3 s
- **validate_samples_proton**
  - probe_jj_flavour_chi2_mg_half_splits: ok, 6m24
  - probe_jj_flavour_chi2_calibration_inverse_square: ok, 7m23
  - probe_jj_flavour_chi2_second_grid_pair_inverse_square: ok, 7m29
  - probe_jj_flavour_chi2_calibration_log: ok, 7m42
  - probe_jj_flavour_chi2_second_grid_pair_log: ok, 12m49
  - probe_proton_samples_p_floor_headroom: ok, 16m55
- **lib**
  - roofline_census: ok, 2 s
  - probe_vertex_leg_binding: ok, 2 s
  - zeroamp_node_reduction_table: ok, 4 s
  - **hel_expand_stats::expansion_bounds_arenas: FAILED, 5 s (EXPECTED).** Its ignore reason: "fails on three of the four processes…".
  - helicity_contribution_spectrum: ok, 5 s
  - **probe_process_diagrams: FAILED, 5 s (EXPECTED).** It needs `VG_PROBE_NAME`.
  - probe_from_diagram_volume: ok, 9 s
  - all_rootings_preserve_amplitude: ok, 9 s
  - execution_order_metrics: ok, 15 s
  - alternative_orders_are_bit_identical_2to6: ok, 28 s
  - pdf::table_oracle::every_member_reads_as_its_coefficient_tables_did: ok, 30 s
  - probe_2to6_tail_points: ok, 39 s
  - probe_shared_outer_tier_replay: ok, 2m50
  - probe_rejected_point_cost: ok, 3m08
  - probe_cut_edge_weight_tail: ok, 3m44
- **diagram_channel**
  - probe_the_channel_maps_of_the_banked_rows: ok, 21 s
  - probe_the_walk_density_tail_against_the_pole_scale: ok, 2m28
  - probe_the_no_scale_production_sigma_against_flat_rambo: ok, 4m13
  - probe_the_photon_pair_corner_against_the_pole_scale: ok, 5m15
  - probe_the_no_scale_spine_on_a_mass_cut_peripheral_process: **interrupted** after about 7.5 min, then rerun in run 1.

**Run 1**
- **diagram_channel**
  - probe_the_no_scale_spine_on_a_mass_cut_peripheral_process: ok, 29m42. Its output reproduces run 0's lines exactly, e.g. "all-timelike … mean 8.79645e3".
- **finite_field_msq**
  - measure_full_msq: ok, 3m34
  - measure_trace_form: ok, 6m36
- **rambo_flat_mc**
  - flat_mc_partonic_sigma: ok, 57 s
- **spine_sigma**
  - probe_the_seed_scatter_of_the_no_scale_spine: ok, 1m30
- **ufo**
  - **test_load_taudecay: FAILED, 1 s (EXPECTED).** It panics with `Lorentz(UnknownOperator("FFCT2"))`, which is its own ignore reason.
- **validate_hadronic**
  - **probe_cluster_scale_spread_over_configurations: FAILED, 3 s.** See section 3.
  - probe_accepted_point_floor_hadronic: ok, 1m19
  - probe_fiducial_bound_on_llj_fixed: ok, 2m22
  - probe_jj_channel_partition: ok, 3m14
  - probe_alpha_survey_budget_hadronic: ok, 4m12
  - probe_dynamic_rows_seed_sweep: ok, 11m08
  - probe_hadronic_seed_headroom: ok, 20m28
- Mid-run at the 02:41:31 kill:
  - probe_llj_dyn_scatter_guard_calibration (running about 20 min)
  - probe_llj_dyn_scatter_guard_calibration_legacy (about 19 min)
  - probe_llj_dyn_scatter_guard_calibration_soft_all (about 12 min)
  - probe_llj_fixed_scatter_guard_calibration (about 3 min)

  All four were later dropped (see section 4).

**Run 2 (validate_hadronic finished at 7 passed, 2 failed, 6469 s)**
- **validate_hadronic**
  - probe_cluster_scale_spread_over_configurations: FAILED again, identical values.
  - **probe_scale_draw_cost: FAILED, 6 s.** See section 3.
  - probe_llj_weight_tail_regions: ok, 3m16
  - sigma_llj_mlm_alps2_vs_madevent: ok, 4m28
  - sigma_llj_xqcut_only_vs_madevent: ok, 9m36
  - sigma_llj_mlm_vs_madevent: ok, 14m45
  - sigma_ll_0j2j_mlm_vs_madevent: ok, 24m02
  - sigma_ttx_0j1j_mlm_vs_madevent: ok, 24m53
  - probe_llj_seed_ensemble: ok, 107m49. Results:
    - pp_to_llj: rel +0.0017 on every rung.
    - pp_to_llj_dyn, 40 seeds a rung: 415.5986 / 415.4747 / 415.4616 pb against MG 414.716 ± 0.895, rel +0.0021 / +0.0018 / +0.0018. 3 of 8 quintets climb monotonically.
- **validate_mlm_dumps**
  - **matched_scales_reproduce_madevents_event_by_event: NOT RUN, 2 s.** The ktdump dumps are absent: "no dump at …/validation/madgraph/output/ktdump/dumps/pp_to_llj_mlm.jsonl.gz". They are outside the bundle and built by validate-mlm-dumps with MadGraph. This is not a vibegraph failure.
  - **the_jet_memo_is_the_channel_s_restricted_jet_count: NOT RUN, 2 s.** Same reason.
  - mg_run_card_matches_madevents_banner: ok, 2 s. 64 of 65 shared fields agree; the one difference is iseed, MadEvent "20260928" against vibegraph "0".
- **validate_samples**
  - **probe_pta_windows_against_madgraph: FAILED, 4 s.** See section 3.
  - probe_samples_p_floor_headroom: ok, 6m24
- **validate_scales**
  - **probe_first_channel_cost_in_alpha_s: FAILED, 1m07.** See section 3.
- **validate_sigma**
  - probe_2to6_density_decomposition: ok, 13 s
  - probe_2to6_eval_cost: ok, 9m38
- Interrupted at the 05:10:11 kill:
  - probe_2to6_sample_cost (running 32m52)
  - probe_accepted_point_floor (32m52)
  - probe_alpha_collapse (32m39)
  - probe_alpha_survey_budget (23m14)

**Run 3**
- **validate_systematics**
  - cms_arguments_reproduce_madgraphs_weights: ok, 1m57. Same numbers as step 3: 9,260,000 of 9,260,000 exact on each run, worst ≤ 5.000e-8.
  - **madgraphs_module_on_other_files: FAILED, 6 s (EXPECTED, environment).** It panics with `SYSTEMATICS_FILES: NotPresent`, as its ignore reason says.
  - **the_pdfas_member_reads_its_own_coupling_and_clamp: FAILED, 6 s (EXPECTED, environment).** It needs `LHAPDF_DATA_PATH`, which the catch-all's environment doesn't set. It **passed** in step 3, where `LHAPDF_DATA_PATH` was set.
- **weighted_search**
  - **a_final_state_gluon_pair_without_a_qcd_vertex_is_refused: FAILED, 1 s (EXPECTED).** It fails with "expected NoDiagrams, got Ok(1)". Its ignore reason: "passes once the search trials each subprocess with its final-state gluons removed".
- **doctests**
  - **diagrams/mod.rs (line 17): FAILED (EXPECTED).** It is an `ignore` snippet that can't compile: E0425 `ufo_path`.
  - **helas/repr/vectorspace.rs impl_add_for_array (line 17): FAILED (EXPECTED).** It fails with "cannot find macro `impl_add_for_array`".

**Tally of the 98 selected tests:**
- 53 ok.
- 15 FAILED. Of these, 9 are expected, 2 are MLM dump oracles that are not-run in effect, and 4 are probes with a broken premise.
- 30 not run. These are the 26 validate_sigma tests that never started plus the 4 interrupted ones. **Correction:** in my mid-run messages I said "30 unfinished validate_sigma tests"; the right count is 26 never started plus 4 interrupted, out of validate_sigma's 32.

## 3. The four failing probes: they fail identically at d87ad3ae (pre-sprint-I) and are not regressions

Each one fails with the same values at d87ad3ae and at 2b26c109.

1. **validate_hadronic::probe_cluster_scale_spread_over_configurations**, at :3316, on the first point of [pp_to_llj_dyn].
   - Values: rebuilt (167.34195398462134, [167.34195398462134, 167.34195398462134]) against the integrand's (91.43648415225013, [91.43648415225013, 91.43648415225013]).
   - What's wrong: the probe assumes the configuration it clusters in is the drawn channel's diagram (`drawn.channel + 1 == config`). The integrand instead draws each group's configuration ∝ AMP2 from the trailing uniform (`proton.rs::per_group_sum`). validate_sigma's `probe_the_scale_draw_reads_the_point_and_not_the_sampler` documents that design: "a function of the point and of the trailing uniform … not of the sampling channel". **Defunct.**
2. **validate_hadronic::probe_scale_draw_cost**, at :4636.
   - Values: left (1, 1), right (1, 0).
   - What's wrong: the probe expects `2 = sde_strategy` to switch the configuration draw off. But `scale_draw_ndim` follows `hadronic.rs::draws_configuration()`, which tests only `PerEvent { channels: Some(_) }`; sde_strategy selects only the weights (`amp2_configuration_weights`). **Defunct.**
3. **validate_samples::probe_pta_windows_against_madgraph**, panicking at hadronic.rs:3056.
   - Values: "a point is 5 map coordinates and 1 scale-draw uniforms", left 5, right 6.
   - What's wrong: the probe builds points without the scale-draw uniform. **Defunct** against the current API.
4. **validate_scales::probe_first_channel_cost_in_alpha_s**, at :1270.
   - Values: the `aS in SMINPUTS` panic.
   - What's wrong: six banked runs are in CLUSTERED_RUNS and have toy-model param cards with no SMINPUTS 3: ll_to_qqx_toy_{dipole,tensor,yukawa}, p3r3_to_p3r3_toy_{epsilon,sextet} and qqx_to_o8o8_toy_dcolor. In sorted order the first is ll_to_qqx_toy_dipole. I inferred this from the cards and did not trace it. **Defunct** against the current bundle.

## 4. Not run, with reasons

- **The 12 skipped by design:**
  - Ten ladders: decay_chain_sampler_ladder, probe_2to6_budget_ladder (ladder-2to6, the user's v0.2.0 call, 6–8 h), probe_bb_budget_ladder and probe_recarded_budget_ladder (run as step 2), probe_h4l_width_ladder, probe_jj_budget_ladder, probe_llj_{deep,dyn,fixed}_budget_ladder, and proton::tests::probe_mirror_visibility_ladder.
  - validate_kt_cluster's two tests: LHAPDF is absent and the dump generator's byte-identity precondition fails (step 4).
- **The five scatter-guard calibrations** (probe_llj_dyn_scatter_guard_calibration[_legacy|_soft_all] and probe_llj_fixed_scatter_guard_calibration[_legacy]): not run, dropped by the user's call that the ignored probes are largely defunct. Four of them were interrupted in run 1, at the times in section 2.
- **validate_sigma, 30 tests**: not run, as probes, dropped on the user's call that the ignored probes are largely defunct; the_long_tier_sigma_rows was measured in step 1.
  - 4 were interrupted: 2to6_sample_cost (32m52), accepted_point_floor (32m52), alpha_collapse (32m39), alpha_survey_budget (23m14).
  - 26 never started: alpha_survey_seed_sensitivity, channel_dedup_census, channel_map_degeneracy, channel_partition_moves_sigma, cluster_scale_spread_over_configurations, ee_nsz_channel_split, event_readout_cost, fixed_density_llj_annihilation, gate_row_seed_headroom, grammar_seed_sweep, grid_adaptation_is_the_residue, llj_parton_seed_stability, non_sm_seed_stability, per_channel_grid_variance_cpu, photon_pole_is_the_instability, polarized_single_runs, qcd_seed_stability, resonant_seed_stability, sampled_channel_cost_in_alpha_s, scale_cost, scale_draw_cost, smeft_capstone_seed_stability, the_scale_draw_reads_the_point_and_not_the_sampler, unweighting_weight_max, vegas_iteration_path and the_long_tier_sigma_rows_match_madgraph. All but the last carry a `probe_` prefix.
- **The two MLM dump tests**: not run, because the ktdump dumps are absent (they are outside the bundle and built by validate-mlm-dumps with MadGraph).

## 5. Classification of all 110 ignored tests

Each entry gives the last commit that touched the test (its fn and doc range, from git blame). Marks:
- **F** = failed here.
- **D** = I think it is defunct.
- **E** = it needs environment input or data outside the bundle.

**Gates and oracles (they compare against MadGraph or a banked record)**
- matched_sample_record_fractions_against_madevent (e360871b)
- pinned_checksums_match_the_upstream_archives (e9d451ba; network)
- pdf::table_oracle::every_member_reads_as_its_coefficient_tables_did (1812407b; E, CMS PDFs)
- all_rootings_preserve_amplitude (d571a5d6; an MG-validated sweep)
- alternative_orders_are_bit_identical_2to6 (94ed9072; a self-equality gate)
- sigma_ll_0j2j_mlm_vs_madevent, sigma_llj_mlm_alps2_vs_madevent, sigma_llj_mlm_vs_madevent, sigma_llj_xqcut_only_vs_madevent (all 403cff84)
- sigma_ttx_0j1j_mlm_vs_madevent (25ab0016)
- the_long_tier_sigma_rows_match_madgraph (2d107ab1)
- cms_arguments_reproduce_madgraphs_weights (e360871b; E)
- the_pdfas_member_reads_its_own_coupling_and_clamp (e360871b; E, F in the catch-all only because LHAPDF_DATA_PATH was missing)
- matched_scales_reproduce_madevents_event_by_event (403cff84; E, dumps absent)
- the_jet_memo_is_the_channel_s_restricted_jet_count (403cff84; E)
- mg_run_card_matches_madevents_banner (e360871b)
- the_clustering_engine_reproduces_madgraphs_own (d54e544c; E)
- derived_channel_forests_match_the_generated_ones (d571a5d6; E)
- test_load_taudecay (d571a5d6; F, a known unsupported FFCT2: a pending gate)
- a_final_state_gluon_pair_without_a_qcd_vertex_is_refused (f2df52c0; F, a pending gate for missing behaviour)
- hel_expand_stats::expansion_bounds_arenas (d8276da4; F, a pending gate)

**Probes and diagnostics (they measure, and assert only their own premise)**
- **cli and samples**
  - probe_sample_sigma_seed_headroom (403cff84)
  - the five probe_jj_flavour_chi2_* (35a1b157)
  - probe_proton_samples_p_floor_headroom (b18a4ea9)
  - probe_samples_p_floor_headroom (02e8b25b)
  - **probe_pta_windows_against_madgraph (cc8b12c0; F, D)**
- **lib**
  - probe_2to6_tail_points (d571a5d6)
  - probe_from_diagram_volume (e31f7c4f)
  - zeroamp_node_reduction_table (d8276da4)
  - roofline_census (aeb96a74)
  - probe_vertex_leg_binding (d8276da4)
  - helicity_contribution_spectrum (d8276da4)
  - execution_order_metrics (052a00ec)
  - probe_cut_edge_weight_tail (dabf284a)
  - probe_rejected_point_cost (3994351c)
  - probe_shared_outer_tier_replay (024f8585)
  - probe_mirror_visibility_ladder (d8276da4)
- **ladders**
  - decay_chain_sampler_ladder (1539abc1)
  - probe_h4l_width_ladder (1c4e628c)
- **diagram_channel and spine**
  - probe_the_channel_maps_of_the_banked_rows (23d3a766)
  - probe_the_no_scale_production_sigma_against_flat_rambo (1c4e628c)
  - probe_the_no_scale_spine_on_a_mass_cut_peripheral_process (23d3a766)
  - probe_the_photon_pair_corner_against_the_pole_scale (23d3a766)
  - probe_the_walk_density_tail_against_the_pole_scale (23d3a766)
  - probe_the_seed_scatter_of_the_no_scale_spine (23d3a766)
- **measurements**
  - measure_full_msq, measure_trace_form (ef84a125)
  - flat_mc_partonic_sigma (d571a5d6)
- **validate_hadronic**
  - probe_accepted_point_floor_hadronic, probe_alpha_survey_budget_hadronic (1c4e628c)
  - probe_bb_budget_ladder, probe_dynamic_rows_seed_sweep, probe_fiducial_bound_on_llj_fixed, probe_hadronic_seed_headroom, probe_jj_budget_ladder, probe_llj_deep_budget_ladder, probe_llj_dyn_budget_ladder, probe_llj_fixed_budget_ladder, probe_recarded_budget_ladder (all cc8b12c0)
  - **probe_recarded_budget_ladder: D (stale source comments only).**
  - probe_jj_channel_partition (f52ac93c)
  - the five *_scatter_guard_calibration* (35a1b157)
  - probe_llj_seed_ensemble (e73b158b)
  - probe_llj_weight_tail_regions (d8276da4)
  - **probe_cluster_scale_spread_over_configurations (b18a4ea9; F, D)**
  - **probe_scale_draw_cost (b18a4ea9; F, D)**
- **validate_scales**
  - **probe_first_channel_cost_in_alpha_s (e73b158b; F, D)**
- **validate_sigma**
  - probe_2to6_density_decomposition (7ddcba2c)
  - probe_2to6_eval_cost (7f132d94)
  - probe_2to6_sample_cost (1cf96dd5)
  - probe_accepted_point_floor (1c4e628c)
  - probe_alpha_collapse (e31f7c4f)
  - probe_alpha_survey_budget (1c4e628c)
  - probe_alpha_survey_seed_sensitivity (cc8b12c0)
  - probe_channel_dedup_census (e0590920)
  - probe_channel_map_degeneracy (d571a5d6)
  - probe_channel_partition_moves_sigma (1cf96dd5)
  - probe_cluster_scale_spread_over_configurations (d8276da4). It wasn't run, and I suspect it shares the hadronic twin's premise. Check before deleting it.
  - probe_event_readout_cost (d571a5d6)
  - probe_gate_row_seed_headroom (28d3135b)
  - probe_grammar_seed_sweep (cc8b12c0)
  - probe_grid_adaptation_is_the_residue (e31f7c4f)
  - probe_llj_parton_seed_stability, probe_non_sm_seed_stability, probe_resonant_seed_stability, probe_smeft_capstone_seed_stability (8d1461fe)
  - probe_per_channel_grid_variance_cpu, probe_photon_pole_is_the_instability (1c4e628c)
  - probe_polarized_single_runs (cc8b12c0)
  - probe_qcd_seed_stability (e61762fd)
  - probe_sampled_channel_cost_in_alpha_s (d8276da4)
  - probe_scale_cost (d571a5d6)
  - probe_scale_draw_cost (d571a5d6). Not run. It is a likely twin of the defunct hadronic one; check it.
  - probe_the_scale_draw_reads_the_point_and_not_the_sampler (e73b158b)
  - probe_unweighting_weight_max (86793c05)
  - probe_vegas_iteration_path (e31f7c4f)
- Not run and E, but probes: probe_2to6_budget_ladder (e487ee8c), probe_ee_nsz_channel_split (1c4e628c) and probe_fixed_density_llj_annihilation (e487ee8c). They take optional VG_* environment knobs.

Three of these validate_sigma probes are the oldest: probe_alpha_collapse, probe_grid_adaptation_is_the_residue and probe_vegas_iteration_path, all at e31f7c4f from 2026-07-26, diagnosing one historical collapse. These are the most likely to be defunct, though that is unverified.

**Tools (dumps that need environment input)**
- probe_process_diagrams (d571a5d6; F, needs VG_PROBE_NAME, E)
- madgraphs_module_on_other_files (e360871b; F, needs SYSTEMATICS_FILES, E)
- **the two doctests (diagrams/mod.rs, afaddaea 2026-11-05; vectorspace.rs): F, D as ignored-tests.** They are `ignore` snippets that `--ignored` tries to compile, and they never can. Making them `text` or `no_run` would keep them out of every catch-all.

## Found

1. **Stale ladder comments.** The source comments on validate_hadronic.rs RECARDED_ROWS (lines 3700–3725) don't match a fresh run, while pp_to_bb's comment matches it exactly. Since pp_to_bb reproduces, the ladder is deterministic, and the other two rows' readings were taken on older code.
   - pp_to_llj: the comment says +0.28/+0.20/+0.22/+0.21%, χ²/dof 1.32/0.49/0.49/1.43, "the lowest rung highest". The run reads +0.25/+0.26/+0.14/+0.19%, χ²/dof 0.45/0.23/1.12/1.07.
   - pp_to_ll_scalefact2: the comment says −0.02/+0.00/+0.03/+0.05%. The run reads +0.03/+0.01/+0.02/+0.04%.
   - pp_to_llj's `ladder_note` string goes into the report cell, so the report repeats the stale numbers.
   - Fix: re-read the readings and their budget justification from a current run.
2. **The kT dump generator demands bit-exactness, which AGENTS.md argues against.** gen_kt_cluster_dumps.sh requires the MadEvent replay to be byte-identical to the bank. On Linux, 2 of 70,001 lines differ in the 11th digit, so validate-kt-cluster can't run here. Its banner normalisation also fails when MG reports itself as an unknown development version. Fix: compare numerically, at a tolerance with measured ulp headroom, and normalise the dev-version banner.
3. **Four ignored probes have broken premises, all pre-sprint-I** (details in section 3):
   - The cluster-scale probe assumes config == channel.
   - The scale-draw-cost probe assumes sde_strategy gates the draw.
   - The pta-window probe builds points without the draw uniform.
   - The first-channel αs probe assumes every Clustered run has SMINPUTS 3.
   Each needs fixing or deleting. Their validate_sigma namesakes should be checked for the same premise.
4. **Toy-model runs are in CLUSTERED_RUNS without SMINPUTS 3.** These are ll_to_qqx_toy_{dipole,tensor,yukawa}, p3r3_to_p3r3_toy_{epsilon,sextet} and qqx_to_o8o8_toy_dcolor. Any validate_scales code that reads αs from the param card must skip them, or take αs from the model.
5. **The catch-all's environment can't satisfy every gated test.** the_pdfas_member test needs LHAPDF_DATA_PATH, which step 3 sets and the catch-all doesn't. The MLM per-event oracles need ktdump dumps that the bundle doesn't ship. A `validate-deep` recipe should set the first and fetch or build the second, or the catch-all reports environment noise as failures and silently doesn't run the finest scale oracle there is.
6. **`--ignored` compiles `ignore` doctests, which always fail.** The two snippets (diagrams/mod.rs:17, vectorspace.rs:17) should be `text` or `no_run`.
7. **Ignored tests share names across binaries.** probe_cluster_scale_spread_over_configurations and probe_scale_draw_cost each exist in both validate_hadronic and validate_sigma. libtest's `--skip`/`--exact` can't select one of a pair, so a resumable catch-all can't skip exactly what is finished. Fix: unique names.
8. **The VEGAS iteration χ²/dof is very large on the 2→6 info rows** in step 1: 60.13 for bbx and 559.83 for uux, at 600000×8. The test's own note says single-seed σ "swings by up to 4.8%". The iteration combination there is not self-consistent, so the quoted error is suspect; the rows are info-only. This is worth a look when the 2→6 ladder is run.
9. **mg_run_card_matches_madevents_banner** reports `iseed` "0" on our side against MadEvent's 20260928. That is benign if intended, but our banner records no seed.
10. Probes with no fixed threshold: probe_llj_seed_ensemble's pp_to_llj_dyn arm sits at +0.18% over MG at every rung (40 seeds a rung, MG ± 0.22%). This is consistent and not a failure. It is informational only.
