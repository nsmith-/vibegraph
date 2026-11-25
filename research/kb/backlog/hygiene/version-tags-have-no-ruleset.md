---
type: Backlog Item
title: "v* release tags have no ruleset, so a tag can be created, moved or deleted by anyone with push access"
description: "Branch rulesets protect main (and, once added, release/v*), but the v*.*.* tags that trigger publishing are unprotected: a mistaken push or a force-moved tag publishes or rewrites a release."
area: hygiene
state: open
priority: medium
closes_when: "A tag ruleset on refs/tags/v* blocks deletion and update (no moving a release tag), and restricts creation to the repository owner. The settings are recorded in tooling/release-branches.md."
blocked_by: []
opened: 2026-11-25
tags: [release, ci, ruleset, tags]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Found by sprint L's release-branch PR (#210). Pushing a `v*.*.*` tag triggers
`release.yml`'s publish job. The history rewrite planned before v0.2.3 makes this
sharper: a tag that points at rewritten history must not be moved silently
afterwards. Changing a ruleset is the user's call.
