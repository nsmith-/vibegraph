---
type: Backlog Item
title: "g2HDM charged-Higgs cards read 2.4–4.4% above MadGraph at high mass"
description: "In v022-1009, g2HDM charged-Higgs (cgbh, rhotu00_rhott06_rhotc04) M300 and M450 pass, but M900–M1750 read +2.4% to +4.4% on every seed (mean pulls 6–8, χ²/dof 0.6–1.9). A steady offset."
area: validation
state: open
priority: high
closes_when: "On one high-mass card (M1000 or above), the excess is attributed against MadGraph per event (scale, merge sequence, Breit–Wigner window or width) or per diagram, not from σ alone. It is then fixed, so the high-mass cards match MadGraph by the multi-seed criterion, or registered as a deviation with its predicted size."
blocked_by: []
opened: 2026-11-22
tags: [g2hdm, mlm, sigma, widths, madgraph-parity, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-22}
sources:
  - {id: log, resource: "../../sprints/v022-smeft-and-refusals/log.md", title: "Sprint K log, v022-1009 verdicts"}
---
CMS cards `g2HDM_charged_higgs_cgbh_M*_rhotu00_rhott06_rhotc04` (889–895).
These are the MLM cards whose v0.2.1 panic #153 fixed. Results from the
post-release campaign `v022-1009` on v0.2.2, 5 seeds, unadapted, against
MadGraph σ × `small_width_ratio`:

| mass | vg/MG | note |
|---|---|---|
| M300 (889) | 1.0039 | pass, mean pull 0.87, MG 10.04 ± 0.039 pb |
| M450 (893) | 1.0064 | pass |
| M900 | 1.025 | mean pull 6–8 |
| M1000 | 1.024 | mean pull 6–8 |
| M1250 | 1.044 | mean pull 6–8 |
| M1500 | 1.041 | one seed timed out at 8 h |
| M1750 | 1.035 | one seed timed out at 8 h |

Every high-mass seed reads high, and χ²/dof is 0.6–1.9, so this is not VEGAS
missing a region.

Candidates from the CERN host:
- the small-width or Breit–Wigner window (the verdict applies
  `small_width_ratio`);
- a width that the param card leaves to `Auto`.

A card with `DECAY … Auto` should be refused today, so first check which widths
the card actually sets, and the value each one takes.

Cost: 2.5–8 h a seed on 4 cores, with about 3.0–3.4 GB peak RSS at the default
target. A single high-mass card under a per-event comparison is the affordable
oracle.

**Claimed for sprint L as a stretch goal** (PR 12, user decision,
2026-11-22). It does not hold the v0.2.3 tag. It is scheduled after sprint L's
VBS work (`vbs-stable-boson-sigma-reads-high`), since the two may share a
cause. Multi-seed σ runs go to the CERN host. Local work is limited to
reduced-budget single seeds and per-event checks.

**MG 3.7.1 check and card 967 (2026-11-22, vg-validator).** MG 3.7.1 agrees with 2.9.18 on the gridpack cards (M1000 1.0041, M300 1.0000), so M1000 is +2.8% over 3.7.1 in vibegraph. Re-resolved Auto widths move Γ(H±) by 0.005%; the `goldstone` flag is read correctly. Card 967 (`SM_alp_UFO`, ALP → ττ, m = 20 GeV, Γ/m = 5%, MLM) reads +1.2% (4.4σ) over MG 3.7.1's 1.864e-5 ± 4.6e-8 pb, MG versions agreeing; it moves here from `sigma-reads-high-on-mlm-and-wbq-cards` as a cheap second wide-resonance probe.
