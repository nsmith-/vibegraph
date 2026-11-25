---
type: Design
title: "Release branches and backports"
description: "release/vX.Y is cut from a release tag; a fix squash-merges to main, its squash commit is cherry-picked with -x into a PR against release/vX.Y, and the patch tag pushed there is built by release.yml."
status: draft
tags: [release, branching, ci, ruleset, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: split, resource: "../decisions/v03-branch-after-v023.md", title: "The v0.3 line splits from main at the v0.2.3 tag"}
  - {id: ci-yml, resource: "https://github.com/nsmith-/vibegraph/blob/eaf09f77/.github/workflows/ci.yml", title: ".github/workflows/ci.yml"}
  - {id: release-yml, resource: "https://github.com/nsmith-/vibegraph/blob/eaf09f77/.github/workflows/release.yml", title: ".github/workflows/release.yml"}
  - {id: gh-rulesets, resource: "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets", title: "GitHub docs: available rules for rulesets"}
---
# Release branches and backports

A maintenance line `vX.Y` lives on a branch `release/vX.Y`. `main` carries the
next minor version. Patch releases `vX.Y.Z` are tagged on the release branch.
The first such line is `release/v0.2`, cut at v0.2.3.[^split]

## Cutting the branch

The branch starts at the release tag, not at `main`'s tip:

```bash
git fetch origin --tags
git push origin v0.2.3^{commit}:refs/heads/release/v0.2
```

The tagged commit is on `main` and passed `main`'s push CI, so its `all-pass`
status already exists on that SHA. Create the branch **before** the
`release/v*` ruleset is active, or with the ruleset's bypass. A ruleset that
requires a pull request can refuse a push that creates a matching branch.

## A fix: main first, then cherry-pick

1. The fix lands on `main` as an ordinary PR. The repository squash-merges, so
   the PR becomes **one commit on `main`**. That squash commit is the one to
   pick, not the PR branch's commits, which `main` never contains.
2. Branch from the release branch and cherry-pick that commit with `-x`, so the
   message records `(cherry picked from commit <sha>)`:

   ```bash
   git fetch origin
   git switch -c backport/<topic> origin/release/v0.2
   git cherry-pick -x <squash-sha-on-main>
   ```

   Resolve any conflict on this branch. A fix that has to be rewritten for the
   release line (the code has diverged) is committed here directly, and the PR
   description says why.
3. Open the PR with **base `release/v0.2`**, not `main`. Its title says it is
   a backport and names the `main` PR. The squash keeps the `-x` line because
   the repository's default squash message is the commit messages.
4. Squash-merge it once `all-pass` is green. The usual
   [attribution](../../../AGENTS.md) rules hold: the branch's commits are
   authored by the human who merges, so the squash adds no model co-author.

A fix that applies only to the release line (it touches code `main` has
removed) skips step 1 and is written against `release/v0.2` directly.

## Which CI runs

`ci.yml` runs every job (`fmt`, `kb`, `test`, `changes`, `banked`,
`all-pass`) on:
- a pull request into `release/v*`, since the `pull_request` trigger has no
  branch filter. `changes` diffs against the PR's base, so a kb-only
  backport skips `banked` as it would on `main`;
- every push to `release/v*`, which is the squash merge itself. On a push
  `changes` always answers `run=true`, so `banked` always runs.

`docs.yml` builds the book on a PR into the branch but deploys only from
pushes to `main`. The site follows `main`, never a release line. No other
workflow runs on branch pushes.

The banked layer on the release branch checks against that branch's own
`validation/manifest.toml` and refdata pin. A backport that needs regenerated
reference data on `main` needs it on the release line too.

## Tagging a patch release

On the release branch, after the backport PRs have merged:

1. A PR into `release/v0.2` adds `docs/release-notes/v0.2.N.md` with no TODO
   lines, and bumps the workspace version. `release-check` reads the notes from
   the tagged commit, so the notes have to be on the release branch. Having
   them on `main` does not count.
2. Tag the merged commit and push the tag:

   ```bash
   git fetch origin
   git tag v0.2.N origin/release/v0.2
   git push origin v0.2.N
   ```

`release.yml` triggers on any pushed tag matching `v*.*.*`. Nothing in it, or
in the `acceptance.yml` run it dispatches, asks which branch the tagged
commit is on:
- `release-check` validates the tag's form and its notes file.
- `build` checks out the tag with full history. `git describe --tags --match 'v*'`
  then names the tag.
- `publish` runs on a tag push (`github.event_name == 'push'` and a `refs/tags/`
  ref), then creates the
  release and dispatches `acceptance.yml`. That dispatch runs the
  default-branch copy of the workflow, which checks out the tag again before
  it runs `scripts/acceptance.sh`, so the script matches the binary.

A tag `v0.2.4` on a commit reachable only from a local `release/v0.2` was
rehearsed against `scripts/release-check.sh` (accepted: `prerelease=false`,
`notes=docs/release-notes/v0.2.4.md`) and against `git describe` (`v0.2.4`).
`cargo build --release --locked --package vibegraph` at that tag gives a
binary whose `--version` reports `vibegraph v0.2.4`. The workflow has not yet
run on such a tag.

**Open: "Latest" after v0.3.0.** `gh release create` is called without
`--latest`, and the gh CLI documents its default as "automatic based on date
and version". Until a v0.3.0 release exists, a v0.2.x patch is the newest
release either way. After that, whether a later-dated v0.2.x would take
"Latest" from v0.3.0 has not been checked. The weekly acceptance run tests
whatever is "Latest".

**Dry run.** `release.yml`'s `workflow_dispatch` builds and smoke-tests every
leg without publishing: `gh workflow run release.yml --ref release/v0.2
-f tag=v0.2.N`. `publish` requires a tag push, so no dispatch publishes,
whatever its ref. The first such dry run on a release branch is run by hand
after the branch is cut from its tag, before the ruleset is activated.

## The ruleset

`release/v*` gets a ruleset that mirrors `main`'s:
- an active ruleset targeting `refs/heads/release/v*`;
- pull request required, squash merges only, no approvals;
- required status check `all-pass` from GitHub Actions, not strict;
- linear history;
- deletion and force-push (non-fast-forward) blocked;
- an empty bypass list.

The ruleset is a repository setting. Only the repository owner can change it.

## Related

- [Release binaries](release-binaries.md): what `release.yml` builds and how
  a release is cut.
- [Release acceptance](release-acceptance.md): the run `publish` dispatches.
- [What a green CI run covers](ci-coverage.md): the jobs `all-pass` aggregates.
- [Merging without an up-to-date banked re-run](merge-queue.md): the merge
  rule on `main`, which a release branch follows too.

[^split]: [The v0.3 line splits from main at the v0.2.3 tag](../decisions/v03-branch-after-v023.md).
