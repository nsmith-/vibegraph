---
type: Session Report
title: "HY1 report: hygiene pass on the CM-clustering fix"
description: "Review of 6740954. Every frame mutation of the integrand (direct, mirrored, unmirrored, unswapped, and the ickkw = 0 half alone) fails the new unit test; the banked replay fails on a lab-frame replay (9 pp_to_jj events) and on a first-configuration-only search. No library boost is reachable from tests/, so partonic_cm stays. One docs commit fixes the frame statements; six stale kb sites and one fixed backlog item go to Found."
status: draft
tags: [hygiene, non-vacuity, mutation-testing, clustering, frames, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: docs-fix, resource: "https://github.com/nsmith-/vibegraph/commit/2dd52f4", title: "docs(scales): say which frame each scale path reads"}
---
This report is from the hygiene-dev agent. The change under review is
`6740954`, which touches `proton.rs`, `coupling/scales.rs`, `hadronic.rs`,
`tests/validate_scales.rs` and `tests/validate_hadronic.rs`. `helas/` was not
touched. Worktree `/home/user/wt/ttx-mlm-at1`, branch `sigma/ttx-mlm-at1`,
base `a8cedce`. The change's files are unchanged between `6740954` and the
base (`git diff --stat 6740954 HEAD -- vibegraph-lib/` names only
`standalone_jamps.rs` and `color_cf_oracle.rs`).

## Findings

The table was written before any edit.

| id | pt | site | evidence | conf | final |
|---|---|---|---|---|---|
| H1 | 1 | `proton.rs:2116` (`per_group_sum` doc) | a reflow left `([`term_at`](Self::term_at)). The` on a line of its own | checked | **fixed** (`2dd52f4`) |
| H2 | 1 | `hadronic.rs:402` (`EventScaleSource::scales`), `scales.rs:152` (`ScaleEvent::incoming`), `hadronic.rs:17` (module `# Frames`) | `scales` said "momenta in the frame [`ScaleEvent::incoming`] describes", and `incoming` names no frame a caller passes, only MadEvent's partonic CM. The integrand hands `scales` **lab** momenta for the closed forms (`event_scales_in` → `scales_of` → `point_scales` → `scales`, `proton.rs:2402–2484`) and CM momenta for the clustering (`term_at`). A reader of `scales` would infer CM for both. The module's Frames section named the matrix element and the cuts only | checked | **fixed** (`2dd52f4`) |
| H3 | 2 | `proton.rs:4897` `clustered_scales_are_read_in_the_partonic_centre_of_mass` | does it fail on each frame regression? Six mutations, below | checked (run) | **not vacuous**: 6 of 6 fail |
| H4 | 2 | `validate_scales.rs:933`, `:1114` (zero-miss asserts) | does the tightened assertion still see the frame, and still need the configuration search? Two mutations, below | checked (run) | **not vacuous**: 2 of 2 fail |
| H5 | 4 | `validate_scales.rs:1566` `partonic_cm` | the library's longitudinal boost is `hadronic::boost_z` (`hadronic.rs:590`), which is `pub(crate)`: an integration test cannot call it. No `pub` boost exists (`grep -rn "fn boost" vibegraph-lib/src`: `lorentz.rs` `pub(crate)`, `onshell.rs`, `resonance.rs`, `kt.rs` private). A second test copy of the same lab→CM boost is `validate_hadronic.rs:2903–2916` (`madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`, from `1539abc1`, before this change); both targets already include `tests/common` | checked | library reuse: **leave** (widening `boost_z` for a test is the wrong trade); the test copies: **Found 3** |
| H6 | 1 | `proton.rs:4896–5013` | the new test is about 117 lines | checked | **leave**: one hand-built point, its frame-flip precondition, and the two `ickkw` passes over it; each part is asserted on and splitting would separate the precondition from the assertion that needs it |
| H7 | – | kb and backlog | `mirror_lab_into` and `TIE_BREAK_MISSES` no longer exist (`grep -rn` over `vibegraph-lib`: no hits) but are named in six kb sites and one backlog item | checked | **Found 1, 2** |

Visibility: the change adds no `pub` item, no `cfg(test)` item and no field
(`git show 6740954 | grep -E '^\+.*\bpub\b|^\+.*cfg\('` returns nothing). The
`lab_mirror_buf` field and `mirror_lab_into` it removed have no remaining
reader. Comment grep (`\bT[0-9]\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer`)
over the change's added lines: no hits.

The brief's three questions, answered:
- **Does the unit test fail for each way the frame could regress?** Yes, for
  all five integrand mutations below, and each half of it (`ickkw = 1`,
  `ickkw = 0`) has power of its own.
- **Does the zero-miss assertion still exercise the configuration search?**
  Yes. On the fixed code, 8 runs need a channel other than the first (for
  example `gu_to_epemu` 7204, `pp_to_llj` 5540, `uux_to_ttx_4f` 9985 of
  10 000), and restricting the search to the first configuration fails both
  banked tests.
- **Does `partonic_cm` duplicate a library boost?** Yes in substance
  (`hadronic::boost_z` with `−β`), but not one a test can call. It stays (H5).

## Fixed

**`2dd52f4` docs(scales): say which frame each scale path reads** (H1, H2).
Docs only.
- `ScaleEvent::incoming`: "beams along ±z, in whichever longitudinal frame the
  caller chooses", ahead of the existing invariance list.
- `EventScaleSource::scales`: the scales are those of the frame the momenta
  are given in, and for the clustering and closed-form choice 1 that frame
  changes them, with a link to `ScaleEvent::incoming`.
- `hadronic.rs` `# Frames`: some per-event scales are not boost-invariant; the
  proton integrand clusters in the partonic CM, as MadEvent does, and
  evaluates the closed-form choices in the lab frame.
- `per_group_sum`: the short line rewrapped.

Evidence: `cargo doc -p vibegraph-lib --no-deps --document-private-items`
before (patch reverse-applied) and after both emit the same 9 warnings at the
same files (`diff` of the sorted `-->` sites is empty). These are the
standing nine of `rustdoc-warnings-ungated`, and none is in the edited lines.
The `ScaleEvent` link from the module doc resolves (it is imported in
`hadronic.rs:43`).

## Mutations

Driven by `scratchpad/ttxhy1-mutate.py`. It applies each mutation, runs one
test, logs the result and restores the file. `git diff --stat` was empty
after each batch. Log: `scratchpad/ttxhy1-mutations.log`, with one `.out` per
mutation.

Unit test: `cargo test -p vibegraph-lib --lib -- clustered_scales_are_read_in_the_partonic_centre_of_mass`.
On the fixed code: `1 passed`.

| mutation | edit | result | message |
|---|---|---|---|
| direct-lab | direct term `term_at(sc, &lab, …)` | **FAILED** | `group 4 ordering 0: beam 1 carries the gluon's scale, got [259.87…, 40.0]` |
| mirror-lab | mirrored term clustered at the rotated, beam-exchanged lab momenta (the `mirror_lab_into` the change removed) | **FAILED** | `group 4 ordering 1: beam 1 carries the gluon's scale, got [259.87…, 40.0]` |
| both-lab | both of the above (the pre-fix code) | **FAILED** | `group 4 ordering 0: …` |
| mirror-unmirrored | mirrored term clustered at `cm` rather than `&mirror` | **FAILED** | `group 4 ordering 1: …` |
| mirror-no-swap | the mirrored term's `mu_f` not swapped back to physical beam order | **FAILED** | `group 4 ordering 1: …` |
| both-lab, `ickkw = 0` pass only | both-lab, with the test's loop cut to `[0]` | **FAILED** | `ickkw = 0: the scales moved at y = -0.3` |

The ordering-1 messages show that the mirrored slot is among those `checked`
counts. The `>= 2` floor is not the only thing standing between the mirrored
term and a silent `continue`.

Banked replay: `cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test validate_scales -- banked_events_reproduce`.
On the fixed code: `2 passed`, with the line "1340000 comparisons over 530000
events in 53 runs within their printing budget".

| mutation | edit | `every_printed_scale` | `aqcdup_from_the_computed_scale` |
|---|---|---|---|
| replay-lab | `general` reads `event.incoming`/`outgoing` unboosted | **FAILED**: `pp_to_jj: 9 of 10000 events outside the printing budget` | **FAILED**: `pp_to_jj: 1 events miss AQCDUP` |
| replay-first-config | `for config in 1..=n_configs.min(1)` | **FAILED**: `ee_to_mumu_tata_qcd0: 262 of 10000 events outside the printing budget` | **FAILED**: `ee_to_mumu_tata_qcd0: 262 events miss AQCDUP` |

replay-lab reproduces the nine events that `partonic_cm`'s doc quotes, so that
count is a measurement. The old test passed this mutation by construction: its
`TIE_BREAK_MISSES` admitted exactly nine signature-matched `pp_to_jj` misses,
and its AQCDUP gate allowed `outside <= 9`. I did not re-run the old test.

**Blind spots (not claimed).**
- The banked replay's sensitivity to frame rests on 9 events of one run
  (`pp_to_jj`), and on 1 event for AQCDUP. A frame error that moves no tie and
  no massive-pair pseudorapidity across MadEvent's printed digits passes it.
  The unit test is the stronger frame oracle.
- The unit test cannot see a frame that differs from the CM by a rotation
  about z, or by a boost that flips no sign at its one point. It says so
  itself. It also does not exercise the massive-pair pseudorapidity measure
  (its final state is `l⁺ l⁻ g`).
- No mutation was run on `validate_hadronic.rs`'s probe edit
  (`probe_cluster_scale_spread_over_configurations`, `ev.lab` → `ev.cm`). It is
  `#[ignore]`d. Its own doc says it compares the rebuilt clustering against the
  integrand's recorded scales on every point, so a frame mismatch there fails
  the probe. That claim was not exercised here.

## Routed

None. No finding moves a computed number.

## Gate

On `2dd52f4`'s tree. `CARGO_INCREMENTAL=0`, `-j 2`, `--test-threads 2`.

- `cargo fmt --all --check`: `fmt rc=0`.
- `cargo clippy --workspace --all-targets -- -D warnings`:
  `Finished \`dev\` profile [optimized + debuginfo] target(s) in 1m 37s`, rc 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  `Finished \`dev\` profile [optimized + debuginfo] target(s) in 30.94s`, rc 0.
- `cargo test --workspace`: rc 0. Summed over every `test result` line:
  1402 passed, 0 failed, 16 ignored. No test was added or deleted.
  `target/debug` was deleted afterwards; it had reached 13 GB, with 2.9 GB
  free on the host at its peak.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items`: rc 0, 9
  warnings before and after, at the same sites. vibegraph-lib is the only
  package touched.
- Banked targets: none re-run after the commit, because no test and no scale
  path changed (the brief's condition). On the unchanged code, `validate_scales`
  `banked_events_reproduce*` passed 2 of 2 (log above). That was the baseline
  for the mutations. `validate_hadronic` was not run.

## Method

- Reading the change's diff and the frame docs against the call graph
  (`shape` → `per_group_sum`/`shared_scale_sum` → `term_at`/`event_scales_in`):
  about 30 %. This found H2, the one doc defect.
- Mutation batches, unit and banked: about 45 %, mostly rebuild wall time
  (160–190 s for each unit mutation, 70–110 s for each banked one). Each brief
  question was settled by a run, not by reasoning.
- Grepping for a second copy of the boost and for stale names: about 15 %. It
  found H5 and H7.
- Backlog check (path tails `/proton.rs`, `/scales.rs`, `/hadronic.rs`,
  `validate_scales.rs`, `validate_hadronic.rs`; 20 items): about 10 %. One item
  (`pp-to-jj-tie-break-no-cluster-dump`) is fixed by the change. None of the
  others has a site this change makes wrong.
- **Leads that did not hold up.**
  - The suspicion that `checked >= 2` might be met by direct terms alone, which
    would leave the mirrored slot unasserted. The mirror mutations fail on
    ordering 1.
  - The suspicion that the `ickkw = 0` pass is decoration. It fails on its own.

## Found

1. **Stale kb sites for removed code.** Each needs its sentence rewritten to
   the CM clustering (`term_at` at `cm`, and at `mirror` for the exchanged
   term).
   - `research/kb/hadronic/beam-mirror-identity.md:141`: `mirror_lab_into`.
   - `research/kb/hadronic/proton-integrand.md:142`: `mirror_lab_into`.
   - `research/kb/scales-pdf/clustering-configuration-draw.md:102`:
     `mirror_lab_into`, and the lab-frame clustering of the mirrored term.
   - `research/kb/validation/scale-replay-gate.md:136`: `TIE_BREAK_MISSES = [("pp_to_jj", 9)]`.
     The replay now asserts zero misses and clusters in the CM.
   - `research/kb/validation/sigma-row-gating-exceptions.md:132`:
     `TIE_BREAK_MISSES`.
   - `research/kb/validation/kt-cluster-dump-oracle.md:163`: "`pp_to_jj`'s nine
     beam-crossing …" describes them as an open tie-break population. They
     were the frame bug.
2. **Backlog item fixed by the code:**
   `research/kb/backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md`. Its
   only site (`validate_scales.rs:315`, `TIE_BREAK_MISSES`) is gone. The nine
   events reproduce in the CM, and replay-lab above brings exactly those nine
   back. The item can be closed and needs no dump.
3. **Two test copies of the lab→CM boost.** They are `partonic_cm`
   (`validate_scales.rs:1566`, β from the summed beam `p_z`/`E`) and the inline
   boost in `validate_hadronic.rs:2903–2916` (β from the beam energies, which
   is equal for massless beams on z). Both targets include `tests/common`. The
   fix is one `common` helper used by both, with the general β. The item
   `kinematic-helpers-hand-rolled` lists only `src/` copies, so these could be
   added there.
4. **(Carried from F1, confirmed by reading.) Closed-form choice 1 is
   evaluated on lab momenta** (`event_scales_in`, `proton.rs:2402`), while
   MadEvent evaluates it on the partonic CM. Its `E_T = E·p_T/|p|` is not
   boost-invariant, so a `dynamical_scale_choice = 1` proton row differs from
   MadEvent pointwise. This is physics and goes to validation-dev. The docs now
   state the lab frame truthfully, so a fix there must update the `hadronic.rs`
   Frames section and the `ProtonIntegrand` doc with it.

## Brief corrections

- The brief's gate asks for `--test validate_hadronic` (the non-ignored tests)
  when a test or scale path changed. The only `validate_hadronic.rs` edit in
  `6740954` is inside an `#[ignore]`d probe, so the non-ignored tests do not
  reach it. Running them would not have tested the change.
- The brief says the host has about 28 GB free; the assignment says about
  17 GB. At this session's start it was 17 GB, and during `cargo test
  --workspace` it fell to 2.9 GB (13 GB of it `target/debug`).
