---
type: Backlog Item
title: The decay-chain census cannot carry a BSM card without the model in the repository
description: "TTToHcToWA, TpTp, VLLD and VLLVLL matched MadGraph only in a scratch harness, because their UFO models (2HDM_UFO, eVLQS012_v8_4FNS_UFO, VLL) are not in the repository and their licences are unchecked."
area: validation
state: open
priority: low
closes_when: "Each model's licence is checked and recorded; the models whose licence allows it are vendored (or fetched by a pinned, checksummed path), the census loads a non-SM model through UFOModel::load, and rows for the BSM survey families pin MadGraph's per-entry diagram counts and doubly reached contents."
blocked_by: []
opened: 2026-11-05
tags: [decay-chains, census, bsm, licensing, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: report, resource: "https://github.com/nsmith-/vibegraph/blob/02bea373ea0e6287f745ad43d3d3f832dbd4bc9a/research/kb/sprints/cms-gridpack-coverage/sessions/H-D3-report.md", title: "Sprint H session H-D3 report, §6 and Found"}
  - {id: census, resource: "../../../../vibegraph-lib/tests/decay_chain_census.rs", title: "vibegraph-lib/tests/decay_chain_census.rs"}
---
Sprint H matched MadGraph entry for entry on every BSM survey card's decay
chains, 890 subprocesses of which 417 are reached by two decay assignments.
The comparison ran only in a scratch harness. The banked census
(`tests/decay_chain_census.rs`) cannot load a model that is not in the
repository. The cards need 2HDM_UFO (TTToHcToWA), eVLQS012_v8_4FNS_UFO (TpTp)
and VLL (VLLD, VLLVLL).

Check each model's licence first: vendoring is allowed only where the licence
permits redistribution. Otherwise, fetch the model through a pinned,
checksummed download as the PDF sets are. The Rust side needs only an
`UFOModel::load` path, a few lines per the H-D3 harness.
