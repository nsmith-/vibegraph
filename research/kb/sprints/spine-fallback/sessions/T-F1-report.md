---
type: Session Report
title: "T-F1 report: the spine is built without a pT cut, on a pole floor from a 10⁴ GeV² default scale"
description: "The pole floor and the support bound are now separate. The bound comes only from Cuts::spacelike_floor. The floor is always on, from the fiducial scale or from DEFAULT_POLE_SCALE = 10⁴ GeV², a value chosen by measured variance. The spine is built for every final state. Walk/density agreement holds on every peripheral diagram with no scale. σ matches flat RAMBO on a converged leptonic process. Seven banked rows move, all within their gates."
status: draft
tags: [phase-space, spine, t-channel, vbf, sprint-e, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
---
The session brief is [T-F1](T-F1.md). Code commit: `c5d44b86` on
`sprinte-spine-fallback`. This report is committed after it.

## What was implemented

- **Bound and floor are separate** (`diagram_channel.rs`, `spine_chain`).
  - Each rung's pole is `m².max(POLE_FRACTION_OF_FIDUCIAL_SCALE · scale)`.
    `scale` is the fiducial scale when it is positive and `DEFAULT_POLE_SCALE`
    otherwise.
  - `t_max_cap` is set only from a positive fiducial scale, exactly as before.
- **`DEFAULT_POLE_SCALE = 1e4` GeV²**, which gives a pole floor of 10 GeV². Its
  doc comment records the variance table below.
- **The spine is built for every final state.** The
  `(n_out == 2 || regulated)` condition is gone.
- **Docs now describe the current code.** Updated: the module docs (regulating
  the pole), `from_diagram`, `from_diagram_regulated`, `from_diagram_with`,
  `from_diagram_capped` (`max_rungs = 0` is now the all-timelike switch),
  `Cuts::spacelike_floor` and its zero-floor test, `use_multichannel`
  (`hadronic.rs`) and `ProtonIntegrand::new` (`proton.rs`).
- **`without_transfer_bound` is now `pub`.**
  `from_diagram_regulated(.., X).without_transfer_bound()` builds the channel at
  any pole scale with the support whole. This is what the variance scan uses.
- **Existing tests adapted.**
  - `SpacelikeCut::derived` in `tests/diagram_channel.rs` is now explicitly the
    all-timelike tree (`from_diagram_capped(.., 0.0, 0)`). Before, it was
    `from_diagram`, which now builds a spine.
  - A unit test in `proton.rs` asserted that no spine is built without a floor.
    It now asserts that the spine is built with and without the scale alike.
- **New hermetic tests**, in `tests/diagram_channel.rs`:
  - `without_a_fiducial_scale_every_peripheral_diagram_gets_its_spine_on_a_pole_floor`
  - `a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step`
- **New ignored probes:**
  - `probe_the_walk_density_tail_against_the_pole_scale`
  - `probe_the_no_scale_spine_on_a_mass_cut_peripheral_process`
  - `probe_the_no_scale_production_sigma_against_flat_rambo`

There are no deviations from the brief's design. One brief correction affects
how the density check is run; see "Corrections to the brief".

## Bias at the density level

`a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step` runs
over every peripheral diagram of five processes at `√ŝ = 500`:

- `u u~ > e+ e- g`
- `g u > e+ e- u`
- `u d > e+ e- u d QCD=0`
- `u u~ > d d~ g`
- `e+ e- > e+ e- mu+ mu-`

That is 75 spines, 39 of them multi-rung. They have 85 massless rungs, and 25 of
those emit a composite subsystem. Each spine gets 2000 uniform draws. The test
requires no draw past `WALK_DENSITY_TOL = 1e-7` (unchanged) and no degenerate
draw. Its control arm floors the pole at 1e-20 GeV² on the same draws.

```
no fiducial scale: 75 spines (39 multi-rung), 85 massless rungs (25 emitting a composite subsystem), pole floor 1e1 GeV²
no fiducial scale, 75 peripheral channels × 2000 draws: worst walk-vs-density gap 2.047e-8 (u d > e+ e- u d QCD=0 diagram 23); with the pole at 1e-20 GeV² the same draws break the identity 51502 times, 16603 of them on 13 of the 25 diagrams emitting a composite subsystem off a massless line
```

**How many points.** The edge flip shows on 51,502 of 150,000 draws, about a
third. It shows on 13 of the 25 composite-emission diagrams; the other 12 never
put the edge within rounding of zero in 2000 draws. A rounding-scale flip in
production would therefore show thousands of times over, not once.

**Why 2000 draws and not more.** The longer sweep below (`10⁵` draws per
channel, 7.5M per arm) finds a tail of about 24 draws past `1e-7`, worst
`2.4e-6`. That tail is the same at every pole scale from 1 to 10⁶ GeV², and the
same on the channels a 400 GeV² fiducial scale already builds. So it comes from
near-threshold conditioning (worst on diagram 8, a massive-W rung emitting one
leg), not from the pole. A density check at that length measures this tail. It
should not be used as the oracle for the pole.

```
probe_the_walk_density_tail_against_the_pole_scale
        pole scale 1e-17 GeV², unbounded:  2472529 of 7500000 draws past 1e-7, 103075 degenerate, worst gap 6.79e25 (e+ e- > e+ e- mu+ mu- diagram 51)
          pole scale 1e0 GeV², unbounded:       28 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
          pole scale 1e1 GeV², unbounded:       24 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
          pole scale 1e2 GeV², unbounded:       24 of 7500000 draws past 1e-7, 0 degenerate, worst gap 3.52e-6 (u d > e+ e- u d QCD=0 diagram 32)
          pole scale 1e3 GeV², unbounded:       24 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
          pole scale 1e4 GeV², unbounded:       24 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
          pole scale 1e5 GeV², unbounded:       22 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
          pole scale 1e6 GeV², unbounded:       26 of 7500000 draws past 1e-7, 0 degenerate, worst gap 2.41e-6 (u d > e+ e- u d QCD=0 diagram 8)
        fiducial scale 400 GeV², bounded:       25 of 7500000 draws past 1e-7, 0 degenerate, worst gap 6.22e-7 (u d > e+ e- u d QCD=0 diagram 8)
```

**Composite emitted subsystems pass.** The bias check does not fail on them, so
the brief's fallback ("build unless the emitted side is composite") was not
needed. A mutation shows what happens without the floor: production spines
with the pole at zero break the identity on 816 of 2000 draws of the `llj`
diagram whose emitted side is the lepton pair (worst gap 6e4). This is the
composite case, and the floor removes it.

An edge-heavy sweep was tried first (every coordinate pushed to within
1e-12…1e-3 of the cube's ends). It was dropped. It breaks the identity on
20–30% of draws for the already-shipped 400 GeV² channels just as much as for
the no-scale ones, so it measures cube-edge degeneracy, not the pole.

## Bias at the σ level

`probe_the_no_scale_spine_on_a_mass_cut_peripheral_process` integrates the
process's own `Σ|M|²` under the run card's cuts. It uses plain Monte Carlo
(no VEGAS), with 8 seeds per arm combined by unweighted mean. Units are
arbitrary but the same for every arm of a process. Each table cell is the seed
mean ± spread (sd/√8), with χ²/dof the seeds' scatter about that mean against
their own errors.

**`e+ e- > e+ e- mu+ mu-`**: `√ŝ = 500`, `ptl = 0`, `mmll > 50`, default
`|η_l| < 2.5`, `ΔR > 0.4`. There are 48 diagrams in 34 channels, 18 of them
spines. Flat RAMBO is converged here.

| arm | draws/seed | mean ± spread | seed χ²/dof | ratio to flat | rel. per-draw variance |
|---|---|---|---|---|---|
| flat RAMBO | 4·10⁷ | 90.825 ± 0.107 | 0.41 | 1 | 1.01e3 |
| spine, production (before the 10⁴ change: scale 10²) | 2·10⁶ | 90.615 ± 0.174 | 0.39 | 0.9977 | 154 |
| spine, pole scale 1 | 2·10⁶ | 90.722 ± 0.370 | 1.21 | 0.9989 | 240 |
| spine, pole scale 10² | 2·10⁶ | 90.615 ± 0.174 | 0.39 | 0.9977 | 154 |
| spine, pole scale 10⁴ (now the default) | 2·10⁶ | 90.778 ± 0.214 | 1.02 | 0.9995 | 90 |
| spine, pole scale 10⁵ | 2·10⁶ | 90.915 ± 0.164 | 0.85 | 1.0010 | 62 |
| spine, pole scale 10⁶ | 2·10⁶ | 91.102 ± 0.185 | 1.15 | 1.0030 | 59 |
| all-timelike (known-wrong) | 2·10⁶ | 90.818 ± 0.667 | 1.30 | 0.9999 | 937 |

At the chosen default (10⁴) the pull against flat RAMBO is
(90.778 − 90.825)/√(0.214² + 0.107²) = −0.2. No arm is more than 1.4σ from flat
(the 10⁶ arm at +1.3). The scan ran before the default was set, so its
"production" row is the 10² trial value and is identical to the 10² row.

**`u c > e+ e- u c QCD=0`**: `√ŝ = 1 TeV`, `ptj = ptl = 0`, `mmjj > 300`,
`mmll > 50`, default `|η_j| < 5`, `ΔR > 0.4`. There are 24 diagrams in 10
channels, all 10 spines.

| arm | draws/seed | mean ± spread | seed χ²/dof | ratio to flat | rel. per-draw variance |
|---|---|---|---|---|---|
| flat RAMBO | 4·10⁷ | 8103 ± 1480 | 10.86 | 1 | 1.3e7 |
| spine, pole scale 1 | 2·10⁶ | 7973 ± 100 | 2.28 | 0.984 | 1368 |
| spine, pole scale 10² | 2·10⁶ | 8129 ± 51 | 0.73 | 1.003 | 817 |
| spine, pole scale 10⁴ | 2·10⁶ | 8091 ± 52 | 1.24 | 0.9985 | 539 |
| spine, pole scale 10⁵ | 2·10⁶ | 8181 ± 134 | 1.90 | 1.010 | 3673 |
| spine, pole scale 10⁶ | 2·10⁶ | 8060 ± 93 | 0.63 | 0.995 | 6149 |
| all-timelike (known-wrong) | 2·10⁶ | 8796 ± 1530 | 2.90 | 1.086 | 5.7e5 |

Flat RAMBO is **not** converged on this process: its seeds run from 5335 to
17926, with χ²/dof 10.9. That is the heavy-tailed failure AGENTS.md describes,
so it cannot serve as the independent reference here. The independent check on
this process is the agreement among pole scales that share no floor (1 to
10⁶ GeV²). They spread over 7973–8181, within 1.5σ of their mean. A
floor-driven bias would move with the floor. The known-wrong all-timelike row
is heavy-tailed: its seeds run from 5616 to 19011.

## Variance, and the default scale chosen

The relative per-draw variance is in the last column of both tables. The value
10⁴ GeV² is the lowest on `u c` (539). On `ee → eeμμ` it is 90, within 1.5× of
the best (59 at 10⁶). The larger scales cost `u c` a factor of 7–11 (3673 at
10⁵, 6149 at 10⁶). The floor that results, 10 GeV², sits more than 10⁸ above
the edge's rounding noise (~4e-8 GeV² at `ŝ = 2.5e5`). Against the all-timelike
tree, the default spine cuts per-draw variance by 10× on `ee → eeμμ` and by
about 1000× on `u c`.

## The production integrator (VEGAS), with the known-wrong row

`probe_the_no_scale_production_sigma_against_flat_rambo` runs
`FixedBeamIntegrand`, `use_multichannel` and `adapt_grids(200000, 8)` with five
seeds (11, 22, 33, 44, 55), combined by unweighted mean. The "before" row is
the same harness with the spine withheld, under the M1 mutation below. That is
today's no-spine behaviour.

| process | arm | peripheral channels | mean (pb) | spread (rel) | seed χ²/dof |
|---|---|---|---|---|---|
| `u c > e+ e- u c` | flat RAMBO + VEGAS | – | 6.5e-4 | 0.84 | 297 |
| | multichannel, no spine (before) | 0/10 | 1.99656e-2 | 7.1e-2 | 15.8 |
| | multichannel, spine (after) | 10/10 | 1.79863e-2 | 2.7e-3 | 1.76 |
| `e+ e- > e+ e- mu+ mu-` | flat RAMBO + VEGAS | – | 5.3e-3 | 0.59 | 5045 |
| | multichannel, no spine (before) | 0/34 | 7.29514e-3 | 4.5e-3 | 2.04 |
| | multichannel, spine (after) | 18/34 | 7.29219e-3 | 1.4e-3 | 2.55 |

Flat RAMBO under VEGAS collapses on both processes, so it is no reference in
pb. The no-spine row on `u c` is 11% above the spine row with seed χ²/dof 15.8.
That is the same heavy-tailed signature as the CMS sweep, here on the high
side. With the spine, the spread drops 26× on `u c` and 3× on `ee → eeμμ`.

## Banked rows whose channel set changed

**Method.** I ran the banked suite at the parent (`90fc171f`, in a scratch
worktree) and on `c5d44b86`'s tree. I diffed every printed measurement. The
only source change between the runs is the channel derivation, so a moved
number means a moved map.

**Census source.** This is a recorded diff, not an inference. Two parts of the
run were cut off:

- The full-suite runs hit the 2 h background limit inside `validate_sigma` and
  `validate_samples`.
- I re-ran those and every later target separately, on both sides.

Every row below was measured on both sides and passed both times. No other
printed σ or sample statistic moved: 307 keyed σ lines compared, plus every
sample line.

| row (gate) | why it changed | before | after |
|---|---|---|---|
| `ee_to_wpwm` (validate_sigma, 5 seeds) | 2→2, neutrino exchange re-poled 0 → 10 GeV² | 7.196298 ± 1.56e-3 pb, pull +0.50, seed χ² 0.54 | 7.196160 ± 1.56e-3, pull +0.47, seed χ² 0.55 |
| `ee_to_wpwm_cw` (validate_sigma) | same | 7.287789 ± 1.62e-3, pull +0.39, χ² 0.40 | 7.287786 ± 1.62e-3, pull +0.39, χ² 0.38 |
| `ee_to_wp0wm` default / transverse / unpolarized (polarization σ, 10 seeds) | same | 0.2578730 (pull +0.07) / 6.938539 (+0.76) / 7.196470 (+0.27) | 0.2578659 (+0.05) / 6.938526 (+0.76) / 7.196430 (+0.26) |
| `ee_to_wpwm` samples, three seeds (validate_samples) | same | min KS p 1.57e-4 / 1.64e-1 / 2.41e-4 / 5.87e-2 | 1.27e-4 / 1.53e-1 / 1.83e-4 / 5.58e-2 |
| `zz_ee` decay chain `e+ e- > z z, z > e+ e-` (informational) | 2→4 with `cut_decays = F`, floor 0: spine now built on the electron exchange | 4.731599e-4, per-seed err ≈5.3e-7, pull +2.91 (informational) | 4.731459e-4, per-seed err ≈3.4e-7, pull +3.20 (informational) |
| `zz_emu` decay chain | same | 9.443556e-4 ± 3.47e-7, pull +0.58, worst seed 1.43 | 9.447290e-4 ± 2.29e-7, pull +1.15, worst seed 1.99 |
| `pp_ttx_lep` decay chain `p p > t t~, t > b e+ ve, …` | 2→6, floor 0: spine now built on the top exchange (m² = 2.99e4, unfloored) | 6.413151 ± 4.13e-3, pull +0.09 | 6.410089 ± 3.70e-3, pull −0.29 |
| `pp_ttx_lep_dyn` events (cli_decay_chain_events) | same | trees χ² 8.7/3 (p 0.033), m_wm p 0.043; replay 2000/2000 | trees χ² 2.8/2 (p 0.25), m_wm p 0.071; replay 2000/2000 |
| `uu_tt` on-shell veto `u u~ > w+ b w- b~ $ t t~` | 2→4, ptb = 0: spine now built | 4.980480e-2 ± 5.0e-5, pull +4.86, χ² 1.41 | 4.975220e-2 ± 3.1e-5, pull +4.69, χ² 0.20 |

**Notes on the table.**

- The `uu_tt` 2% offset from MadEvent is unchanged by this session. It was
  +4.86σ before and +4.69σ after, and the gate passes on both sides.
- The `zz_ee` excess was informational before and still is.
- No `e+ e-` four-fermion σ row with `ptl = 10` moved
  (`ee_to_mumu_tata_qcd0` and its controls): its floor is 100 GeV².
- No hadronic llj/jj/bb row moved.
- `pp_to_bb*` (`ptb = 0`) did not move. Its only spacelike line is the b quark
  (m² = 22 GeV² > 10).
- The lib unit test over `e+ e- > mu+ mu- ta+ ta-` with `from_diagram` now
  builds spines. Its Z-pole variance gains moved from 3.1× / 3.3× to
  2.8× / 3.0× against flat.

## Mutations

Each mutation was applied to `diagram_channel.rs`, then the two new tests were
run, then the file was restored from a backup. The restore was checked by diff.

- **M1, spine withheld again.** Restored
  `&& (n_out == 2 || fiducial_scale > F::zero())`.
  ```
  thread 'without_a_fiducial_scale_every_peripheral_diagram_gets_its_spine_on_a_pole_floor' panicked at vibegraph-lib/tests/diagram_channel.rs:2806:9:
  assertion `left == right` failed: u u~ > e+ e- g QCD=2 QED=2: a 1-line peripheral diagram built 0 rungs with no fiducial scale
  test result: FAILED. 1 passed; 1 failed
  ```
  The walk/density test passed under M1, because a timelike tree is
  self-consistent. I then added an assertion that it is sweeping spines, so M1
  now fails it too, at its first diagram. The M1 production run is the "before"
  row in the VEGAS table.
- **M2, pole floor off with no scale.** Restored
  `t_mass2: if bounded { m2.max(pole_floor) } else { m2 }`.
  ```
  thread 'a_spine_without_a_fiducial_scale_keeps_its_walk_and_its_density_in_step' panicked at vibegraph-lib/tests/diagram_channel.rs:2920:9:
  assertion `left == right` failed: u u~ > e+ e- g QCD=2 QED=2 diagram 2: 816 draws past the walk-density tolerance (worst 5.984e4) and 1 degenerate ones without a fiducial scale
  thread 'without_a_fiducial_scale_every_peripheral_diagram_gets_its_spine_on_a_pole_floor' panicked at vibegraph-lib/tests/diagram_channel.rs:2815:17:
  u u~ > e+ e- g QCD=2 QED=2: rung 0 draws against a massless pole with no floor under it
  test result: FAILED. 0 passed; 2 failed
  ```

## Gates

The commands were run on `c5d44b86`'s tree, before the report commit. The
report commit touches only `research/kb/`.

- **fmt:** `cargo fmt --all --check` → exit 0.
- **clippy:**
  - `cargo clippy --workspace --all-targets -- -D warnings` → `Finished … exit=0`.
  - `cargo clippy --workspace --all-targets --features vibegraph/extended-validation -- -D warnings`
    → `Finished … exit=0`.
  - The first run flagged `type_complexity` on a probe; it is fixed with the
    `ProbeArm` alias.
- **hermetic:** `cargo test --workspace` → 48 targets, 1706 passed, 0 failed,
  22 ignored, exit 0.
  - `diagram_channel`: 26 passed, 3 ignored (the new probes).
  - lib: 1108 passed.
- **banked**, `--profile release-debug --features vibegraph/extended-validation,vibegraph-lib/extended-validation --no-fail-fast`:
  - every target to `validate_scales` in the full run, then `validate_sigma`
    and `validate_samples … weighted_search` separately;
  - every `test result:` line reads `0 failed`.
  - Representative lines:
    - `validate_sigma`: `ok. 8 passed; 0 failed; 33 ignored` (1320 s).
    - `validate_hadronic`: `ok. 20 passed; 0 failed; 26 ignored`.
    - `validate_samples`: `ok. 6 passed; 0 failed`.
    - `validate_unweighting`: `ok. 1 passed`.
  - The collator (`validation-report`) was not run.
- **kb lint:** `python3 scripts/kb.py lint` → `998 concepts, 333 backlog items;
  0 errors, 4 warnings`. The 4 warnings are pre-existing `state: blocked`
  backlog items.

## Corrections to the brief

1. **The density check cannot be run "over enough points" at the unchanged
   tolerance.** The walk/density identity has a pole-independent tail of about
   3 draws per million past `1e-7` (worst `2.4e-6`). It is present on the
   channels production already builds with a 400 GeV² scale. The test
   therefore stays below the tail at 2000 draws a channel, where the
   rounding-noise flip shows 51,502 times. The probe documents the tail.
2. **The suggested σ process is not one flat RAMBO can serve as reference
   for.** `u c > e+ e- u c` with only invariant-mass cuts (finite only through
   the default `|η_j| < 5`) is heavy-tailed under flat RAMBO and under
   flat-RAMBO VEGAS. The converged σ-level reference is
   `e+ e- > e+ e- mu+ mu-`. On `u c`, agreement across pole scales stands in
   for it.
3. **The banked census is wider than "a final state of more than two legs".**
   The pole floor is now on for `2 → 2` rows with no pT cut too: the
   `ee_to_wpwm` family's neutrino exchange. Decay-chain and on-shell-veto rows
   with `cut_decays = F` or `ptb = 0` have floor 0 even though their cards set
   `ptl`.

## Found

- **WALK_DENSITY_TOL's calibration covers only short sweeps.** Its doc says the
  worst gap measured is 7.1e-9. Over 10⁵ draws per channel, production
  channels (bounded, 400 GeV²) exceed `1e-7` on 25 of 7.5M draws, worst
  6.2e-7. Unbounded, the worst is 2.4e-6, on `u d > e+ e- u d` diagram 8, a W
  rung emitting a single leg. Evidence: `probe_the_walk_density_tail_against_the_pole_scale`.
  To close: locate the near-threshold cancellation (dump the worst point's
  invariants), then either reformulate it or re-derive the tolerance from a
  long sweep.
- **Edge-of-cube degeneracies in both walk and density.** With coordinates
  within 1e-12 of 0 or 1, every arm (fiducial 400 GeV² included) gives
  `density = inf` or `weight = 0` with nonzero density on some draws (about
  20k of 1.5M in the edge-heavy sweep). Under uniform sampling this is measure
  zero. A multichannel combiner gives such a point zero weight. To close:
  decide whether `density` should return a finite value at those
  configurations, and pin it with a test.
- **Artifacts do not record the map derivation.** `FORMAT_VERSION` stays 14.
  An artifact integrated before this change, for a process with no pT cut and
  more than two legs (or the `ee_to_wpwm` family), would replay its grids
  against different maps. `check_channel_keys` compares `(group, diagram)` keys
  and grid dimensions, which both still match. To close: bump the format, or
  bank a digest of the channel `map_key`s and compare it at generate time.
- **Possible collinear divergence in the CMS cards (unverified hypothesis).**
  With massless quarks, `ptj = 0` and no `etaj` cut, photon exchange between
  the quark lines of `l+ l- j j QCD=0` is log-divergent in the quasi-real
  limit. The `u c` probe is finite only because the default `|η_j| < 5` keeps
  the jets off the beam. To close (T-V1): read `etaj`/`drjj` off the three
  genproductions cards. If no cut regulates the region, MadGraph's 13.12 /
  7.574 / 117.6 pb is a sampling artefact and no ladder will converge to it.
- **Flat RAMBO + VEGAS collapses on peripheral processes** (σ low by 30×–10⁴×
  with iteration χ²/dof up to 2e8 on `u c`). Any gate or probe that uses the
  `n_survey = 0` path as an "independent" reference on such processes is
  unsound. To close: document it at `with_integrand`'s `n_survey = 0` option,
  or refuse it there.
