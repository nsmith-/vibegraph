---
type: Sprint
title: "Proton runs with unequal beam energies"
description: "A proton run with unequal beams uses √s = 2√(E1E2), partons at x_i·E_i and MadGraph's cm_rap boost, bit-identical for equal beams, pinned by a unit test and a banked unequal-beam MadGraph σ row."
status: draft
active: true
tags: [beams, kinematics, proton, madgraph-parity, sprint-d]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-21}
---
## Goal and exit criteria

Done when the item's `closes_when` holds. Its banked unequal-beam σ row needs
a MadGraph run (M3 only): this PR carries the banking request, and the item
closes with that row or by the user's call.

## Scope

Backlog items claimed (one `Backlog: <slug>` line in the draft PR):

- `collider-energy-sum-of-beam-energies`

Split out of the cut-refusal PR (#59) by the user's decision of 2026-10-21.
The item was filed on that PR's branch, so this branch starts from it
(stacked on #59) and merges `main` in once #59 lands.

The design was scoped by the cut-refusal F2 session
([its report](../cut-refusal/sessions/F2-report.md), part 3): the
fixed-energy path is already exact; on the proton path, √s, the parton
energies, the lab boost, the lab-frame cuts and the written momenta all
assume equal beams. Equal-beam runs stay bit-identical: 2√(E1E2) equals
E1 + E2 exactly when E1 = E2, and every validated row has equal beams.
MadGraph's massive stot (0.938 GeV beams) is not used for √s, because it
would move every bit-for-bit proton row by an ulp.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| F1 | feature-dev (Opus) | – | hermetic, fmt, clippy, kb-lint; equal-beam bit-identity | the item (subject to banking) |
| V1 | validation-dev | F1 | banked σ gates unmoved (CI); banking request | – |
| HY1 | hygiene-dev (Sonnet) | V1 | hermetic, fmt, clippy | – |
