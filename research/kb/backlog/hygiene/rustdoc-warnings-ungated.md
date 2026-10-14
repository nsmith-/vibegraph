---
type: Backlog Item
title: "Nine rustdoc warnings remain and no CI step denies them"
description: "8 vibegraph-lib and 1 vibegraph doc warnings remain; cargo doc --workspace also races on the vibegraph lib/bin doc name; test-target links cannot be checked."
area: hygiene
state: open
priority: low
closes_when: "cargo doc -D warnings runs per package in CI with no warning, and the lib/bin doc name collision is resolved."
blocked_by: []
opened: 2026-10-10
tags: [docs, ci]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-b-report, resource: "../../sprints/hygiene/sessions/F-B-report.md", title: "F-B report (hygiene sprint)"}
  - {id: f-c-report, resource: "../../sprints/hygiene/sessions/F-C-report.md", title: "F-C report (hygiene sprint)"}
  - {id: f-g2-report, resource: "../../sprints/hygiene/sessions/F-G2-report.md", title: "F-G2 report (hygiene sprint)"}
---
Remaining sites: `coupling/cluster/configs.rs:4`, `phasespace/channel.rs` (`ScaledChannel::ndim`), `coupling/alphas.rs`, `coupling/scales.rs` ×2, `ufo/parameters.rs`, `lhef/mod.rs` ×2, `vibegraph-cli/src/assets.rs:5` (F-B Found 3, F-C Found 5, F-G2 Found 10).

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-B](../../sprints/hygiene/sessions/F-B-report.md), [F-C](../../sprints/hygiene/sessions/F-C-report.md), [F-G2](../../sprints/hygiene/sessions/F-G2-report.md).
