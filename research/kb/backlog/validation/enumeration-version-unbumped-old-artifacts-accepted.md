---
type: Backlog Item
title: "ENUMERATION_VERSION was not bumped in v0.2.2, so v0.2.1 artifacts bring back the 4× repeated-decay σ"
description: "Main records enumeration version 2, as v0.2.1 did, though #150 changed stitching. A v0.2.1 artifact for a repeated-decay card passes integrate --check and integrates to exactly 4× σ."
area: validation
state: open
priority: high
closes_when: "ENUMERATION_VERSION is bumped with its doc entry naming the changes since 2; a v0.2.1 artifact is refused by name; and the known-answer set pins a card with identical decays on interchangeable slots, so an unbumped stitching change fails it."
blocked_by: []
opened: 2026-11-19
tags: [artifacts, diagrams, decay-chains, silent-wrong-answer, release-blocking]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
sources:
  - {id: artifact, resource: "../../../../vibegraph-lib/src/diagrams/artifact.rs", title: "ENUMERATION_VERSION contract"}
  - {id: kd1, resource: "../../sprints/v022-smeft-and-refusals/sessions/K-D1-report.md", title: "K-D1 report"}
---
`ENUMERATION_VERSION`'s contract (`diagrams/artifact.rs`) says any change to
enumeration output, including a fix, must bump it. The amplitude probe of an
integrate artifact recompiles the banked diagrams, so it agrees with itself
whatever enumeration produced them. #150 (stitch repeated decays once) changed
which diagrams exist. #141 (orders spent vertex by vertex) may have changed
them too. Neither bumped the version: it is 2 on v0.2.1 (e4a90128) and on
main 43d2429. `enumeration_output_matches_the_pinned_version` did not fail,
because its card set has no identical decays on interchangeable slots.

Measured on 2026-11-19 by the sprint session with K-D1's NMSSM card
(`p p > w± h01, (h01 > a01 a01, a01 > b b~, a01 > b b~)`) and the M15 param
card:

| step | v0.2.1 build | main build (73e787f1, the same tree as 43d2429) |
|---|---|---|
| `enumerate` | 16 diagrams in 4 subprocesses | 8 diagrams in 4 subprocesses |
| main `integrate --check` on the artifact | `check ok` | `check ok` |
| main `integrate --fixed-budget --neval 20000 --niter 3 --seed 7` on the artifact | 105.10 ± 0.53 fb | 26.27 ± 0.13 fb |

The ratio is 4.00. simshop banks artifacts, so its stored v0.2.1 artifacts for
these cards would integrate silently 4× high under v0.2.2.
