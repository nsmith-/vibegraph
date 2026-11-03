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

### Final dispositions

| id | final | commit |
|---|---|---|
| H1 | fixed (MD1 and MD2 both survived the old tests; the new asserts fail them) | `39c92205` |
| H2 | **not reproduced**: MD5 fails `the_rung_ordering_test_fires_on_a_swapped_chain` at `tests/diagram_channel.rs:2280`, the three-rung reversal's check that the last rung's projection does not move. Left alone | – |
| H3 | fixed (MD3 fails it, and the precondition now asserts) | `e4c89e65` |
| H4, H5, H6 | fixed | `e4c89e65` |
| H7 | fixed, readings from the T-F2 report's recalibration table | `e4c89e65` |
| H8 | fixed; printed output identical on 200000 lines | `e4c89e65` |
| H9 | fixed; the new census runs, 76 rows censused, 1 not (`wpwm_to_wpwmz_cw`, `nhel`), the same split as T-F2's 15 + 61 + 1 | `e4c89e65` |
| H10 | fixed; byte-identical dump | `5eefe683` |
| H11–H15 | fixed | `983c08ff` |
| H16 | fixed | `e4c89e65` |
| H17 | routed to validation-dev | – |
| H18, H19, H20 | Found | – |
| H21, H22, H23 | leave | – |

**Backlog items naming reviewed paths.** `feature/three-body-spine-requires-fiducial-scale`
(the item this PR closes; its body describes the old policy, which is the
manager's close-out); `performance/generic-scalar-constants-via-numcast`
(`diagram_channel.rs` is one of its sites, untouched here);
`hygiene/kb-concepts-stale-after-sprint-d` (cites `artifact.rs` at version 14,
now 15); `hygiene/cli-tests-define-pdf-dir-per-file` (eight files; the new
`validate_ewk_lljj_sigma.rs` is a ninth); the `generate.rs` and
`bank_seeded_sigma.sh` items concern other code. None is a finding here; the two
stale counts go to Found.

## Fixed

- **H10, `5eefe683`.** `Spine::draw_rest_invariant` and
  `Spine::rest_invariant_measure` choose a remainder's map (the mixture where
  `rest_mixed`, otherwise `draw_invariant` under `rest_resonance`); `sample_spine`
  and `spine_jacobian` call them instead of branching each on its own.
- **H11–H15, `983c08ff`.** `PointAnatomy` (struct, fields, both methods),
  `ProtonIntegrand::point_anatomy`, `MultiplicitySum::point_anatomy`,
  `HeavyTrial` (fields documented), `HEAVIEST_KEPT`, `UnweightStats::heaviest`,
  `VegasGrid::jacobian_factors` and `ProtonIntegrand::channel_map_keys` are
  `pub(crate)`. `PointAnatomy::outer_jacobian`, never read, is deleted.
  `cargo check --workspace --all-targets` gives no warning, so no user outside
  the crate existed.
- **H1, `39c92205`.** `a_channel_map_digest_that_moved_is_refused` and
  `a_hadronic_channel_map_digest_that_moved_is_refused` assert the banked
  digest differs from `channel_map_digest::<&str>(&[])`.
- **H3–H9, H16, `e4c89e65`.** As listed in the table. The census probe is now
  `probe_the_channel_maps_of_the_banked_rows`: one line a row with its
  channel count, longest chain, fiducial scales and the digest of its keys,
  with its decay-chain blind spot (T-F2's Found) written into its doc.

## Mutations

Driver `T-HY1-mut.py` / `T-HY1-mut2.py` (scratchpad): apply by exact string
replacement with an asserted count, run, restore from a copy. After each batch
`git diff --stat` showed only the session's own edits.

| id | mutation | old tests | new tests |
|---|---|---|---|
| MD1 | `hadronic.rs`: both `self.channel_map_keys = map_keys;` → `Vec::new()` | `cli_generate::a_channel_map_digest_that_moved_is_refused ... ok`; lib `channel_map` + `hadronic::` 25 passed | `... FAILED`, `cli_generate.rs:721: the multichannel artifact banks the digest of no channel's map` |
| MD2 | `proton.rs`: drop `channel_map_keys.push(channel.map_key())` | lib `channel_map` + `proton::` 48 passed; the CLI test was **not run** on the old code (the first batch omitted `--features extended-validation`, so cargo refused the target) | `... FAILED`, `cli_generate_proton.rs:681: the hadronic artifact banks the digest of no channel's map` |
| MD3 | `channel_map_digest` sorts its keys | both digest tests FAILED (`artifact.rs:896`; `diagram_channel.rs:4511` "the digest reads the channel order") | same, now without the conditional |
| MD4 | `map_key` never writes `M` | `map_key_separates_every_field_the_map_reads` FAILED: "'base' and 'remainder mixture' share a key" | unchanged test |
| MD5 | `with_rung_order` no longer re-derives `rest_mixed` | `the_rung_ordering_test_fires_on_a_swapped_chain` FAILED (`tests/diagram_channel.rs:2280`) | H2 not reproduced |
| MD6 | `Spine::rest_mixed` drops its `i + 1 < len` guard | every `phasespace::` and `diagram_channel` test passes | **unreachable**: every constructor already clears the last rung's flag, so the guard is redundant; no test can see it |
| ID-M | (oracle power) `rest_invariant_measure` never takes the mixture | – | the identity dump differs on 35100 of 79200 lines |

Unreachable or unclaimed: MD6 above. The AWAITING path of
`validate_ewk_lljj_sigma` (H17) cannot fail on this side's σ while the row is
planned; that is routed, not fixed. `point_anatomy` has no test (H20).

## Byte-unchanged proofs

- **H10.** A temporary ignored test (`hy1_identity_dump`, appended to
  `tests/diagram_channel.rs`, then removed by restoring the file from its copy)
  wrote every `to_bits` of the walk's weight, its momenta and the rebuilt
  density for 300 draws per arm over the 75 no-scale peripheral diagrams, in
  four arms: production, the 400 GeV² fiducial scale, flat remainders, and every
  multi-rung chain reversed through `with_rung_order` — 79200 lines.
  Before and after: `sha256 e082777e8508608886493efd24222dc58ba6ae823e83af89e0bb701c32d3f0f8`
  both, `cmp` identical. The dump sees the mixture: the ID-M mutation moves
  35100 lines.
- **H8.** A temporary ignored test (`hy1_seed_sweep_identity`, removed the same
  way) fed 100000 synthetic seed sets (5 or 8 seeds, scales 10⁻⁴…10⁴) to the
  old private formulas and to the new `combine` path, formatting both with the
  probes' own format strings: **200000 printed lines, 0 differ**. The f64 values
  are not bit-identical (66500 of 100000 sets differ in some bit:
  `sqrt(var/n)` against `sd/sqrt(n)`, and the χ² term's division order), so the
  claim is the probes' printed output, not their bits.
- **H11–H15.** No executable change: visibility, and a field nothing read.
- **H9** changes what the probe prints, by design; it is not claimed unchanged.
  Its run: `CENSUSED (76)`, `NOT CENSUSED (1)`, 11 s, e.g.
  `bbx_to_ccx_emmm_qcd0: 615 channels, longest chain 2 rungs, fiducial scale {"4e2"}, maps c6691efc…`.

## Gate

On `a8bbd140`'s tree (the code is `e4c89e65`'s; the later commits touch only
`research/kb/`), detached, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0`, `-j 2` (`T-HY1-gate.log`):

```
fmt rc=0            cargo fmt --all --check
clippy rc=0         cargo clippy --workspace --all-targets -- -D warnings
clippy-ev rc=0      cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
clippy-ev-lib rc=0  cargo clippy -p vibegraph-lib --all-targets --features extended-validation -- -D warnings
test rc=0           cargo test --workspace --no-fail-fast -- --test-threads 2
                    49 suites, 1745 passed, 0 failed, 24 ignored
doc-lib rc=0        cargo doc -p vibegraph-lib --no-deps --document-private-items   8 warnings, none in a touched file
doc-cli rc=0        cargo doc -p vibegraph --no-deps --document-private-items       0 warnings
kblint rc=0         python3 scripts/kb.py lint   1020 concepts, 333 backlog items; 0 errors, 4 warnings
```

The suite count moves by no test of this session: the only test-set change is
the ignored census probe's rename. The four kb warnings are pre-existing
`state: blocked` items.

**Banked targets**, `--profile release-debug --config profile.release-debug.debug=0
--features extended-validation`, rerun because `5eefe683` touches the spine's
walk and density:

```
spine-sigma rc=0 177s   --test spine_sigma
  16 seeds: mean 1.799563e-2 pb, spread 1.65e-5 (rel 9.15e-4), seed χ²/dof 1.27
  test result: ok. 1 passed; 0 failed; 1 ignored
cli-gen-ev rc=0 184s    -p vibegraph --test cli_generate_proton    ok. 6 passed; 0 failed; 2 ignored
decay-chain rc=0 144s   -p vibegraph --test cli_decay_chain        ok. 1 passed; 0 failed
onshell rc=0 207s       -p vibegraph --test cli_onshell_veto -- a_top_pair   ok. 1 passed; 0 failed; 3 filtered out
sigma rc=0 986s         --test validate_sigma                      ok. 8 passed; 0 failed; 33 ignored
```

`spine_sigma` reads T-F2's gate run on `d54f0e46` to every printed digit
(1.799563e-2, 9.15e-4, 1.27): the refactor is identical end to end on the
production integrator as well as in the dump. Not run: the long-tier
`validate_ewk_lljj_sigma` (34 minutes, untouched by any fix) and the MLM σ
targets.

## Routed

- **H17 → validation-dev.** `validate_ewk_lljj_sigma.rs:220-226`: while the
  row is planned the test integrates five seeds (about 34 minutes) and passes
  whatever they read. The side's own χ²/dof band (`CHI2_PER_DOF_BAND`,
  0.023–4.617) needs no record and the recorded reading (2.42) sits inside it,
  so it could be asserted on the AWAITING path; doing so changes what the gate
  means while planned, which is not a hygiene call.

## Found

- **H18. Seeded-σ CLI gates copy one scaffolding.** `validate_ewk_lljj_sigma.rs`
  and `validate_param_card_sigma.rs` each define `output_dir`, `pdf_dir`, a
  CLI `integrate` into a temp dir, the seed-record `reference`, the
  pull/rel/χ² check and the `IntegralsRow` write; the first adds a ninth
  `pdf_dir` to the eight of `hygiene/cli-tests-define-pdf-dir-per-file`, whose
  count is now stale. To close: a `tests/common` seeded-gate helper both use.
- **H19. `rest_mixed` is per rung but means per spine.** `spine_chain`,
  `with_rung_order`, `with_mixed_remainders` and `Spine::rest_mixed` each
  restate "every rung but the last" (MD6 shows the guard is redundant). A
  spine-level flag removes the restatements, but `map_key` would encode it
  differently and every digest would move. To close: decide whether a key
  format change is worth it; if so, do it with the next map change that moves
  digests anyway.
- **H20. `point_anatomy` is untested.** Its doc claims the densities are the
  ones the weight was formed from; a wrong channel index in
  `channel_weight_at_recording` would go unseen and mislead an overweight
  diagnosis. To close: a unit test that rebuilds `value_in_channel(j, u)` as
  `αⱼ f/g` from the anatomy for points drawn in several channels.
- **Stale kb and item sites** (not edited; bookkeeping):
  - `research/kb/phase-space/channel-contract.md:136-139`: "A spine is built
    for more than two outgoing legs only when that scale is positive …
    otherwise the channel is the all-timelike tree."
  - `research/kb/phase-space/spacelike-floor.md:110-113`: "When the floor is 0
    … a final state of more than two legs gets no spine"; its `dc-chain` source
    anchor `#L1643-L1708` no longer covers `spine_chain`.
  - `research/kb/phase-space/t-channel-spine.md:116`: "floor no spine is built
    past two outgoing legs".
  - None of the three mentions the remainder mixture or `DEFAULT_POLE_SCALE`.
  - `hygiene/kb-concepts-stale-after-sprint-d` cites `artifact.rs` at
    version 14; it is 15.
  - `hygiene/cli-tests-define-pdf-dir-per-file`: eight files, now nine (H18).

## Method

- Diff read in full, file by file, with the five reports beside it (about
  35%). Leads: the stale calibration (H7) came from reading the docs' numbers
  against T-F2's recalibration table; the history comments from the grep plus
  reading.
- Caller greps for every new `pub` item and accessor (about 15%): H11–H15 and
  the unread field H12, and H1's "no test reads `channel_map_keys`".
- Mutations run, not reasoned (about 30%): H1 confirmed, H2 **not
  reproduced** (an existing three-rung check catches MD5), MD6 unreachable.
- Identity dumps for the two refactors (about 15%). For H8 the bit-level
  comparison failed as expected and the printed-output comparison is the
  claim.
- Leads that did not hold: H2; a suspected silent change of kind for other
  `scripts/*.mg5` rows under the new `card` rule in `bank_seeded_sigma.sh`
  (only `pp_to_epemjj_qcd0` and the five explicit MLM rows have a committed
  `<row>_run_card.dat`); `jacobian_factors` as a second copy of a bin lookup
  (none exists).

## Brief corrections

- The brief's mutation suggestion for the digest refusal ("each gate must be
  able to fail") was met by the existing tests for a *changed* digest; the gap
  was the opposite case, a digest that is trivially equal on both sides (H1).
- The first mutation batch's hadronic CLI target needs
  `--features extended-validation` even in the hermetic layer
  (`cli_generate_proton` is `required-features`).
