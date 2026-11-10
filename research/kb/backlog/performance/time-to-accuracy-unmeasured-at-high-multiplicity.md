---
type: Backlog Item
title: Time to accuracy against MadEvent is unmeasured at 2→4 and above
description: "The time-to-0.1% comparison has one 2→4 row and one MLM row and nothing above 2→4; add banked 2→4 and 2→6 rows, partonic and hadronic, with a release whose Linux assets scale with threads."
area: performance
state: open
priority: medium
closes_when: "integration-vs-madgraph.md carries time to 0.1% on σ against MadEvent, on one host in one sitting at a recorded release tag, for at least three 2→4 rows and two 2→6 rows (one each hadronic), with ≥3 seeds per row, single-core CPU and wall clock at equal core count on both sides, iterations and evaluations per seed, and the σ pull against MadEvent. A row that cannot reach 0.1% within a stated cap is reported at the accuracy both sides reached, never dropped."
blocked_by: []
opened: 2026-11-10
tags: [performance, madgraph-comparison, integration, time-to-accuracy, multiplicity, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: int, resource: "../../performance/integration-vs-madgraph.md", title: "Integration against MadGraph: throughput and time to accuracy"}
  - {id: protocol, resource: "../../performance/end-to-end-timing-protocol.md", title: "End-to-end timing protocol"}
---
The time-to-accuracy rows stop at 2→3, apart from `ud_to_epemud_qcd0` (2→4)
and `pp_to_ll_0j2j_mlm` added for v0.2.0. Per point, vibegraph's advantage
over MATRIX1 is largest at high multiplicity (0.57–0.84× on the 2→4 and
2→6 rows at x86-64-v3), but whether that survives to time to accuracy, where
the multichannel map and the channel count dominate, is not measured.

Use a release that carries the musl allocator fix
([musl allocator thread scaling](../../performance/musl-allocator-thread-scaling.md)):
an equal-cores wall-clock column is part of the measurement, and the v0.2.0 Linux
assets run slower at `-j 4` than at `-j 1`.

Rows with a banked MadEvent σ to start from (`validation/manifest.toml`):
- 2→4: `ud_to_epemud_qcd0`, `ee_to_mumu_tata_qcd0`;
- 2→6: `uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0`;
- matched: `pp_to_ttx_0j1j_mlm` (MLM, up to 2→3 at the matrix element).

Missing, and needed for the hadronic half: a proton-beam 2→4 such as
`p p > e+ ve mu- vm~` and a proton-beam 2→6 such as dileptonic
`p p > t t~, (t > b l+ vl), (t~ > b~ l- vl~)`. Each needs a MadEvent
reference run, made in the same sitting as the timing, not banked.

Follow the protocol's §5 (MadEvent's cumulated_time scaled to the δ each of
our seeds reached) and, as for the overshooting v0.2.0 rows, also report
time to 0.1% as a user sees it where a run's iteration floor carries it past
the target.
