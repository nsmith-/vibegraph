---
type: Session Report
title: "I-Z1 report: release tooling for v0.2.0"
description: "build.rs describes v* tags only; hyphenated tags publish as prereleases; notes come from docs/release-notes/<tag without suffix>.md and a missing, empty or TODO file fails; workspace 0.2.0; a release.yml dispatch ran green with publish skipped."
status: draft
tags: [release, ci, build, sprint-i]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-11-08}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-08}]
sources:
  - {id: commit, resource: "https://github.com/nsmith-/vibegraph/commit/9929dcd30140ead84673c0488c13ffbed324fa41", title: "9929dcd3 on sprinti-release"}
  - {id: run, resource: "https://github.com/nsmith-/vibegraph/actions/runs/42455545280", title: "release.yml dispatch on sprinti-release"}
---
Manager's spot checks: the commit's author and trailer; that
`origin/sprinti-release` is at 9929dcd3; the run's conclusion (`success`,
`workflow_dispatch`, head 9929dcd3) through the Actions API; and a reading of
`scripts/release-check.sh` and the `publish` step's `--prerelease` and
`--notes-file` wiring. The per-leg `--version` lines and the mutation results
below are the session's claims.

## Implemented

1. **`--match 'v*'`.** `git describe` runs with `--tags --match v* --always --dirty=-dirty`. The arguments live in `vibegraph-cli/build_support/describe.rs`, which `build.rs` and a test both include through `#[path]`. The `CARGO_PKG_VERSION` fallback is unchanged.
2. **Prereleases.** `scripts/release-check.sh TAG [ROOT]` accepts `vX.Y.Z` or `vX.Y.Z-suffix` and prints `prerelease=…` and `notes=…`. `publish` adds `--prerelease` for a tag with a hyphen. Acceptance needs no change: the `-f tag=` dispatch takes any tag. The weekly `releases/latest` lookup excludes prereleases by GitHub's definition of latest; that is recalled, not checked against the documentation, and not exercised.
3. **Release notes.** Notes are read from `docs/release-notes/<tag without -suffix>.md`, so an rc reads its release's notes. A missing file, an empty one, or any line containing `TODO` is refused. A `release-check` job runs first and `publish` needs it: on a tag push a refusal is fatal, on a dispatch it only reports. `--generate-notes` is gone. `docs/release-notes/v0.2.0.md` is a stub with TODO lines.
4. **Version.** `Cargo.toml` and `pixi.toml` are at 0.2.0. `Cargo.lock` changes only the three workspace packages (3 insertions, 3 deletions).
5. **Docs.** `research/kb/tooling/release-binaries.md` and `release-acceptance.md` are updated. The "red until next v* release" paragraph of `release-acceptance.md` is left for the close-out.

## Tests and mutations

- `vibegraph-cli/tests/release_tooling.rs` has 4 tests:
  - with a `v0.1.0` tag and a newer `refdata-9`, the describe result starts `v0.1.0-2-g`;
  - with only a `refdata` tag, it is the bare hash;
  - a commit tagged `v0.2.0-rc1` describes as `v0.2.0-rc1`;
  - `scripts/test-release-check.sh` passes its 10 cases.
- Mutations, each failing:
  - the old describe arguments: 2 passed, 2 failed (`expected the v* tag, got refdata-9`);
  - `TODO` replaced by `NOPE` in the grep: the TODO-stub cases;
  - the prerelease line deleted: `v0.2.0-rc1 gave: prerelease=false`.

## Gates (9929dcd3, `-p vibegraph` only)

`cargo fmt --all --check`, `cargo clippy -p vibegraph --all-targets -- -D warnings` and `cargo test -p vibegraph` (117 unit tests and the integration binaries) pass. The session did not run the workspace clippy, the workspace tests or `validate.sh`.

## The dispatch

Run 42455545280 ended `success`:
- `release-check` succeeded, with the self-test printing `all release-check cases passed`. On `v0.2.0-rc1` it printed `dry run: v0.2.0-rc1 would be refused by the publish job` (the TODO stub).
- All four build legs succeeded. Each printed `vibegraph v0.1.0-107-g9929dcd3` for `--version`.
- `publish` was skipped.

Not exercised: `gh release create` with `--prerelease` and `--notes-file`, the acceptance dispatch from `publish`, and the fatal tag-push path.

## Found

- A `release.yml` comment still names `zstd-sys` as the only C dependency; `ring` is one too.
- Until `v0.2.0` is tagged, builds report `v0.1.0-N-g…`; the `0.2.0` shows only through the fallback.
