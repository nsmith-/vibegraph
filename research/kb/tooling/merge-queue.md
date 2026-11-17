---
type: Design
title: "Merging without an up-to-date banked re-run"
description: "A PR merges once fast checks pass on a head that merges cleanly; banked runs on every push to main and a red main is reverted or fixed first. ci.yml already runs on merge_group for a future queue."
status: draft
tags: [ci, merge-queue, banked, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
sources:
  - {id: gh-mq, resource: "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue", title: "GitHub docs: managing a merge queue"}
---
# Merging without an up-to-date banked re-run

## Why

The merge rule used to be that a PR merges only when CI, `banked` included, is
green on a head that contains current `main`. `banked` takes about an hour, so
every merge forced each other ready PR to merge `main` in and wait another
hour. Sprint K had six green PRs at once, which meant a six-hour serial train.

Committed per-folder kb indexes made it worse. Every sprint PR added a session
brief, so every PR conflicted with every other in the same generated
`sessions/index.md`. Those indexes are no longer committed
([knowledge bundle](../workflow/knowledge-bundle.md)).

## The rule (the user, 2026-11-17)

1. A PR needs green CI, `banked` included, on its own head once.
2. If it later conflicts with `main` only in generated files, its author merges
   `main` in and pushes. It then merges as soon as test, fmt and kb pass on that
   push, without waiting for `banked` to re-run.
3. A PR that merges cleanly, with no conflict, merges on its existing green CI.
4. A conflict in code means the full CI, `banked` included, runs again before
   the PR merges.
5. `banked` runs on every push to `main`. A red `main` is fixed or reverted
   before anything else merges.

The cost is up to about an hour between a merge and the `main` run that would
catch two PRs interacting badly.

## A merge queue, later

A GitHub merge queue would close that hour. It tests each queued PR on
`main`'s tip plus every PR ahead of it, and a batch shares one `banked` run.
GitHub offers merge queues only on repositories owned by an organization
(public ones, or private ones on Enterprise Cloud). `nsmith-/vibegraph` is
owned by a personal account, so the setting does not exist there.

`ci.yml` already runs on `merge_group`. The `changes` job diffs a group
against `merge_group.base_sha`, so a kb-only group still skips `banked`.
Turning the queue on after a move to an organization is then a ruleset on
`main`:
- squash merges, groups of up to 5;
- `all-pass` as the only required check (`docs.yml` does not run on
  `merge_group`);
- no requirement that branches be up to date;
- the default squash message set to "pull request title and description".

GitHub's squash adds a `Co-authored-by` line for every commit author other
than the merger, so commits on a queued branch must be authored by that person
(AGENTS.md, "Attribution"). The PR body's `Assisted-by` line becomes the
commit's trailer.

The move itself needs:
- git remotes updated on every host;
- the hard-coded `nsmith-/vibegraph` in `scripts/kb.py`, the docs and the
  release fetch URLs updated;
- the Claude GitHub App reinstalled for the new owner.
