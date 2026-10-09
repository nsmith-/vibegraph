---
type: Backlog Item
title: Several validation/manifest.toml notes describe a superseded state
description: "Manifest notes call the unary-minus defect live, count 90000 kt-dump events, credit qqx_to_o8o8 to a reverted rule, and misstate the gu_to_epemu and ud_to_epemud gates."
area: hygiene
state: open
priority: medium
closes_when: "Each manifest note listed in the body states what its gate currently does and measures."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, manifest, validation-report, hygiene-sprint]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: manifest, resource: "../../../../validation/manifest.toml", title: "validation/manifest.toml"}
---
The manifest's notes are rendered into the validation report, so a stale note
misstates a gate there. Each was checked on 2026-10-09:

- **`couplings-mg` standalone note** (~:1443): describes `-ee**2/(2.*cw)` being read as `(-ee)**2/…` as a live `KNOWN_CRATE_DEFECTS` entry. The list is empty (`vibegraph-lib/tests/coupling_oracle.rs` ~:120-130).
- **kt-cluster standalone rationale** (~:1378): "90000 events". `kt_cluster_dump_manifest.json` pins eight runs of 10000 (`pp_to_llj_qcd2_qed2` retired).
- **`qqx_to_o8o8_toy_dcolor` amplitudes note** (~:1013): says a Dirac-matrix-free line "takes no per-propagator reversal flip … It is the falsifier for that rule". Since 7f523ad the row passes through the `SSS1`/`SSSS1` scalar-sink −1, and the line takes the propagator sign.
- **`gu_to_epemu`, `gux_to_epemux` integrals notes**: "rel_tol 0.02 … The pull (+5.21) is reported and NOT asserted". `validate_sigma.rs` gates both at `rel_tol 0.005` (~:510-514), and `PULL_REPORTED_NOT_ASSERTED` holds only `ee_to_mumua` (~:199).
- **`ud_to_epemud_qcd0` amplitudes note** (~:479): "every one of the 35 diagrams against MadGraph's bare AMP() at 6.6e-15 under a single global phase". The row is multi-flow (NCOLOR = 2), banks `jamp_coefficients: null`, and so runs no per-diagram fit (`amplitude_oracle.rs` ~:968). The 6.6e-15 is the per-configuration check, and `G` comes from the per-flow fit.

Found by drafters D1 and D8, verifiers V1 and V3, and the Phase 3 cross-fix pass.
