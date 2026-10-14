---
type: Session Brief
title: "R-E: hygiene review of hadronic integration, PDFs and scales"
description: "Review proton, hadronic, pdf and coupling (couplings, alpha_s, scales, kT clustering) on the four hygiene points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/proton.rs`, `hadronic.rs`, `pdf/`, `coupling/` (about 21k lines) and their in-module tests.

**Already claimed, read and don't re-report:** configuration-weights-wrong-at-sde1-with-tmin, and the `proton.rs`/`hadronic.rs` sites in sampler-and-phase-space-doc-comments-stale and validation-test-comments-stale.

**Look especially at:** `proton.rs` (6.5k lines) and `hadronic.rs` (4.7k) as candidates for a module split, with a proposed boundary; run-card branches (`SDE_strategy`, `dynamical_scale_choice`) where one arm is untested or unreachable; probes and `#[ignore]`d tests that duplicate a gate.
