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

Gridpack 126 was also the repro for a multichannel sampling miss, closed in
sprint L ([L-P1 report](../../sprints/v023-orders-majorana-timeouts/sessions/L-P1-report.md)).
Its closing comparison used a MadGraph σ with the scheme off; on that card,
MadGraph 3.7.1 measures the scheme's own effect at −0.04%.

## The interference cards, and a confound in the 3–40% reading (2026-11-25)

The scheme is recorded on 14 VLQ gridpacks, not 9. The five TprimeBToTH
interference cards (124, 125, 610, 612, 614, `VLQ^2==2`) also set it. gpdb's
attempt 2 strips the line (`strip:set`), so every vibegraph σ on those cards so
far is real-mass against a complex-mass MG reference.

Main (ff45724c) at campaign budget, 5 seeds, against the signed references:
- 124, 125 and 610 (5–10% widths) read 0.958–0.997.
- 612 (M800, Γ = 160) reads 0.807 ± 0.032.
- 614 (M700, 20%) reads 0.523 ± 0.060.

The "3–40% low" above was measured before sprint L's PR 5 fixed the
multichannel sampling miss on this family. On PR 5's head the wide TZ cards
615–618 read 0.987–1.012 against the same complex-mass references. MG 3.7.1 puts
the scheme's own effect on 126 at −0.04%. So the scheme's size on the
pair-production cards is not established, and on the interference cards, where
σ is a cancellation, it may be large or small. MG 3.7.1 on 612 with the scheme
on against off, and everything else identical, measures it directly.

### Measured: the scheme accounts for 612/614's gap against their references

MG 3.7.1 on each card's own cards, with zerowidth_tchannel True and
hard_survey 2, run with the scheme off and then on:

| card | MG off | MG on | on/off | MG on / gridpack reference | vibegraph main 8× / MG off |
|---|---|---|---|---|---|
| 612 | −8.912e-4 (3 seeds, sd 3.4%) | −1.094e-3 (3 seeds) | 1.228 | 0.993 | 0.986 ± 0.019 |
| 614 | −3.078e-4 (8 seeds, sd 11.2%) | −5.579e-4 (3 seeds) | 1.74 ± 0.15 | 0.938 ± 0.06 | 0.982 ± 0.044 |

vibegraph agrees with MG in the real-mass scheme on both cards. Against
complex-mass references, the VLQ interference cards are not comparable until
the scheme is supported.

Sprint L's L-I1 report (fixed-point bank) explains the sensitivity. Near the
pole the scheme rotates the T' propagator's phase, so the interference moves by
O(1) while |M0|² and |M2|² move about 1%. That is why pair production barely
feels the scheme.
