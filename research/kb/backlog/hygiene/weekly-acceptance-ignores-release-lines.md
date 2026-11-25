---
type: Backlog Item
title: "Weekly acceptance tests only the latest release, so v0.2.x binaries go untested once v0.3.0 ships"
description: "acceptance.yml's scheduled run resolves the most recent release; with two release lines, the release/v0.2 binaries stop being exercised after v0.3.0."
area: hygiene
state: open
priority: low
closes_when: "The scheduled acceptance run covers the newest tag of each supported release line (v0.2.x and v0.3.x), each from its own tag's acceptance script, or tooling/release-branches.md records that the v0.2 line is no longer exercised."
blocked_by: []
opened: 2026-11-25
tags: [release, ci, acceptance, release-branches, v0.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Found by sprint L's release-branch PR (#210). The weekly acceptance run picks
one release. After v0.3.0 that release is on the v0.3 line, and a regression
reachable only on release/v0.2 would surface only when a patch is cut.
