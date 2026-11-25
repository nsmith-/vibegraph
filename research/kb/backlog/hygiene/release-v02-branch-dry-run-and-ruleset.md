---
type: Backlog Item
title: "Cut release/v0.2 from the v0.2.3 tag, dry-run release.yml on it, and activate the release/v* ruleset"
description: "#210 made CI run on release/v* and stopped dispatches from publishing. The remaining steps are the user's: cut the branch, run the non-publishing dispatch from it, and activate the ruleset."
area: hygiene
state: needs-user
priority: high
closes_when: "release/v0.2 exists at the v0.2.3 tag; `gh workflow run release.yml --ref release/v0.2 -f tag=v0.2.3` completes release-check, the self-test and all four build legs with publish skipped; the release/v* ruleset from #210's body is active (PR required, squash only, all-pass required, linear history, no deletion or force-push)."
blocked_by: []
opened: 2026-11-25
tags: [release, ci, ruleset, release-branches, needs-user]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: concept, resource: "../../tooling/release-branches.md", title: "Release branches and the backport flow"}
---
#210 closed `release-branch-ci-and-ruleset-missing` as far as it could go:
- CI runs on `release/v*`;
- the backport flow is documented in [release branches](../../tooling/release-branches.md);
- `publish` requires a genuine tag push.

A dispatch from a branch ref builds every leg and skips `publish`. Running one
from `release/v0.2` is the evidence that a tag pushed from a non-main branch
builds.

The ruleset settings mirror main's ruleset (id 22539060) and are given in #210's
body as UI steps and as a `gh api` payload. Activate the ruleset after the
branch is cut.
