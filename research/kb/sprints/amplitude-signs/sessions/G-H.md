---
type: Session Brief
title: "G-H: hygiene review of the t t~ + jet per-flow gate"
description: "Review PR #29's change for maintainability, test non-vacuity, visibility and reusable abstractions, write the findings first, then fix the local ones."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [G1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #29 makes against `main` (`git diff origin/main...HEAD`),
following [hygiene review](../../../workflow/hygiene-review.md). The code is
the five rows in `validation/madgraph/gen_standalone_jamps.py`, the five tests
and the comment in `vibegraph-lib/tests/standalone_jamps.rs`, and the
`color_cf_oracle.rs` comment. The [G1 report](G1-report.md) gives the intent.

Points the manager already knows; check them rather than re-find them:
- G1's falsifiers were temporary mutations. Whether the four-gluon contact
  permutation should become a permanent negative test (G1 Found 2) is a
  finding to judge, with its cost.
- Five near-identical `#[test]` functions: consider whether they read better
  table-driven, while keeping one test name per row in the output.

Physics findings go to the manager, not into fixes. Do not regenerate any
table and do not touch the σ gates.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test -p vibegraph-lib --test standalone_jamps --test color_cf_oracle`,
and `cargo test --workspace`.
