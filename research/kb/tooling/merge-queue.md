---
type: Design
title: "Merging through a GitHub merge queue"
description: "ci.yml runs on merge_group, so queued PRs are tested together against main's tip and need no up-to-date re-run; the queue itself is a ruleset on main that only a repository admin can turn on."
status: draft
tags: [ci, merge-queue, banked, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
sources:
  - {id: gh-mq, resource: "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue", title: "GitHub docs: managing a merge queue"}
---
# Merging through a GitHub merge queue

## Why

The merge rule was that a PR merges only when CI, `banked` included, is green
on a head that contains current `main`. `banked` takes about an hour, so every
merge forced each other ready PR to merge `main` in and wait another hour.
Sprint K had six green PRs at once, which meant a six-hour serial train.

Committed per-folder kb indexes made it worse. Every sprint PR added a session
brief, so every PR conflicted with every other in the same generated
`sessions/index.md`. Those indexes are no longer committed
([knowledge bundle](../workflow/knowledge-bundle.md)).

## How it works

`ci.yml` runs on `merge_group` as well as `pull_request` and pushes to `main`:
- **The tested tree.** When a PR enters the queue, GitHub builds a temporary
  branch holding `main`'s tip plus every PR ahead of it in the queue plus this
  one, and runs the required checks on it.
- **Merging.** If they pass, the PR merges. If they fail, it leaves the queue,
  and the PRs behind it are retested without it.
- **Batching.** With a group size above 1, several queued PRs share one group
  and one `banked` run.
- **`banked` decision.** The `changes` job diffs the group against
  `merge_group.base_sha` with the same skip list as on a pull request, so a
  group of kb-only PRs still skips `banked`.

A PR therefore needs green CI only on its own head, plus no textual conflict
with `main`. It never needs to be brought up to date just to re-run `banked`.

## Turning it on (repository admin)

In Settings → Rules → Rulesets, add or edit a branch ruleset for `main`:
1. **Require a merge queue:** merge method squash, build concurrency 5, minimum
   group size 1, maximum group size 5, wait 5 minutes for the group to fill,
   and "only merge non-failing pull requests".
2. **Require status checks to pass:** `all-pass` only. `docs.yml` does not run
   on `merge_group`, so requiring its `build` check would stall the queue. Its
   PR run still shows a broken site before merge.
3. **Do not** require branches to be up to date. The queue makes that rule
   redundant.

## Squash message

The queue squash-merges, and GitHub's squash adds a `Co-authored-by` line for
every commit author other than the person merging. Commits on a queued branch
must therefore be authored by that person (AGENTS.md, "Attribution"). The
queue cannot take a per-merge commit message, so set the repository's default
squash message to "pull request title and description". The PR body's
`Assisted-by` line then becomes the commit's trailer.
