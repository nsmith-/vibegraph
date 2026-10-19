---
type: Session Report
title: "HY1 report: hygiene review of the opt-in grid rules"
description: "Twelve findings on P1's grid-rule change; four test-power gaps closed with run mutations (cap constant, over-tight bound, Hill count/threshold, untested K refusal), one pub narrowed, one log duplication merged; the default rule is bit-identical before and after on a library digest and two CLI artifacts."
status: draft
tags: [vegas, grid-adaptation, hygiene, non-vacuity, mutation-testing, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: 1b2d6fc, resource: "https://github.com/nsmith-/vibegraph/commit/1b2d6fc", title: "vegas: grid-rule tests catch a moved cap, an over-tight bound and an unchecked K"}
  - {id: a7727d4, resource: "https://github.com/nsmith-/vibegraph/commit/a7727d4", title: "unweight: pin Hill's estimator to its closed form on Pareto quantiles"}
---
Scope: P1's code commits `1ed69b7` and `97a5927` (`git diff 09cc6d4..97a5927`:
`vegas.rs`, `unweight.rs`, `budget.rs`, `hadronic.rs`, `multiplicity.rs`,
`vibegraph-cli/src/integrate.rs`). Worktree `/home/user/wt/vegas-low-acceptance-bins`,
branch `vegas-low-acceptance-bins`, verified with `git rev-parse --show-toplevel`
and `git branch --show-current` before any action. Scratch logs are under
`/tmp/claude-0/-home-user-vibegraph/c98eb2f4-0e99-502e-b316-ad96baf4894f/scratchpad/hy1/`
(prefixed `vegasacc-HY1-`).

## Identity oracle for the default rule (chosen before the first edit)

- **O1, library digest.** A temporary test (`vegasacc-HY1-oracle.patch`, applied
  and reverted by `vegasacc-HY1-oracle.sh`, never committed) writes the `to_bits`
  of `integral`, `std_dev`, `chi2_per_dof` and every grid edge after 6 iterations
  of `VegasGrid::adapt` (both iteration combinations), a frozen pass on each
  trained grid, and `adapt_parallel_seeded` under 3 threads at chunk 7 and 1000,
  on three integrands (smooth polynomial, a 3-D peak, the 4 %-acceptance strip):
  46 lines covering both `BinHistogram` callers and `refine_grid`.
- **O2, CLI artifacts.** `vibegraph integrate` (release-debug) at
  `--fixed-budget --neval 20000 --niter 4 --seed 20261017 -j 4` on
  `e+ e- > mu+ mu- a` (`ee_to_mumua` run card, `FixedBeamIntegrand`) and
  `p p > l+ l- j` (`pp_to_llj` run card, `MultiplicitySum`), sha256 of
  `grid.bin.zst`.

| | before (HEAD `f788ccd`, unedited) | after (edited tree, = `a7727d4` modulo `cargo fmt`) |
|---|---|---|
| O1 digest | `d8351a44…bda6887` | `d8351a44…bda6887`, `cmp` identical |
| O2 `ee` artifact | `ae2dba5d…b265301` | `ae2dba5d…b265301` |
| O2 `pp` artifact | `73054082ec65…7def0` | `73054082ec65…7def0` |

(`vegasacc-HY1-sums-before.txt` / `-after.txt`; `diff` of the hash columns
prints nothing.) The edits after this run were `cargo fmt` whitespace in one
`unweight.rs` statement. No edit touched the hot path: the default arithmetic in
`BinHistogram::add`/`finish` and `refine_grid` is unchanged in the diff.

On the brief's hot-path question, read from the code: under the default rule
`BinHistogram::add` takes `v = fval2` and `accepted = None`, so it adds `fval2`
to `hist[dim][k]` in dimension order, point by point in draw order, exactly the
loop it replaced in both `run_iter_batched` and `adapt_blocks_iteration`;
`finish` returns `hist` untouched. O1/O2 confirm the same bits.

## Findings

Written before any edit. Confidence after settling (every *suspected* was
settled by a run).

| id | pt | site | evidence | mutation | conf. | disposition |
|---|---|---|---|---|---|---|
| H1 | 2 | `vegas.rs` `test_acceptance_rescale_is_capped` | asserts `got == 4.0 * MAX_ACCEPTANCE_RESCALE`, the constant it guards | M1: `MAX_ACCEPTANCE_RESCALE = 1.0e3` — **survived** all 33 `vegas::tests::test_` (the hand replay's own literal `1.0e4` never binds: ratio ≤ 3000/8-ish) | checked | **fixed** `1b2d6fc` |
| H2 | 2 | `vegas.rs` `test_bin_width_bound_holds_through_refinement`, `test_rules_on_a_low_acceptance_region` | both assert only `widest ≤ K/nbins`, which the uniform grid satisfies | M2b: `bound_bin_width(.., (max_width + 1.0) / 2.0)` (a tighter bound) — **survived**; M2a (`max_width / nbins`) was caught, because it drives `ρ` negative and switches the bound *off* | checked | **fixed** `1b2d6fc`: widest bin must also reach `K/nbins` (to 1e-9); the toy's bounded width must equal 4 |
| H3 | 2 | `unweight.rs` `hill_tail_index_recovers_a_pareto_index` | tolerance `a/√20` = 22 % on *deterministic* quantiles; the estimate there is exactly `a·k/Σⱼ ln((k+½)/(j−½))` = 0.99259 a at k = 20 (`python3`), so the test checks one constant to 22 % | M3: numerator `k → k−1` (×0.943) — **survived**; M8: threshold `sorted[n−k−1] → sorted[n−k]` — **survived** | checked | **fixed** `a7727d4`: closed form to 1e-9 |
| H4 | 2 | `unweight.rs:283` `if !(threshold > 0.0)` | no test reaches a zero threshold | M4: guard disabled — **survived** (returns `Some(0.0)`) | checked | **fixed** `a7727d4` |
| H5 | 2 | `integrate.rs:281-286` `--grid-max-bin-width` refusal | `grep -rn "grid-max-bin-width" vibegraph-cli/tests` → nothing; the library's `with_rule` assert is the only other guard (a panic) | M9: `k > 1.0 → k >= 1.0` — **survived** (0 tests matched) | checked | **fixed** `1b2d6fc`: `cli_hard_errors::cli_a_bin_width_bound_not_above_one_is_refused`, `=value` spelling, message and value matched, no `panicked` |
| H6 | 3 | `vegas.rs:155` `pub const MAX_ACCEPTANCE_RESCALE` | `grep -rn MAX_ACCEPTANCE_RESCALE` → only `vegas.rs` | — | checked | **fixed** `1b2d6fc`: private; `GridRule`'s pub doc states `10⁴` |
| H7 | 1 | `unweight.rs:528-542` | two `tracing::debug!` arms identical but for a suffix | — (log text identical by construction) | checked | **fixed** `a7727d4` |
| H8 | 2 | `vegas.rs` `bound_bin_width` break `>=` | algebra: admitting a bin at density exactly `ρ` leaves `ρ' = (rest − ρw)/(K − W − w) = ρ` | M5: `>= → >` — survives before and after; **equivalent mutation**, no test can reach it | checked | **leave** |
| H9 | 2 | `test_rules_on_a_low_acceptance_region` thresholds (default > 5, \|f·w\| > default, rescale < 2.5) from one seed | settled with 20 seeds (temporary test, reverted): default 6.80–7.65, \|f·w\| 8.88–9.17, rescale 1.775–1.788, pair 2.158–2.166, bound 4.000 on every seed; pulls within ±3.0 | — | checked | **leave** (the claims are the rules', not the seed's) |
| H10 | 1 | `integrate.rs:583, 933, 1048` `args.grid.rule()?` evaluated three times; the last two `?` cannot fire | quoted lines | — | checked | **leave**: matches `args.maps.options()`'s per-path pattern; threading the rule through `integrate_proton → integrate_hadronic` adds a parameter to three functions for no behaviour |
| H11 | 4 | `K > 1 && finite` written twice (CLI `rule()` refusal, lib `with_rule` assert) | quoted lines | — | checked | **leave**: one is the user refusal, the other a `pub(crate)` invariant; H5 now pins the user path. See Found F4 for the `pub` setter |
| H12 | 1 | artifact does not record the rule; can `generate` assume one? | `grep -nE "adapt\|refine\|integrate_channels\|grid_rule"` over `generate.rs`, `unweight.rs`, `artifact.rs` → only the trait default `unweight.rs:242` and a test's `grid.adapt` (`artifact.rs:1108`); the only reader of `grid_rule()` is `budget.rs:673` (`integrate_channels`) | — | checked | **Found** F1. `generate` never refines a loaded grid, so it reads no rule; nothing silently assumes one today |

Also checked, no finding:
- Comments: `git diff 09cc6d4..97a5927 -- '*.rs' | grep '^+' | grep -nE '\bT[0-9]+\b|\bC[0-9]\b|Phase [0-9]|note [0-9]+|used to|no longer|F-A|P1|backlog|sprint|session|today'` → nothing.
- Docs against bodies: `GridRule::acceptance_rescale` ("the constant cancels", "the mean over its accepted points": `h·N_acc/n_k = N_acc·mean(v)`), `max_bin_width` (`ρ = rest/(K − W_raised)` with `W` in absolute units and `K` dimensionless is the fixed point of `ρ = Σm'/K`), `set_grid_rule`'s list (`adapt_grids`, `adapt_grids_with`, `adapt_grids_budget` all call `integrate_channels`, `hadronic.rs:3018-3074`).
- Citations: `dsample.f:1890` (`grid(1,i,j) + abs(wgt)`), `:2113-2117` (`min(non_zero/inon_zero, 10000.)`), `:753` (`data use_cut/2/`) read in `research/refs/mg5amcnlo/Template/LO/Source/dsample.f`. Note there: MadEvent rescales its `Σwgt²` grid (`grid2`) by the factor **squared**; this crate's `Squared` + rescale applies power 1, as P1's deviations record (and as `use_cut = 3` does for `grid(1)`). A design choice, not a defect; it is an untested arm (Found F3).
- Visibility: `BinWeight`, `GridRule`, `GridRule::is_default`, both `set_grid_rule` are used by the CLI; `with_rule`, `HILL_TAIL_POINTS`, `hill_tail_index`, `ChannelScan::tail_index` are `pub(crate)`.
- Duplication of the two `set_grid_rule` setters: two three-line setters on two types, read through one trait method; not worth a trait setter.
- Block-split/sequential identity test: it compares two callers of the same `BinHistogram`, so it cannot see a defect inside `BinHistogram` (the hand-replay test does; M6, rescale squared, is caught by both the replay and the cap test).

## Fixed

- **`1b2d6fc`** (H1, H2, H5, H6): `vegas.rs`, `vibegraph-cli/tests/cli_hard_errors.rs`.
- **`a7727d4`** (H3, H4, H7): `unweight.rs`.

## Mutations

Driver: `vegasacc-HY1-mutate.py <label> <ids>` (apply, `cargo test`, restore, log);
logs `vegasacc-HY1-mutations-pre.log` (old tests) and `-post.log` (new tests).
Commands: `cargo test -p vibegraph-lib --lib vegas::tests::test_`,
`cargo test -p vibegraph-lib --lib unweight::tests::hill`,
`cargo test -p vibegraph --test cli_hard_errors bin_width`.

| id | mutation | old tests | new tests |
|---|---|---|---|
| M1/M1p | `MAX_ACCEPTANCE_RESCALE = 1.0e3` | survived (33 passed) | caught: `test_acceptance_rescale_is_capped` |
| M2a | bound at `max_width / nbins` | caught (bound switched off) | caught |
| M2b | bound at `(max_width + 1) / 2` | survived (33 passed) | caught: `test_bin_width_bound_holds_through_refinement`, `test_rules_on_a_low_acceptance_region` |
| M3 | Hill `/ (k − 1)` | survived | caught |
| M4 | Hill zero-threshold guard off | survived | caught |
| M5 | `>=` → `>` in the floor's admission | survived | survived — equivalent (H8) |
| M6 | rescale factor squared in `finish` | caught | caught |
| M8 | Hill threshold `sorted[n − k]` | survived | caught |
| M9 | CLI `k > 1.0` → `k >= 1.0` | survived (0 tests ran) | caught: `cli_a_bin_width_bound_not_above_one_is_refused` |
| M10 | `ρ` not updated (`rest / max_width`) | caught | caught |

Unreachable: M5 (equivalent). The hand-replay test restates the implementation
(`fval != 0.0`, `min(.., 1e4)`), so a mutation `hit = fval > 0.0` is invisible on
the non-negative toy; production weights are `|M|²`-positive, so it is also
invisible in production.

## Gate

All with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
(`vegasacc-HY1-gate.log`).

- `cargo fmt --all --check`: first run flagged the new `map_or` chain's layout;
  `cargo fmt --all` then `--check` → exit 0 (whitespace only, before the commit).
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`, no warnings.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → `Finished`, no warnings.
- `cargo test --workspace --no-fail-fast` → exit 0, **1486 passed, 0 failed, 16
  ignored** (P1 reported 1485/0/16 at step 2; +1 is
  `cli_a_bin_width_bound_not_above_one_is_refused`; no test deleted).
  `cli_reference_docs`: 1 passed.
- `cargo test -p vibegraph-lib --lib vegas::tests::test_ unweight::tests::hill` → 34 passed.
- `cargo doc -p vibegraph-lib --no-deps --document-private-items` → 8 warnings, all
  in untouched files (`coupling/cluster/configs.rs`, `phasespace/channel.rs`,
  `coupling/alphas.rs`, `coupling/scales.rs`, `ufo/parameters.rs`, `lhef/mod.rs`);
  `cargo doc -p vibegraph --no-deps --document-private-items` → 1 warning
  (`cache::resolve::locate`), untouched. No new warnings.
- Identity oracle O1/O2: identical (table above).
- Banked targets: none touched (test files and a `pub` → private constant; the
  default rule is bit-identical), so none run, per the brief.

## Method

- Mutation runs (driver script), about 40 %: produced all five test findings.
  Reasoning predicted M2a would survive; it was caught because a sub-1 bound
  drives `ρ` negative and disables the floor, which is why M2b (a valid but
  tighter bound) was the useful mutation.
- Closed-form oracle for a deterministic test (Hill), 10 %: turned a 22 % spread
  into an exact identity.
- Identity oracle (digest + artifacts), 20 % (mostly build time).
- Workspace greps for `pub` users, complaint strings in tests, and refinement
  callers on the `generate` path, 15 %.
- Seed sweep of the toy's fixed-seed thresholds, 5 %: the lead (seed artifact)
  did not hold.
- Citation read in `dsample.f`, backlog path-tail grep, 10 %.

## Found

- **F1. The grid rule is not recorded in the artifact.** `VegasGrid::rule` is
  `serde(skip)`; a deserialized grid carries `GridRule::default()`. Harmless
  today: the only reader of `grid_rule()` is `integrate_channels`
  (`budget.rs:673`) and `generate` never refines a loaded grid (grep above).
  Becomes wrong the moment a rule is a default, a banked run uses one, or any
  path resumes adaptation from an artifact. Close: an artifact metadata field
  for the rule, with a format-version decision. (P1 filed the same; one item.)
- **F2. The bin-width bound's `K` is untuned and the rescale + bound arm is
  unmeasured** — P1's Found, unchanged; noted so the manager files one item.
- **F3. `Squared` + acceptance rescale uses power 1 where MadEvent's `Σwgt²`
  grid gets the factor squared** (`dsample.f:2118-2120`, `grid2 * min(..)**2`).
  Untested arm; a performance measurement (`performance-dev`) would decide
  whether the squared histogram should take factor² (crediting the mean
  *weight* squared) rather than factor¹ (the mean of squares).
- **F4. `set_grid_rule` (pub) accepts a `GridRule` whose `max_bin_width` is ≤ 1
  or not finite** and the library panics later inside `integrate_channels`
  (`with_rule`'s assert). The CLI refuses first (H5), so no user reaches it.
  Close: validate in `set_grid_rule` (or a `GridRule` constructor returning a
  `Result` both the CLI and the library use), removing the duplicated predicate
  (H11).
- **F5. Stale backlog site:** `research/kb/backlog/performance/sampler-tunings-predate-pooled-stop.md`
  cites `vibegraph-cli/src/integrate.rs` `--max-iters` at `~:320-328`; P1's
  `GridRuleArgs` (+52 lines at :247) moved it to `~:345-381`
  (`grep -n max_iters vibegraph-cli/src/integrate.rs` → 381; at `09cc6d4` → 329).
  Other items naming the reviewed paths (`alpha-survey-iteration-limited-on-wide-splits`
  `:100`, `adapt-survey-bounds-untested-at-hundreds-of-channels` `:97-98`,
  `vegas-chi2-per-dof-overflows-on-wide-splits` `budget.rs ~454`,
  `stop-scale-inflated-by-few-accepted-iterations` `budget.rs ~:472-510`,
  `vegas-refinement-noises-constant-integrand`) still point at the right lines
  or name no line.

## Brief corrections

- The brief says review `git diff origin/main...HEAD`: that works; the local
  `main` is stale (`dee4def`), and `git diff main...HEAD` lists 430 files of
  other merged PRs. The P1 code diff is `09cc6d4..97a5927`.
- The brief's "K vs K/nbins" mutation is caught by the old tests (it disables
  the bound rather than tightening it); the gap was a valid tighter bound (M2b).
- "`<` vs `≤` in the density floor" is an equivalent mutation (H8); no test can
  catch it, and none is needed.
- The CLI package is `vibegraph` (directory `vibegraph-cli`); `cargo build -p
  vibegraph-cli` fails.
