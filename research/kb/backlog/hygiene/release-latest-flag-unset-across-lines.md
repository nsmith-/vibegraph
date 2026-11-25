---
type: Backlog Item
title: "A v0.2.x patch release published after v0.3.0 would take GitHub's Latest flag"
description: "release.yml calls gh release create without --latest, so GitHub marks the most recently created release as Latest. Once release/v0.2 and the v0.3 line both publish, a later v0.2.x would become Latest."
area: hygiene
state: open
priority: medium
closes_when: "release.yml sets --latest=true only when the tag is the highest semver tag in the repository, and --latest=false otherwise. A check pins the rule on a v0.2.x-after-v0.3.0 case, and release-binaries.md documents it."
blocked_by: []
opened: 2026-11-25
tags: [release, ci, release-branches, v0.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Found by sprint L's release-branch PR (#210). The v0.3 split runs two release
lines at once. `gh release create` without `--latest` lets GitHub choose the
newest-dated release, so a v0.2.x backport would replace v0.3.0 as "Latest" on
the releases page. Anyone following "latest" would then get the older line.

It must close before the first v0.2.x patch that ships after v0.3.0.
