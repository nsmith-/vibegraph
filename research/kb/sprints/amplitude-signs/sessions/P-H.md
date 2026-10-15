---
type: Session Brief
title: "P-H: hygiene review of the probe–production bridge"
description: "Review PR #31's change for maintainability, test non-vacuity, visibility and reusable abstractions, write the findings first, then fix the local ones."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [P1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #31 makes against `main` (`git diff origin/main...HEAD`),
following [hygiene review](../../../workflow/hygiene-review.md). The code is
`vibegraph-lib/src/helas/eval/run/probe_bridge.rs` and its declaration in
`run.rs`. The [P1 report](P1-report.md) gives the intent and the measured
falsifiers.

Points the manager already knows; check them rather than re-find them:
- P1 says `test_whole_amplitude_equals_diagram_sum_eemumu` is subsumed by the
  bridge's first row. Confirm that the bridge checks everything the old test
  checks, and delete the old test only if it does.
- The bridge is about 400 lines in one test. Judge whether its helpers read
  clearly, and whether any duplicates an existing helper in `run.rs`.

The evaluator (`helas/eval`) is a hot module: change nothing outside
`#[cfg(test)]` code. Physics findings go to the manager.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test -p vibegraph-lib --lib probe_bridge -- --nocapture`, and
`cargo test --workspace`.
