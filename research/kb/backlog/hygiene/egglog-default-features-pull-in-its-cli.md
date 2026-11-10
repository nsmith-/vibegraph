---
type: Backlog Item
title: egglog's default features compile its command-line tool into the library
description: "vibegraph-lib depends on egglog = \"2.0.0\" with default features, whose bin feature pulls in clap, env_logger, chrono, mimalloc and graphviz serialisation that the library does not use."
area: hygiene
state: open
priority: low
closes_when: "vibegraph-lib depends on egglog with default-features = false and only the features it uses, the workspace builds and tests pass, the musl CLI still declares mimalloc itself, and the change in clean build time and release binary size is recorded in the PR."
blocked_by: []
opened: 2026-11-10
tags: [dependencies, build-time, binary-size, egglog]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: musl, resource: "../../performance/musl-allocator-thread-scaling.md", title: "musl allocator thread scaling"}
---
egglog 2.0.0's `default = ["bin"]`, and `bin` enables `serde`, `graphviz`,
`dep:clap`, `dep:env_logger`, `dep:chrono` and `dep:mimalloc`: its own
command-line tool. `vibegraph-lib/Cargo.toml` takes `egglog = "2.0.0"`, so all of
them compile into every build. Found while fixing the musl allocator, which is
why the musl CLI's mimalloc added no new C source
([musl allocator thread scaling](../../performance/musl-allocator-thread-scaling.md)).

Check which of `serde` and `graphviz` the library needs before dropping them.
`vibegraph-cli` declares `mimalloc` for musl directly, so turning off `bin` must
not remove the allocator; confirm the musl build still links it.
