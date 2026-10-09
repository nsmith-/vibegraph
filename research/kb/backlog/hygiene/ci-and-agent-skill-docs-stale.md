---
type: Backlog Item
title: release.yml and the extended-validation skill state stale facts
description: "release.yml calls zstd-sys the only C dependency though ring compiles C too, and the extended-validation skill omits that build.sh regenerates any missing MadGraph output directory."
area: hygiene
state: open
priority: low
closes_when: "release.yml's musl comment names every C-compiling dependency, and the skill's --skip-deps section says that a task depending on build-diagrams regenerates each missing output directory."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, ci, agent-skill, tooling]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: release, resource: "../../../../.github/workflows/release.yml", title: ".github/workflows/release.yml musl leg (~:39-40)"}
  - {id: skill, resource: "../../../../.agents/skills/extended-validation/SKILL.md", title: "extended-validation skill, --skip-deps section (~:57-70)"}
  - {id: build, resource: "../../../../validation/madgraph/build.sh", title: "validation/madgraph/build.sh, per-directory skip-if-present (~:126-131)"}
---
- **`.github/workflows/release.yml:39-40`**: "the crate's only C dependency is zstd-sys's bundled … libzstd". `ring` 0.17.14 (through `ureq`/`rustls`) also compiles C and assembly through `cc`. The musl choice is unaffected; the comment is wrong.
- **`.agents/skills/extended-validation/SKILL.md:57-70`** (`--skip-deps`): says what regeneration costs, but not what triggers it. `build.sh` regenerates every missing `validation/madgraph/output/<proc>/` and skips present ones (~:126-131). Every task depending on `build-diagrams` therefore starts a MadGraph build in a work area that lacks the bundle, while `pixi run validate` only fetches it. A folded fact recorded this; the skill was not updated.

The `acceptance.yml` header's "until a first release exists" is already in
[acceptance-yml-weekly-schedule](acceptance-yml-weekly-schedule.md). Found by
drafter D15 and verifier V15.
