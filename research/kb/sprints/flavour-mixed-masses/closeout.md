---
type: Sprint Record
title: "A flavour group with mixed outgoing masses is split, not refused"
description: "A card whose subprocesses differ in outgoing masses splits into one proton integrand per mass set, summed like multiplicities, as MadGraph writes one P directory each; p p > l+ l- with l = e, ta gates against MadGraph on both parts."
closed: 2026-10-26
status: draft
tags: [proton, flavour-groups, multiplicity, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: maps, resource: "map-changes-for-the-maps-session.md", title: "Map changes for the maps session"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/72", title: "nsmith-/vibegraph#72"}
---
One PR ([nsmith-/vibegraph#72](https://github.com/nsmith-/vibegraph/pull/72),
merged 2026-10-26) closing `flavour-group-mixed-outgoing-masses-refused`, which
blocked two surveyed CMS LO cards. Sessions: [F1](sessions/F1-report.md)
(feature-dev, Opus), [V1](sessions/V1-report.md) and [V2](sessions/V2-report.md)
(validation-dev, Sonnet), [HY1](sessions/HY1-report.md) (hygiene-dev, Opus). The
M3 banked the row. By the user's scope decision (2026-10-22) no phase-space map
code was edited; [the maps note](map-changes-for-the-maps-session.md) records
that none is needed.

## What was established

- **MadGraph's grouping.** MadGraph keys subprocess classes on every leg's mass
  (`group_subprocs.py:482-490`), so `p p > l+ l-` with `l = e, ta` becomes two P
  directories. This crate now splits such a card before flavour grouping
  (`multiplicity::split_by_outgoing_masses`): one `ProtonIntegrand` per mass set,
  each with its own channels on its own mass shell, combined by
  `MultiplicitySum`. Integrate, generate and the LHE writer all use the split.
  MadGraph keys on the mass parameter's name, this crate on its value; they
  differ only where distinct parameters share a value, where merging is exact.
  *Manager-verified:* the split's design against `group_subprocs.py`, and that no
  map code changed.
- **A pre-registered prediction.** Before the bank, F1 measured five seeds at
  120000 × 12: e part 934.541 ± 0.616 pb (pull +1.68 against the banked
  `dy13_default`, the same process and card), τ part 923.778 ± 0.610, total
  1858.318 ± 0.867.
- **The gate** (`pp_to_ll_etau`, banked on the M3 with MadGraph 3.7.1). MadGraph
  wrote `P1_qq_ll` and `P1_qq_taptam`; each directory's per-seed sum agrees with
  `results.dat` to ≤ 0.04 pb, so the gate reads the right quantity. On the M3's
  run (quoted, not reproduced here):

  | part | vibegraph | MadGraph | pull |
  |---|---|---|---|
  | e | 934.520 ± 0.616 pb | 933.580 ± 0.255 pb | +1.41 |
  | τ | 923.762 ± 0.610 pb | 923.374 ± 0.262 pb | +0.58 |
  | total | 1858.282 ± 0.867 pb | 1856.940 ± 0.365 pb | +1.43 |

  *Known-wrong control:* the e part read against the τ directory misses by pull
  +16.7, rel +1.21 % (the manifest's earlier "−1.15 %" was τ relative to e).
  The directory names were assumed until the bank: MadGraph's massless group is
  `ll`, not `epem`, and V2 classifies on the label.
- **Two red tests at the branch tip, fixed by HY1.** Unplanning the row
  (`ee6f18c`) turned the collator's planned-row test into a no-op and left the
  mixed-mass gate failing on any checkout without the record. HY1 also added the
  scale assertions the neighbouring gate makes and a `should_panic` test for the
  one-per-phase-space invariant.

## Backlog

- Deleted: `flavour-group-mixed-outgoing-masses-refused`.
- Drafted for filing at close-out: `fixed-energy-path-refuses-multi-flavour-card`
  (F1 Found 1), `multiplicity-sum-construction-copied` (HY1 Found 1),
  `kb-concepts-stale-after-sprint-d` (HY1 Found 3, 4, with F1 Found 2),
  `time-stages-keeps-its-own-row-card-map` (V1 Found 2),
  `absent-run-escapes-spelled-per-gate` (V1 Found 4),
  `mixed-mass-directories-classified-by-label` (V2 Found 1),
  `mixed-mass-control-borrows-the-e-part` (V2 Found 2),
  `collator-has-no-control-cell-kind` (V2 Found 3).
- Proposed edits, not new items: `sigma-gate-scaffold-hand-copied` gains HY1
  Found 2's sites; `stochastic-rounding-refuses-mixed-multiplicity` needs its
  `closes_when` per part, not per `@N` (HY1 Found 5).
- Not filed:
  - F1 Found 3 (`resolve_mom` unused in a release build): already
    `resolve-mom-dead-code-under-release-debug-tests`;
  - F1 Found 4 (the survey names no card): the item closed on the banked row
    instead;
  - V1 Found 1 (the positional `bank-seeded-sigma` in a request): the M3 used
    `ROWS=`, and the request is spent;
  - V1 Found 3, 5 (directory names, the per-directory quantity): settled by the
    bank, as above.

## Process notes

The map-code boundary held: the split needs no map change because each part is
its own integrand. The sprint's merge order (#67, #72, #74, #60) left this PR's
row `bundled = false` until #60 pinned refdata-11.
