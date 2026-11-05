---
type: Session Report
title: "G-HY2 report: hygiene review of G-V1's bounded τ draw and G-V4's Breit–Wigner τ mixture"
description: "Fifteen findings on the diagnoses PR's τ maps, cuts ceiling and NoUsableWeight refusal. Six mutations the existing tests passed (the shape rule's τ ceiling, the mixture's segment search, z_hi and ln_range on a bounded range, the no-pole arm, the union's narrow flag) now fail four new tests. The τ range, pole-in-range test and log draw are written once, identity shown on a to_bits dump of 47000 draws. The CLI NoUsableWeight test, broken by main's integrate-side refusal, is rebuilt on a real route: one scan point per channel on a narrow-band ptl card."
status: draft
tags: [hygiene, non-vacuity, tau-map, breit-wigner, visibility, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: brief, resource: "G-HY2.md", title: "G-HY2 brief"}
  - {id: g-v1, resource: "G-V1-report.md", title: "G-V1 report"}
  - {id: g-v4, resource: "G-V4-report.md", title: "G-V4 report"}
---
Worktree `/home/user/wt/G2-HY2`, branch `sprintg-diagnoses-hy2`. Base 9e49f31. It
was merged with `sprintg-diagnoses` four times: 8198287 (the compile fix), e6572c9
(main through #99), 61d0c39 and 2384b9f (main through #100). The scope is `git diff 75131e1 6561527`, f6fc78e and
`git diff 546c566 faa34b9`.

## Findings (written before any edit)

| id | pt | site (at 9e49f31) | evidence | conf | disposition |
|---|---|---|---|---|---|
| H1 | 1 | proton.rs:1546 `ln_inv_tau_min` | the field's doc reads "`ln(τ_max/τ_min)`" and its init is `(tau_max / tau_min).ln()`, but the name says `ln(1/τ_min)` | checked | **fixed** (`ln_tau_range`) |
| H2 | 1, 4 | proton.rs:1192, 2350, 2361 | `tau_min.powf(1.0 - v) * tau_max.powf(v)` is written three times: the Log arm, the BW `None` arm and the mixture's log segment | checked | **fixed** (`log_tau`) |
| H3 | 4 | proton.rs:1877–1881 against maps.rs:176–177; proton.rs:1152 against maps.rs `narrow_in_tau` | the τ range `[ŝmin/s, min(1, ŝmax/s)]` is computed twice, once without `.max(tau_min)`. The pole-in-range test `M²/s ∈ (τmin, τmax)` is also written twice | checked | **fixed** (`tau_range`, `pole_in_tau` in maps.rs) |
| H4 | 1 | maps.rs:48, :45; proton.rs:1878, 2331–2341; maps.rs:309–319; multiplicity.rs:263–270 | `TauMap::Log`'s doc says `τ = τ_min^(1−u)`, which predates the ceiling. "both maps" and "Either measure" are left over from when there were two maps, and one doc line runs past 120 columns. The BW measurement was spliced into `resolve`'s doc in front of the `p p > j j` ratios. `union_shape`'s doc says the rule only ever keeps the log map | checked | **fixed** |
| H5 | 3 | proton.rs:2181 `pub fn tau_max` | its only callers are proton.rs:3712 and 3785, both in `mod tests` | checked | **fixed** (deleted; the tests read the field) |
| H6 | 3 | cuts.rs:715 `pub fn shat_max` | its callers are proton.rs:1881 and maps.rs:177, both inside the crate | checked | **fixed** (`pub(crate)`) |
| H7 | 3 | unweight.rs:268–271, :583 | `NoUsableWeight`'s `pub` fields are read only by its own `Display`. `try_scan_with`'s only callers are generate.rs:513 and 1001 | checked | **fixed** (private fields, `pub(crate)`) |
| H8 | 2 | maps.rs:177 `ProcessShape::of`'s τ_max | no test puts a forced whole-state chain through `ProcessShape`. Yet this line is what keeps card 048's chain on `log` | suspected, then run: Ma survives | **fixed** (test) |
| H9 | 2 | proton.rs:1189–1211 `TauMixture::draw`, :1177 `z_hi`, :2359 `ln_range` | the only mixture test has one pole and τ_max = 1 | suspected, then run: Mb, Mc, Mh survive | **fixed** (direct three-pole bounded-range test) |
| H10 | 2 | proton.rs:2358–2362 BW `None` arm; maps.rs:61 "With no pole in range it is Log bit for bit" | no test builds a BW integrand without a pole in range | suspected, then run: Me survives | **fixed** (to_bits test; the arm merged with Log) |
| H11 | 2 | multiplicity.rs:244 the union's narrow OR | the only user, validate_hadronic.rs:5688, unions wide DY parts | suspected, then run: Mf survives | **fixed** (unit test) |
| H12 | 2 | generate.rs:1009 the proton path's `?` | the CLI test runs fixed beams through `generate_sample`, and no test reaches the proton site | checked by reading | **Found** (no hermetic proton fixture) |
| H13 | 1 | proton.rs:1196–1206 `std::ptr::eq` to find the last pole | the last pole is identified by pointer rather than by index | checked | **fixed** (index), identity oracle |
| H14 | 4 | cuts.rs:898 `dsqrt_shatmax` | `shat_max()` reads only the forced windows, so the τ draw ignores the run card's own ŝ ceiling | checked | **Found** (performance) |
| H15 | 1 | proton.rs ~6200, ~6286–6288, ~6391–6393 | three older tests invert the log map by hand with `ln(1/τ_min)`, which is valid only at τ_max = 1 | checked | **leave** (outside the change), Found |

Added after the manager's message:

| id | pt | site | evidence | conf | disposition |
|---|---|---|---|---|---|
| H16 | 2 | cli_hard_errors.rs `cli_generate_refuses_a_grid_whose_scan_finds_no_weight` | on the merged tree, `integrate` refuses the fixture first ("every one of the 5120 points … none passed the cuts", exit 1, no artifact) | checked | **fixed** (rebuilt on a reachable route) |
| H17 | 1 | unweight.rs `NoUsableWeight` doc and message | they name "an artifact whose integration saw almost no point inside its cuts", which `integrate` and `generate` now refuse before any scan. The remedy given, re-integrating, is not the one that works | checked | **fixed** |

**Backlog check.** I grepped `research/kb/backlog` for `/proton.rs`, `/maps.rs`,
`/cuts.rs`, `/unweight.rs`, `/diagram_channel.rs`, `/generate.rs`, `/multiplicity.rs`
and `cli_hard_errors`. None of the items found names a site this PR changed, with
two exceptions:

- `performance/tau-rule-reads-whole-process` still holds. The new
  `narrow_whole_state_resonance` is OR'd per process and over parts, like the flag
  the item names.
- `performance/sampler-tunings-predate-pooled-stop` cites `resolve`'s doc, which
  H4 reordered but did not change in substance.

## Fixed

| finding | commit | change |
|---|---|---|
| H1–H3, H13, H4, H5, H6, H8–H11 | f834d20 | `tau_range`, `pole_in_tau` and `log_tau` hold the range, the predicate and the draw once. `TauMixture` owns its range through `from_counts`, so `draw` takes only `u0`. `map_point_with` matches on `(map, mixture)`, so BW without a pole is the Log arm itself. Docs updated, `tau_max()` deleted, `shat_max` is `pub(crate)`, and four tests were added |
| H7, H16, H17 | b27052f | `NoUsableWeight`'s fields are private and `try_scan_with` is `pub(crate)`. The doc and message describe a scan that missed. The CLI test is rebuilt on a reachable route |
| H16 (ldjson) | 98de949 | `cli_error` maps `NoUsableWeight` by name to **`run`** and appends `(--scan-points)`. The test checks `error_kind` under `--output-ldjson` |

**Identity oracle for f834d20, chosen before the edit.** A temporary test
(never committed) wrote `to_bits` of `(x1, x2, √ŝ, jac)` for 3000 stream points under
each of Log, InverseSquare and BreitWigner on three processes:

- `p p > e+ e-` at `mmll > 50`, where the mixture is present;
- `p p > z, z > e+ e-`, where the window bounds the range and the mixture is bounded;
- `p p > e+ e-` at `mmll > 150`, where BW has no pole in range.

It also wrote 20000 midpoint draws of a two-pole mixture. Before the edit the mixture
was built as a struct literal with `of`'s formulas, and after it through `from_counts`.
The 47000 lines are byte-identical:
`sha256 85d60510ac91626a13c50190b44d3d6f7e0ed3c8344bdb7fb477a180bcb47702` on both.

**Reachability of `NoUsableWeight` from the CLI (the manager's question, case 1: it is
reachable).** After #88, `integrate` refuses a run with fewer than 20 effective points
and `generate` refuses σ = 0. The scan can still miss, because `--scan-points N`
fixes the draws per channel.

| fixture (`e+ e- > mu+ mu-`, 45.6 GeV beams, 1000 × 2) | integrate | generate `--scan-points 1`, seeds 1–12 |
|---|---|---|
| `ptl = 50` | exit 1: `CutsRejectAll`, no artifact | — |
| `ptl = 45.0` | σ = 253.9 ± 2.9 pb, effective 2655 | refused (exit 1, named) on seeds 9 and 10, otherwise exit 0 |
| `ptl = 45.5` | σ = 105.3 ± 2.0 pb, effective 1643 | refused on seeds 3, 9 and 10, otherwise exit 0 |

None of these runs panicked. The test integrates at `ptl = 45.5` and generates on
seeds 1 to 40 until the first refusal. Every earlier seed must exit 0, the refusal
must exit 1 with the named message and the flag, and a seed missing a refusal in 40
has odds of about `0.8⁴⁰ ≈ 1e-4`. It then reruns that seed under `--output-ldjson`.

**Error kind: `run`.** The artifact is sound: its σ is supported by about 1600
effective points. What failed is the scan this run drew, and a larger scan fixes it.
`artifact` would say the file is at fault.

## Mutations

The driver `mutate.py` (session scratchpad, `G-HY2/`) applies one mutation, runs the
named tests, logs the result and restores the file. `git diff --stat` was clean after
every batch. The test command was `cargo test -p vibegraph-lib --lib -- tau maps::
multiplicity:: shape|union|without_a_pole`. Logs: `G-HY2-mut-pre.log`,
`G-HY2-mut-post.log` and `G-HY2-mut-cli.log`.

| id | mutation | old tests (9e49f31 + merge) | new tests |
|---|---|---|---|
| Ma/Na | `ProcessShape::of`'s τ_max = 1 | **pass** (25 ok) | **fail**: `the_tau_rule_reads_the_range_a_decay_window_bounds` |
| Mb/Nb | drop the `break` in the pole-segment search | **pass** | **fail**: `a_three_pole_tau_mixture_…` |
| Mc/Nc | `z_hi` at `τ = 1` instead of `τ_max` | **pass** | **fail**: `a_three_pole_…` |
| Md | no dedupe of identical poles | fail (`the_breit_wigner_tau_mixture_…`) | — |
| Me | BW `None` arm drops `τ_max` (old); BW `None` drawn as inverse-square (new) | **pass** | **fail**: `the_breit_wigner_map_without_a_pole_in_range_is_the_log_map` |
| Nd | `pole_in_tau` drops `pole > τ_min` | — | fail: the new no-pole test and `narrowness_…` |
| Mf/Nf | the union drops the narrow OR | **pass** | **fail**: `the_union_of_shapes_keeps_a_part_s_narrow_resonance` |
| Mh/Nh | the mixture's `ln_range` at `ln(1/τ_min)` | **pass** | **fail**: `a_three_pole_…` |
| Mi | drop the last pole's `start = 1 − w` | fail (`the_breit_wigner_tau_mixture_…`'s `u₀ = 1` edge) | — |
| Mj/Nj | density without component weights | fail | fail (both mixture tests) |
| Mk | narrowness without the factor 2 | fail (`narrowness_…`) | — |
| Ml | the ceiling's fold over sets is min instead of max | fail (`a_whole_state_forced_line_bounds_shat_from_above`) | — |
| Nn | the segment start is not advanced | — | **pass** on the first, two-pole version (the last pole's reset masks it); **fail** after the third pole was added |
| P1 | the fixed-beam scan back on panicking `scan_with` | the old CLI fixture is refused by `integrate` before any scan | **fail**: seed 3 exits 101 |
| P2 | `NoUsableWeight` mapped to `artifact` | — | **fail** at the ldjson `error_kind` assert |

**The three-pole test's bounds come from the grid, not from a measurement.** The draw
runs on a midpoint grid of 400000 points, so the count of draws below `t` differs from
the measure by less than one point per segment, which bounds it by `4/n`. Each
component's integral is off by less than one grid step of its largest value, `1/c`,
per segment edge, which bounds it by `4·5/n = 5e-5`. On the fixed code the integrals
read 0.999999993, 1.000000000, 1.000000000 and 1.000000003. The mutations miss by far
more:

| mutation | reading | against |
|---|---|---|
| Nh | log component's integral 1.487 | 1 ± 5e-5 |
| Nj (two-pole version) | log component's integral 0.692 | 1 ± 5e-5 |
| Nn | `P(τ ≤ 1.5e-4)` 0.218 | 0.0179 ± 1e-5 |
| Nb | `P(τ ≤ 1.5e-4)` 0.418 | 0.0179 ± 1e-5 |

**Sites no mutation reaches.** The proton path's `?` in generate.rs:1009 is one (H12).
The `.clamp(0.0, 1.0)` on `v` in `draw` is another: removing it moves nothing that the
`τ` clamp after it does not already catch.

## Routed

None. No finding moves a computed number. The τ ceiling that ignores `dsqrt_shatmax`
would change sampling order, so it is filed as Found for performance-dev.

## Gate

At e25051b, which is `origin/sprintg-diagnoses` 2384b9f merged. Cargo ran with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 --jobs 3`,
detached (`G-HY2/gates.sh`), with logs in `G-HY2-gate-<step>.log`.

```
$ cargo fmt --all --check                                                   -> exit 0
$ cargo clippy --jobs 3 --workspace --all-targets -- -D warnings            -> exit 0
    Finished `dev` profile [optimized] target(s)
$ cargo clippy ... --features vibegraph/extended-validation -- -D warnings  -> exit 0
$ cargo clippy ... --all-features -- -D warnings                            -> exit 0
$ cargo test --jobs 3 --workspace --no-fail-fast -- --test-threads 3        -> exit 0
    55 result lines: 1957 passed, 0 failed, 25 ignored
    test cli_generate_refuses_a_grid_whose_scan_finds_no_weight ... ok
$ cargo doc -p vibegraph-lib --no-deps --document-private-items             -> exit 0
    warning: `vibegraph-lib` (lib doc) generated 7 warnings
$ cargo doc -p vibegraph --no-deps --document-private-items                 -> exit 0 (no warnings)
$ cargo test -p vibegraph --profile release-debug --features extended-validation \
    --test cli_generate_proton -- --test-threads 1                          -> exit 0
    test result: ok. 11 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 49.70s
$ cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_sigma -- --test-threads 3                               -> exit 0
    test result: ok. 8 passed; 0 failed; 33 ignored; 0 measured; 0 filtered out; finished in 778.15s
$ cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
    --test validate_hadronic -- --test-threads 3                            -> exit 0
    test result: ok. 20 passed; 0 failed; 26 ignored; 0 measured; 0 filtered out; finished in 402.34s
$ python3 scripts/kb.py lint                                                -> 0 errors, 5 warnings
    (the gate-time run read 1 error, the sessions index stale on this report's uncommitted
    draft; `kb.py index` regenerated it, adding this report's line only)
```

**Suite count.** 55 result lines: 1957 passed, 0 failed, 25 ignored. The manager's
gate of 2384b9f read 1952 passed and 1 failed (this test). That makes +1 for the
repaired test and +4 for the new ones:
`a_three_pole_tau_mixture_draws_its_density_over_a_bounded_range`,
`the_breit_wigner_map_without_a_pole_in_range_is_the_log_map`,
`the_tau_rule_reads_the_range_a_decay_window_bounds` and
`the_union_of_shapes_keeps_a_part_s_narrow_resonance`. The CLI test was rewritten in
place, under the same name.

**Doc warnings.** `vibegraph-lib`'s 7 doc warnings are all outside the files this
session touched (`coupling/`, `lhef/mod.rs`, `phasespace/channel.rs`), so none is new.

**Why the banked targets.** `cli_generate_proton` replays proton artifacts through
the τ maps. `validate_sigma` and `validate_hadronic` run every banked row, the brief's
condition for a change to the τ maps or the cuts. The refactor is bit-identical, so
they must read exactly as before.

**Earlier runs.** At 35dad4b (main through #99), the whole gate passed through
`validate_sigma` (1955 passed; `validate_sigma` 8 passed in 1053.57 s). It was
stopped (by PID) during `validate_hadronic` when #100 arrived. Before that, at
b27052f: fmt, the three clippy runs, the workspace (1933
passed, 0 failed), docs and `cli_generate_proton` (11 passed) were all green. Its
`validate_sigma` was stopped (by PID) when main's #99 arrived, and the gate was rerun
in full on the merge.

## Method

| technique | share | yield |
|---|---|---|
| A one-line mutation per new or changed test, reasoned and then run in a batch | ~45 % | H8–H11: 6 survivors of 12. It also showed that Mi, a suspected blind spot, was already caught by the `u₀ = 1` edge |
| Grepping a new `pub` item's readers across the workspace | ~10 % | H5–H7 |
| Grepping a distinctive body line workspace-wide | ~10 % | H2, H3 |
| Reading docs against bodies after the change | ~10 % | H1, H4, H17 |
| The reachability investigation (H16), CLI runs at three `ptl` values over 12 seeds | ~20 % | case 1, a real route |
| Backlog grep by path tail | ~5 % | none new |

Leads that did not hold up:

- Md (the dedupe) and Mi (the last-pole reset) were caught by G-V4's test.
- The first version of the new mixture test had only two poles and passed Nn. A
  segment search needs a pole that is neither first after the log map nor last. Only
  the batch showed this.
- `NARROW_TAU_SHARE` is named and justified at its declaration, which is where it is
  read, so there was no finding on the brief's point about the threshold.

## Found

1. **The τ draw ignores the run card's own ŝ ceiling.** `Cuts::shat_max` reads only
   the decay-chain windows (cuts.rs `forced_shat_ceiling`). `dsqrt_shatmax`
   (cuts.rs:898, `shat_max_sq`) is applied as a cut but does not bound τ, so a card
   with a ŝ ceiling draws points above it that the cut zeroes. To close
   (performance-dev): fold `shat_max_sq` into the ceiling, bump the artifact format,
   and measure on a card that sets it.
2. **The proton path's `NoUsableWeight` is reached by no test** (generate.rs, in
   `generate_proton_sample`). The two scan-and-report blocks of `generate_sample` and
   `generate_proton_sample` are copies. To close: a banked `cli_generate_proton` case
   with `--scan-points 1` on a narrow-band card, or one shared scan helper so a single
   site serves both.
3. **Three older proton tests invert the log τ map by hand with `ln(1/τ_min)`**
   (`the_tau_map_is_loose_below_the_true_threshold_by_a_measured_margin` and the two
   at ~6286 and ~6391). They hold only at τ_max = 1. To close: invert through
   `log_tau` and `tau_range`, or assert τ_max = 1 where they read the map.
4. **`ProcessShape::of` also settles the τ choice of fixed-beam runs**
   (hadronic.rs:2105), where τ is not drawn. A fixed-beam card with a cut-implied
   floor and a narrow whole-state pole would bank and print `breit-wigner (auto)` for a
   map it never uses, as it already does for `log` and `inverse-square`. To close:
   settle `tau` only for hadronic runs, or document it as inert there.
5. **The `tau-rule-reads-whole-process` item's sites have grown by one.**
   `narrow_whole_state_resonance` is OR'd per process and in `union_shape`, like
   `whole_state_resonance`. To close: add it to the item.

## Brief corrections

- **`cli_hard_errors` is hermetic, not banked.** It has no `required-features` entry
  in vibegraph-cli/Cargo.toml and runs in `cargo test --workspace`.
- **The base did not compile its lib tests** (maps.rs:605, `from_model_card` made
  fallible by #88). The manager fixed it in 8198287. At the base, the CLI
  `NoUsableWeight` test also failed, because `integrate` refuses its fixture.
- The brief's gate lists `--features vibegraph/extended-validation` clippy. The agent
  definition's `--all-features` was run as well.
