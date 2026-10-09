---
type: Session Brief
title: "R-F: hygiene review of I/O, run cards, the CLI and the report"
description: "Review runcard, artifact, lhef, cache, config, the validation module, vibegraph-cli and validation-report on the four hygiene points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/` `runcard.rs`, `runcard/`, `artifact.rs`, `lhef/`, `cache/`, `config.rs`, `progress.rs`, `validation.rs`, `validation/`, `bin/`; all of `vibegraph-cli/` (src and tests); `validation-report/` (about 30k lines).

**Already claimed, read and don't re-report:** artifact-reader-arm-names-format-version, runcard-opaque-defaults-unverified, no-network-variable-read-two-ways, and the `artifact.rs` sites in process-model-and-artifact-doc-comments-stale.

**Look especially at:** error and refusal messages that name no flag or field; version and upgrade arms with no round-trip test; CLI logic that duplicates library logic; CLI integration tests that assert only an exit code.
