---
type: Session Report
title: "P3 report: the bin-width bound default flipped to K = 6 (interim)"
description: "GridRule::default() bounds every bin at six uniform widths, --grid-max-bin-width none restores the unbounded grid, and the artifact records the rule at format version 12 while an unbounded artifact stays byte-identical to the old writer's; the one-sitting timing was launched detached and the combined-tree banked run was not started when the session was ended."
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

**Status: interim.** The harness asked for the hand-back before scope items 4
and 5 finished. Items 1-3 are done, gated and committed (`23b8a42`). Item 4
(one-sitting timing) is running detached; item 5 (banked layer on
`p3-combined`) has its merge made but was not run. No variance-per-CPU claim
appears in any doc or comment.

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

## Not finished

- **Item 4, timing.** `vegasacc-P3/campaign.sh` was launched detached at
  2026-10-17T22:09:15Z (PIDs 3012/3016): release binary
  `vegasacc-P3/vibegraph-p3` (sha256 `be00b4ef…`, `cargo build --release`
  of `23b8a42`), arms `--grid-max-bin-width none` and `6` spelled
  explicitly, 20 seeds × `pp_to_llj`, `pp_to_llj_mlm`, `pp_to_jj` at the
  gates' budgets, seed-major with rows inner and arm order alternating by
  seed parity, results in `vegasacc-P3/c3/results.jsonl`, about 2.3 h. The
  summary is `python3 vegasacc-P3/summary.py` (var·CPU K6/unbounded per row,
  ratio of means with seed-spread error and paired per-seed ratio, spread²
  with F interval). No result was read before the hand-back.
- **Item 5, banked layer.** `p3-combined` exists locally (not pushed):
  `418f923`, a clean merge of `origin/map-default-guards` (`5e46f05`) into
  `23b8a42`, tree `25bc9c39921403a1968cd2fbd2175b3e0c794b29`. `pixi run
  --skip-deps validate` on it was **not run** (it must follow the timing,
  4 cores), so there is no moved-cell list against
  `mapguards-V1-report-after/`. Run it detached on `p3-combined` after the
  campaign ends, copy `target/validation-report`, and diff with
  `python3 -I mapguards-V1-diff.py mapguards-V1-report-after <copy>`.

## Corrections to the brief

- "generate … read the rule from the artifact": `generate` never refines a
  grid, so there is no rule for it to read; the artifact field is provenance.
- The backlog item `artifact-has-no-sampler-kind` says adaptation settings
  stay out of the wire format and that a VEGAS-only artifact stays at "the
  version it has today"; after this commit a default artifact is version 12,
  so that item's byte-identity criterion now refers to version 12 (or 9-11
  for unbounded runs).

## Found

- **Items 4 and 5 are outstanding** (above): the one-sitting var·CPU figure
  and the combined-tree banked run with its moved-cell diff. Close: collect
  `c3/results.jsonl` through `summary.py`, then run the banked layer on
  `p3-combined` and diff against `mapguards-V1-report-after`.
- **`importance-sampler-seam.md` predates the recorded rule.** Its table says
  the rule is `serde(skip)` on the grid (still true) and its serialise step
  puts `GridRule` in a per-kind config; the artifact now carries it at top
  level as `Option<GridRule>`. Close: one line in that concept when the
  sampler-kind work lands.
- **The VEGAS golden no longer covers the bound.** See "Seeded goldens".
  Close: a pinned-seed golden on an integrand where the default bound binds.

**Banking request: none.**
