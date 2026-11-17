---
type: Backlog Item
title: "The merge queue is not enabled on main"
description: "ci.yml runs on merge_group, but main has no ruleset requiring a merge queue, so PRs still need an up-to-date head and a fresh banked run before each merge."
area: hygiene
state: needs-user
priority: high
closes_when: "A ruleset on main requires a merge queue (squash, groups of up to 5, `all-pass` the only required check, no up-to-date requirement), the default squash message is the PR title and description, and one PR has merged through the queue with banked run once on its group."
blocked_by: []
opened: 2026-11-17
tags: [ci, merge-queue, banked, process, repository-settings]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Only a repository admin can do this, in Settings → Rules → Rulesets. The
settings and the reasons for them are in
[merge queue](../../tooling/merge-queue.md).

Until it is on, the interim rule (the user, 2026-11-17) applies. A PR whose
only conflict with `main` is in generated files merges once its fast checks
pass, and `banked` is watched on `main` instead of re-run on the PR.
