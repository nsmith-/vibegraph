---
type: Design
title: Release binaries
description: "release.yml builds static musl Linux (baseline and x86-64-v3) and native macOS binaries on tag push, publishes bare executables with SHA256SUMS, and dispatches acceptance."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [release, ci, distribution, musl]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n24-u1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/24-user-distribution-and-proton-events-plan.md#L2098-L2352", title: "Note 24 §U1 outcome: release workflow, musl decision, version scheme"}
  - {id: release-yml, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/.github/workflows/release.yml", title: ".github/workflows/release.yml"}
  - {id: build-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-cli/build.rs", title: "vibegraph-cli/build.rs"}
---
# Release binaries

`.github/workflows/release.yml` turns a version tag into downloadable,
self-contained `vibegraph` binaries for a user with no Rust toolchain.

## Trigger and jobs

- **Trigger**: `push` of a tag matching `v*.*.*` (which also matches a
  release candidate such as `v0.2.0-rc1`), plus `workflow_dispatch` for a dry
  run of the build matrix against any ref (input `tag`, default `v0.2.0-rc1`,
  names the tag whose notes the dry run reports on). `publish` runs only on a
  tag ref, so a dispatch builds and smoke-tests without releasing.
- **`release-check`** (`contents: read`): runs `scripts/release-check.sh` on
  the tag, then `scripts/test-release-check.sh`. On a tag push a refusal fails
  the run and `publish` (which needs it) is skipped; on a dispatch the real
  check is only reported, and the self-test exercises acceptance of
  `v0.2.0`/`v0.2.0-rc1` against a fixture notes file and refusal of a missing
  file, an empty file, a `TODO` stub, notes for another version and a
  non-version tag. `cargo test -p vibegraph` runs the same self-test.
- **`build`** (matrix, `fail-fast: false`, `contents: read`):

| asset | target | runner | note |
|---|---|---|---|
| `vibegraph-aarch64-apple-darwin` | `aarch64-apple-darwin` | `macos-15` | Apple Silicon, native |
| `vibegraph-x86_64-apple-darwin` | `x86_64-apple-darwin` | `macos-15-intel` | Intel, native; no Rosetta |
| `vibegraph-x86_64-unknown-linux-musl` | `x86_64-unknown-linux-musl` | `ubuntu-latest` | static, baseline x86-64 |
| `vibegraph-x86_64-unknown-linux-musl-v3` | same triple | `ubuntu-latest` | `RUSTFLAGS=-C target-cpu=x86-64-v3` (AVX2, FMA, BMI); opt-in, SIGILLs on pre-Haswell/Excavator CPUs |

  Runner labels are explicit rather than `macos-latest`, whose architecture has
  moved before and would silently change what "macOS x86_64" means.
  `macos-15-intel` is the last x86_64 macOS image Actions offers; the workflow
  comment dates its retirement to Fall 2027, after which Intel macOS has no
  build host. The cache key ends in `--` because one asset name is a prefix of
  another (`…-musl` of `…-musl-v3`) and a restore-key is a prefix match.
  Each leg runs `cargo build --release --locked --target <triple> --package
  vibegraph` (fat LTO, per `[profile.release]`), then a smoke test, then copies
  the binary to `dist/<asset>`.
- **`publish`** (`contents: write`, `actions: write`): downloads all assets,
  writes `SHA256SUMS`, and runs `gh release create <tag> --notes-file <notes>`
  (plus `--prerelease` when the tag contains a hyphen) with the binaries, `SHA256SUMS`, `LICENSE-MIT`, `LICENSE-APACHE` and
  `THIRD-PARTY-NOTICES`. It uses the preinstalled `gh` CLI, no third-party
  release action. It then dispatches `acceptance.yml` with `-f tag=<tag>`,
  because a release created with the workflow's own `GITHUB_TOKEN` raises no
  `release` event that starts a workflow; `workflow_dispatch` is one of the
  documented exceptions. See [release acceptance](release-acceptance.md).

## Release notes and prereleases

Notes are committed, never generated: `docs/release-notes/<tag without
prerelease suffix>.md`, so `v0.2.0-rc1` and `v0.2.0` both read
`docs/release-notes/v0.2.0.md` and a candidate publishes the same drafted text
as the final. `scripts/release-check.sh` fails when the file is missing, empty
or has a line containing `TODO` (a new version starts from a stub with
`Highlights`, `Changes` and `Known issues` sections and `TODO` lines), and
`publish` calls it again for the path, so notes can never fall back to the
generated PR list.

A tag containing a hyphen is published with `--prerelease`. GitHub's "latest
release" is the newest non-prerelease, non-draft release, so a candidate never
becomes what `releases/latest` (the weekly acceptance run's default) returns;
it can still be tested through the `-f tag=` dispatch, which `publish` issues
for every tag.

## Bare executables carry their own notices

Each asset is the bare executable, runnable after `chmod +x`, with no archive
to unpack. So the binary itself must carry the licence notices its contents
demand: `vibegraph --version` prints `THIRD-PARTY-NOTICES` and both of
vibegraph's licence texts, compiled in. The smoke test greps the built binary
for `Copyright (c) 2009, 2013, the MadTeam` and for the MIT text. Why the
notice is owed is [licensing](licensing.md).

## Linux: musl, not a glibc floor

The crates in the binary's graph that compile C are `zstd-sys`, which builds
its bundled zstd via `cc` rather than linking a system `libzstd` (`pkg-config`
is only an optional probe),[^n24-u1] and `ring`, the `rustls` crypto backend
that `ureq` (the PDF fetch, `vibegraph-cli/src/fetch.rs`) pulls in, which
builds its own C and assembly via `cc`. Neither links a system library, and
there is no `openssl` or `native-tls`. `libmimalloc-sys` (through egglog, and
the CLI's musl allocator below) is a third, also built via `cc`. So
`x86_64-unknown-linux-musl` costs only `musl-tools` (`musl-gcc`) on the runner
and yields one fully static binary that runs on any x86_64 Linux kernel, with no
"which glibc does this distro ship" matrix. A new C dependency that links a
system library would reopen this.

**The musl builds use mimalloc as the global allocator** (`vibegraph-cli`,
`cfg(target_env = "musl")`). musl's own allocator takes a process-wide lock, so
the v0.2.0 assets ran slower at `-j 4` than at `-j 1` and about 20–28% slower
than glibc single-threaded. The measurements and the alternatives are in
[musl allocator thread scaling](../performance/musl-allocator-thread-scaling.md).

**The glibc floor was measured and rejected.**
- `cargo zigbuild --target x86_64-unknown-linux-gnu.2.17` builds on an ordinary
  runner, needs at most `GLIBC_2.17`, and ran under CentOS 7's glibc. But it adds
  zig to CI, compiles the C dependencies with `zig cc`, runs about 15% slower than
  musl + mimalloc, and would rename the assets.
- A native glibc build needs `GLIBC_2.39` and starts on none of EL7–EL9.
- Static glibc still carries glibc's NSS loader.

`release.yml` deliberately does not raise the codegen floor for the baseline
assets; `ci.yml` builds its own tests with `-C target-cpu=x86-64-v3`, which is
also what shows the runner executes v3 code.

## Smoke test

Per leg, on the binary as built: `--version` (plus the two notice greps), then
a partonic run that needs no PDF set, no UFO file and no network because the
SM is compiled in:

```
generate e+ e- > t t~      # lpp = 0, ebeam 250 + 250
vibegraph integrate proc_card.dat --run-card run_card.dat --out out --neval 5000 --niter 2
test -f out/grid.bin.zst
```

It exercises diagram generation, HELAS evaluation and VEGAS end to end. It
does not check σ against anything; physics gates live in the validation
suite. Linux is smoke-tested on one runner here and exercised again by
acceptance; macOS is smoke-tested only here.

## The version string

`vibegraph-cli/build.rs` sets `VIBEGRAPH_VERSION` from
`git describe --tags --match 'v*' --always --dirty=-dirty` (arguments in
`vibegraph-cli/build_support/describe.rs`), so the newer `refdata-*` tags do
not name the version; `tests/release_tooling.rs` builds a repository with both
kinds of tag and pins this. It falls back to
`CARGO_PKG_VERSION` when there is no git binary or no repository (a GitHub
"Source code" tarball). `main.rs` uses it for both `version` and
`long_version`, so a tagged build reports the tag (`vibegraph v0.1.0`), an
untagged one the abbreviated hash (`vibegraph c6f3c32-dirty`), and a source
tarball `vibegraph 0.1.0`. All three paths were checked by building
locally.[^n24-u1] The build script emits `rerun-if-changed` on the git common
dir's `HEAD` and `refs`, so a moved tag rebuilds without touching tracked
files; the release checkout uses `fetch-depth: 0` so `git describe` sees the
tags.

## To cut a release

```bash
# 1. write docs/release-notes/vX.Y.Z.md (no TODO lines) and bump the workspace version
git tag vX.Y.Z-rc1 && git push origin vX.Y.Z-rc1   # prerelease candidate
git tag vX.Y.Z && git push origin vX.Y.Z
# watch Actions: publish needs every build leg green, then dispatches acceptance
```

A bad test release is removed with `gh release delete <tag> --yes` and deleting
the tag locally and on the remote.

[^n24-u1]: Note 24 §U1 outcome: the Linux target decision and the `--version` checks.
