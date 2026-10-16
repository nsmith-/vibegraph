---
type: Session Brief
title: "Q-H: hygiene review of the same-flavour four-fermion per-graph gate"
description: "Review PR #35's change for maintainability, test non-vacuity, visibility and reusable abstractions, write the findings first, then fix the local ones."
status: draft
agent: hygiene-dev (Opus)
depends_on: [Q1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #35 makes against `main` (`git diff origin/main...HEAD`),
following [hygiene review](../../../workflow/hygiene-review.md). The code is
the `bank_amps` option and the `ee_to_ee_4f` row in
`validation/madgraph/gen_standalone_jamps.py`, and the per-graph level
(`MG_GRAPH_DIAGRAMS` and its comparison) in `vibegraph-lib/tests/standalone_jamps.rs`.
The [Q1 report](Q1-report.md) gives the intent, the falsifier and the blind spots.

This per-graph route is now reused by two other open PRs (the five-vector and
Identity-sign work merge this branch), so its shape matters beyond this row.
Judge it as shared machinery: whether the `MG_GRAPH_DIAGRAMS` declaration and
the per-graph comparison read clearly for a second and third row, and whether
the `matrix.f` parsing of JAMP coefficients in `bank_amps` is robust or merely
fits this process.

The doc comment on the `ee_to_ee_4f` entry carries a counting-convention
statement that awaits the user's approval. Leave its wording alone.

Physics findings go to the manager. Do not regenerate the table.

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test -p vibegraph-lib --test standalone_jamps -- --nocapture`, and
`cargo test --workspace`.
