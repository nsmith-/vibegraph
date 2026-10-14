---
type: Design Decision
title: "Hygiene sprint visibility: mechanical demotion only"
description: "Demote pub items nothing outside their crate uses; anything another crate or a test target uses stays pub; the supported library surface is proposed for the user, not decided."
decided: 2026-10-09
decided_by: human:nsmith-
status: stable
tags: [hygiene, visibility, api]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: "human:nsmith-", at: 2026-10-09}]
sources:
  - {id: user, resource: "../log.md", title: "Hygiene sprint log, 2026-10-09: the user's answers in the planning session"}
---
Chosen over "propose and apply" and "audit only".

- **What is demoted.** A `pub` item in `vibegraph-lib` that nothing outside
  the crate names becomes `pub(crate)`, or private where its module alone
  uses it. The same applies to `vibegraph-cli` and `validation-report`.
- **What counts as a user outside the crate.** The other workspace crates,
  the integration tests under `tests/`, benches, and the `bin/` targets. All
  of them are built with `--all-targets --all-features`, so tests gated by
  `required-features` count. An item used only by integration tests stays
  `pub`, and the proposal marks it test-only.
- **What is not decided.** Which of the items the CLI uses form the supported
  library surface is the user's call on
  [lib-pub-api-surface-unaudited](../../../backlog/feature/lib-pub-api-surface-unaudited.md).
  V1's report lists the surface that remains, grouped as
  *used by the CLI*, *used by validation-report* and *test-only*. That list is
  the proposal.
- **Dead code.** An item that demotion leaves unused (clippy's `dead_code`
  fires) is handled by who still uses it. If only unit tests use it, it is
  gated with `#[cfg(test)]` or moved into the tests. If only feature-gated
  code uses it, it gets that code's `cfg`. If no code uses it, it is deleted
  and the docs naming it are reworded. A bare `#[allow(dead_code)]` is not a
  disposition. (Amended 2026-10-09 after V1, which met an earlier rule with
  144 allows.)
- **Docs document private items.** Intra-doc links to demoted items stay
  links. Every rustdoc build (`docs-api`, `scripts/build-docs.sh`) passes
  `--document-private-items`, so the API site stays a developer reference
  until the supported surface is decided. (Amended 2026-10-09.)
- **Inert by construction.** A visibility change cannot change behaviour, so
  V1's gate is the build, the lints and the hermetic tests.
