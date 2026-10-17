---
type: Session Report
title: "P1 report: a constant integrand integrates once"
description: "A single-channel helicity-summed two-body decay at rest is declared constant from the process, draws one iteration, and reports the analytic width to 1e-13; every non-constant run is bit-identical to before."
status: draft
tags: [vegas, decays, width, performance, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-17}
sources:
  - {id: 79eacd9, resource: "https://github.com/nsmith-/vibegraph/commit/79eacd9", title: "vegas: a structurally constant integrand integrates once"}
---
Backlog: `vegas-refinement-noises-constant-integrand`. Regime: deterministic
(a seeded run's non-constant paths are gated bit-for-bit; the constant path
has no sampling to gate and is gated against the analytic width). The change
is **order-preserving** for every integrand that does not declare itself
constant, and for the declared ones it replaces a statistical estimate with a
deterministic one, so "bit-for-bit" does not apply to them and is not claimed.

Everything below was run in `/home/user/wt/vegas-constant-integrand`, branch
`vegas-constant-integrand`, logs in the session scratchpad with the prefix
`vegasconst-P1-`.

## What was implemented

- `ChannelIntegrand::is_constant()` (`vibegraph-lib/src/unweight.rs`), default
  `false`.
- `FixedBeamIntegrand::has_constant_integrand()` (`hadronic.rs`) implements it.
  **The condition**, all of it read from the process and never from sampled
  values:
  1. initial state is a decay at rest (not two beams);
  2. exactly two final-state bodies;
  3. no polarized leg in any subprocess (`AmplitudeEvaluator::is_polarized`);
  4. one sampling channel;
  5. `Cuts::admits_every_point()` (new, `cuts.rs`): no ŝ window, no decay-chain
     window, no `mmnl` window, no pair threshold, and every single-leg
     threshold at a value no momentum fails;
  6. no configuration draw (`scale_draw_ndim() == 0`).
- `integrate_channels` (`budget.rs`): for a declared integrand the warm-up is
  0, and after the first iteration the run stops with the new
  `StopReason::Constant` without refining, with `kept_iterations = 1`. The CLI
  prints `constant integrand: one iteration, no grid refinement (relative error
  …)` and, in target mode, `met exactly (constant integrand, drawn once)`.
- **Why it cannot misfire.** A two-body decay at rest has one kinematic degree
  of freedom, the direction of a daughter. Flux, spin/colour average, symmetry
  factor and two-body phase-space weight are point-independent; with the
  helicities of all legs summed, `Σ|M|²` is rotation invariant; the coupling
  scale is built from rotation-invariant kinematics. Conditions 1-6 are each
  needed for that argument (a beam pair gives `1 + cos²θ`; three bodies a
  Dalitz plane; a cut a step; a configuration draw a random scale). Sampled
  variance is not used to decide. As a second line, the one iteration is
  checked: if its relative error exceeds `CONSTANT_REL_ERROR = 1e-7`
  (`budget.rs`) the declaration is withdrawn with a `WARN` and the run
  continues as an ordinary run (warm-up 0, since it was set before the check).
  The 1e-7 sits above the cancellation floor of `Σf²/n − mean²`
  (`√ε ≈ 1.5e-8` relative on the mean at any n; measured 2e-9 to 6e-9 at
  `--neval` 1.2e5, 2e6 and 1e7 on `h > b b~`, see below).
- Polarized legs are excluded even though a fixed-helicity daughter with the
  mother summed is isotropic: the conservative choice, pinned by a test
  (`t > w+{0} b`). Initial-state polarization of a decaying particle is
  refused upstream (`DecayedPolarization`), so condition 3 is exercised only
  on a final-state polarized leg.

## Deviations from the brief

- The brief says "reports the exact width (zero or ulp-scale error)". The
  quoted error is **not** ulp-scale: it is the cancellation residue of the
  one iteration's variance, 2e-9 to 6e-9 relative (or `1.5e-154`, the
  `MIN_POSITIVE` floor `combine_kept` applies, when the variance is exactly
  zero, e.g. `t > w+ b` in the lib test). The width itself is ulp-scale
  (relative difference from `decays.py` 1e-14 to 5e-13, below). The output
  states the relative error and what it is. I did not set the error to 0.0: a
  zero would be a claim, whereas this number is what the iteration measured.
- The checked-in `cli_decay.rs::partial_widths_match_madevent_over_seeds` had
  to change (see Gates). This is a change to a gate and is my call, flagged
  for review.

## Gates (command, then outcome)

All with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`.

| Gate | Command | Result |
|---|---|---|
| format | `cargo fmt --all --check` | exit 0 (after `cargo fmt --all`) |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | `clippy=0` (`vegasconst-P1-clippy.log`) |
| hermetic suite | `cargo test --workspace` | `exit=0`, 36 test binaries all `ok`, 0 failed (`vegasconst-P1-workspace.log`) |
| two-body widths, once | `cargo test -p vibegraph-lib --test decay_widths -- --nocapture` | 19 two-body rows, all `1 iteration(s)`, `rel` to `decays.py` 1.3e-14 to 4.6e-13 (gate 1e-10) |
| CLI decay gate | `cargo test -p vibegraph --test cli_decay -- --nocapture --test-threads=1` | `3 passed; 0 failed`, 90.9 s (`vegasconst-P1-clidecay2.log`) |
| MG net | `pixi run --skip-deps validate` | **not run** |

What I did **not** run: `pixi run --skip-deps validate` and the
`extended-validation` gates. Rationale, not an assertion that they pass: they
integrate scattering processes, for which `is_constant()` is `false`
(`FixedBeamIntegrand::has_constant_integrand` requires `InitialState::Decay`,
pinned by `two_beams_into_two_bodies_are_not_a_constant_integrand`) and the
code path is the unchanged loop. The oracle I used for "nothing else moved" is
below. The decay rows of that net (`t_bev`, `h_4l`, the two `_cuts` rows) ran
in `cli_decay` and read as before: see `vegasconst-P1-clidecay2.log`.

## Nothing else moved (oracle: artifact bytes at fixed seed)

Script `vegasconst-P1-inert.sh`: the same proc card, run card and seed through
a binary built from `HEAD` before my change (a detached worktree,
`release-debug`) and through the new binary; compare the printed result and
`sha256` of `grid.bin.zst` (the full trained grids, channel weights and
results). `--target-rel 5e-3 --max-iters 8` unless noted.

| Process (run card) | seeds | base == new, artifact sha256 |
|---|---|---|
| `t > b e+ ve` | 1, 2 | identical |
| `t > b e+ ve` `--fixed-budget --neval 20000 --niter 4` | 3 | identical |
| `z > e+ e-` (lepton cuts that bite) | 1, 2 | identical |
| `h > e+ e- mu+ mu-` | 1, 2 | identical |
| `e+ e- > mu+ mu-` (500 GeV beams) | 1, 2 | identical |
| `e+ e- > mu+ mu- a` (500 GeV beams) | 1, 2 | identical |
| `t > w+ b` (lepton cuts only; none touch `b`, `w+`) | 1, 2 | **differs, by design** (constant) |

11 of 11 non-constant comparisons are bit-identical (value and hash). The
seeded `VegasGrid::adapt` golden (`test_pinned_seed_regression_grid_adapt`) is
in `cargo test --workspace` and passed; `vegas.rs` is untouched.

## Before / after (release-debug, this host, 4 cores shared with siblings)

Default `vibegraph integrate` (target 0.1 %), the "before" being the `HEAD`
binary. Γ in GeV, wall clock of the whole command including enumeration and
the α survey.

| Process | exact (`decays.py`) | before, seeds 1/2/3 | after, seeds 1/2/3 |
|---|---|---|---|
| `h > b b~` | 5.390779e-3 | 5.388806e-3 / 5.390555e-3 / 5.392335e-3, each ± 1.78e-6 (0.033 %), 720 000 evals over 6 iters, 0.72-0.82 s | 5.390779e-3 ×3, ± 2.09e-11, 120 000 evals over 1 iter, 0.35-0.44 s |
| `z > e+ e-` | 8.396539e-2 | 8.393467e-2 / 8.396191e-2 / 8.398962e-2, ± 2.77e-5 | 8.396539e-2 ×3, ± 4.45e-10 |
| `t > w+ b` | 1.491472 | 1.490926 / 1.491410 / 1.491903, ± 4.9e-4 | 1.491472 ×3, ± 1.14e-9 |
| `w+ > u d~` | 0.682692 | 0.682442 / 0.682664 / 0.682889, ± 2.25e-4 | 0.682692 ×3, ± 1.60e-9 |

6x fewer evaluations; wall time is dominated by the fixed setup (the survey
runs unchanged, see Found). Regressions elsewhere: none measured; a
non-constant run executes the same arithmetic as before (bit-identical
above). Cross-host times are not compared.

Side effect, seen once and not gated: `vibegraph generate` from the new
`z > e+ e-` artifact (seed 5, 2000 events) has unweighting efficiency 1.0
against 0.49 from the old, noise-refined artifact (`INFO sampling` lines),
because the unrefined grid sees an exactly flat integrand.

## Tests added and mutation evidence

- `budget.rs`: `a_declared_constant_integrand_draws_once_and_reports_its_level`
  (Fixed and Target budgets: `StopReason::Constant`, 1 iteration, 4000 points,
  value within 1e-12, grid still uniform), `an_undeclared_constant_integrand_is_integrated_as_any_other`
  (same values, undeclared: 6 iterations, grid refined: the shortcut is the
  declaration's, not a sampled-variance heuristic), `a_declaration_the_iteration_contradicts_is_withdrawn`.
- `cuts.rs`: `a_filter_with_every_threshold_off_admits_every_point`,
  `each_threshold_withdraws_the_claim_that_nothing_is_rejected` (12
  thresholds one at a time), `a_shat_window_or_a_decay_window_withdraws_the_claim`.
- `hadronic.rs`: `two_beams_into_two_bodies_are_not_a_constant_integrand`.
- `decay_widths.rs`: `two_body_widths_integrate_once_and_exactly` (19 rows,
  8 iterations budgeted, `StopReason::Constant`, 1 iteration, 20 000 points,
  `rel < 1e-10`, quoted `err/Γ < 1e-7`) and
  `only_a_structurally_constant_decay_skips_refinement` (`t > b e+ ve`,
  `t > w+{0} b`, `z > e+ e-` under the LO default lepton cuts: not constant,
  all iterations run, the cut width below 0.99 of analytic).
- `cli_decay.rs`: `a_two_body_width_is_integrated_once_and_a_three_body_one_is_not`
  (the printed width against `decays.py` to 1e-6, the print resolution; log
  says one iteration). **Gate change:** in
  `partial_widths_match_madevent_over_seeds` the two exact two-body rows
  (`t_wb`, `z_ee`) are now held to the exact width at 1e-6 (printed
  resolution) and skip the seed-scatter statistics. Left as it was, they
  fail: a deterministic result has ten identical seeds, so χ²/dof about the
  mean is 0 or, with the `MIN_POSITIVE` floor, enormous, against a window of
  [0.15, 3.1] built to catch a sampler that misses a region. I did not widen a
  window; the rows moved to a tighter test than the one they had (3e-4 noise
  scale to 1e-6), selected by the process string (two daughters, no run
  card), not by their result. Verified: the first run of this gate with only
  the code change failed on exactly `["t_wb", "z_ee"]`
  (`vegasconst-P1-clidecay.log`), the second, after the edit, passed.

Mutation evidence: `vegasconst-P1-mutate.py` applies one edit to the source,
runs `cargo test -p vibegraph-lib --lib -- budget:: cuts:: two_beams_into` and
`--test decay_widths`, and restores the file. Outcome (logs
`vegasconst-P1-mutate.log`, `-mutate2.log`):

| Mutation | Caught by |
|---|---|
| M1 drop two-body condition | `only_a_structurally_constant_decay_skips_refinement` |
| M2 drop cuts condition | same |
| M3 drop unpolarized condition | same |
| M4 drop decay-at-rest condition | `two_beams_into_two_bodies_are_not_a_constant_integrand` |
| M5 drop single-channel condition | **not caught** |
| M6 drop no-configuration-draw condition | **not caught** |
| M7 no runtime check (always stop) | `a_declaration_the_iteration_contradicts_is_withdrawn` |
| M8 keep default warm-up | `a_declared_constant_integrand_draws_once…`, `two_body_widths_integrate_once_and_exactly` |
| M9 never break | same two |
| M10 error bound 1e-7 to 1e-12 | `two_body_widths_integrate_once_and_exactly` (not the toy: its variance is exactly 0) |
| M11 cuts: ignore pair thresholds | `each_threshold_withdraws…` |
| M12 cuts: ignore `eta_max` | same |
| M13 cuts: ignore `pt_min` | same |
| M14 cuts: ignore decay windows | `a_shat_window_or_a_decay_window…` |
| M15 cuts: ignore `mmnl` | `each_threshold_withdraws…` |
| M16 cuts: ignore ŝ window | `a_shat_window_or_a_decay_window…` |

M5 and M6 are not caught because no SM tree-level `1 → 2` decay has more than
one diagram, and a two-body decay has no scale draw; I could not construct a
reachable case, and kept both as guards for models that have one. The
mutation script was killed once mid-run by the background time limit with M8
applied; I reverted that edit by hand, re-grepped every mutation site and re-ran
the remaining mutations (M4, M8-M16) detached. `git diff` shows no residue.

## Brief errors / corrections

- "Reports … zero or ulp-scale error": see Deviations; the error is
  cancellation-scale, 1e-9 relative.
- "`cargo test` … Gate: hermetic": the old CLI exact-row gate was not
  compatible with a deterministic result (above). The brief did not anticipate
  a statistical gate over rows that become exact.

## Found

1. **The α survey runs for a single channel.** On `h > b b~` the log shows
   `alpha_survey: surveying 1 channels over 6 × 40000 points` before the
   one iteration, 240 000 evaluations to learn `α = [1.0]`, twice the
   integration's 120 000. Closing it: skip `adapt_alphas` when `built.len() ==
   1` in `FixedBeamIntegrand::use_multichannel` (and the proton analogue).
   Not done: it is outside scope and it touches the streams of every
   single-channel run (the survey has its own stream, so integration should
   not move, but that needs its own bit-for-bit check).
2. **`vibegraph generate` from a constant-integrand artifact prints `overweight:
   rate 1.000e0 … largest w/w_max 1.000`** (seed 5, `z > e+ e-`). Every
   weight equals `w_max` to rounding, so the `>` comparison fires on ulp noise
   and the line reads as total overweight. Evidence: the `INFO overweight`
   line in the `generate` output of my run. Closing it: compare with a
   relative tolerance, or test the pre-existing behaviour on any exactly flat
   integrand.
3. **`ConvergenceReport` and the CLI's `quoted 0.0000 %`** read an error of
   `1.5e-154` (`MIN_POSITIVE`) as a width error when a constant integrand's
   variance is exactly zero (`t > w+ b` in the lib test prints `± 1.5e-154`).
   Honest, but it looks like a bug. Closing it: print `exact` or the rounding
   floor when `StopReason::Constant`.
4. **Same structural argument for `1 → 2` with N-body flat phase space is not
   claimed.** `h > e+ e- mu+ mu-` and every N ≥ 3 decay still refine. Nothing
   to do; recorded so nobody extends the condition by analogy.
5. **Polarized final legs of a two-body decay are excluded though constant.**
   `t > w+{0} b` is isotropic (mother summed, daughter helicity defined along
   its own momentum). Closing it: relax condition 3 to initial-state
   polarization only, with a test against `decays.py`'s polarized widths if
   the oracle has one; none was available.
6. **`pixi run --skip-deps validate` was not run on this branch** (host
   shared with two siblings). The manager should run it, or accept the
   bit-identity table above, before merge; no cell is expected to move.

## Files touched

All under `/home/user/wt/vegas-constant-integrand/`:
`vibegraph-lib/src/budget.rs`, `vibegraph-lib/src/cuts.rs`,
`vibegraph-lib/src/hadronic.rs`, `vibegraph-lib/src/unweight.rs`,
`vibegraph-lib/tests/decay_widths.rs`, `vibegraph-cli/src/integrate.rs`,
`vibegraph-cli/tests/cli_decay.rs`; this report. Code commit `79eacd9`.
