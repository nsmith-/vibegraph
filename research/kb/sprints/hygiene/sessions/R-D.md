---
type: Session Brief
title: "R-D: hygiene review of phase space and sampling"
description: "Review phasespace, vegas, budget, cuts and the unweighting and statistics modules on the four hygiene points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/phasespace/`, `vegas.rs`, `budget.rs`, `cuts.rs`, `multiplicity.rs`, `unweight.rs`, `stats.rs`, `select.rs` (about 17k lines) and their in-module tests.

**Already claimed, read and don't re-report:** sampler-and-phase-space-doc-comments-stale, madgraph-line-citations-predate-pin.

**Look especially at:** statistical tests that pass on a fixed seed alone (AGENTS.md "Samplers gate statistically"); configuration knobs whose non-default arms no production path selects; doc comments justifying a tuning by a combination rule or stop rule that has since changed.
