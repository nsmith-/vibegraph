---
type: Sprint Record
title: "gg_to_gg_cg σ offset"
description: "The −0.22 % offset is the banked reference being one high MadEvent draw; this side's quadrature agrees with fresh MadEvent seeds at 10× budget to 6e-6. The re-bank goes with the reference-scatter work."
closed: 2026-10-15
status: draft
tags: [sigma, madevent-reference, smeftsim, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: v1, resource: "sessions/V1-report.md", title: "V1 report"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/27", title: "nsmith-/vibegraph#27"}
---
One PR ([nsmith-/vibegraph#27](https://github.com/nsmith-/vibegraph/pull/27))
closing `gg-to-gg-cg-sigma-offset`. Chronology in [log.md](log.md).

## What was established

| estimate | σ (pb) |
|---|---|
| banked reference (one MadEvent run, iseed 21) | 271240 ± 230 |
| MadEvent, 20 seeds at the banked budget | 270654 ± 61 |
| MadEvent, 8 fresh directories at 10× budget | 270652.5 ± 23 |
| this side, deterministic quadrature of the production integrand | 270650.9 (converged to about 1e-5) |

- **No defect on either side.** The bank is a +2.1-sd draw. |M|² agrees with
  MadGraph standalone to 2.2e-15 across αs ∈ [0.08, 0.20], and a fixed-scale
  control agrees.
- **The frame does not matter here.** At fixed symmetric beams the lab and the
  partonic CM coincide, and choice 3 is boost-invariant in any case.
- **The cell stays `info`** until the row is re-banked as ≥ 10 fresh-directory
  seeds. That re-bank belongs with every other single-draw σ reference, in
  [reference-sigma-seed-scatter-untested](../../backlog/validation/reference-sigma-seed-scatter-untested.md)
  (PR #25). Expected re-banked value: 2.7065e5.

## Trust

V1 is machine-confirmed by the manager: the banner seed; the seed means
recomputed from the raw per-seed file; the quadrature channel sum. The slice
table and the |M|² comparison are the session's.

## Filed

- [madevent-windowed-runs-split-low](../../backlog/validation/madevent-windowed-runs-split-low.md)
- [madevent-heavy-tail-seed-frequency](../../backlog/validation/madevent-heavy-tail-seed-frequency.md)
