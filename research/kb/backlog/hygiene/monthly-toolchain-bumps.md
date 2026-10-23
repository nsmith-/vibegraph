---
type: Backlog Item
title: Freeze the Rust and pixi toolchains, and bump them on a monthly schedule
description: "CI floats on Rust stable and pins pixi v0.63.2 while dev hosts run newer pixi; both drifts have broken PRs. Pin both in the repo and bump them deliberately once a month."
area: hygiene
state: open
priority: medium
closes_when: "rust-toolchain.toml pins an exact Rust release that every workflow reads; pixi.toml's requires-pixi matches the CI pin; a monthly scheduled job opens the bump PR; the routine is recorded where contributors read the gates."
blocked_by: []
opened: 2026-10-22
tags: [ci, toolchain, clippy, pixi]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: pr51, resource: "https://github.com/nsmith-/vibegraph/pull/51", title: "PR #51 CI failure on 5e46f05: a clippy lint only CI's newer stable knew"}
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71", title: "PR #71 CI failure: pixi 0.82 wrote lock-file version 7, which CI's pixi v0.63.2 cannot read"}
---
Toolchain versions drift in two directions, and each direction has turned a PR red
for reasons unrelated to its change:

- **Rust floats.** Five workflow steps use `dtolnay/rust-toolchain@stable`, and
  the repo has no `rust-toolchain.toml`. A dev host on 1.97 passed every local
  gate, then CI's 1.99 clippy flagged `chunks_exact_to_as_chunks` (PR #51).
- **pixi is pinned in CI only.** The three `setup-pixi` steps pin v0.63.2, but
  nothing stops a newer local pixi from rewriting `pixi.lock`. pixi 0.82
  wrote lock-file version 7, and every pixi job in CI failed at
  `pixi install --locked` (PR #71).

Resolution:

1. **Freeze Rust.** Add `rust-toolchain.toml` with an exact `channel` (e.g.
   `1.99.1`) and the `clippy` and `rustfmt` components. Point every workflow's
   toolchain step at that file instead of `@stable`, so local and CI builds run
   the same compiler and lints.
2. **Freeze pixi.** Set `requires-pixi` in `pixi.toml`'s `[workspace]` to the
   version CI pins. A mismatched local pixi then refuses to run instead of
   rewriting the lock.
3. **Bump monthly.** A scheduled workflow, on the first of each month, opens one
   PR that:
   - moves the Rust channel to the current stable;
   - moves the pixi pin in `pixi.toml` and the workflows to the current release,
     and re-locks;
   - bumps the GitHub Actions' major versions;
   - fixes any new lints in the same PR, so the bump lands green.

   A bump whose fixes are too large for that PR gets its own item.
