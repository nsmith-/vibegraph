---
type: Sprint
title: "H1 census of banked grouped rows"
description: "List every banked grouped MadEvent row with a clustering scale and non-identity symmetry permutations, measure H1's share on each affected one, and re-bank or register it."
status: draft
active: false
tags: [h1, madevent-reference, scales, grouping]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

Grouped MadEvent's first `setclscales` call clusters the unpermuted `PP`
while this crate clusters `P1` (H1, a registered deviation). R1 measured −4.0 %
on an unmatched row. Done when every banked grouped row using a clustering
scale (`dynamical_scale_choice = -1` or matching) has been checked for
non-identity permutations, and each affected one is re-measured against an
unbiased reference (a user banking run with the first-call patch or
`group_subprocesses False`) or carries H1 as a registered deviation. The
vectorised path (`update_scale_coupling_vec`) is checked too.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- `h1-ickkw0-grouped-rows-unchecked` (closed by this PR; see [closeout](closeout.md))

Sibling PRs of the same planner hand-off ("cross sections that disagree
with MadEvent, cause unattributed") are `sigma-2to4-offset`, `sigma-ttx-mlm-at1`,
`sigma-h1-ickkw0-census`, `sigma-reference-scatter`, `sigma-ee-mumua`,
`sigma-gg-cg` and `sigma-uux-wbwb-veto`. Five of the seven offsets put this
side high, which makes reference-side under-coverage a candidate common cause;
`sigma-2to4-offset` is the cheapest test of it and its result is relayed to the
others as a watch item.

**Banking.** Diagnostic MadEvent runs happen here; new banked reference
data is produced only by the user on the M3 host. A banking need becomes one
comment on this PR listing exactly what to run.

## Sessions

| Session | Agent type | Depends on | Gate | Closes |
|---|---|---|---|---|
| [V1](sessions/V1.md) | validation-dev (Opus) | — | census + diagnostic runs | h1-ickkw0-grouped-rows-unchecked |
| [HY1](sessions/HY1.md) | hygiene-dev (Sonnet) | V1 | census script fails loudly; `runs` byte-identical | — |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
