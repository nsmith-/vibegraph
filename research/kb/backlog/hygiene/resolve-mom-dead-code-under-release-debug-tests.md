---
type: Backlog Item
title: NodeAnalysis::resolve_mom warns "never used" in release-debug test builds
description: Building vibegraph-lib's tests under --profile release-debug warns that NodeAnalysis::resolve_mom (helas/eval/analysis.rs:208) is never used; the default test profile does not show it.
area: hygiene
state: open
priority: low
closes_when: "cargo test -p vibegraph-lib --profile release-debug --no-run builds without the dead-code warning, by a cfg on the function or by removing it if nothing needs it."
blocked_by: []
opened: 2026-10-20
tags: [lint, dead-code, build-profile]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: m1, resource: "../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1 report, Found"}
---
Seen by the M1 measurement session on tree `b0a9312`; it predates that session.
CI's clippy gate builds the default profile and does not see it, which is why
it has survived. Decide whether the function has a caller outside the profiles
that warn: gate it with a matching `cfg`, or remove it.

Detail: [M1 report, Found](../../sprints/vegas-absolute-grid-coords/sessions/M1-report.md).

Seen again four times. It was filed twice more as duplicates, which are now
folded in here:
- K-C2 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C2-report.md)), and K-H4 (#158) under `cargo check --release`;
- L-Z8 ([report](../../sprints/v023-orders-majorana-timeouts/sessions/L-Z8-report.md));
- L-I1, sprint L's interference-sign probe.

The warning shows in every release-profile build, not just the release-debug
test build.
