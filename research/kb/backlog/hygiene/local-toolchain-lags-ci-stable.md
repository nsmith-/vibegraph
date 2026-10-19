---
type: Backlog Item
title: Local gates run an older Rust than CI's floating stable, so new clippy lints surface only in CI
description: "CI installs dtolnay/rust-toolchain@stable (1.99 on 2026-10-17); the cloud host had 1.97, whose clippy lacks chunks_exact_to_as_chunks, so PR #51 went red on a lint no local gate could see."
area: hygiene
state: needs-user
priority: medium
closes_when: "The toolchain is pinned (rust-toolchain.toml, updated deliberately) or the dev hosts track CI's stable, and the choice is recorded where contributors read the gates."
blocked_by: []
opened: 2026-10-18
tags: [ci, toolchain, clippy]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-18}
sources:
  - {id: pr51, resource: "https://github.com/nsmith-/vibegraph/pull/51", title: "PR #51 CI failure on 5e46f05 (validate_hadronic.rs chunks_exact)"}
---
Pinning trades surprise lint failures for deliberate toolchain bumps; the
user decides.

Filed at the close-out of [vegas-low-acceptance-bins](../../sprints/vegas-low-acceptance-bins/sprint.md).
