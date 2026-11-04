---
type: Session Brief
title: "G-HY1: hygiene review of #87"
description: "Review the lazy cache root, the mixed-flavour line mapping and the new CLI and banked tests for maintainability, test non-vacuity, visibility and reusable abstractions; findings before edits; fix with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [G-V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/G`, branch `sprintf-grid-worker` (PR #87). Start at
the commit that adds this brief. The change under review is
`git diff 86b8e5c..HEAD -- ':!research'`: G-F1's 10a3209, G-F2's e9041eb
and G-V1's 314c60a. Read the three reports in this folder and
`research/kb/workflow/hygiene-review.md`. Pixi is not installed. The crate
packages are `vibegraph-lib` and `vibegraph`. This brief may be wrong: if the
code says otherwise, say so.

Points to look at, beyond your own review:
- **G-F1's CLI tests** change uid to reach a lookup with no home directory,
  and skip when they may not. On an unprivileged CI runner they therefore
  pass without exercising the fix, and only the lib unit test covers it.
  Decide whether the skip is visible enough, or whether the CLI could reach
  the same path in a way CI exercises. The hygiene item
  `cache-root-resolved-two-ways` is related; do not widen into it.
- `NoCacheRoot`'s message hardcodes `$VIBEGRAPH_HOME` in the library while
  the constant lives in the CLI (G-F1's Found).
- G-F2's `carried_flavour` and the charge rule it falls back to: is the
  precedence documented in the function's own terms, and are both branches
  pinned by unit tests that fail when swapped?
- G-V1's weighted-share estimator: is it shared, or worth sharing, with the
  llj test? Do not promote the llj test; that is a filed item.

Physics or performance findings go in your report for the manager, not into
edits. Gates: fmt, clippy (default and extended features), the default
`cargo test`, and the banked `cli_generate_proton` and `cli_pdf_cache`
targets after any edit to them.

**Report.** `G-HY1-report.md`, findings written before edits, mutation
evidence for each test change. Commit as described in your agent definition.
Do not push.
