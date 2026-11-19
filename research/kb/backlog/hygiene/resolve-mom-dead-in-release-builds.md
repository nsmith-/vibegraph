---
type: Backlog Item
title: "resolve_mom is dead code in release builds only"
description: "helas/eval/analysis.rs resolve_mom is unused under cargo build --release -p vibegraph; CI's clippy over all targets does not see it."
area: hygiene
state: open
priority: low
closes_when: "resolve_mom is removed or gated to the configuration that uses it, and a release build of the vibegraph crate has no dead-code warning."
blocked_by: []
opened: 2026-11-19
tags: [dead-code, release, clippy]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-C2 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C2-report.md), Found).

**K-H4 (PR #158)** saw the same warning independently under `cargo check --release`; clippy does not report it.
