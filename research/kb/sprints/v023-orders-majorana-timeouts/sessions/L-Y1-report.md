---
type: Session Report
title: "L-Y1 report: group index and run-global position on ldjson progress records"
description: "Per-group progress stages carry group, groups, label and channels; every progress object carries a monotone phase and, once planned, phases, read off the phase instrument. Additive extras under schema_version 1, with σ and grid bytes unchanged against main."
status: draft
tags: [cli, ldjson, progress, output, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-23}
---
Branch `sprintl-progress` off main abc3b31 (brief commit 05412c4f). Closes
`ldjson-progress-has-no-run-global-position`; its file is deleted.

## What changed

**Group extras** (`vibegraph-lib/src/progress.rs`, `multiplicity.rs`). A
thread-local `Group` scope (`enter_group`) puts `group`, `groups`, `label` and
`channels` on every `progress::step` taken inside it; outside a scope the four
fields are absent. Two stages repeat per group, and both are scoped:

- `alpha_survey`: `MultiplicitySum::adapt_alphas` scopes each part, with its
  channel count (proton path); the fixed-energy and decay path scopes group 1 of
  1 around `use_multichannel`, with the diagram count.
- `compile`: `multiplicity_groups_of` scopes each part around
  `derive_flavor_groups` (proton path, no `channels`: the channels are built
  later); the fixed-energy compile in `integrate` and in the library's
  `generate` is group 1 of 1.

`vegas`, `weight_scan`, `unweight`, `ufo_load` and `enumerate` run over all
groups at once or before the split and carry no group. `weight_scan` was read at
`unweight.rs`: it counts channels pooled over every part, in a rayon region.

`label` is the group's tags, `@1` (`@1 @2` when lines share a final state), or
`N legs` when another group carries the same tags. A card with no `@N` still has
tags (the line position), so the leg-count form arises only for repeated tags.

**Phase position** (`vibegraph-cli/src/phase.rs`, `logging.rs`). `phase::start`
bumps a counter of phases entered; the ldjson progress writer stamps every
`progress` object's `fields` with `phase` (that count, absent before the first
phase) and `phases` (the planned count, absent until planned). `phase` is the
phase instrument's own sequence, and the counter never decreases.
`phases` comes from `integrate_phases(enumerates, proton, check)` and
`generate_phases()`; `integrate` plans after its run card resolves.

## Corrections to the brief

- "`phases` is known up front from the run's mode" holds for `generate` (always
  `input`, `pdf`, `generate`; planned before it reads anything) but not for
  `integrate`: whether the density phase runs depends on the beams, and a
  default run card is read off the card's own enumeration. Objects written
  before the run card resolves (`ufo_load`, an early `enumerate`) carry `phase`
  and no `phases`; once present it never changes.
- "or the leg count when the card has no tag": every line has a tag (`@N`, or its
  position), so the fallback applies when tags repeat across groups.
- `compile` also repeats per group and the brief names only `alpha_survey`.
- In `compile`, `groups` counts parts before a single-particle part that its
  cuts zero is dropped; `alpha_survey` counts the survivors. Documented.
- `validation/madgraph/pp_to_llj_mlm_run_card.dat` needs model `sm` for the
  σ-comparison card: `sm-no_b_mass` refuses with a massless b in the beam.
- No overall fraction is written; the docs derive `(phase, group, done)`.

## Oracles

Every command ran in `/home/user/wt/sprintl-progress`, dev profile with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | clean |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| clippy, features | same with `--features vibegraph/extended-validation,vibegraph-lib/extended-validation` | clean |
| hermetic suite, data hidden | `cargo test --workspace --no-fail-fast` with `research/refs/mg5amcnlo`, `validation/madgraph/output`, `validation/pdf/NNPDF23_lo_as_0130_qed` moved away | 2318 passed, 0 failed, exit 0 (summed over every `test result` line; log `ly1--test.log`) |
| new tests | `cargo test -p vibegraph --test cli_progress_position` | 5 passed |
| lib units | `cargo test -p vibegraph-lib --lib -- group_labels progress::` | 8 passed |

The new CLI tests are in `vibegraph-cli/tests/cli_progress_position.rs`. The
proton runs use a smooth stand-in PDF set written by the test under the name the
default run card resolves, so nothing needs fetched data. They check:

- two groups (`generate p p > e+ e-`, `add process p p > e+ e- j`, `sm-no_b_mass`):
  groups 1 of 2 and 2 of 2, labels `@1` and `@2` against the result's
  `process_ids`, `channels` against the result's per-part `channels`,
  `(phase, group, done)` non-decreasing, `phases` 10;
- one group: 1 of 1, `channels` equal to the `channels` phase size, `phases` 9;
- every progress object's `phase` equals the number of `phase` start records
  before it, and `phases` equals the phases the run goes on to enter (`--check`
  5, `generate` 3);
- ordering: with the extras removed, each stage's key set is the pre-existing
  one, and the stage and phase order is pinned.

**σ and grid, against main** (both binaries dev profile, `-j 1`, seed 5,
`--fixed-budget --neval 3000 --niter 3`, `--no-tui`; main built in a detached
worktree of abc3b31 with its own target, removed afterwards):

| card | σ main | σ branch | `sha256sum out/grid.bin.zst` (both) |
|---|---|---|---|
| `e+ e- > mu+ mu-`, 45.6 GeV beams | 2.0295 ± 0.0089 nb | 2.0295 ± 0.0089 nb | b7dc695591b2ac1569e1e1985bfc171daaa1a2870391283fd90343342cf921fe |
| MLM `pp_to_llj_mlm_run_card.dat`, `p p > e+ e-` + `add process p p > e+ e- j` | 937 ± 29 pb | 937 ± 29 pb | 3b2479a0243231886a9a9e160e30547c552b7cf0e21f3a9b160be909c60d70cf |

The branch under `--output-ldjson` writes the same two hashes (progress writes
do not touch the sampling) and σ 2029.545411444088 and 937.0178812779731 pb. Its
MLM survey records are `(group, groups, label, channels, phase, phases)` =
(1, 2, `@1`, 1, 7, 10) and (2, 2, `@2`, 6, 7, 10), over 12 records.

## What the tests cannot detect

- The per-record `phase` check compares against the phase records of the same
  stream; a stage emitted outside any phase is not covered (none exists).
- Nothing here exercises a card with repeated tags
  across groups in a CLI run (the label fallback is a unit test only), nor a
  dropped single-particle part (the compile `groups` count is by construction).
- The σ comparison is two cards at one seed; it shows the sampling order is
  untouched for them, not a sweep. The extras do no sampling, so none is needed.

## Found

- `groups` of `compile` and `alpha_survey` differ when a single-particle part is
  dropped (documented); closing it means deciding the drop before the compile
  loop. Minor.
- The proton `phase_timing` test is gated behind `extended-validation` for the
  PDF set; it could use this file's stand-in set and run in the default suite.
