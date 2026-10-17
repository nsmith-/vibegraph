---
type: Backlog Item
title: "Six χ² p-value prints show an underflowed tail as 0"
description: "Test-log prints of χ² p-values in the samples and decay-chain gates show 0.000e0 below f64's range; report cells and seed-record already print a bound, through two copies of the same rule."
area: hygiene
state: open
priority: low
closes_when: "Every printed χ² p-value goes through one shared formatter that prints an exact 0 as a bound, or the duplication is recorded as intended."
blocked_by: []
opened: 2026-10-16
tags: [chi2, p-value, formatting, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: hy1, resource: "../../sprints/seed-record-tooling/sessions/HY1-report.md", title: "seed-record-tooling HY1 report, H7 and H8"}
---
`statrs`'s upper incomplete gamma underflows to exactly 0 only below about
1e-308, so no verdict depends on this. These sites still print such a p as a
value: `vibegraph-lib/tests/validate_samples.rs` ~:880 and ~:906,
`vibegraph-cli/tests/validate_samples_proton.rs` ~:508 and ~:528, and
`vibegraph-cli/tests/cli_decay_chain_events.rs` ~:500 and ~:508. The rule
"exact 0 prints `< 1e-300`" exists twice, as `p_text` in `seed-record` and
`cells::pval` in the collator, two binaries of one crate without a lib
target ([HY1](../../sprints/seed-record-tooling/sessions/HY1-report.md), H7
and H8). A shared formatter in `vibegraph-lib::stats` would serve the six
sites.
