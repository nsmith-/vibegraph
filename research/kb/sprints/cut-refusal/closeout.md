---
type: Sprint Record
title: "Unimplemented cuts: refused early, and no-op values let through"
description: "An active unimplemented cut is refused from the run card alone, before diagram enumeration, naming the cut and value; single-object pT maxima at or above half the beam-energy sum read as their default."
closed: 2026-10-20
status: draft
tags: [cuts, runcard, refusal, cms, sprint-d, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: f1, resource: "sessions/F1-report.md", title: "F1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
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
