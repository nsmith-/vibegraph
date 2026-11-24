---
type: Backlog Item
title: "A process line that the cuts zero is reported only as unscanned channels"
description: "With ptj2min > 0, an MLM sum's Z+1j line has σ = 0 (MadGraph's cuts.f does the same), but the user sees only a weight-scan warning that wrongly calls its share of σ missing."
area: feature
state: open
priority: medium
closes_when: "integrate reports σ per process line (each generate / add process) in its integrated record, and when the cuts reject every point of a whole @N process or multiplicity part, integrate names that process and the cut that zeroes it (here ptj2min against a one-jet final state) as a warning, and generate's weight-scan warning no longer calls a zero-σ channel's share missing; a test pins both on a p p > z j / p p > z j j card with ptj2min set."
blocked_by: []
opened: 2026-11-24
tags: [cuts, run-card, mlm, warnings, user-facing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
sources:
  - {id: mg-cuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/3.7.1/Template/LO/SubProcesses/cuts.f#L812-L820", title: "MadGraph 3.7.1 cuts.f: too few jets for an active ptjNmin fails the point"}
---
simshop's v0.2.2 gridpack for `generate p p > z j, z > b b~` +
`add process p p > z j j, z > b b~` (MLM, `ptj = 20`, `ptj2min = 20`)
writes no Z+1j event. Its `<init>` declares process 1 with `XSECUP = 0`, and
generate warns that 3 of 18 channels "produced no point in the weight scan …
their share of the banked cross section is missing from the sample".

The physics is MadGraph's: `cuts.f:812-820` fails every point with fewer than
two jets when `ptj2min > 0`. With `maxjetflavor = 4` the Z's b quarks are not
jets, so the one-jet line can never pass. The defects are the reporting:

- Nothing says a whole process line was zeroed, or by which cut.
- The scan warning calls the line's share of σ missing. That share is zero,
  which is the same wording problem as
  [weight-scan-warning-counts-vanishing-channels-as-lost](../hygiene/weight-scan-warning-counts-vanishing-channels-as-lost.md).

The user asked on 2026-11-24, relayed through simshop, for σ per process line
in `integrate`'s `integrated` record, so a production tool can flag a zeroed
line itself. The field name and shape are open.
