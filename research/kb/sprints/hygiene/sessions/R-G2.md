---
type: Session Brief
title: "R-G2: hygiene review of the sampling, scale, PDF and event test targets"
description: "Review the vibegraph-lib integration tests on sampling, scales, PDFs, MLM and events, and the manifest notes, for non-vacuity first, then the other three points."
status: draft
agent: claude (Opus), read-only
depends_on: [V1b]
closes: []
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
---
Follow the [review protocol](review-protocol.md). Review the sprint branch
at V1b's commit (the dispatch names the commit).

**Cluster.** `vibegraph-lib/tests/`: `alphas_reference_grid`, `decay_chain_ladder`, `diagram_channel`, `rambo_flat_mc`, `rambo_oracle`, `scales_run_cards`, `validate_alphas`, `validate_hadronic`, `validate_kt_cluster`, `validate_lhef`, `validate_mlm_dumps`, `validate_pdf_grid`, `validate_samples`, `validate_scales`, `validate_sigma`, `validate_unweighting`, `validate_vegas`, `common/`; and `validation/manifest.toml`'s notes (about 25k lines).

**Already claimed, read and don't re-report:** validation-test-comments-stale, validate-scales-module-doc-stale, validate-hadronic-calibration-comments-superseded, manifest-notes-describe-superseded-state, jj-banked-orderings-eta-uses-wrong-components, and the test-file sites in madgraph-line-citations-predate-pin. Not claimed but known: llj-gate-comments-quote-pre-floor-ladders and sigma-calibration-comments-stale, so don't re-report them either.

**Look especially at:** σ gates whose tolerance and seed count are justified by a quoted measurement, and whether that measurement is recorded anywhere (AGENTS.md "A report is only evidence if every green cell is a recorded measurement"); exception lists (`PULL_REPORTED_NOT_ASSERTED`, `TIE_BREAK_MISSES`, and the like) whose entries no longer need the exception; `validate_sigma.rs` and `validate_hadronic.rs` (about 5k lines each) as candidates for shared gate scaffolding.
