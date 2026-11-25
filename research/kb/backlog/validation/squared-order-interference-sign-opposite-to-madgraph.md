---
type: Backlog Item
title: "VLQ^2==2 interference σ has the opposite sign to MadGraph on TprimeBToTH cards 612 and 614"
description: "At campaign budget vibegraph gives +4.78e-4 pb (612) and +7.83e-4 pb (614) for the VLQ^2==2 term; MadGraph's signed σ is −1.10e-3 and −5.95e-4. The |M|² order classes sum consistently."
area: validation
state: open
priority: high
closes_when: "A per-diagram × per-helicity complex dump of 'g u > H t b~ d' at one point, against MadGraph 3.7.1 standalone with matched parameter provenance, locates the relative sign or phase between the VLQ-order-0 and VLQ-order-2 amplitudes. The fix is checked arithmetically on the dump, a test pins the interference sign against MG, and 612/614 agree with MG's signed σ over 5 seeds. Or the dump shows the amplitudes agree and the discrepancy is in the reference."
blocked_by: []
opened: 2026-11-25
tags: [squared-orders, interference, sign, vlq, madgraph-parity, cms, release-blocker]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: method, resource: "../../validation/bit-exact-amplitude-debugging.md", title: "Bit-exact amplitude debugging"}
---
Found by the sprint L validator on PR 1's request A (bin-pr1 9f196713). The run
used 5 seeds at campaign budget (fixed, neval 120000, niter 3):

| card | vibegraph σ (pb) | sd | MG signed σ (pb) | ratio |
|---|---|---|---|---|
| 612 | +4.779e-4 | 4.1% | −1.101e-3 ± 3.19e-5 | −0.43 |
| 614 | +7.828e-4 | 3.1% | −5.95e-4 ± 2.97e-5 | −1.32 |

The order classes are self-consistent on 612 (seed 1). VLQ enters the amplitude
only at orders 0 and 2:
- `^2==0`: 0.065912;
- `^2==2`: 0.000472;
- `^2==4`: 0.001281.

Their sum, 0.06767, matches `VLQ<=2`, 0.06719 ± 0.0004, at 1.1σ. `^2<=2` matches
`==0` plus `==2` at 1.1σ. So the selection partitions |M|² correctly. The |M|²
pieces cannot see a relative sign or phase between the two amplitude classes, but
the interference term can. That fits a relative-sign error between diagram
classes, such as a fermion-flow or vertex convention, not a selection bug. This
is a hypothesis until the dump settles it.

## The reference was wrong too

Earlier comparisons read vibegraph "10–16× low", against the wrong reference.
gpdb's `choose_reference` rejects the signed gen_log σ because it is negative,
and falls back to `results.dat` column 1. For a negative-weight run, that column
is the absolute integral. The same error affects 15 squared-order interference
cards:
- VLQ: 124, 125, 610, 612, 614;
- VBS EFT interference: 707–709, 724–727, 760, 762;
- 898.

Every earlier ratio on those cards is void until their references are
re-derived.

## Release impact

If the dump confirms a sign error, every squared-order selection that keeps an
interference term between amplitude classes may be wrong in sign. `generate`
already refuses these selections, because their weights are negative, but
`integrate` reports their σ. v0.2.3 should not tag with this open unless the
user decides to ship it as a known issue.
