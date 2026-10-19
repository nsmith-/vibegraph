---
type: Session Report
title: "P3 report: the bin-width bound default flipped to K = 6"
description: "GridRule::default() bounds every bin at six uniform widths, --grid-max-bin-width none restores the unbounded grid, and the artifact records the rule at format version 12 while an unbounded artifact stays byte-identical to the old writer's; in one sitting K = 6 costs 0.87-0.98 of the unbounded grid's quoted variance x CPU, and the banked layer on the tree combined with the map-default flip passes every cell."
status: draft
tags: [vegas, grid-adaptation, weight-tail, artifact, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: 23b8a42, resource: "https://github.com/nsmith-/vibegraph/commit/23b8a42", title: "vegas: bound every refined bin at six uniform widths by default"}
---
Host: the 4-core cloud container. Worktree `/home/user/wt/vegas-low-acceptance-bins`,
branch `vegas-low-acceptance-bins`, verified before any action.
**Classification: sampling (statistical)** for every default integration; the
unbounded rule is bit-for-bit what it was (below).

Items 1-3 landed in `23b8a42`, with two test fixes after it (`87a3711` by the
manager, `502d75f`); items 4 and 5 are below. No variance-per-CPU claim went
into any doc or comment.

## What landed (`23b8a42`)

- **The flip, in one place.** `impl Default for GridRule` (`vegas.rs`) sets
  `max_bin_width: Some(DEFAULT_MAX_BIN_WIDTH)`, `DEFAULT_MAX_BIN_WIDTH = 6.0`;
  `GridRule::UNBOUNDED` names the old rule. Every integrand (`FixedBeamIntegrand`,
  `ProtonIntegrand`, `MultiplicitySum`, the `ChannelIntegrand` trait default)
  and `VegasGrid::new` read it, so the library tests and the banked layer get
  the bound without a CLI.
- **(a) The way back.** `--grid-max-bin-width <K|none>`; the default is
  printed from the library (`[default: 6]`), numbers keep the `K > 1`
  refusal (`cli_hard_errors` unchanged and passing). `docs/src/cli/reference.md`
  regenerated with `scripts/gen-cli-docs.sh`.
- **(b) The rule in the artifact.** `IntegrateArtifact::grid_rule:
  Option<GridRule>` (last field), `FORMAT_VERSION = 12`
  (`GRID_RULE_VERSION`). By `version_for`'s convention (oldest version that
  holds what is banked), `version_for(channels, grid_rule)` returns 12 for any
  rule but `UNBOUNDED`, and the key-derived 9/10/11 otherwise; an unbounded
  artifact is written through a `v9` schema struct, byte for byte the old
  writer's file. `write_to_path` refuses a version < 12 for any other rule
  (`ArtifactError::UnrecordableGridRule`). Files of versions 3-11 read back as
  `Some(GridRule::UNBOUNDED)`. `None` is reserved for an artifact holding no
  VEGAS grid, so the sampler-kind tag of `artifact-has-no-sampler-kind` can go
  beside the field without a reshuffle (a non-VEGAS writer writes `None`).
  `GridRule`/`BinWeight` derive serde; their field order is the wire format
  (stated on the type). `integrate` records `integ.grid_rule()`, the rule the
  integrand actually used. `generate` replays frozen grids and never refines,
  so it reads no rule; nothing else consumes it.
- Concepts updated to state K = 6: `phase-space/vegas-integrator.md`
  (refinement step 4, the artifact record), `phase-space/vegas-grid-weight-tail.md`
  (new section "The default bounds the bin width at six uniform widths",
  quoting only P2's ε_unw, Hill and per-evaluation variance, no CPU figure).

Tests: `artifact.rs` — `a_recorded_grid_rule_is_a_version_12_file_and_reads_back`
(default, absolute+rescale+2.5, rescale, `None`), `an_unbounded_artifact_is_the_file_a_writer_without_the_rule_wrote`
(bytes equal a test-local version-11 struct's encoding), `write_refuses_a_rule_its_version_cannot_record`,
and `grid_rule == UNBOUNDED` asserted in the v3, v4, v5 and v6-8 upgrade
tests. `cli_fixed_energy.rs` — `cli_integrate_records_its_grid_rule_and_none_unbounds_it`
(`e+ e- > mu+ mu- a`: default artifact byte-identical to `--grid-max-bin-width 6`,
version 12, widest bin ≤ 6 uniform widths; `none` is version 9, rule
`UNBOUNDED`, widest bin > 6, so the bound binds there). `vegas.rs` rule tests
re-based on `GridRule::UNBOUNDED` (they compared rules against the old
default) and extended to the new default: on the low-acceptance strip the
widest bin is unbounded 7.519, K = 6 5.892, K = 4 4.000 uniform widths.

Mutations (each reverted, `cmp` against kept copies): `version_for` ignoring
the rule → `a_recorded_grid_rule…` fails; the v9 upgrade recording the
default → `an_unbounded_artifact…` fails; the library default set to `None`
→ the CLI test fails (`format_version` 9 ≠ 12). Log
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/vegasacc-P3-mut.log`.

## Gates (on `23b8a42`'s tree)

`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/vegasacc-P3/gates.sh`
(log `…/vegasacc-P3-gates.log`, `…/vegasacc-P3-test2.log`):

| gate | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| same, `--features vibegraph/extended-validation,vibegraph-lib/extended-validation` | exit 0 |
| `pixi run --skip-deps kb-lint` | exit 0 (775 concepts, 0 errors, 3 pre-existing warnings) |
| `cargo test --workspace --no-fail-fast` | exit 0; 1538 passed, 0 failed, 16 ignored |

The first workspace run failed two `cli_generate` tests
(`a_pre_draw_artifact_…`): they forge a stale artifact by stamping version 6
onto a fresh one, which now carries the K = 6 rule and is refused by the new
`UnrecordableGridRule` check. Fixed by also stamping `grid_rule =
UNBOUNDED`, which is what a version-6 file records.

**Seeded goldens: none moved.** `test_pinned_seed_regression_grid_adapt`
(`u₀² + u₁`, 50 bins) passes unchanged under the new default: no bin there
reaches the floor, and `bound_bin_width` leaves every bin's importance
untouched (`max(m, ρw) = m`) when none does, so the default is bit-identical
on it. That golden therefore no longer exercises the bound's arithmetic; the
bound's own tests do.

## Byte identity across builds

`vegasacc-P3/inert.sh` (log `…/vegasacc-P3-inert.log`): base = `98240e0`
built from this worktree (`vibegraph-base-dev`, sha256 `db2d95bd…`), tip =
`23b8a42` (`vibegraph-p3-dev`, `14e6f882…`), dev profile, `integrate
--fixed-budget --neval 20000 --niter 4 --seed 20261017 -j 4`:

| row | base default = tip `none` | tip default = tip `6` | σ base / tip default (pb) |
|---|---|---|---|
| `ee_to_mumua` | identical, `ae2dba5d…` | identical, `b8f997aa…` | 0.099919 / 0.100363 |
| `pp_to_llj` | identical, `73054082…` | identical, `286e6c93…` | 502.4665 / 507.7448 |
| `pp_to_llj_mlm` | identical, `a99e89e8…` | identical, `39d038bc…` | 268.3583 / 270.8413 |

`a99e89e8…` is also P1's banked inertness hash for the same command.

## Feature-gated tests the flip could break

CI's banked job on `3b392dd` failed `cli_generate_proton.rs:878`
(`format_version == 11`, now 12 under the default rule); the manager's
`87a3711` asserts 12 and a merged channel. Grepping every
`extended-validation` test (`cli_integrate`, `cli_decay_chain*`,
`cli_generate_proton`, `validate_samples_proton`, `cli_pdf_cache_fetched`,
`cli_reweight_proton`, `vibegraph-lib/tests/validate_*`) for `format_version`,
`grid_rule`, `GridRule`, `to_bits`, hashes and byte-identity found **one more
break in the same test**: lines 1004 and 1034 stamp versions 9 and 10 onto
the fresh (K = 6) artifact to test the refusals, which `write_to_path` now
refuses. Fixed in `502d75f` (the stale copy records `UNBOUNDED`, as a
version-9 file does). The other hits are map-density bit comparisons and
coupling ulp checks, which the grid rule cannot reach. Both proton tests pass
in the banked run below (`a_mixed_multiplicity_card_is_integrated_and_sampled_as_a_sum
... ok`).

## Item 4: one-sitting timing, K = 6 against unbounded

`vegasacc-P3/campaign.sh`, 2026-10-17T22:09:15Z-23:12:47Z, nothing else
heavy on the host: release binary `vegasacc-P3/vibegraph-p3` (sha256
`be00b4ef…`, `cargo build --release` of `23b8a42`), arms
`--grid-max-bin-width none` and `--grid-max-bin-width 6`, seeds
20261101-20, the gates' budgets (`pp_to_llj`, `pp_to_llj_mlm` 150k × 10,
`pp_to_jj` 75k × 10), seed-major with the rows inner and the arm order
alternating by seed parity; 120 runs, none failed. Every run's σ, error and
ε_unw is identical to P1's `base` and P2's `bound6` record on the same seed
(120 of 120), so only the CPU columns are new. Summary:
`python3 vegasacc-P3/summary.py` (output `vegasacc-P3-summary.md`).

| row | CPU per run | quoted² per evaluation | **quoted²·CPU** | spread² (68 %, F p) | spread²·CPU (68 %) |
|---|--:|--:|--:|--:|--:|
| `pp_to_llj` | 0.948 ± 0.011 | 0.963 ± 0.031 | **0.913 ± 0.031** | 1.12 [0.70, 1.78], p 0.81 | 1.06 [0.67, 1.69] |
| `pp_to_llj_mlm` | 0.934 ± 0.007 | 0.932 ± 0.014 | **0.870 ± 0.015** | 1.22 [0.77, 1.94], p 0.67 | 1.14 [0.71, 1.81] |
| `pp_to_jj` | 0.983 ± 0.011 | 0.995 ± 0.007 | **0.978 ± 0.013** | 0.80 [0.50, 1.27], p 0.63 | 0.78 [0.49, 1.25] |

Ratios K = 6 / unbounded. Errors: the ratio of the two arms' means with each
arm's seed-to-seed scatter (`s/√20`) in quadrature; the paired per-seed ratios
agree (0.919 ± 0.020, 0.872 ± 0.011, 0.980 ± 0.011). CPU is the `integrate`
process's user + system seconds. **Reading:** the bound costs less CPU per run
(×0.93-0.98: more points land where the cuts reject, which the cut-first
evaluation prices at almost nothing) at equal or lower quoted variance per
evaluation, so the quoted variance × CPU falls 9 %, 13 % and 2 %, each
resolved from 1 by 1.7-8.7 of its error. The seed-spread form resolves
nothing at 20 seeds (every interval contains 1). ε_unw and Hill are P2's
(×1.38, ×1.59, ×1.22; worst-channel Hill +0.23, +0.48, +0.53).

## Item 5: the banked layer on the combined tree

`p3-combined` (local, not pushed) is `3127deaab5df6c133a48a63a19496f1e2b6a56e0`,
a clean merge of the local `map-default-guards` at
`87e36951c882e8b5c4f3e1a16a522db1923f394a` into `502d75f`; **tree
`6aecfc62dfa916cbbcb7061886359823de0bb8c7`**. (A first `p3-combined`,
`418f923`, tree `25bc9c39…`, merged `origin/map-default-guards` into
`23b8a42`; it was discarded unrun.)

`vegasacc-P3/validate.sh` runs `pixi run --skip-deps validate`:
23:13:53Z-00:24:10Z, exit 0 (log `vegasacc-P3-validate.log`, report copied to
`vegasacc-P3-report/`); clippy with the extended-validation features clean;
1924 passed, 0 failed, 95 ignored; the collator: "64 rows × 4 categories =
256 cells: 203 measured (197 ✅, 6 ⚠️, 13 ⏳) … every gate cell passed",
`problems: []`.

**The brief's baseline cannot isolate the flip.** `mapguards-V1-report-after`
was run at `fba61d4`, which predates #42 (`28d3135`, the five-seed σ gate
policy, `MIN_GATE_SEEDS`); `map-default-guards` merged main afterwards. In that
report 17 of 52 integral rows have several seeds; in the combined one, 51.
`python3 -I mapguards-V1-diff.py mapguards-V1-report-after vegasacc-P3-report`
(full output `vegasacc-P3-diff.txt`) moves 49 integral cells, 34 of them with
a different seed count, with no status change and the same tally on both
sides (197 ✅, 6 ⚠️, 13 ⏳, 15 —, 25 uncovered; `ok` true, no problems). To
attribute moves to the bound, I ran the banked layer on the map flip alone at
the merged commit `87e3695` (tree `715ed6cf…`), 00:26:46Z-01:43:14Z, exit 0,
1912 passed, 0 failed, 95 ignored, the same tally
(`vegasacc-P3-validate-mapbase.log`, `vegasacc-P3-report-mapbase/`). The tables
below diff that run (before) against `p3-combined` (after); the two trees
differ by this sprint's commits only (`vegasacc-P3-moved.md`). Diagrams and
amplitudes: no file differs, durations aside. 20 integral rows are
bit-identical (every `ee_to_mumu*`, `ee_to_ttx*`, `ee_to_zh*`, `ee_to_wp0wm`,
`gg_to_ttx*`, `uux_to_ttx_4f`, `tata_to_ttx_tensor4f`,
`qqx_to_o8o8_toy_dcolor`, `ll_to_qqx_toy_yukawa`, `p3r3_to_p3r3_toy_epsilon`):
the bound does not bind on their grids.

**Integrals: 32 moved, every one inside its limits.** Pull limit 3.5;
`rel_tol` read from the merged tree's `plan_for` (blank where it is not a
literal there).

| row | mode | seeds | σ before → after (pb) | rel before → after | rel_tol | pull before → after (limit 3.5) | χ²/dof before → after |
|---|---|--:|--:|--:|--:|--:|--:|
| ddx_to_epemg | gate pass→pass | 5 | 0.620409 → 0.620423 | +0.206% → +0.208% | 1% | +2.27 → +2.29 | 0.28 → 0.27 |
| ee_to_ee | gate pass→pass | 5 | 155.756 → 155.797 | -0.004% → +0.023% | 1% | -0.04 → +0.58 | 2.48 → 0.07 |
| ee_to_ee_nsz | gate pass→pass | 10 | 157.639 → 157.526 | +0.065% → -0.007% |  | +1.09 → -0.20 | 1.48 → 1.12 |
| ee_to_ee_nsz__unrestricted | gate pass→pass | 10 | 155.733 → 155.803 | +0.467% → +0.512% |  | +1.63 → +1.80 | 0.84 → 0.96 |
| ee_to_mumu_tata_qcd0 | gate pass→pass | 5 | 0.00137543 → 0.00137625 | +0.081% → +0.140% | 2% | +0.61 → +1.06 | 1.49 → 1.65 |
| ee_to_mumua | gate pass→pass | 5 | 0.10064 → 0.100736 | -0.132% → -0.036% | 3% | -1.76 → -0.42 | 0.29 → 1.60 |
| ee_to_tatah | gate pass→pass | 5 | 0.00188422 → 0.00188384 | +0.026% → +0.006% | 2% | +0.46 → +0.11 | 0.58 → 0.42 |
| ee_to_wp0wm__transverse | gate pass→pass | 10 | 6.93857 → 6.93854 | +0.051% → +0.050% |  | +0.77 → +0.76 | 1.56 → 1.55 |
| ee_to_wp0wm__unpolarized | gate pass→pass | 10 | 7.19648 → 7.19647 | +0.018% → +0.018% |  | +0.27 → +0.27 | 1.74 → 1.74 |
| ee_to_wpwm | gate pass→pass | 5 | 7.1963 → 7.1963 | +0.037% → +0.037% | 0.8% | +0.50 → +0.50 | 0.54 → 0.54 |
| ee_to_wpwm_cw | gate pass→pass | 5 | 7.2878 → 7.28779 | +0.029% → +0.029% | 0.8% | +0.39 → +0.39 | 0.40 → 0.40 |
| gg_to_gg | gate pass→pass | 5 | 142950 → 142991 | +0.093% → +0.121% | 0.8% | +1.46 → +1.76 | 1.39 → 1.65 |
| gg_to_gg_cg | info info→info | 1 | 270709 → 270604 | +0.019% → -0.020% |  | +0.15 → -0.17 | 0.82 → 0.72 |
| gu_to_epemu | gate pass→pass | 5 | 0.108786 → 0.108773 | +0.052% → +0.039% | 0.5% | +0.56 → +0.41 | 0.70 → 1.20 |
| gux_to_epemux | gate pass→pass | 5 | 0.108791 → 0.108773 | +0.131% → +0.115% | 0.5% | +1.42 → +1.24 | 0.60 → 1.05 |
| ll_to_qqx_toy_dipole | gate pass→pass | 5 | 78.1492 → 78.1488 | -0.014% → -0.015% | 0.5% | -0.52 → -0.55 | 1.23 → 1.19 |
| ll_to_qqx_toy_tensor | gate pass→pass | 5 | 8038.21 → 8038.11 | +0.067% → +0.066% | 0.5% | +0.97 → +0.95 | 2.77 → 2.73 |
| p3r3_to_p3r3_toy_sextet | gate pass→pass | 5 | 0.00241473 → 0.00241472 | +0.038% → +0.037% |  | +1.10 → +1.11 | 0.68 → 0.61 |
| pp_to_bb | gate pass→pass | 5 | 3.76328e+08 → 3.76276e+08 | +0.078% → +0.064% |  | +1.62 → +1.33 | 1.38 → 1.71 |
| pp_to_bb_fixed | gate pass→pass | 5 | 2.14806e+06 → 2.14821e+06 | +0.161% → +0.168% |  | +1.92 → +1.96 | 0.69 → 1.00 |
| pp_to_bb_qcd2 | gate pass→pass | 5 | 3.76335e+08 → 3.76392e+08 | +0.083% → +0.098% |  | +1.68 → +1.87 | 0.58 → 1.12 |
| pp_to_jj | gate pass→pass | 5 | 6.79781e+08 → 6.79773e+08 | -0.045% → -0.046% |  | -0.47 → -0.51 | 1.22 → 1.03 |
| pp_to_ll__default | gate pass→pass | 5 | 933.647 → 933.921 | +0.007% → +0.037% |  | +0.13 → +0.73 | 1.23 → 0.80 |
| pp_to_ll__mmll_60_120 | gate pass→pass | 5 | 644.473 → 644.457 | +0.052% → +0.050% |  | +0.97 → +1.06 | 0.41 → 0.16 |
| pp_to_ll_scalefact2 | gate pass→pass | 5 | 1957.47 → 1957.59 | -0.029% → -0.023% |  | -0.24 → -0.19 | 1.09 → 0.65 |
| pp_to_llj | gate pass→pass | 5 | 505.143 → 505.313 | +0.163% → +0.197% |  | +0.87 → +1.05 | 0.40 → 0.49 |
| pp_to_llj_dyn | gate pass→pass | 5 | 415.592 → 415.653 | +0.211% → +0.226% |  | +0.94 → +1.01 | 1.03 → 0.53 |
| pp_to_llj_fixed | gate pass→pass | 5 | 423.989 → 424.269 | +0.218% → +0.284% |  | +0.97 → +1.29 | 1.39 → 0.55 |
| ud_to_epemud_qcd0 | gate pass→pass | 5 | 0.0141255 → 0.0141306 | +0.199% → +0.235% | 1% | +1.24 → +1.47 | 0.67 → 0.37 |
| uux_to_epemg | gate pass→pass | 5 | 0.556136 → 0.556083 | +0.195% → +0.185% | 1% | +1.94 → +1.85 | 0.71 → 0.59 |
| uux_to_mumu | gate pass→pass | 5 | 0.0767395 → 0.0767398 | +0.010% → +0.010% | 0.5% | +0.20 → +0.21 | 1.18 → 1.14 |
| uux_to_uux | gate pass→pass | 5 | 28271.5 → 28270.8 | +0.032% → +0.029% | 0.5% | +0.62 → +0.48 | 0.92 → 1.45 |

The closest a gate integral sits to a limit after the flip is
`ddx_to_epemg`'s pull +2.29 (0.65 of 3.5; +2.27 before); then
`pp_to_bb_fixed` +1.96, `pp_to_bb_qcd2` +1.87, `uux_to_epemg` +1.85. Pulls
move by at most 1.34 (`ee_to_mumua`, −1.76 → −0.42), in both directions. No
cell leaves its limit.

**Samples: 28 moved, every gate cell passing.** Floor `P_FLOOR = 1e-4`.

| row | mode | status | min KS p before → after (floor 1e-4) | min χ² p before → after | min scale KS p before → after |
|---|---|---|--:|--:|--:|
| ddx_to_epemg | gate | pass→pass | 1.73e-03 → 2.69e-03 | 6.98e-01 → 7.82e-01 | 7.62e-02 → 3.64e-02 |
| ee_to_ee | gate | pass→pass | 6.14e-02 → 1.63e-01 | 1.37e-01 → 2.16e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_ee_nsz | gate | pass→pass | 7.44e-02 → 1.34e-01 | 2.70e-01 → 3.85e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_mumu_tata_qcd0 | gate | pass→pass | 8.29e-02 → 2.20e-02 | 1.75e-01 → 1.26e-01 | 6.29e-01 → 2.80e-01 |
| ee_to_mumua | info | info→info | 6.26e-04 → 9.33e-04 | 1.64e-02 → 1.24e-01 | 1.01e-02 → 3.75e-02 |
| ee_to_tatah | gate | pass→pass | 7.37e-02 → 7.29e-02 | 4.42e-01 → 3.94e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_tlt | gate | pass→pass | 1.57e-01 → 1.58e-01 | 7.60e-01 → 6.93e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_wpwm | gate | pass→pass | 1.57e-04 → 1.57e-04 | 2.89e-01 → 2.89e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_wpwm_cw | gate | pass→pass | 2.41e-04 → 2.41e-04 | 1.10e-01 → 1.10e-01 | 1.00e+00 → 1.00e+00 |
| ee_to_z0h | gate | pass→pass | 1.26e-01 → 9.92e-02 | 1.95e-01 → 1.95e-01 | 1.00e+00 → 1.00e+00 |
| gg_to_gg | gate | pass→pass | 2.40e-01 → 1.11e-01 | 1.45e-01 → 1.40e-01 | 1.00e+00 → 1.00e+00 |
| gg_to_gg_cg | gate | pass→pass | 3.60e-02 → 1.61e-02 | 2.23e-02 → 3.80e-02 | 3.60e-02 → 1.61e-02 |
| gu_to_epemu | gate | pass→pass | 5.25e-04 → 4.05e-03 | 1.04e-01 → 6.99e-02 | 9.06e-01 → 1.47e-01 |
| gux_to_epemux | gate | pass→pass | 2.20e-02 → 2.32e-02 | 8.73e-02 → 1.30e-01 | 4.69e-01 → 7.92e-01 |
| p3r3_to_p3r3_toy_sextet | gate | pass→pass | 7.56e-02 → 7.40e-02 | 3.63e-01 → 3.63e-01 | 1.00e+00 → 1.00e+00 |
| pp_to_bb | gate | pass→pass | 4.54e-02 → 9.83e-02 | 3.26e-01 → 1.91e-01 | 6.30e-01 → 6.93e-01 |
| pp_to_bb_fixed | gate | pass→pass | 1.49e-02 → 8.35e-02 | 7.20e-02 → 1.56e-01 | 1.00e+00 → 1.00e+00 |
| pp_to_bb_qcd2 | gate | pass→pass | 3.51e-02 → 3.15e-02 | 5.17e-01 → 2.54e-01 | 4.03e-01 → 4.58e-01 |
| pp_to_jj | gate | pass→pass | 6.67e-02 → 1.32e-01 | 5.58e-02 → 6.11e-03 | 8.06e-01 → 7.35e-01 |
| pp_to_ll__default | gate | pass→pass | 1.25e-02 → 2.77e-03 | 1.24e-02 → 1.95e-02 | 1.00e+00 → 1.00e+00 |
| pp_to_ll__mmll_60_120 | gate | pass→pass | 5.71e-02 → 2.84e-03 | 7.71e-02 → 1.88e-01 | 1.00e+00 → 1.00e+00 |
| pp_to_ll_scalefact2 | gate | pass→pass | 3.48e-02 → 3.37e-02 | 2.87e-01 → 4.86e-01 | 1.02e-01 → 1.14e-01 |
| pp_to_llj | gate | pass→pass | 1.76e-02 → 2.32e-02 | 9.80e-02 → 3.33e-02 | 2.45e-01 → 3.85e-02 |
| pp_to_llj_dyn | gate | pass→pass | 5.85e-03 → 1.91e-03 | 3.37e-01 → 5.19e-01 | 4.68e-01 → 7.14e-01 |
| pp_to_llj_fixed | gate | pass→pass | 8.88e-02 → 9.76e-02 | 4.53e-02 → 8.63e-02 | 1.00e+00 → 1.00e+00 |
| ud_to_epemud_qcd0 | gate | pass→pass | 3.90e-02 → 2.17e-02 | 1.11e-01 → 2.60e-01 | 1.00e+00 → 1.00e+00 |
| uux_to_epemg | gate | pass→pass | 1.33e-02 → 7.07e-03 | 5.38e-01 → 5.91e-01 | 9.98e-02 → 1.06e-02 |
| uux_to_uux | gate | pass→pass | 3.14e-01 → 2.99e-01 | 2.66e-01 → 3.66e-01 | 1.00e+00 → 1.00e+00 |

The lowest gated p-value after is `ee_to_wpwm`'s KS 1.57e-4, unmoved (its σ
moves in the last digits, its sample does not); among cells that moved, the
lowest is `pp_to_llj_dyn`'s KS 1.91e-3 (from 5.85e-3), 19× the floor.

## Corrections to the brief

- Item 5's baseline, `mapguards-V1-report-after`, predates #42's five-seed σ
  gates, so a diff against it does not isolate this flip; the moved-cell
  list above is against a banked run of `map-default-guards` alone at the
  merged commit, with the brief's diff summarised beside it.

- "generate … read the rule from the artifact": `generate` never refines a
  grid, so there is no rule for it to read; the artifact field is provenance.
- The backlog item `artifact-has-no-sampler-kind` says adaptation settings
  stay out of the wire format and that a VEGAS-only artifact stays at "the
  version it has today"; after this commit a default artifact is version 12,
  so that item's byte-identity criterion now refers to version 12 (or 9-11
  for unbounded runs).

## Found

- **#51's banked report is not a baseline for a tree merged after #42.**
  `mapguards-V1-report-after` was taken at `fba61d4`, before the five-seed σ
  gate policy; a diff of it against any later tree attributes the policy's
  moves to whatever else changed (34 of the 49 integral cells it moves here).
  Close: the re-record (#45) takes its "before" from a run of the tree it
  lands on, e.g. `vegasacc-P3-report-mapbase/` (map flip alone, `87e3695`).
- **`importance-sampler-seam.md` predates the recorded rule.** Its serialise
  step puts `GridRule` in a per-kind config; the artifact now carries it at
  top level as `Option<GridRule>`. Close: one line in that concept with the
  sampler-kind work.
- **The VEGAS golden no longer covers the bound.** It does not bind on
  `u₀² + u₁`. Close: a pinned-seed golden on an integrand where the default
  bound binds.
- **The brief's gate list misses the feature-gated tests.** It names the
  hermetic suite and clippy with features, not `cargo test` with
  `extended-validation`; the `cli_generate_proton` breaks surfaced only in
  CI. Close: add the feature-gated test binaries (or the banked run) to the
  pre-commit gate of any change that moves artifact versions or defaults.

**Banking request: none.**
