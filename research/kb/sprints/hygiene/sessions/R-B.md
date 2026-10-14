---
type: Session Brief
title: "R-B: hygiene review of the Lorentz, colour and wavefunction layer"
description: "Review helas/repr, helas/color and the helas root files on the four hygiene points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/src/helas/repr/`, `helas/color/`, `helas/vertex.rs`, `helas/wavefn.rs`, `helas/mod.rs` (about 9k lines) and their in-module tests.

**Already claimed, read and don't re-report:** the `helas/repr` sites in evaluator-doc-comments-stale (`wavefn.rs`, `repr/lorentz.rs`, `color/flow_tags.rs`).

**Look especially at:** generic-over-`F` trait bounds that only one instantiation uses; `PhantomData` markers whose distinction no code relies on; convention claims (signs, ε, transposes) that no test pins, per AGENTS.md "Convention claims are hypotheses".
