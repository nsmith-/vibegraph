---
type: Session Brief
title: "M-HY1: hygiene review of #86"
description: "Review the knot-only PDF layout, the table oracle, the bytes-per-member guard and the CMS fetch script for maintainability, test non-vacuity, visibility and reusable abstractions; write findings before editing; fix with mutation evidence."
status: draft
agent: hygiene-dev (Sonnet)
depends_on: [M-V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
---
Worktree `/home/user/wt/M`, branch `sprintf-pdf-memory` (PR #86). Start at
the commit that adds this brief. The change under review is
`git diff 86b8e5c..HEAD -- ':!research'`: M-P1's 428c4d8 and M-V1's 300674a.
Read the [M-P1](M-P1-report.md) and [M-V1](M-V1-report.md) reports and
`research/kb/workflow/hygiene-review.md`. Pixi is not installed. The crate
packages are `vibegraph-lib` and `vibegraph`. The 880 CMS members are in
`/home/user/cms-pdfs` (read only). This brief may be wrong: if the code says
otherwise, say so.

Points to look at, beyond your own review:
- `table_oracle.rs` carries a copy of the removed algorithm. Is it the
  smallest faithful copy, and is its provenance (86b8e5c) stated in its own
  terms?
- The `pdf_member_bytes` guard uses a counting allocator. Check two things:
  - Is it isolated from other tests' allocations? It is the only test in its
    binary.
  - Does its 10 bytes per knot bound still fail on a layout that keeps only
    one of the dropped tables? M-V1 showed only the full baseline failing.
- The visibility of `XCell`, `QCell` and any accessor widened for the
  oracle.
- `fetch_cms.sh` against `fetch_common.sh` conventions.

Physics or performance findings go in your report for the manager, not into
edits. Gates: fmt, clippy (default and extended features), the default
`cargo test`, `pdf::table_oracle` over the CMS tree after any edit to
`src/pdf`.

**Report.** `M-HY1-report.md`, findings written before edits, mutation
evidence for each test change. Commit as described in your agent definition.
Do not push.
