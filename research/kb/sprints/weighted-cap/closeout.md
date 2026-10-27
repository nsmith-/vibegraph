---
type: Sprint Record
title: "The automatic WEIGHTED search reaches MadGraph's cap"
description: "The search runs MadGraph's loop: start at get_minimum_WEIGHTED, step by one, cap at (n_ext−2) × the largest interaction WEIGHTED, no search for a top-level decay; four MadGraph rows pin counts and the bound each search accepts."
closed: 2026-10-21
status: draft
tags: [diagrams, coupling-orders, weighted, madgraph-parity, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: v2, resource: "sessions/V2-report.md", title: "V2 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/64", title: "nsmith-/vibegraph#64"}
---
One PR ([nsmith-/vibegraph#64](https://github.com/nsmith-/vibegraph/pull/64),
merged 2026-10-21) closing `auto-weighted-search-cap-uses-hierarchy`, which left
four CMS BSM cards with no diagrams. Sessions: [F1](sessions/F1-report.md) and
[V1](sessions/V1-report.md) (Opus), [HY1](sessions/HY1-report.md) and
[V2](sessions/V2-report.md) (Sonnet). The M3 banked four rows.

## What was established

- **MadGraph's loop, element by element** (`b7687064`). The per-interaction
  WEIGHTED is divided by `max(n_particles − 2, 1)` (`base_objects.py:893-899`);
  the cap is `(n_ext − 2)` × the largest such value; the search starts at
  `get_minimum_WEIGHTED`, steps by one, and returns at the first trial with
  diagrams (`diagram_generation.py:2013-2115`); a one-particle initial state that
  is not a decay chain runs no search (`:1978-1980`). *Manager-verified:* those
  lines read at the pinned commit.
- **Corrections to the brief.** The SM cap does not rise (per-vertex WEIGHTED
  counts per equivalent cubic vertex, so the SM maximum stays 2). The 1→n rule
  changes top-level decays without orders: `z > u u~ d d~` goes from 4 to 17
  diagrams. No decay the suite computes moves (all are single-order), and the
  search's bound agrees with MadGraph's recorded `WEIGHTED<=N` on all 109
  committed cards that generate (V1).
- **The bound, not only the count.** The M3's build logs showed `Trying` lines
  4,5 / 6,7,8 / 4 / none on the four rows, where V1 had predicted 4,5 / 6,7 /
  none / none. V2 read why: MadGraph prints the line before each trial, so a
  successful trial is a printed line. `the_search_accepts_the_bound_madgraph_does`
  pins each row's sequence from the loop's own log. Capping row 2 at 8 or stepping
  by 2 keeps every count test green and fails only this test (V2's mutations).
  *Manager-verified:* `diagram_generation.py:2013-2014, 2104-2107, 2115`;
  `weighted_search` 8 passed.
- **The rows** (banked `67ee2c2`): three on a toy UFO whose hierarchies differ
  and SM `z > u u~ d d~`, with 2, 6, 4 and 17 diagrams, gated by
  `validate_madgraph_diagrams` (61 passed locally, *manager-verified*).
- **A pre-existing gap, confirmed.** MadGraph drops final-state gluons from each
  trial (`diagram_generation.py:2040-2059`), so on SMEFTsim's SM limit it refuses
  `e+ e- > z g g` at `WEIGHTED<=6`, where this crate returns 1 diagram at 107.
  The old loop behaved the same; the M3 confirmed MadGraph's
  `NoDiagramException`. It is pinned as an ignored test.

## Backlog

- Deleted: `auto-weighted-search-cap-uses-hierarchy`.
- Filed during the sprint: `weighted-search-gluon-free-trial-missing` (F1 Found
  1, V1 Found 1), `coupling-orders-fallback-differs-from-madgraph` (F1 Found 2,
  V1 Found 2), `diagram-count-test-helper-duplicated` (HY1 Found 1), and a dated
  note on `weighted-order-computed-three-ways` naming the fourth site (F1 Found 3,
  HY1 Found 2).
- Drafted for filing at close-out: `weighted-search-bound-read-from-debug-log`
  (V2 Found 2 and 3, with the private subscriber's callsite-interest race).
- Proposed edit: `weighted-search-gluon-free-trial-missing`'s last
  `closes_when` clause (a MadGraph run confirming the refusal) is met by the
  M3's run on #64.
- Not filed: V1 Found 3 (MadGraph's trial drops polarizations, moving only the
  bound of a card refused on both sides), V2 Found 1 (V1 §4's superseded trial
  table) and V2 Found 4 (shared-scratchpad file names): nothing to change.

## Process notes

The diagram counts matched on the first M3 build while two predicted trial
sequences did not; the manager held the commit until the loop's accepted bound
was pinned, since a count cannot tell a stop at 8 from generation at the cap 9.
