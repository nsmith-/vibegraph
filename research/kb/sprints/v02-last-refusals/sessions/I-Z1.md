---
type: Session Brief
title: "I-Z1: release tooling for v0.2.0: version tags only, rc tags as prereleases, committed release notes, the version bump"
description: "build.rs describes only v* tags; release.yml publishes hyphenated tags as prereleases and reads the notes from a committed file, failing without it; the workspace moves to 0.2.0; all proven by a manual release.yml run with no tag pushed."
status: draft
agent: feature-dev (Sonnet)
depends_on: []
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
---
The user's instruction for the close-out PR, relayed by the vibegraph manager
(2026-11-08). Read `research/kb/tooling/release-binaries.md`,
`research/kb/tooling/release-acceptance.md`, `.github/workflows/release.yml`,
`.github/workflows/acceptance.yml` and `vibegraph-cli/build.rs` first.

## Worktree & long-command discipline

- **First action**: `cd /home/user/wt/sprinti-release`, then
  verify `git rev-parse --show-toplevel` and `git branch --show-current` (`sprinti-release`). If you
  find yourself in the shared main checkout, STOP and report — never edit it.
  Re-verify `pwd` after any resume; resume is when isolation leaks.
- **Background anything expected to exceed ~2 minutes**
  (`run_in_background: true`, output redirected to a log file prefixed with your
  worktree/session name — parallel siblings share one scratchpad and have
  clobbered each other's unprefixed logs). A backgrounded command notifies on
  exit: never spawn sleep-based watcher shells (they compound into
  self-sustaining wake-up rings); draft your notes and report between completion
  notifications instead of idle-waiting. On resume after a stall, first check
  whether the interrupted command finished (`ps`, result artifacts) before
  re-running it.
- **`git status` can hang for minutes** in a worktree full of untracked build
  data; spot-check with `git log` / `git diff --stat` instead.
- **Run gates with `--skip-deps`** unless the assignment explicitly says
  reference data must be regenerated — without the reference data present, a
  bare `pixi run validate` silently launches a multi-hour MG regeneration.
- **Disk is finite, and a full disk looks like a failure.** A debug workspace
  build with incremental data reaches ~15 GB per worktree, and two such trees
  have filled a cloud container mid-gate. On a shared or small host, run cargo
  with `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`
  (they change disk use only, not what a gate checks). An `ENOSPC`, a linker
  `Bus error` or a truncated log is a kill, not a result: free only your own
  `target/` and re-run. Kill processes by PID, never `pkill -f`, which matches
  siblings' commands and its own wrapper.


This container restarts without warning. Run every long job fully detached
(`nohup setsid`, not a chain shell), with per-seed output appended to a file,
so a restart keeps every finished seed and you rerun only the missing ones.
Run heavy jobs (seed sweeps, `validation/validate.sh`, local MadGraph) one at a
time (4 cores), and ask the manager before starting one: a sibling session may
be running its own. Commit and push at checkpoints. Prefix logs `iz1-`.
pixi is not installed here: run gates as plain `cargo` and
`bash validation/validate.sh`, which fetches nothing (the `--skip-deps`
equivalent); the reference data is already copied into your worktree.

## Scope

1. **`vibegraph-cli/build.rs`.** `git describe --tags` takes the newest tag of
   any kind: on main it gives `refdata-9-95-gd87ad3ae` where it should give
   `v0.1.0-106-gd87ad3ae`. Add `--match 'v*'`, keep the `--always` and
   `CARGO_PKG_VERSION` fallbacks, and add a test or CI check that fails if a
   reference-data tag is picked (say which, and show it failing on the old
   arguments).
2. **Prereleases.** `v*.*.*` also matches `v0.2.0-rc1`, and `gh release create`
   is called without `--prerelease`, so an rc tag would publish a full release
   that becomes "latest", which is what the weekly acceptance run tests. Publish
   a tag containing a hyphen as a prerelease. Check that acceptance still
   works: the `-f tag=` dispatch runs on an rc, and the weekly run's
   `releases/latest` lookup excludes prereleases (GitHub's `latest` already
   does; cite the documentation or the API behaviour you rely on).
3. **Release notes.** `--generate-notes` replaces drafted notes with a PR list.
   Read the notes from a committed file instead (`--notes-file`). Choose the
   path and naming convention (e.g. `docs/release-notes/<tag-without-rc>.md`,
   or per tag; decide how an rc finds its notes), and fail the publish job if
   the file is missing: never fall back to generated notes. Add the v0.2.0 file
   as a stub with the section headings (Highlights, Changes, Known issues) and
   a `TODO` line the publish job also refuses (so an unfinished stub cannot
   ship); the manager writes the content.
4. **Version bump.** The workspace version 0.1.0 → 0.2.0 (`Cargo.toml`, and
   `Cargo.lock` via `cargo update -w`, nothing else in the lock moving: show
   `git diff --stat Cargo.lock`). Check for other places that state the version
   (docs, README, acceptance) and update or list them.
5. **Docs.** Update `release-binaries.md` and `release-acceptance.md` to the new
   behaviour, with the path convention.

## Evidence (no real tag is pushed, ever)

- Push the branch, then trigger `release.yml` by `workflow_dispatch` on
  `sprinti-release` through the GitHub MCP tools (`ToolSearch` for
  `mcp__github__actions_run_trigger`; `gh` is not available). Record the run
  URL, its conclusion and what the smoke test printed for `--version` on each
  leg (from the job logs). The publish job must be skipped on a dispatch: show
  that it was.
- The notes-file check and the prerelease decision should be testable without
  publishing: factor them into a step or script the build job (or a dry-run
  path) exercises, and show it accepting `v0.2.0-rc1`/`v0.2.0` with the file
  present and refusing a missing file and the `TODO` stub. Say exactly what
  was not exercised.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, and `cargo test -p vibegraph` green. You need no reference data:
  do not run `validate.sh`.

## Rules for this sprint

- Commits are authored by Nicholas Smith: `git -c user.name="Nicholas Smith" -c user.email="nick.smith@cern.ch" commit …`, and end with exactly one trailer, `Assisted-by: claude-code:<your model id>`. Never `Co-authored-by` or `Signed-off-by`.
- Do not create, edit or delete backlog items, or anything else under `research/kb/`, unless your scope says so; the manager deletes the items your PR closes. New work goes in your report's **Found**.
- Comments describe the code in its own terms: never backlog items, sessions or sprints.
- Statistical claims: ≥ 5 seeds a side, unweighted mean, with the spread and χ²/dof (AGENTS.md, "Samplers gate statistically"). Every number in your report is a recorded measurement with its command beside it, on a stated commit.
- Lints are gates: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- Correct this brief if its premise is wrong. Stop and report rather than build on a wrong premise.

## Report

Your final message is the report: the commits; each oracle with its command and
output; mutation evidence for each new test (it fails before the change); the
gates run, with their output; and a **Found** section. The manager records it
as this session's `-report.md`.
