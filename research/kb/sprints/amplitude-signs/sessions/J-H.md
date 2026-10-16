---
type: Session Brief
title: "J-H: hygiene review of the jioxxx chirality and longitudinal test"
description: "Review PR #34's change for maintainability, test non-vacuity, visibility and reusable abstractions, write the findings first, then fix the local ones."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [J1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #34 makes against `main` (`git diff origin/main...HEAD`),
following [hygiene review](../../../workflow/hygiene-review.md). The code is
`test_eval_jioxxx` and its helpers in `vibegraph-lib/src/helas/eval/run.rs`,
and the HELAS reference port `jioxxx` in `vibegraph-lib/src/helas/vertex.rs`.
The [J1 report](J1-report.md) gives the intent and the measured mutations.

Points the manager already knows; check them rather than re-find them:
- The test grew by about 250 lines. Judge whether the massive and massless
  regimes share their setup cleanly, and whether its ALOHA transcriptions
  (`ffv2_3`, `ffv4_3`) duplicate anything already in the tree.
- J1's mutation M5 (`chiral_weighted`) passes this test and is caught only by
  other `helas::` tests. That is a stated blind spot, not a finding to fix here.

Change nothing outside `#[cfg(test)]` code and the test-only reference port.
The production kernels stay as they are. Physics findings go to the manager.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test -p vibegraph-lib --lib helas::`, and `cargo test --workspace`.
