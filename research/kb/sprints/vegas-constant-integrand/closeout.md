---
type: Sprint Record
title: "Constant integrands integrate once"
description: "A structurally constant integrand (an unpolarized, uncut two-body decay at rest, one channel, no configuration draw) draws one iteration and stops, reporting the exact width; a declaration the iteration contradicts is withdrawn."
closed: 2026-10-17
status: draft
tags: [vegas, decay-width, two-body, constant-integrand, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: p1, resource: "sessions/P1-report.md", title: "P1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/50", title: "nsmith-/vibegraph#50"}
---
One PR ([nsmith-/vibegraph#50](https://github.com/nsmith-/vibegraph/pull/50))
closing `vegas-refinement-noises-constant-integrand`. Sessions:
[P1](sessions/P1-report.md) (performance-dev, Sonnet) and
[HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **The condition is structural, never sampled.**
  `FixedBeamIntegrand::has_constant_integrand` requires all of: a decay at
  rest, two bodies, no polarized leg, one channel,
  `Cuts::admits_every_point()`, and no configuration draw.
  - With helicities summed, Σ|M|² of a two-body decay at rest is rotation
    invariant. Flux, averages and phase-space weight do not depend on the
    point.
  - `admits_every_point` destructures `Cuts` and `SingleLegCut` in full, so a
    new cut field does not compile until it is classified.
- **The run.** A declared integrand draws one iteration with no warm-up and
  stops with `StopReason::Constant`. If the iteration's relative error exceeds
  `CONSTANT_REL_ERROR = 1e-7`, the declaration is withdrawn and the run
  continues as an ordinary one. The measured error is 2e-9 to 6e-9 at 1.2e5
  to 1e7 points.
- **Widths.** All 19 SM two-body rows match `decays.py` to 1e-10. `h > b b~`
  reads 5.390779e-3, against 5.388806e-3 ± 0.033 % before, in about half the
  wall time.
- **The `cli_decay` gate.** `t_wb` and `z_ee` integrate exactly, so their ten
  seeds are one number and the seed-scatter window has nothing to read. They
  are held to the exact width at 1e-6, the resolution of the printed result
  and about 300× tighter than their old noise. No window moved.
- **Inert elsewhere.** Every scattering integrand is pinned non-constant
  (`two_beams_into_two_bodies_are_not_a_constant_integrand`). 11 of 11
  non-constant CLI comparisons against the base binary are bit-identical in
  result and grid hash.

## Trust

Machine-confirmed by the manager:
- the commits and trailers;
- `decay_widths` (5 passed) and `cli_decay` (3 passed) re-run on P1's
  commit;
- the review of the `cli_decay` gate change;
- one of HY1's mutations re-run (the single-channel condition dropped): it
  fails `a_second_channel_or_a_configuration_draw_withdraws_the_declaration`.

Asserted by the sessions: the bit-identity table, workspace clippy and tests,
and the other mutations. `pixi run --skip-deps validate` was not run on this
host; CI's banked job on the PR is the banked-layer evidence.

## Filed

- [alpha-survey-runs-for-one-channel](../../backlog/performance/alpha-survey-runs-for-one-channel.md)
- [flat-weights-print-rounding-noise](../../backlog/hygiene/flat-weights-print-rounding-noise.md)
- [constant-two-body-excludes-polarized-daughters](../../backlog/feature/constant-two-body-excludes-polarized-daughters.md)
- [integrate-channels-monolithic](../../backlog/hygiene/integrate-channels-monolithic.md)
