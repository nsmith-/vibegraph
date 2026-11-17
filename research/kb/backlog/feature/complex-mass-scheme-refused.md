---
type: Backlog Item
title: "set complex_mass_scheme is refused"
description: "9 CMS TprimeB cards set complex_mass_scheme; vibegraph refuses it. Run without it, the wide-width (Γ/M 10–30%) cards read 3–40% low; the narrow M1200 card agrees."
area: feature
state: open
priority: medium
closes_when: "The complex-mass scheme is implemented as MadGraph does it (complex masses in propagators and the couplings derived from them), with per-diagram amplitudes against MadGraph standalone under the scheme and σ on a wide-width TprimeB card by the multi-seed criterion, and the 9 cards integrate."
blocked_by: []
opened: 2026-11-17
tags: [complex-mass-scheme, widths, proc-card, refusal, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
The CERN host's survey refused gridpacks 126, 127, 611, 613 and 615–619
(TprimeBToTZ/TH) on `set complex_mass_scheme`. Its driver stripped the line,
and the wide-width cards then read 3–40% low against MadGraph's σ, while the
narrow M1200 card passed. The refusal was correct.

Gridpack 126 is also the repro for
[`multichannel-subdominant-channel-carries-dominant-region`](../validation/multichannel-subdominant-channel-carries-dominant-region.md).
That item's sampling miss was measured with the scheme stripped, so it should
be re-checked once this lands.
