---
type: Backlog Item
title: "No merge queue: personal-account repositories cannot have one"
description: "GitHub offers merge queues only to organization-owned repositories; until vibegraph moves to an org, a clean merge relies on main's own banked run to catch interacting PRs."
area: hygiene
state: needs-user
priority: low
closes_when: "The repository moves to an organization and a ruleset on main requires a merge queue (squash, groups of up to 5, `all-pass` only, no up-to-date requirement), and one PR has merged through it with banked run once on its group; or the user decides to keep the interim rule and closes this."
blocked_by: []
opened: 2026-11-17
tags: [ci, merge-queue, banked, process, repository-settings]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
The user decided on 2026-11-17 to keep the interim merge rule for now and to
look at moving to an organization later. The rule, the queue settings and what
a move involves are in [merge queue](../../tooling/merge-queue.md). `ci.yml`
already runs on `merge_group`, so the CI side is done.
