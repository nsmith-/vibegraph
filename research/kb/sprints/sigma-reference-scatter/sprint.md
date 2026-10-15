---
type: Sprint
title: "Reference σ seed scatter"
description: "Give every gated σ reference independent MadEvent seeds and a second budget, record their scatter in the bundle, and size the σ gates on it; regenerate the llj MLM references from fresh directories."
status: draft
active: true
tags: [validation, refdata, madgraph, seeds, convergence, mlm, llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
## Goal and exit criteria

The hadronic σ references are mostly one MadEvent run at one budget, and
the MLM references' extra seeds share a directory; MadEvent seeds have scattered
at χ²/dof 3–14 at the 0.1 % level. The three llj MLM rows all sit +0.03 to
+0.23 % high, which is the concrete case. Done when every gated σ reference
has independent seeds (fresh directories) and a second budget, the bundle
records each row's seed χ²/dof and budget shift, the σ gates use an error that
covers the measured scatter where it exceeds the quoted one, and the llj
offset is attributed.

The two items share the generators (`gen_mlm_references.sh`,
`gen_hadronic_sigma.sh`), the bundle schema and the σ gates, so one review
reads them as one change. The design (D1) needs the user's approval before
implementation, and the re-banked references come from the user's M3 run.

## Scope

Backlog items claimed (one `Backlog: <slug>` line each in the draft PR):

- [reference-sigma-seed-scatter-untested](../../backlog/validation/reference-sigma-seed-scatter-untested.md)
- [llj-mlm-rows-sit-high-together](../../backlog/validation/llj-mlm-rows-sit-high-together.md)

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
| [D1](decisions/D1-reference-error-design.md) | manager | V1 measurements from sibling PRs | user approval | — |
| [V1](sessions/V1.md) | validation-dev (Opus) | — | diagnostic only | — |
| F1 | feature-dev (Opus) | D1 approved | hermetic + `--skip-deps validate` unmoved (inert until re-banked) | — |

Further sessions (a fix, its validation, performance where it applies, then
hygiene) are added once the diagnosis says which side owns the defect.
