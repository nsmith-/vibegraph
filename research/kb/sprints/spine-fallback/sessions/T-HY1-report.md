---
type: Session Report
title: "T-HY1 report: hygiene review of #81"
description: "Hygiene review of the no-scale spine change: the pole floor, the remainder mixture, artifact version 15 with the channel-map digest, the overweight dump, the spine σ and EWK ℓℓjj gates, and the bank driver's card kind. Findings written before any edit, then fixed with mutation evidence and byte-for-byte identity dumps."
status: draft
tags: [hygiene, review, spine, artifact, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
---
The session brief is [T-HY1](T-HY1.md). The review covers
`git diff origin/main...HEAD` on `sprinte-spine-fallback` at `63d9c16a`,
leaving out `research/kb/sprints/` and the backlog items.

## Findings (written before any edit)

Confidence is *checked* where the quoted lines or a command settle it, and
*suspected* where a mutation run still has to settle it; the mutation section
below settles every *suspected* row. The disposition column was filled in when
the table was written and is updated in the final column after the work.

| id | pt | site | evidence | mutation the test should catch | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 2 | `hadronic.rs:2520,2567`, `proton.rs:1656` | No test reads `channel_map_keys()`. The two refusal tests change only the stored digest; integrate and generate both read the integrand's keys, so a key list that is never filled gives the empty-list digest on both sides and round-trips. `grep -rn channel_map_keys vibegraph-*/tests` returns nothing. | MD1: `self.channel_map_keys = map_keys;` → `Vec::new()` (fixed energy); MD2: drop `channel_map_keys.push(..)` (hadronic) | suspected | fix: assert the banked digest is not the empty-list digest and covers one key per channel |
| H2 | 2 | `diagram_channel.rs:1047-1051` (`with_rung_order`) | The rung-order re-derivation of `rest_mixed` (`d54f0e46`) is production (`maps.rs` calls `with_rung_order`). No test named in the T-F2 report reorders a production chain and reads the flags. | MD5: delete the re-derivation loop | suspected | fix if it survives: unit test on a reordered two-rung chain |
| H3 | 2 | `diagram_channel.rs:4510` | `if reordered != spine_keys { assert_ne!(..) }`: the order check is skipped silently if the reversal is a no-op. | MD3: sort the keys inside `channel_map_digest` (order-blind digest); `artifact.rs`'s own test catches it, this one may not run its arm | checked | fix: assert the precondition instead of branching on it |
| H4 | 1 | `tests/diagram_channel.rs:2794` | "Both halves are what the scale used to decide." History. | – | checked | fix |
| H5 | 1 | `tests/diagram_channel.rs:3110-3111` | "which is what a final state past two legs got with no scale before the spine was built for it". History. | – | checked | fix |
| H6 | 1 | `tests/diagram_channel.rs:3387,3392,3425` | `R1 seed-1`, labels `"R1 seed 1, …"`: `R1` is a campaign rung defined only in the T-V2 report (`--neval 10000 --niter 3`). | – | checked | fix: describe the budget |
| H7 | 1 | `tests/spine_sigma.rs:66-87` | The `CHI2_PER_DOF_BAND` and `REL_SPREAD_MAX` docs quote T-V2's calibration (sd/quoted 0.91, per-seed 3.3e-3, gated 1.06 and 1.05e-3, "an earlier gate of eight of them 1.55"). T-F2 changed the maps and re-measured: 40-seed sd/quoted 1.05, per-seed 3.11e-3, 16-of-40 spread 3.9e-4/7.7e-4/1.08e-3, χ²/dof 0.28/1.08/2.09, gated 9.15e-4 and 1.27 (T-F2 report, "Recalibration from 40 seeds per arm"). The doc describes maps the code no longer builds, and "an earlier gate" is history. | – | checked | fix: the recorded T-F2 readings |
| H8 | 1, 4 | `tests/diagram_channel.rs:3065-3083`, `:2700-2707` (pre-merge numbering `3362`) | `seed_sweep` and the inline mean/sd/χ² in `probe_the_no_scale_production_sigma_against_flat_rambo` re-implement `common::seeds::combine` (`tests/common/seeds.rs:67`), which `spine_sigma.rs` already uses. | – | checked | fix: route both through `combine`; identity oracle below |
| H9 | 1 | `tests/diagram_channel.rs:3746-3853` (`probe_the_banked_rows_whose_maps_moved`) | Its "old" arm rebuilds a map no code path builds any more (pole scale `10⁴`, flat remainders) and its doc reads "Which banked σ rows the spine changes move": a probe of this PR's change, history once merged. | – | checked | fix: make it a per-row channel-map digest census that two builds can diff, no reconstructed past |
| H10 | 4 | `diagram_channel.rs:2731-2739` (walk), `:2853-2857` (density) | The walk and the density each decide on their own whether `R_i` is drawn from the mixture or from `draw_invariant`. The two must agree for the estimator to be unbiased; one helper per direction on `Spine` keeps the choice in one place. | – | checked | fix: refactor; identity oracle below |
| H11 | 3 | `proton.rs:1201` `PointAnatomy`, `:2794` and `multiplicity.rs:395` `point_anatomy` | `grep -rn point_anatomy\|PointAnatomy` finds only `generate.rs`, `multiplicity.rs`, `proton.rs`. | – | checked | fix: `pub(crate)` |
| H12 | 3 | `proton.rs:1207` `PointAnatomy::outer_jacobian` | Written at construction, read nowhere (`grep -rn outer_jacobian`: the definition and the constructor). Hidden from `dead_code` only by `pub`. | – | checked | fix: delete |
| H13 | 3 | `unweight.rs:334,339,345` `heaviest`, `HEAVIEST_KEPT`, `HeavyTrial` | Read only in `unweight.rs` and `generate.rs`. | – | checked | fix: `pub(crate)` |
| H14 | 3 | `vegas.rs:770` `VegasGrid::jacobian_factors` | Called only from `generate.rs`. | – | checked | fix: `pub(crate)` |
| H15 | 3 | `proton.rs:1970` `ProtonIntegrand::channel_map_keys` | Called only from `multiplicity.rs`; the CLI reads `MultiplicitySum::channel_map_keys`. | – | checked | fix: `pub(crate)` |
| H16 | 1 | `diagram_channel.rs:1750` | A 130-column doc line in `DEFAULT_POLE_SCALE`'s comment (rustfmt does not wrap docs). | – | checked | fix with H-docs |
| H17 | 2 | `validate_ewk_lljj_sigma.rs:220-226` | The `AWAITING THE BANK` path integrates five seeds and passes whatever they read; the side's own χ²/dof band needs no record but is checked only once one exists. | – | checked | route to validation-dev: enforcing it changes the gate's meaning while planned |
| H18 | 4 | `validate_ewk_lljj_sigma.rs` against `validate_param_card_sigma.rs` | Same scaffolding: `output_dir`, `pdf_dir`, `integrate` through the CLI into a temp dir, the seed-record reference, the pull/rel/χ² check and the `IntegralsRow` write. Cross-file. | – | checked | Found |
| H19 | 4 | `diagram_channel.rs` `SpineRung::rest_mixed` | A per-rung flag for what is a property of the spine and the position (every rung but the last, or none), so three sites (`spine_chain`, `with_rung_order`, `with_mixed_remainders`) and `Spine::rest_mixed` each restate "but the last". A spine-level flag removes them but changes `map_key`'s encoding and so every digest. | – | checked | Found |
| H20 | 2 | `proton.rs:2794` `point_anatomy` | No test: its densities are claimed to be the ones the weight was formed from, and nothing checks it. Diagnostic only. | – | checked | Found |
| H21 | 1 | `bank_seeded_sigma.sh:439-449` | The `card` arm of `row_setup` is the `mlm` arm with one more clause of generator text, and the two kinds are handled alike everywhere else. | – | checked | leave: the generator text is written into each seed record, so a merge saves three lines and changes either kind's record text or keeps a branch anyway |
| H22 | 2 | `tests/spine_sigma.rs` `census` | The census rebuilds channels with `from_diagram_regulated` instead of reading what `use_multichannel` built, and only prints `spines of channels`. The spread bound fails under M1 (T-V2, T-F2), so the gate is not vacuous. | – | checked | leave |
| H23 | 1 | `POLE_FRACTION_OF_FIDUCIAL_SCALE` name | It now scales the default too; its doc says "Fraction of the pole scale". | – | checked | leave: two kb concepts cite the name; renaming is churn against them |

**Backlog items naming reviewed paths.** `feature/three-body-spine-requires-fiducial-scale`
(the item this PR closes; its body describes the old policy, which is the
manager's close-out); `performance/generic-scalar-constants-via-numcast`
(`diagram_channel.rs` is one of its sites, untouched here);
`hygiene/kb-concepts-stale-after-sprint-d` (cites `artifact.rs` at version 14,
now 15); `hygiene/cli-tests-define-pdf-dir-per-file` (eight files; the new
`validate_ewk_lljj_sigma.rs` is a ninth); the `generate.rs` and
`bank_seeded_sigma.sh` items concern other code. None is a finding here; the two
stale counts go to Found.
