---
type: Sprint Record
title: "Proton runs with unequal beam energies"
description: "√s = 2√(E1E2), partons at x_i·E_i and MadGraph's cm_rap boost, bit-identical at equal beams; the 6500 x 4000 row exposed MadEvent's mirrored-ordering cut defect, registered as a deviation, and gates at pull +0.95 against an emulation."
closed: 2026-10-26
status: draft
tags: [proton, beams, kinematics, madgraph-defect, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: v3, resource: "sessions/V3-report.md", title: "V3 report"}
  - {id: v4, resource: "sessions/V4-report.md", title: "V4 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/67", title: "nsmith-/vibegraph#67"}
---
One PR ([nsmith-/vibegraph#67](https://github.com/nsmith-/vibegraph/pull/67),
merged 2026-10-26) closing `collider-energy-sum-of-beam-energies`, split out of
the cut-refusal PR (#59) by the user's decision of 2026-10-21. Sessions:
[F1](sessions/F1-report.md) (feature-dev, Opus), [V1](sessions/V1-report.md),
[V3](sessions/V3-report.md) and [V4](sessions/V4-report.md) (validation-dev,
Opus), [V2](sessions/V2-report.md) (validation-dev, Sonnet). The planned hygiene
session (HY1) did not run. The M3 banked the row.

## What was established

- **The frames.** On `lpp = 1` the proton path assumed equal beams. It now
  uses √s = 2√(E₁E₂), partons at `x_i·E_i` and MadGraph's `cm_rap` boost
  (`genps.f:345`), `β = (x₁ − r·x₂)/(x₁ + r·x₂)` with `r = E₂/E₁`. At equal
  beams every quantity reduces exactly, so every existing row is bit-identical,
  and a test pins it. *Manager-verified:* the three new `proton::tests` pass, the
  boost reduces exactly, and `vegas.rs`, `budget.rs` and `phasespace/` are
  untouched.
- **Guards.** A proton beam below 0.938 GeV (or non-finite) is refused at parse.
  The library's `check_collider_energy` refuses an artifact whose recorded run
  card gives another √s.
- **Massive beams.** MadGraph's √stot (0.938 GeV beams) equals 2√(E₁E₂) exactly
  at equal beams and differs by −1.0e-9 at 6500 × 4000; the ≈2e-8 residual is the
  lab boost's rapidity, present at equal beams too (V1's correction to F1).
- **The row exposed a MadEvent defect.** At 6500 × 4000 the M3's first run read
  vibegraph 754.956 ± 0.352 pb against MadGraph 717.25 ± 0.145 (+5.26 %), with
  KS failures on the rapidity and beam columns. V3 traced it to MadEvent:
  grouped MadEvent runs `passcuts` once per point (`dsample.f:181`) before
  `select_grouping` picks the mirrored ordering (`:222`), whose own `PASSCUTS` is
  commented out (`super_auto_dsig_group_v4.inc:835`). The mirrored half is cut in
  a lab frame `ln(E₁/E₂)` off, which vanishes at equal beams. Emulating that cut
  gave 717.575 ± 0.309 pb. MadGraph's own events confirm it (M3): 0 of 99807
  direct-ordering events lie outside |η| ≤ 2.5, against 8429 of 100193 mirrored
  ones, all inside [−2.014, 2.986]. 3.8.1 has the fix; 3.5.17 LTS does not.
- **Registered deviation (user, 2026-10-22).** This crate keeps the correct cut.
  The σ gate compares a test-only emulation of MadEvent's cut (compiled only
  under `cfg(test)` or `extended-validation`): 717.575 ± 0.309 against the seed
  record's 717.250 ± 0.145, pull +0.95, rel +0.05 %. The crate's own σ is
  reported beside it. The samples cell gates the columns the defect cannot move
  and the direct-ordering sub-sample on every column. The defect is catalogued in
  `validation/madgraph-defects.md` with an unfiled upstream draft. *Manager-
  verified (V4):* the production path is unchanged outside the test-only
  emulation.
- **Known-wrong control** (M3). With the old energy-sum frames, the gate reads
  755.890 ± 0.341 pb, pull +104.4. The control moves the emulated cut's frames
  too, so it shows the gate sees the change, not how the 5.39 % splits.

## Backlog

- Deleted: `collider-energy-sum-of-beam-energies`.
- Drafted for filing at close-out: `fixed-beam-energy-not-validated` (V1 Found 5,
  V2 Found 4), `proton-massless-beam-residuals-unrecorded` (F1 Found 2, V1 Found
  4), `absent-run-escapes-spelled-per-gate` (V2 Found 1),
  `time-stages-keeps-its-own-row-card-map` (V2 Found 2).
- Proposed edit, not a new item: `madgraph-defect-reports-unfiled` should name
  the mirrored-ordering cut, whose draft is
  `research/upstream/madgraph-mirrored-cut-unequal-beams/`.
- Not filed:
  - F1 Found 1, 4 (no beam-energy check; a stale artifact √s): the parse guard
    and `check_collider_energy`;
  - F1 Found 3 (the pT-maximum no-op ceiling is `(E₁+E₂)/2` where `√(E₁E₂)` is
    exact): sound, only conservative, and no surveyed card is affected;
  - V1 Found 1 (the CI compile break): fixed by the main merge (`49d6802`);
  - V1 Found 2 (`validate_hadronic` at constant beams): V2 reads each card's
    beams, `to_bits`-identical on the dy13 cards;
  - V1 Found 3 (`collider_energy`'s doc): it now states the equal-beam
    condition and the unequal-beam expression;
  - V2 Found 3 (the m_ll spectrum judged at 10.2 TeV): the M3 ran it at the
    pushed tree and it passes.

## Process notes

A gate that failed by 5 % was diagnosed instead of re-toleranced, and the
diagnosis carried its own falsifier (the η window of MadGraph's mirrored events),
which the M3's events then confirmed. V1's permission layer refused `git fetch`
and `git merge`; the manager put that to the user rather than working around it.
The bank request's positional `bank-seeded-sigma` form was wrong (the script
takes `ROWS=`); the M3 caught it.
