---
type: Backlog Item
title: "A fixed-beam card whose subprocesses' decays carry different legs now goes pairing-weighted, unmeasured against MadGraph"
description: "The fixed-beam path reads one ForcedResonances over every subprocess; such cards moved from cutting every leg to per-subprocess exemptions in v0.2.1, a σ change no row measures."
area: validation
state: open
priority: low
closes_when: "One fixed-beam σ row (e.g. e+ e- > d d~ z, z > u u~ with add process e+ e- > z u u~, z > d d~, cut_decays = F) is compared with MadGraph over ≥ 5 seeds a side."
blocked_by: []
opened: 2026-11-12
tags: [cuts, decay-chains, fixed-beam, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: jw1, resource: "../../sprints/v021-fixes/sessions/J-W1-report.md", title: "J-W1 report, Found 3"}
---
Found by J-W1 (Found 3). The fixed-beam path (`integrate.rs`, `generate.rs`)
reads one `ForcedResonances` over the diagrams of every subprocess. That
predates v0.2.1. Under the old intersection rule, a fixed-beam card whose
subprocesses' decays carry different legs had every leg of every subprocess
cut. One example is `e+ e- > d d~ z, z > u u~` with
`add process e+ e- > z u u~, z > d d~`.

Since the pairing-weighted cut, such a card goes weighted, and each
subprocess's single pairing gets its own exemption. That is MadGraph's
behaviour on those subprocesses, but no row measures the σ change.
A lepton-collider core that shares the decay's quarks
(`e+ e- > z u u~, z > u u~`) is refused at enumeration as ambiguous.
