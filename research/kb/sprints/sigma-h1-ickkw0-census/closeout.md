---
type: Sprint Record
title: "H1 census of banked grouped rows"
description: "Every banked grouped MadEvent row checked for the permuted first call: only pp_to_ll_0j2j_mlm is measurably affected, the rest are identity, immune or negligible; no re-bank needed."
closed: 2026-10-15
status: draft
tags: [h1, madevent-reference, scales, grouping, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/24", title: "nsmith-/vibegraph#24"}
---
One PR ([nsmith-/vibegraph#24](https://github.com/nsmith-/vibegraph/pull/24))
closing `h1-ickkw0-grouped-rows-unchecked`. Chronology in [log.md](log.md).

## What was established

- **Exposure criterion, corrected.** The item's "`dynamical_scale_choice = -1`
  or matching" was wrong in both directions. The first call also clusters at
  `xqcut > 0` with `ickkw = 0`. It does not cluster at `-1` with all three
  scales fixed. And it can move σ only through PDFs, an α_s coupling, or a
  rejection.
- **Census.** All 89 banked runs were read. Every run is 3.7.1, apart from the
  six deliberate 3.5.7 e+e- runs, which H1 cannot reach. Each exposed row was
  regenerated at the pin through `output`, with the R1 reproducer as a
  positive control that read correctly first. The regenerated directories
  match the bundle's.
- **Same-seed screen** (unpatched against patched, 2000 events): the 2 → 2
  rows and `pp_ttx_lep_dyn` are identical line for line. `uu_full` and `uu_tt`
  move by ≤ 3e-5 of σ. R1 moves +2.1 %. With #23's paired measurement on
  `pp_to_ttx_0j1j_mlm` `@1` (+0.03 ± 0.11 pb), every banked row is now
  classified. The table is in
  [the permuted first call](../../validation/madgraph-permuted-first-call.md#census-of-the-banked-rows).
- **Vectorised path.** It has the same defect, but no banked run takes it.

No reference needed re-banking, so nothing was requested of the M3 host.

## What landed

- `validation/madgraph/census_permuted_first_call.py`. HY1 made it fail loudly
  on missing or malformed inputs: five synthetic fixtures that exited 0 now
  exit 1, and its `runs` output is byte-identical. It shares
  `extract_sigma.parse_run_card_scalars` (3875 parameters identical across
  every banked card).
- The [permuted first call](../../validation/madgraph-permuted-first-call.md)
  and [defect policy](../../validation/madgraph-defect-policy.md) concepts
  record the census.

## Trust

- V1 and HY1 are machine-confirmed by the manager. The checks: the commits,
  the raw screen summary, the 49-row permutation table, `runs` re-run and
  byte-identical to V1's output, and a bad path exiting 1.
- The per-row screen numbers themselves are the agent's measurements, read
  from the summary file and not re-run.

## Filed

- [validate-scales-replay-blind-to-h1](../../backlog/validation/validate-scales-replay-blind-to-h1.md)
- [census-permuted-first-call-no-fixture](../../backlog/hygiene/census-permuted-first-call-no-fixture.md)

Not filed: handing the first call `P1` shifts written p_z and E at about 1e-9
on mirrored events (worst 3.4e-5). It only affects a patched MadEvent's event
record, and it is cosmetic.
