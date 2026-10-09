---
type: Backlog Item
title: The FeynGraph reference submodule is pinned to a different commit than the build
description: "research/refs/feyngraph is at 1dc4ea7 while vibegraph-lib builds against FeynGraph fd5aa83, eight commits later, so a reader of the submodule sees code the crate does not use."
area: hygiene
state: open
priority: low
closes_when: "research/refs/feyngraph points at the rev vibegraph-lib/Cargo.toml builds against, and research/refs/README.md names it."
blocked_by: []
opened: 2026-10-09
tags: [references, submodule, feyngraph]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: cargo, resource: "../../../../vibegraph-lib/Cargo.toml", title: "vibegraph-lib/Cargo.toml ~:34, feyngraph rev fd5aa8306746b432e098c40dcd96d7cb7ef20125"}
  - {id: refs, resource: "../../../refs/README.md", title: "research/refs/README.md"}
---
`git ls-tree HEAD research/refs/feyngraph` gives
`1dc4ea768855bcbae034eff8c0e6e8c8b0aae47c`. The crate builds against
`fd5aa8306746b432e098c40dcd96d7cb7ef20125` (`vibegraph-lib/Cargo.toml` ~:34).
Between them are eight commits: a drawing rewrite, the `add_particle` rework,
`AsRef<str>` APIs and raw-string UFO parsing. A citation or line number read
from the submodule can describe behaviour the build does not have. Bump the
submodule to the build's rev and keep the two moving together. Found by
Phase 3 verifier V16.
