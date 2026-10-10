---
type: Sprint Record
title: "Hygiene sprint"
description: "The first sprint in the folder shape: eight cluster reviews (176 findings), ten fix sessions (125 fixed, at least 112 mutations run), 25 items closed, 54 filed, the pub surface cut from 2601 to 1359 lines, and the lessons that design the hygiene agent."
closed: 2026-10-10
status: draft
tags: [hygiene, sprint, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: triage, resource: "triage.md", title: "Triage of the eight reviews"}
  - {id: lessons, resource: "../../workflow/hygiene-review.md", title: "Hygiene review lessons (draft)"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/19", title: "nsmith-/vibegraph#19"}
---
The hygiene sprint reviewed the whole codebase for maintainability, test
non-vacuity, minimal visibility and reusable abstractions. It fixed what was
local, filed what was not, and recorded how the review went, as the input to
the hygiene agent's design. It ran 2026-10-09 to 10-10 on one branch and draft
PR ([nsmith-/vibegraph#19](https://github.com/nsmith-/vibegraph/pull/19)).
The chronology, including two container restarts and every user decision, is
in [log.md](log.md).

## What was banked

**Visibility (V1, V1b).**
- `vibegraph-lib`'s `pub` lines went from 2601 to 1359, and both binary crates
  are crate-private.
- Dead types were deleted. 144 dead-code allows were replaced by `cfg` gating
  or deletion.
- Intra-doc links were restored, and every rustdoc build documents private
  items.
- Reports: [V1](sessions/V1-report.md), [V1b](sessions/V1b-report.md).

**Tooling (T1).** Five script and CI fixes ([report](sessions/T1-report.md)).

**Reviews (R-A to R-G2).** 176 findings over eight module clusters, read-only
([triage](triage.md)). The manager spot-checked 26 claims, and all
reproduced.

**Fixes.** Ten sessions fixed 125 triaged findings. At least 112 mutations
were run to show that tightened tests fail on the defect they guard, and 41
of 47 non-vacuity fixes are confirmed that way
([lessons](../../workflow/hygiene-review.md)). Real bugs fixed under the
stop-rule:
- `asin`/`acos` evaluated as `acsc`/`asec`;
- a Lorentz divisor silently dropped;
- `recompute` reviving a restriction-locked parameter;
- reweight cards refusing any name containing "scan";
- `make_anti` negating singlet and octet colour;
- a library reweight guard that could never fire (now a required argument,
  [F-C2](sessions/F-C2-report.md));
- the duplicated `SDE_strategy` predicate;
- the `tmin_for_channel` card, whose prescribed weight MadGraph itself
  computes from an unassigned value, now refused;
- the Drell–Yan σ gate's OR (an effective 1% bound);
- a stale RAMBO reference value.

Per-session reports: [F-A](sessions/F-A-report.md), [F-B](sessions/F-B-report.md),
[F-C](sessions/F-C-report.md), [F-C2](sessions/F-C2-report.md),
[F-D](sessions/F-D-report.md), [F-E](sessions/F-E-report.md),
[F-F](sessions/F-F-report.md), [F-CLI](sessions/F-CLI-report.md),
[F-G1](sessions/F-G1-report.md), [F-G2](sessions/F-G2-report.md).

**Unchanged physics.**
- The evaluator's emitted program is unchanged on 45 subprocesses.
- The VEGAS goldens and the parallel/sequential checks are bit-identical.
- The seed-headroom probe gives identical readings, over all 68 rows, at the
  pre-sprint commit and the sprint head.

**Census change.**
- The `smeftsim` and `toy_models` targets moved from the banked to the
  hermetic layer, as two new hermetic standalone rows.
- The hermetic suite went from 1348 passed (V1 baseline) to 1391.

**Lessons.** The draft [hygiene review procedure](../../workflow/hygiene-review.md):
- yield per point;
- which techniques worked;
- calibration;
- vacuity evidence;
- a proposed checklist and report shape for the hygiene agent;
- the operational lessons.

## Backlog

**Closed (files deleted):** 25 items. They are hygiene-sprint, the four
`dead-types…`/`evaluator…`/`ufo-asin…`/`make-anti…` items, and every other
claimed item except one; the list is in `sprint.md`, "Scope".
configuration-weights-wrong-at-sde1-with-tmin closed by a refusal, not by its
prescribed weight; [F-E's report](sessions/F-E-report.md) gives MadGraph's
source.

**Released:** acceptance-yml-fails-on-refdata-releases. The fix landed in
`a4db76f`, but its `closes_when` needs a published `refdata-*` release to
observe, and none was published during the sprint.

**Filed:** 54 new items (`190c13e`). They cover every triaged *file* finding
and every Found entry. Three are `needs-user`:
- the seed-combination rule;
- three-seed χ² gates;
- fixed-beam closed-form scales.

The highest-priority new item is sigma-seed-calibrations-drifted. It predates
the sprint.

## Gates on the final commit

- **fmt and clippy, both configurations:** clean.
- **Hermetic suite:** 37 suites, 1391 passed, 0 failed, 16 ignored (manager
  re-run at F-G2's head).
- **Full banked `validation/validate.sh`:** see the log entry that records it.
