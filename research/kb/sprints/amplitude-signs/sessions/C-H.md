---
type: Session Brief
title: "C-H: hygiene review of the Identity and derivative-scalar sign fixes"
description: "Review PR #32's change for maintainability, test non-vacuity, visibility and reusable abstractions, write the findings first, then fix the local ones."
status: draft
agent: hygiene-dev (Opus)
depends_on: [C2]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #32 makes against `main` (`git diff origin/main...HEAD`),
following [hygiene review](../../../workflow/hygiene-review.md). The code is
C1's standalone rows, toy-UFO vertex and card, and C2's production change:
- the two sign arms in `helas/eval/root_lorentz.rs`;
- the mixed-sign refusal (`uniform_sign`, `RootLorentzError::UnsupportedVertex`) in
  `helas/eval/diagram_eval.rs`;
- `root_diagram.rs`;
- the `toy_models.rs` split-sum test.

The [C1](C1-report.md) and [C2](C2-report.md) reports give the intent.

The evaluator is a hot module, and a follow-up session restructures these same
role-sign functions. Prefer local, clearly commented code over new abstractions
here, and file anything larger. Physics findings go to the manager.

Points to check rather than re-find:
- C2 Found 1, the `tensor()` asserts that still panic, is already filed
  (`tensor-uniformity-asserts-panic`).
- Does the mixed-sign refusal's error name enough (vertex, structures,
  coupling order) for a user with an arbitrary UFO to act on it?

Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace`, and `pixi run --skip-deps validate` if anything outside
`#[cfg(test)]` changes.
