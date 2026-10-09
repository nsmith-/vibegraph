---
type: Backlog Item
title: The vendored SMEFTsim checksum check is not in the default test run
description: "vendored_copy_matches_its_manifest lives in tests/smeftsim.rs, which requires extended-validation, so a plain cargo test never checks the vendored UFO against its SHA256SUMS."
area: validation
state: open
priority: low
closes_when: "The vendored-copy checksum test runs under a plain cargo test (hermetic layer), and the manifest records it there."
blocked_by: []
opened: 2026-10-09
tags: [smeftsim, vendored, hermetic, ci-coverage]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: test, resource: "../../../../vibegraph-lib/tests/smeftsim.rs", title: "tests/smeftsim.rs vendored_copy_matches_its_manifest (~:124-131)"}
  - {id: cargo, resource: "../../../../vibegraph-lib/Cargo.toml", title: "vibegraph-lib/Cargo.toml [[test]] smeftsim, required-features = [extended-validation]"}
  - {id: note35, resource: "../../history/notes/35-ufo-lorentz-sprint-plan.md", title: "Note 35 L1, which planned the check hermetic"}
---
`vendored_copy_matches_its_manifest` (`vibegraph-lib/tests/smeftsim.rs`
~:124-131) checks `validation/ufo/SMEFTsim_topU3l_MwScheme_UFO/` against its
`SHA256SUMS` in both directions. The whole test target has
`required-features = ["extended-validation"]` (`vibegraph-lib/Cargo.toml`), so
CI's default `cargo test` never runs it. A drifted or extended vendored copy
would change what every SMEFTsim gate measures, and nothing hermetic would say
so. Note 35 L1 planned this check as hermetic. It needs no network and no
MadGraph, so it can move to an ungated target. Found by Phase 2 drafter D13.
