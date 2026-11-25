---
type: Backlog Item
title: "VLQ^2==2 interference on 612/614: confirm main's signed σ against MadGraph (the opposite sign came from a pre-#169 binary)"
description: "The campaign binary bin-pr1 9f196713 lacked #169's chirality fix and gave VLQ^2==2 the wrong sign. On main, L-I1 matches MG standalone per diagram; 612/614's σ on main is not yet measured."
area: validation
state: open
priority: medium
closes_when: "Main, at or after 3f4dd815, on cards 612 and 614 with their own gridpack cards at campaign budget over 5 seeds, gives a negative σ consistent with gpdb's signed MG reference (612 −1.101e-3, 614 −5.95e-4 pb), read against the seed spread. If it misses by more than the spread, MG 3.7.1 at hard_survey = 2 on 612 decides which side is off. The same run covers PR 1 request A's other VLQ rows, 124, 125 and 610."
blocked_by: []
opened: 2026-11-25
tags: [squared-orders, interference, sign, vlq, chirality, madgraph-parity, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: l-i1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-I1-report.md", title: "L-I1 report: per-diagram VLQ split against MG standalone"}
  - {id: method, resource: "../../validation/bit-exact-amplitude-debugging.md", title: "Bit-exact amplitude debugging"}
---
## What was seen

PR 1's request A (bin-pr1 9f196713) ran at campaign budget over 5 seeds. It gave
612 +4.78e-4 pb and 614 +7.83e-4 pb for `VLQ^2==2`, against MadGraph's signed
−1.101e-3 and −5.95e-4.

## Diagnosis (L-I1)

9f196713 does not contain a5bf8fca (#169, the charged-scalar chirality fix):
`git merge-base --is-ancestor a5bf8fca 9f196713` fails. Main contains it.

L-I1 compared against MG 3.7.1 standalone on `g u > h t b~ d`, which has 19
diagrams, 5 of them VLQ=2, at 10 points, 6 of them near the T' pole:
- **Main:** G0 = G2 = −i, and G2/G0 = 1 to 1e-15. Every diagram pairs with one MG
  graph. |M0|², |M2|² and 2Re(M0·M2*) all agree to 1e-12.
- **9f196713:** G2/G0 is 0.31–0.55, with residuals up to 1.29. |M2|² is off by
  0.27–1.83×. This is not a pure sign.
- **Carriers:** the 5 VLQ=2 diagrams. They share `V_24 [t~, tp, H]`, an FFS2 bare
  ProjM vertex on a non-self-conjugate pair, which is #169's bug class.
- **Attribution:** removing only the projector exchange from main reproduces
  9f196713 bit for bit.
- **Pruning:** ruled out. Runs with and without it are identical.

The earlier reasoning ("the classes partition, so only the interference is
wrong") checked vibegraph against itself. Against MG, |M2|² itself was wrong.

On main, a reduced-budget run of 612 with card 126's run card gives σ = −937 ±
27 ab against MG's −1101 ± 32 ab. The run card is not 612's own, so only the
sign counts.

## The reference

gpdb used to compare negative-weight runs against `results.dat`'s absolute
integral. nsmith-/vibegraph-valdb#3 makes it use the signed σ. The live DB was
migrated to schema 5 on 2026-11-25, and exactly 15 references changed: 124,
125, 610, 612, 614, 707–709, 724–727, 760, 762 and 898. Ratios on those cards
from before the migration are void.

Five NLO cards, including 227, 228 and 230, take their reference from column 1
of an aMC@NLO `results.dat`. What that column holds there has not been checked.
