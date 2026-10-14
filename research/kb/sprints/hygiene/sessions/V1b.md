---
type: Session Brief
title: "V1b: replace V1's dead-code allows and restore its doc links"
description: "Continue V1 in its worktree: turn the 144 dead_code allows into cfg(test)/cfg_attr gating or deletions, restore the de-linked intra-doc links, and document private items in every doc build."
status: draft
agent: feature-dev (Opus), fresh session seeded with V1's report
depends_on: [V1]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
A continuation of [V1](V1.md), run by a fresh agent rather than a resumed one,
with [V1's report](V1-report.md) as its context. It applies two decisions
the user made on V1's results ([D2](../decisions/D2-visibility-mechanical-demotion.md),
amendment of 2026-10-09 in [log.md](../log.md)).

## Scope

1. **Dead-code allows.** For each `#[allow(dead_code)]` V1 added (list:
   `git diff 4bdd921 HEAD -- vibegraph-lib/src | grep '^+.*allow(dead_code)'`):
   - **used only by unit tests:** `#[cfg(test)]` on the item, or move it into
     the test module that uses it;
   - **used only by feature-gated code:** gate it the same way as its user,
     with `#[cfg(feature = "...")]` or
     `#[cfg_attr(not(feature = "..."), allow(dead_code))]`. Prefer the
     `cfg` where it compiles cleanly;
   - **no code user, only a doc or the kb:** delete the item and reword the
     doc sentence that names it. Kb mentions go to Found for close-out; don't
     edit `research/kb/`. Write-only fields count here: drop the field and
     the write.

   No unexplained `#[allow(dead_code)]` may remain. One that must stay
   carries a comment saying why.
2. **Test-only re-exports.** Remove the `#[allow(unused_imports)]`
   re-exports (`helas/mod.rs`: `ffv2_4_3`, `j3xxxx`, `VectorWf`;
   `helas/eval/mod.rs`: `Sym`). Have the unit tests import from the
   defining modules.
3. **Doc links.**
   - Revert V1's conversions of intra-doc links into plain code spans, so each
     link names its target again.
   - Build every doc with `--document-private-items`: the `docs-api` task
     in `pixi.toml` and `scripts/build-docs.sh`.
   - Links that still fail under `--document-private-items` are fixed, not
     de-linked.
4. **Nothing else.** No other refactors. `intertwiner.rs` and the
   `FIELD_CLASSES` alignment stay as V1 left them.

## Gate

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo clippy --workspace --all-targets -- -D warnings` (CI's configuration)
- `cargo test --workspace` (hermetic): the manager's V1 baseline is 35
  suites, 1348 passed, 0 failed, 17 ignored. A changed count must be
  explained (a deleted test-only helper is not a test).
- `cargo doc --workspace --no-deps --document-private-items`: report the
  warning count against the same command at `4bdd921`. It must not grow.

## Report

The usual session report, plus:
- the allow count before and after, with the command;
- a table of each allow's disposition: `cfg(test)`, moved, feature-gated,
  deleted, or kept with its reason;
- the number of links restored.
