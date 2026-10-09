---
type: Design
title: Measurement provenance
description: "The measured: block (commit, pr, landed_in, host, command) and how a later session decides whether a number needs re-measuring across squash merges."
status: draft
tags: [measurement, provenance, git, knowledge-bundle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n42-meas, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L216-L256", title: "Note 42 §5: measurements and staleness"}
  - {id: n42-decisions, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/42-okf-knowledge-bundle-plan.md#L514-L535", title: "Note 42 §9: decisions 5 and the deferred dependency tracking"}
---
Measurements go stale when **code** changes, not when time passes. Nothing
tracks that automatically. A measurement records the facts that let a later
session judge whether re-measuring is warranted. That judgement is the
session's.[^n42-meas]

## The `measured:` block

Every `Measurement` concept, and any other concept that quotes a measured
number, carries a `measured:` frontmatter block. OKF allows unknown keys, so
this needs no spec change.

```yaml
measured:
  commit: 4f2c9e1     # the tree the numbers came from
  pr: 12              # fetch with `git fetch origin pull/12/head`
  landed_in: 1539abc  # the squash commit on main, filled at close-out
  host: "M3 Max, macOS 15"
  command: pixi run bench-eval -- gg_ttg
```

- A concept measured on several hosts or at several commits carries a **list**
  of such mappings.
- Unknown keys are left out. A commit is never invented. For a number older
  than the repository's history, the date and what the source says are all
  there is, and the body states that.
- `stale_after` stays empty for anything tied to code. It is kept for the rare
  fact that genuinely expires with time.

## Why both `commit` and `pr`

PRs land on `main` as **one squash commit**, often a whole sprint: `1539abc`
is all of `process-grammar` (nsmith-/vibegraph#12). So a commit a measurement
was taken at inside a sprint never appears on `main`, and its branch may be
deleted. GitHub keeps `refs/pull/<n>/head`, so the PR number keeps the commit
fetchable. `landed_in` records where the change reached `main`.

| Field | Answers |
|---|---|
| `commit` | Which tree produced the numbers |
| `pr` | How to fetch that tree after the branch is gone |
| `landed_in` | Where it reached `main`; missing means the PR has not merged or close-out skipped it, and `pr` tells which |
| `host` | Whether two numbers are **comparable**, not whether one is stale (see [benchmark hosts](../performance/benchmark-hosts.md)) |
| `command` | How to reproduce it |

## Deciding whether to re-measure

A session weighing a recorded number runs:

```sh
# what changed on main since the measurement landed
git log --oneline <landed_in>..origin/main -- <paths the body names>
# what changed between the measurement and the merge
git fetch origin pull/<pr>/head
git log --oneline <commit>..FETCH_HEAD
# a direct tree comparison; works across a squash once the object is fetched
git diff --stat <commit> origin/main -- <paths>
```

Cloud sessions are **shallow clones**, so a brief that asks for this says to
fetch the PR ref or deepen history first. Otherwise the commits are simply
absent and every log comes back empty.

When the paths a number depends on have changed heavily since `landed_in`, the
number is a re-measurement candidate. A [sprint](sprint-lifecycle.md) checks
for these in its topic before designing. Measurements are best taken **after
the sprint's last code change**, so `commit` is close to what lands. At
close-out, `landed_in` is filled once the PR merges.

## What is deferred

The convention starts with the migration. Numbers drafted from the archived
notes carry whatever host and commit the notes gave, and nothing more.[^n42-decisions]

Two refinements wait for evidence that the simple version falls short:

- deriving each measurement's code dependencies, for example from a coverage
  run of its command;
- computing staleness from per-file git blob IDs.

Until then, "the paths the body names" is the dependency set.

The trust-tier and close-out rules that use this block are in the
[sprint lifecycle](sprint-lifecycle.md). The bundle's frontmatter as a whole is
in [the knowledge bundle](knowledge-bundle.md).

[^n42-meas]: Note 42 §5.
[^n42-decisions]: Note 42 §9, decision 5, and "Still deferred".
