---
type: Sprint Record
title: "A tolerance for the MLM Pythia comparison"
description: "The matched MLM comparison through Pythia is a gate: five |pull| < 3 tests plus jet-rate χ² and per-bin legs under look-elsewhere at 0.5 % each, enforced only while MadEvent's own null passes; its first recorded run passes."
closed: 2026-10-17
status: draft
tags: [mlm, pythia, matching, gate-promotion, look-elsewhere, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: d1, resource: "decisions/D1-tolerance.md", title: "D1 and its amendment"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: hy1, resource: "sessions/HY1-report.md", title: "HY1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/52", title: "nsmith-/vibegraph#52"}
---
One PR ([nsmith-/vibegraph#52](https://github.com/nsmith-/vibegraph/pull/52))
closing `mlm-pythia-comparison-no-tolerance`.
- **The decision:** [D1](decisions/D1-tolerance.md), proposed by the manager.
  The user approved it with an amendment that adds the per-bin leg. The
  manager chose α = 0.5 % per leg.
- **Sessions:** [V1](sessions/V1-report.md) (validation-dev, Opus) and
  [HY1](sessions/HY1-report.md) (hygiene-dev, Sonnet).

## What was established

- **The gate.** The rule is in
  [the gate concept](../../validation/mlm-pythia-matched-comparison.md#the-gate-decided-2026-10-17).
  The driver applies it and writes a collator row. The `mlm-pythia` standalone
  row is `mode = "gate"`, which the collator checks against the row file.
- **The first recorded run passes** (2026-10-17, linux-64 Pythia 8.312).
  - vibegraph against MadEvent: pulls −0.76 / −1.79 / −1.99 / −1.38 / −0.39;
    χ² leg min p × 3 = 0.26; per-bin max |pull| 2.91 against z* = 3.98 at
    72 bins.
  - MadEvent's null passes: 0.19, and 2.93 against 3.97 at 69 bins.
- **Pythia results depend on the build.** The same files and seeds moved by
  up to 1.7 shower-σ between the M3 record and this host. The row now records
  the invocation's build.
- **The verdict logic is pinned.** A scripted mutation batch found 16 of 41
  mutants missed by V1's tests. With HY1's tests (30 Python, 11 + 38 Rust),
  43 of 46 are caught; the remaining three are two equivalent `<` / `≤` and
  the exit code, covered by a negative control.

## Trust

Machine-confirmed by the manager:
- the 16, then 30, driver tests;
- the cached recorded run, twice: `verdict: PASS` with D1's numbers, and with
  the build recorded after HY1;
- one HY1 mutation re-run (the per-bin trial factor halved), which fails
  `test_per_bin_leg_threshold_is_z_star_at_the_total_bin_count`;
- the commits.

The 410 Pythia runs, the sample generation and the other mutations are the
sessions' own.

## Filed

- [mlm-pythia-at1-files-scatter](../../backlog/validation/mlm-pythia-at1-files-scatter.md)
- [mlm-match-py-statistics-follow-ups](../../backlog/hygiene/mlm-match-py-statistics-follow-ups.md)
- [mlm-pythia-trial-factors-uncalibrated](../../backlog/validation/mlm-pythia-trial-factors-uncalibrated.md)
