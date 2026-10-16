---
type: Session Brief
title: "B-H: hygiene review of the root-free convention sign, and the concepts it supersedes"
description: "Review PR #30's convention-sign change for maintainability, test non-vacuity, visibility and reusable abstractions; fix the local findings; and rewrite the five kb concepts that still describe the anchor convention."
status: draft
agent: hygiene-dev (Opus)
depends_on: [B]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Review the change PR #30 makes against `main`, following
[hygiene review](../../../workflow/hygiene-review.md). The code is B′'s
(`f1bb5bb`, `6f2df1e`):
- `helas/eval/root_diagram.rs`, `root_lorentz.rs`, `rooting_soundness.rs`,
  `stitching.rs`;
- the new `helas/eval/relabelling.rs`;
- the test changes in `amplitude_oracle.rs`, `standalone_jamps.rs` and
  `toy_models.rs`.

The [B′ report](B-report.md) gives the derivation, the mutations and the blind
spots. The evaluator is a hot module: any refactor must leave every
`standalone_jamps` and `amplitude_oracle` printed number unchanged (identity
check), and must not change the emitted program.

Points to check rather than re-find:
- `SEVEN_DIGIT_COUPLING_ROWS` is being replaced by a per-graph exemption on
  PR #33. Leave it alone.
- C-H's H13: the four `build_child` bilinear arms repeat the
  pick-the-wrapped-index logic. B′ removed their flips; judge the unification now.
- `rooting_soundness` now checks only kernels. Say what it still pins and
  whether its name and docs say so.

**This session is also assigned the concept rewrite.** It may edit these
`research/kb/` concepts, keeping each one's frontmatter valid:
`amplitudes/rooting-invariance-and-anchor.md`, `amplitudes/vector-vertex-signs.md`,
`amplitudes/fermion-line-sign.md` (its reversal account of `(−1)^P`),
`amplitudes/convention-sign-inventory.md` and
`validation/convention-channel-coverage.md` (it also cites removed test names).
Rewrite them to describe `convention_sign` as it is now, in their own terms,
with no history framing. Superseded content gets no narrative, and git keeps
the history. Rename a concept only if its title becomes false, and update the
inbound links if you do. Set each rewritten concept to `status: draft`; the
manager promotes them after review.

Gate: fmt, clippy (`--all-targets`, and `--all-features`), `cargo test --workspace`,
`pixi run -e kb kb-lint` (0 errors), and `pixi run --skip-deps validate` if
anything outside `#[cfg(test)]` changes.
