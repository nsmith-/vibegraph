---
type: Design
title: "Pinned Rust and pixi, bumped monthly"
description: "rust-toolchain.toml pins Rust and pixi.toml's requires-pixi pins pixi; every workflow reads the pins; a scheduled workflow opens the monthly bump PR and reports the lint gates."
status: draft
tags: [ci, toolchain, pixi, clippy, release, process]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-13}
sources:
  - {id: pr51, resource: "https://github.com/nsmith-/vibegraph/pull/51", title: "PR #51: a clippy lint only CI's newer stable knew"}
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71", title: "PR #71: pixi 0.82 wrote a lock CI's pixi v0.63.2 could not read"}
  - {id: j, resource: "../sprints/v021-fixes/closeout.md", title: "Sprint J: Rust 1.100 reddened main and the release PR"}
---
# Pinned Rust and pixi, bumped monthly

## Why

Floating toolchains turned PRs red for reasons unrelated to their changes,
three times:
- CI's newer stable clippy flagged a lint no dev host knew (#51).
- A newer local pixi rewrote `pixi.lock` in a format CI's pixi could not read
  (#71).
- `dtolnay/rust-toolchain@stable` moved to Rust 1.100 during the v0.2.1
  release, and its `needless_range_loop` reddened `main` and the release PR
  (sprint J).

A pinned toolchain changes only in a PR that is about the change.

## The pins

| tool | pin | who reads it |
|---|---|---|
| Rust | `rust-toolchain.toml`: exact `channel`, plus the `rustfmt` and `clippy` components | rustup, everywhere: every workflow's `rustup toolchain install` step (`ci.yml`, `docs.yml`, `release.yml`), and local `cargo` in the checkout. The release cache key hashes the file. |
| pixi | `pixi.toml` `[workspace] requires-pixi = "==X.Y.Z"` | pixi itself, which refuses to run at any other version and names `pixi self-update --version`. Each workflow's `setup-pixi` `pixi-version` is set to the same version. |
| GitHub Actions | the `uses:` versions | Dependabot (`.github/dependabot.yml`), monthly, one grouped PR |

A contributor on another Rust gets the pinned one downloaded by rustup on first
use. A contributor on another pixi runs `pixi self-update --version X.Y.Z`. Any
pixi other than the pinned one is refused, newer or older, because only the
pinned pixi writes the lock format CI reads.

## The monthly bump

`toolchain-bump.yml` runs on the 1st of each month, and on demand
(`workflow_dispatch`):
1. It reads the current stable Rust (`channel-rust-stable.toml`) and the latest
   pixi release. It does nothing if the pins already match, or if that bump's
   branch already exists.
2. `scripts/bump-toolchains.sh <rust> <pixi>` moves all the pins together, then
   re-locks `pixi.lock` with the new pixi and installs the new Rust.
3. It runs the lint gates CI enforces on the new Rust: fmt, clippy, and clippy
   with extended-validation. Then it opens a PR,
   `toolchain-bump/rust-<rust>-pixi-<pixi>`, whose body lists each gate and the
   lint sites that failed.

**Then, by hand:** fix every new lint on that branch, either by rewriting the
site or with a documented `allow`, per AGENTS.md's lint rule. Merge once CI is
green, banked included. A fix too large for the bump PR gets its own item, and
the bump waits for it. **Never bump while a release is in flight**: hold the
PR until the tag is cut.

An off-schedule bump runs the same script by hand, with the target pixi on
`PATH`:

```bash
pixi self-update --version X.Y.Z
scripts/bump-toolchains.sh 1.NNN.0 X.Y.Z
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --features vibegraph/extended-validation,vibegraph-lib/extended-validation -- -D warnings
```

## Caveats

- **A PR opened with the workflow's own `GITHUB_TOKEN` starts no workflows.**
  CI then does not run on the bump PR until someone pushes to its branch. The
  lint fixes are that push, so CI runs once they land. The secret
  `TOOLCHAIN_BUMP_TOKEN` avoids the wait: a fine-grained token with contents
  and pull requests set to write, on this repository only. The PR is then
  opened as the token's owner and CI starts at once. Without the secret, the
  repository setting "Allow GitHub Actions to create and approve pull requests"
  must be on.
- **Re-locking moves only what the pixi version requires.** `pixi lock` keeps
  the locked packages and upgrades the file format when needed. 0.63.2 → 0.84.1
  changed the format from v6 to v7 and left all 1874 package entries unchanged.
