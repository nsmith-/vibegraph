---
type: Session Brief
title: "R-G1: hygiene review of the amplitude, model and diagram test targets"
description: "Review the vibegraph-lib integration tests on amplitudes, colour, couplings, models and diagrams for non-vacuity first, then the other three points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/tests/`: `amplitude_oracle`, `color_cf`, `color_cf_oracle`, `color_flow_tags_oracle`, `coupling_oracle`, `decay_chain_census`, `decay_widths`, `diagrams`, `finite_field_msq`, `gluon_parke_taylor`, `helas_kernel_composition`, `polarization_census`, `polarization_frame`, `proc_grammar_oracle`, `reweight_mg_oracle`, `schannel_census`, `sm_interned_blob`, `smeftsim`, `standalone_jamps`, `toy_models`, `ufo`, `validate_helas`, `validate_madgraph_diagrams`, `validate_scale_couplings`; and `vibegraph-lib/benches/` (about 15k lines).

**Already claimed, read and don't re-report:** the `tests/amplitude_oracle.rs` site in evaluator-doc-comments-stale; config-amp-phase-and-sign-unpinned; smeftsim-vendored-checksum-not-hermetic. The protocol's backlog check finds the unclaimed items filed against these files.

**Look especially at:** each oracle's blind spot (AGENTS.md "Every oracle has a blind spot"): for every gate, what error class it provably cannot see, and whether some other test covers that class. Census tests whose expected counts were copied from the output they check. Shared helpers duplicated across test files that belong in `tests/common/`.
