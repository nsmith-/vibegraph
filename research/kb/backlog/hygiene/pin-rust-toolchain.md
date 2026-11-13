---
type: Backlog Item
title: "Pin the Rust toolchain so a new stable cannot break CI or a release mid-flight"
description: "CI, docs and release workflows use dtolnay/rust-toolchain@stable; Rust 1.100's clippy broke CI during the v0.2.1 release. Pin it in rust-toolchain.toml with a deliberate bump."
area: hygiene
state: open
priority: high
closes_when: "A rust-toolchain.toml pins an exact version; ci.yml, docs.yml and release.yml install that version rather than @stable; and a documented bump procedure (bump, fmt and both clippy runs clean, CI green) exists."
blocked_by: []
opened: 2026-11-12
tags: [ci, toolchain, release, reproducibility]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: close, resource: "../../sprints/v021-fixes/closeout.md", title: "Sprint J record"}
---
Every Rust job installs `dtolnay/rust-toolchain@stable`: `ci.yml` (three
jobs), `docs.yml` and `release.yml`. When stable moved to Rust 1.100 during
the v0.2.1 release, its clippy's `needless_range_loop` began firing on code
that had not changed. That turned `main` and the release PR (#129) red until
three sites were rewritten. A release build can pick up a toolchain that no
PR was ever gated on.

What would resolve it:
- add a `rust-toolchain.toml` with an exact `channel` and the `rustfmt` and
  `clippy` components;
- have every workflow install that version, for example
  `dtolnay/rust-toolchain` with `toolchain:` read from the file, or plain
  `rustup show`, which honours it;
- write down the bump procedure: change the pin in one PR, run
  `cargo fmt --all --check` and both `cargo clippy --workspace --all-targets
  -- -D warnings` runs (with and without extended-validation), fix or
  explicitly allow any new lint, and merge once CI is green. Never bump while a
  release is in flight.

Local builds follow the pin too, through rustup. A contributor on an older
toolchain then gets it downloaded automatically.
