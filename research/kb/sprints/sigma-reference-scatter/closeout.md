---
type: Sprint Record
title: "Reference σ seed scatter"
description: "Every gated σ reference is now five fresh-directory MadEvent seeds plus two at 4×, banked with hard_survey = 2 and read with its measured scatter; the llj MLM offset was the reference's."
closed: 2026-10-16
status: draft
tags: [sigma, madevent-reference, seeds, refdata, mlm, llj, sprint-record]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: sprint, resource: "sprint.md", title: "Sprint overview"}
  - {id: log, resource: "log.md", title: "Sprint log"}
  - {id: d1, resource: "decisions/D1-reference-error-design.md", title: "D1: reference error design"}
  - {id: pr, resource: "https://github.com/nsmith-/vibegraph/pull/25", title: "nsmith-/vibegraph#25"}
---
One PR ([nsmith-/vibegraph#25](https://github.com/nsmith-/vibegraph/pull/25))
closing `reference-sigma-seed-scatter-untested` and
`llj-mlm-rows-sit-high-together`. Chronology in [log.md](log.md); the design
and its four amendments in [D1](decisions/D1-reference-error-design.md).

## What was established

- **The references.** refdata-10 banks every gated σ row (55) as five
  fresh-directory MadEvent seeds at its budget plus two at 4×, all with
  `hard_survey = 2`. Each record keeps per seed the reported σ, the
  equal-weight recombination, every channel's iterations, the survey the run
  made and its wall time. A σ gate reads the seed-policy mean with error
  max(quoted, spread/√n); the equal-weight value gates only where an
  adaptation-free check validated it.
- **The screens.** The 4× seeds against the 1× mean (one row held,
  `ll_to_qqx_toy_dipole`, whose 20 + 20 ladder then found no shift); the
  effective survey rate (no row inert); the look-elsewhere scatter rule (53
  of 55 within chance; the two flagged rows are long-tier `info`).
- **The gates.** The first banked run on the records is green, 47 σ pulls
  with max |pull| 2.45. `ee_to_mumua`'s +1.04 % offset is gone (−0.074 %,
  pull −0.42): it was the old single-run default-survey bank.
- **llj MLM.** The three rows' shared high reading was on the reference side:
  `hard_survey = 2` closes `xqcut_only` over 20 fresh seeds (−0.005 %), and
  equal weights close the two `rewgt` rows (−0.07 %, −0.05 %). Against the
  refdata-10 records they read +0.14 %, +0.11 %, +0.31 %, each within 1.4σ
  of a reference whose five seeds are coarser than the 20-seed diagnostics.
- **MadEvent timing.** At `nb_core = 12` MadEvent's `cumulated_time` is
  unchanged (geomean 0.98 over 31 processes), so the M3 speed ratios against
  it were not inflated by its 16-job layout.

## Trust

Machine-confirmed by the manager: every session's commits and trailers; the
gate and mutation logs; the refdata-10 archive hash; the rescreen of all 55
records; the `cumulated_time` ratios and the llj offsets, recomputed from the
records and the banked `results.dat`; the ladder σ from the user's table.
The banking itself, the ladder runs and the timing pass are the user's, on
the M3. The CI report is the run's own.

## Left open

- The σ pulls lean high as a population (+0.65 mean over 47 rows), in the
  direction of MadEvent's iteration-combination bias: filed below.
- `ee_to_mumua`'s pull is still reported, not asserted, and its manifest note
  describes the old offset; that change belongs to PR #26.
- The records in refdata-10 carry the pre-F3 `heavy_tail` summary; the reader
  ignores it, and `seed-record --rescreen` gives the verdict of record.

## Filed

- [madevent-generators-accept-empty-results](../../backlog/validation/madevent-generators-accept-empty-results.md)
- [cli-integrate-dy13-skips-seed-record](../../backlog/validation/cli-integrate-dy13-skips-seed-record.md)
- [pta-windows-need-full-mg-work-area](../../backlog/validation/pta-windows-need-full-mg-work-area.md)
- [sigma-pulls-lean-high-on-seeded-refs](../../backlog/validation/sigma-pulls-lean-high-on-seeded-refs.md)
- [hadronic-sigma-cells-unnamed-reference](../../backlog/hygiene/hadronic-sigma-cells-unnamed-reference.md)
- [madevent-shell-refusals-untested](../../backlog/hygiene/madevent-shell-refusals-untested.md)
- [lib-tests-hand-roll-temp-dirs](../../backlog/hygiene/lib-tests-hand-roll-temp-dirs.md)
- [madgraph-file-parsers-duplicated](../../backlog/hygiene/madgraph-file-parsers-duplicated.md)
- [seed-record-ladder-verdict-and-tie-break](../../backlog/hygiene/seed-record-ladder-verdict-and-tie-break.md)
- [chi2-p-value-floors-at-zero](../../backlog/hygiene/chi2-p-value-floors-at-zero.md)

Updated: [madevent-cumulated-time-host-dependence](../../backlog/performance/madevent-cumulated-time-host-dependence.md)
now asks only where a MadEvent job's non-`MATRIX1` CPU goes.
