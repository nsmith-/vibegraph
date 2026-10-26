---
type: Session Report
title: "HY1 report: hygiene review of the HT cuts and their gates"
description: "Findings on the ht-cuts PR's diff (maintainability, non-vacuity, visibility, reuse), written before any edit, then their dispositions, mutation evidence, gates and Found."
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-23}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-23}
---
Reviewed `git diff origin/main...HEAD` at `7daaab9` (F1 `d4731f0`, V1 `4cb654d`,
`ed5d8ea`) in `/home/user/wt/K`. The worktree has no `validation/madgraph/output`
and an empty `research/refs/mg5amcnlo`; `cuts.f` was read from
raw.githubusercontent.com at `b7687064`.

## Findings (written before any edit)

| id | point | site | evidence | mutation the test should catch | confidence | disposition |
|---|---|---|---|---|---|---|
| H1 | 2 (bug) | `vibegraph-lib/tests/validate_hadronic.rs:3352-3369` `seed_record_banked` | After `ed5d8ea` no manifest row is `planned` (`grep -n 'status = "planned"' validation/manifest.toml` returns only the comment at `:72`); both HT rows are `bundled = false`. The function checks the seed record, then `planned_rows()`, then calls `require`. It never reads `bundled`, so a checkout with the pinned bundle, which does not carry these rows, fails both gates with "needs a banked seed record" instead of printing `AWAITING BUNDLE` as `dyn_run_present` does for every other `bundled = false` row. The planned branch is unreachable for any row in this tree | the gate run on a checkout without the rows' runs must not fail | checked (run below) | fix |
| H2 | 2 | `validation-report/src/main.rs:586-622` `each_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing` | asserts `pp_to_jj_ht` and `pp_to_bb_iht` are planned banked rows in the committed manifest. They are not since `ed5d8ea`, so the hermetic suite is red at `7daaab9`. Its "ordinary" half, `text.replace("status = \"planned\"\n", "")`, is a no-op on the committed text | removing the collator's planned branch must fail it | checked (baseline run below) | fix |
| H3 | 1 | `vibegraph-lib/src/cuts.rs:2846-2848`, doc of `htj_is_the_scalar_jet_sum_with_inclusive_edges` | cites "`cuts.f:908`'s `<` and `>`" for the `htj` edges; `cuts.f:892` is the `htj` check (`if(htj.lt.htjmin.or.htjmax.ge.0d0.and.htj.gt.htjmax)`), `:908` is the inclusive one | – | checked (`grep -n 'htj.lt.htjmin' cuts.f` → `892`) | fix |
| H4 | 1, 4 | `vibegraph-lib/src/cuts.rs:2813-2829` `along_x`, `along_y`, `along_minus_x` | the same module's test helper `lep(pt, y, phi)` (`:1483`) is `V::new(pt·cosh y, pt·cos φ, pt·sin φ, pt·sinh y)`; `along_x(pt, y)` is `lep(pt, y, 0.0)` component for component, and the other two are `lep` at `π/2` and `π` | – (refactor; identity oracle: `to_bits` of `transverse` and `rapidity` at every call site's arguments, then the suite) | checked | fix |
| H5 | 2 | `vibegraph-lib/src/cuts.rs:2284` `unbindable_value_does_not_excuse_other_cuts` (line the PR changed) | `assert!(detect_unimplemented(&lhc("1e5 = missetmax\n1e5 = ht2max\n")).is_err())` is an any-refusal assert. `missetmax` precedes `ht2max` in `UNIMPLEMENTED_CUTS` (positions 3 and 31), so if the collider-ceiling excusal of `missetmax` were lost the refusal would name `missetmax` and the assert would still pass | drop the `SINGLE_PT_MAXIMA` ceiling skip in `detect_unimplemented` | checked by reading; run below | fix |
| H6 | 2 | `vibegraph-lib/src/cuts.rs` `ht_minima_and_maxima_are_not_refused`, second loop | `is_err()` on `ht2min`..`ht4max`. The same test asserts the base `lhc` card plus each compiled HT name is `Ok`, so a refusal with one partial-sum name set can only name that field; no one-line mutation separates `is_err` from a name match | – | checked | leave |
| H7 | 4 | `vibegraph-lib/src/cuts.rs:574`, `:1052`, `:1056` | the default-off rule ("a maximum is read only when `≥ 0`, a minimum only when `> 0`") is spelled inline per threshold: in HT's activity guard and twice in `ht_ok`, and the same way for `pt`/`e`/`eta` (`:963-975`), `dr`/`mm`/`ptll` (`:987-1007`, `:313-317`), `mmnl` (`:550`, `:1027`) and the ŝ window (`:942-951`). HT follows the module; merging HT's two alone would make it the odd one | – | checked | Found |
| H8 | 1 | brief question: one place for the jet and b classes | `classify` (`cuts.rs:1207-1213`) is the only decision; `ht_cut` reads `LegInfo::letter` (`:577-586`), as the single-leg cuts and `cut_class` do. No finding | `htj` takes the b class; HT ignores `maxjetflavor` | checked (mutations below) | no finding |
| H9 | 1 | brief question: `do_cuts` not read | module doc `:58-64` and `HtCut` doc `:330-334` say so; `decay_products_count_for_ht` and `a_heavy_jet_class_leg_counts_for_ht` pin it (F1's M5). No finding | – | checked | no finding |
| H10 | 1 | stale kb sites the change leaves | `research/kb/phase-space/multi-tier-vegas-and-cut-edges.md:98` says `htjmin` is "refused today" at `cuts.rs:379`; `research/kb/validation/cms-gridpack-survey-2026-10.md:203` lists the HT cuts' late refusal under "Other refusals" while `:246`, edited by this PR, says closed | – | checked | Found |
| H11 | 3 | `vibegraph-lib/tests/common/manifest.rs:110` `planned_rows`, collator `:361-368` | the byte-identical port; after H1's fix `planned_rows` has no caller in this tree (the module carries `#![allow(dead_code)]` for that reason) and the collator branch is reached by no committed row. Kept identical per the brief | – | checked | leave |

Backlog check: `grep -rlE 'cuts\.rs|validate_hadronic\.rs|validation-report/src/main\.rs|common/manifest\.rs' research/kb/backlog` — results below; none of the listed items names a site H1-H11 touch wrongly.

The backlog grep lists 11 items (`banked-param-card-join-copied-in-tests`, `madgraph-file-parsers-duplicated`, `kinematic-helpers-hand-rolled`, `local-toolchain-lags-ci-stable`, `madgraph-citations-unverified-outside-cuts`, `restrict-card-lookup-triplicated`, `validation-test-docs-glued-and-vegas-claim`, `three-body-spine-requires-fiducial-scale`, `ckkw-l-merging-refused`, `spacelike-floor-too-loose`, `drawn-scale-config-frequency-law-ungated`). None of them covers a site in H1-H11. The brief calls the planned-row spellings "a filed follow-up", but `git grep -lE 'dyn_run_present|seed_record_banked|planned.row'` finds no item under `research/kb/backlog` on this branch, `origin/main` or `main`.

## Mid-session change of base

While H2 was in progress, `git fetch origin sprintd-ht-cuts` showed the remote at `a1caa84` (`7daaab9..a1caa84`). That range holds `f694e5a`, "declare the planned-row fixture in the test", plus a merge commit. `f694e5a` re-inserts `status = "planned"` but leaves `bundled = false` in the planned half. On the manager's instruction I discarded my uncommitted test edit, fast-forwarded to `a1caa84` (`git merge --ff-only`), and committed the three-state test on top of it.

## Dispositions

| id | disposition | commit |
|---|---|---|
| H1 | fixed | `cd24673` |
| H2 | fixed (re-done on `a1caa84`, where `f694e5a` had made it green but blind to the planned rule) | `cabd7e7` |
| H3 | fixed | `34e00fa` |
| H4 | fixed | `34e00fa` |
| H5 | fixed | `34e00fa` |
| H6 | leave: the same test asserts `Ok` for the base card with each compiled name, so a refusal can only name the one partial sum set; MH below fails it as it stands | – |
| H7 | Found | – |
| H8, H9 | no finding; mutations MA, MB below | – |
| H10 | Found | – |
| H11 | leave (byte-identical port, per the brief) | – |

## Fixed

- **H1** (`cd24673`, `vibegraph-lib/tests/validate_hadronic.rs` `seed_record_banked`). The function now calls `dyn_run_present(gate, row)` first, then reads the seed record, then calls `require(gate, "a banked seed record", row)`. The planned branch is gone: no row in this tree is planned, and `dyn_run_present` is the existing rule for rows the bundle does not carry. When both the run and its record are present, the path into `measure_recarded_sigma` is the same as before.
- **H2** (`cabd7e7`, `validation-report/src/main.rs`). The test is now `an_unbundled_or_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing`. It replaces `each_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing`; the suite count is unchanged. For each HT row it splices the row's own `script = …\nbundled = false\n` lines, asserting that the splice matches exactly once, and checks three declarations by `(mark, body, problem count)`:

  | declaration | expected |
  |---|---|
  | as committed | ⏳ "awaiting the bundle", 0 problems |
  | `bundled = false` → `status = "planned"` | ⏳ "planned: no reference run yet", 0 problems |
  | neither | ❌ "not measured", 1 problem |

  The production hunks of the port are untouched.
- **H3, H4, H5** (`34e00fa`, `vibegraph-lib/src/cuts.rs` tests). The citation now reads `:892`. The three helpers are deleted, and the call sites use `lep(pt, y, 0.0 | FRAC_PI_2 | PI)`. `unbindable_value_does_not_excuse_other_cuts` now matches `UnimplementedCutActive { name }` with `name == "ht2max"`.

H4 identity oracle, chosen before the edit and then deleted: a temporary test `tmp_hy1_identity_oracle` checked, for every `pt` in {150, 149.99, 150.01, 200, 40.2, 100, 100.01, 99.99, 400, 50} and every `y` in {-1.5, -1, 0, 1, 1.5, 2}:
- `along_x(pt, y) == lep(pt, y, 0.0)`;
- `to_bits` equality of `transverse`, `rapidity`, `E` and `pz` between `along_y` / `along_minus_x` and `lep` at π/2 / π, with `transverse(lep) == pt` to the bit;
- the m² bits of both back-to-back pairs (`pt` = 150 and 40.2).

```
$ cargo test -p vibegraph-lib --lib cuts::tests::tmp_hy1 -- --nocapture
HY1 identity oracle: all bits equal
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1085 filtered out
```

After the refactor, MA and MB fail exactly the tests they failed before it (below).

## Mutations

Each one was applied, run and reverted by a driver script (scratchpad `htK-HY1-mut/drive.py`), which asserts that the file is byte-identical after the revert.

| id | mutation | command | before the fixes | after the fixes |
|---|---|---|---|---|
| MA | `ht_ok`: `htj = Σjets + Σb`, with `iht` unchanged (the brief's "b class counted in `htj`") | `cargo test -p vibegraph-lib --lib cuts::tests::` | FAILED 63/2: `b_quarks_count_for_iht_and_for_htj_only_as_jets`, `charm_leaves_htj_below_maxjetflavor_four` | same 63/2, same tests |
| MB | `ht_cut` classes legs as jets = `g` or `\|pdg\| ≤ 4`, b = `\|pdg\| = 5`, ignoring `maxjetflavor` | same | FAILED 62/3: `a_heavy_jet_class_leg_counts_for_ht`, `charm_leaves_htj_below_maxjetflavor_four`, `b_quarks_count_…` | same 62/3, same tests |
| MC | `detect_unimplemented`: `if false && SINGLE_PT_MAXIMA.contains(&name)` | same | FAILED 63/2: `pt_maximum_threshold_is_half_the_collider_energy`, `unbindable_pt_maxima_read_as_default`; **`unbindable_value_does_not_excuse_other_cuts` passed** | FAILED 62/3, now including `unbindable_value_does_not_excuse_other_cuts` |
| MH | `"ht2max"` removed from `UNIMPLEMENTED_CUTS` | same | – | FAILED 63/2: `ht_minima_and_maxima_are_not_refused`, `unbindable_value_does_not_excuse_other_cuts` |
| MD | collator: `if false && process.status == Some(ProcessStatus::Planned)` | `cargo test -p vibegraph-validation-report --bin validation-report planned_row` | on `f694e5a`'s test: **ok, 1 passed** (blind) | FAILED, 0 passed 1 failed |
| ME | collator: `if false && !process.bundled` | same | – | FAILED, 0 passed 1 failed |
| MF | `seed_record_banked` without its `dyn_run_present` check, which is the `7daaab9` behaviour for a non-planned row | `cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_jj_ht sigma_bb_iht --nocapture --test-threads 1` | – | FAILED 0/2: "`sigma_jj_ht_window_vs_mg` needs a banked seed record (pp_to_jj_ht)", and the same for bb |
| MG | manifest: `bundled = false` removed from `pp_to_jj_ht` | same | – | FAILED 1/1: bb "AWAITING BUNDLE", jj "needs a banked run directory (pp_to_jj_ht)" |

With no mutation, on the fixed code: `test result: ok. 2 passed`, and both rows print "AWAITING BUNDLE (the manifest marks this row bundled = false and this checkout does not have its run, …)".

H1's third state is a run directory that is present without its seed record. It was checked by hand: `mkdir -p validation/madgraph/output/pp_to_jj_ht/Cards && touch …/run_card.dat`, then the jj test, which gave "`sigma_jj_ht_window_vs_mg` needs a banked seed record (pp_to_jj_ht)", `test result: FAILED. 0 passed; 1 failed`. The directory was then removed (it is gitignored).

The baseline at `7daaab9` showed H2:

```
$ cargo test --workspace --no-fail-fast
test tests::each_planned_row_with_no_cell_waits_and_an_ordinary_one_is_missing ... FAILED
thread '…' panicked at validation-report/src/main.rs:599:13
error: 1 target failed: `-p vibegraph-validation-report --bin validation-report`
47 results: passed 1657, failed 1, ignored 19
```

Unreachable sites:
- **The present-record path of the HT gates.** The worktree has no reference bundle, so the σ path was not re-run. It is unchanged: `measure_recarded_sigma` is called exactly as before.
- **H6.** No one-line mutation separates `is_err` from a name match.

## Gate

Run at `34e00fa` with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

```
$ cargo fmt --all --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [optimized] target(s) in 45.12s     exit 0
$ cargo clippy --workspace --all-targets --all-features -- -D warnings
    Finished `dev` profile [optimized] target(s) in 38.40s     exit 0
$ cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings
    Finished `dev` profile [optimized] target(s) in 48.43s     exit 0
$ cargo test --workspace --no-fail-fast
exit 0; 47 results: passed 1658, failed 0, ignored 19
$ cargo doc -p vibegraph-lib --no-deps --document-private-items
exit 0; 8 warnings, all outside the PR's files (coupling/scales.rs:455, :528; phasespace/channel.rs:774;
ufo/parameters.rs:274; coupling/alphas.rs:58; lhef/mod.rs:4, :58; coupling/cluster/configs.rs:4)
$ cargo doc -p vibegraph-validation-report --no-deps --document-private-items
exit 0, no warnings
$ cargo test -p vibegraph-lib --features extended-validation --test validate_hadronic -- sigma_jj_ht sigma_bb_iht
test result: ok. 2 passed (both AWAITING BUNDLE; no reference bundle in this worktree)
$ python3 scripts/kb.py index
wrote 1 index files
$ python3 scripts/kb.py lint
915 concepts, 295 backlog items; 0 errors, 4 warnings (the four blocked-without-blocked_by items, untouched)
```

The suite count was 1657 passed and 1 failed at `7daaab9`. It is now 1658 passed: the failing collator test is replaced by one test, with no net change in count. The banked target run is `validate_hadronic`, because H1 changes its HT gates. No other banked target reads code this session changed.

## Found

1. **Default-off rule spelled per threshold across `cuts.rs` (H7).** The rule is "a maximum is read only when `≥ 0`, a minimum only when `> 0`". These sites write it out inline:
   - `ht_cut` `:574` and `ht_ok` `:1052`, `:1056`;
   - the single-leg `pt`/`e`/`eta` checks `:963`, `:971`, `:975` and `admits_every_point` `:930-934`;
   - `PairCut::is_active` `:313-317` and the pair checks `:987-1007`;
   - `mmnl_cut` `:550` and its check `:1027`;
   - the ŝ window `:942-951`;
   - `pair_mass` `:1456-1461`.

   To close it, add one `Threshold { min, max }` (or `fails(x, min, max)`) helper in `cuts.rs` that every family uses, with an identity oracle on the compiled `Cuts` values and the existing edge tests.
2. **Stale kb sites (H10), one entry each.**
   - `research/kb/phase-space/multi-tier-vegas-and-cut-edges.md:98`. The table row still reads "`htjmin` … refused today: `UNIMPLEMENTED_CUTS` [code `cuts.rs:352-400`, `htjmin` at `:379`]". The four HT names are now compiled, and only `ht2min`..`ht4max` remain refused. To close it, rewrite the row's status, keeping its aligned/non-aligned remark.
   - `research/kb/validation/cms-gridpack-survey-2026-10.md:203-205`. The bullet "`htjmin`/`ihtmin`/`htjmax` are refused only after full enumeration (up to 92 s)…" is still listed under "Other refusals", while `:246`, which this PR edited, says the item is closed. To close it, mark the bullet as closed by the ht-cuts sprint, the way `:246` does.
3. **No backlog item for the planned/unbundled spellings.** The brief says one is filed, but none exists. These test helpers each decide what to do when a row's input is absent:
   - `dyn_run_present`, `run_present`, `check_dy_run` and `not_yet_banked` (as V1 counted);
   - the `unbundled_rows()` readers in `validate_alphas.rs:126`, `validate_scales.rs:358`, `validate_sigma.rs:1319, 4490, 4531, 4668, 4850, 4946, 5493` and `vibegraph-cli/tests/validate_samples_proton.rs:1260`.

   H1 reduced the HT gates to `dyn_run_present`. To close the rest, add one helper in `tests/common/manifest.rs` that returns a row's declared state (bundled, unbundled or planned), and route every gate through it.
4. **`planned_rows()` has no caller on this branch after H1** (`tests/common/manifest.rs:110`). It is kept because it is a byte-identical port. Once #67 or #72 merges it has callers again; if neither does, delete it.

## Method

| technique | share | yield |
|---|---|---|
| Re-reading the gate plumbing against the manifest *as committed now*, not as the V1 report describes it (`grep 'status = "planned"'`, then reading `seed_record_banked` and `resolve_cell` in order) | 25% | H1 and H2, the only bugs |
| A baseline `cargo test --workspace` before any edit | 10% (wall time) | confirmed H2 as a red hermetic gate |
| Running the brief's named mutations (MA, MB) and asking which one-line mutation each changed assert is blind to (MC against `is_err`) | 30% | H5; MA and MB settled H8 |
| Reading the upstream `cuts.f` at the pinned commit for every citation in the diff | 10% | H3 |
| Grepping a new helper's body against the module's existing test helpers | 5% | H4 |
| Backlog and kb grep for stale sites | 10% | H10 |
| Visibility census of the diff | 5% | nothing: no new `pub`, and every new item is private |

Leads that did not hold up:
- Merging HT's default-off checks locally (H7) would have made HT the odd family out.
- The `along_*` helpers' "to the bit" claim was justified, and it holds for `lep` at the same angles (oracle above).
- H6 cannot be separated from a name match by any one-line mutation.
- The brief's leg-class question is answered by the existing shared `classify`.

## Brief corrections

- The brief says the branch is at `7daaab9 (pushed)`. During the session the remote moved to `a1caa84`, which adds `f694e5a`, a first fix of H2 that left the planned half blind (MD). It is rebased on as described above.
- "The planned-row spellings are a filed follow-up": no such item exists on this branch, `origin/main` or `main` (Found 3).
- The two HT rows are no longer `planned` (`ed5d8ea`), so the brief's questions about the planned-row port apply to code that this tree's rows no longer reach (H11). The real defect was the gate's `bundled = false` handling (H1).
- `research/refs/mg5amcnlo` is empty in this worktree, so `cuts.f` was fetched from raw.githubusercontent.com at `b7687064`.
