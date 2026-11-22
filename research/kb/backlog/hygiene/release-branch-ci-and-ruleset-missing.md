---
type: Backlog Item
title: A maintenance branch has no CI trigger, ruleset or cherry-pick guide
description: "ci.yml runs on pushes to main only and the ruleset covers main only, so release/v0.2 would get no push CI, no all-pass gate and no documented backport flow."
area: hygiene
state: open
priority: medium
closes_when: "ci.yml runs on pushes to release/v*; the ruleset requires all-pass on release/v* as on main; a tooling concept documents the backport flow (fix on main, cherry-pick, tag on the release branch); and release.yml is shown to build a tag pushed from a branch other than main."
blocked_by: []
opened: 2026-11-22
tags: [release, branching, ci, v0.3]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
---
The v0.3 line splits from `main` at the v0.2.3 tag
([decision](../../decisions/v03-branch-after-v023.md)). Today `ci.yml`'s
`push` trigger lists only `main`. Pull requests into any branch still run CI,
but a direct push, such as a cherry-pick merge, does not. The branch ruleset
that requires `all-pass` covers `main` only. `release.yml` fires on any
`v*.*.*` tag, so tagging from `release/v0.2` should already build. That has
never been exercised. The ruleset change is a repository setting that only the
user can make.

This has to be done before the v0.2.3 tag is cut.
