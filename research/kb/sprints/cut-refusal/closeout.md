---
type: Sprint Record
title: "Unimplemented cuts: refused early, and no-op values let through"
description: "An active unimplemented cut is refused from the run card alone, before diagram enumeration, naming the cut and value; single-object pT maxima at or above half the beam-energy sum read as their default."
closed: 2026-10-21
status: draft
tags: [cuts, runcard, refusal, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: f2, resource: "sessions/F2-report.md", title: "F2 report"}
  - {id: hy2, resource: "sessions/HY2-report.md", title: "HY2 report"}
  - {id: f3, resource: "sessions/F3-report.md", title: "F3 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/59", title: "nsmith-/vibegraph#59"}
---
One PR ([nsmith-/vibegraph#59](https://github.com/nsmith-/vibegraph/pull/59))
closing `unimplemented-cut-refused-after-enumeration` and
`no-op-cut-values-refused-as-active`. Sessions: [F1](sessions/F1-report.md)
(feature-dev, Sonnet) and [HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **One check, two callers.** `cuts::detect_unimplemented` reads the run card
  alone. The CLI's run-card load (shared by `integrate` and `generate`, on
  decay, fixed-energy and proton paths) calls it before model-dependent work
  reaches enumeration; `Cuts::compile_with` still calls it for library callers.
  The message names the cut and its value, with no flavour-group prefix.
- **What "before enumeration" is pinned by.** The order of two failures on a
  process that cannot enumerate (`e+ e- > mu+ mu- QED=0`): with the cut, the
  cut refusal; without it, the enumeration error. No test sees elapsed time or
  whether the check precedes model loading.
- **The no-op rule.** In the partonic rest frame a single object's momentum,
  or a final-state subset's vector sum, is at most √ŝ/2, and pT is invariant
  under the longitudinal boost. The nine names it covers are each a
  single-object pT or a subset's vector sum in MadGraph's `cuts.f`
  (`missetmax`: the neutrinos' vector sum, `cuts.f:347-372`; `ptj<n>max`,
  `ptl<n>max`: the n-th hardest jet or lepton, `:853-855`, `:962` at
  b7687064). The ceiling is half of `ebeam1 + ebeam2`, the generator's own
  √s convention (only looser than √s/2 for unequal beams). HT-sum maxima stay
  refused here; their no-op rule belongs with the HT cuts.
- **Tests that can see the list.** The tests name the nine fields literally,
  and a second test checks every other unimplemented cut at `1e5` stays
  refused, so the list can neither shrink nor grow unnoticed.

The manager re-ran the new CLI tests (13 in `cli_hard_errors`) and the 54
`cuts::` unit tests from the committed tree.

## Backlog

- Deleted: `unimplemented-cut-refused-after-enumeration`,
  `no-op-cut-values-refused-as-active`.
- Filed from Found: `collider-energy-sum-of-beam-energies` (validation),
  `cli-test-helpers-copied-per-file` and `generate-reads-run-card-after-artifact`
  (hygiene).
- HT-sum maxima (`htjmax`, `ihtmax`, `ht2max`…`ht4max`) at or above √s: not
  filed separately; the HT-cuts PR of the same sprint implements those cuts
  outright.
- `release-scope.md`'s refusal table now names the early call and the no-op rule.

## Process notes

F1 once used `pkill -f`, killing only its own wrapper; later briefs forbid it.
HY1 waited on its mutation log with a `sleep` loop, against the brief.

## Widened at the user's request (2026-10-20/21)

The user asked on the PR for three items filed from it to be fixed here.

- **`generate-reads-run-card-after-artifact`** (closed, [F2](sessions/F2-report.md)):
  `generate` loads the run card before the artifact, so a run-card refusal
  comes first in both commands; a test pins the order and fails both if the
  order is reversed and if the card goes unchecked.
- **`cli-test-helpers-copied-per-file`** (closed, F2, [HY2](sessions/HY2-report.md),
  [F3](sessions/F3-report.md)): every CLI test file invokes the binary and
  reads its output through `vibegraph-cli/tests/common`; test lists per binary
  identical before and after. Seven binaries use the ambient-environment
  helper (filed: `cli-tests-on-ambient-environment`).
- **`collider-energy-sum-of-beam-energies`** (moved): F2 found the fixed-energy
  path already exact and the whole proton frame chain equal-beam only
  (`build_frames` places partons at x_i·√s/2 and boosts by (x1−x2)/(x1+x2)).
  By the user's choice it moved to its own PR,
  [nsmith-/vibegraph#67](https://github.com/nsmith-/vibegraph/pull/67), stacked
  on this branch; the item's text now describes the whole chain.
- **Also filed:** `fixed-beam-written-event-frame-unsettled` (validation),
  with the CERN host's measurement and a candidate MadGraph defect entry
  (the written frame decided by a one-ulp fallback in `unwgt.f`).

The manager re-ran the touched CLI binaries from the committed tree after
each session (151 default CLI tests at F3).
