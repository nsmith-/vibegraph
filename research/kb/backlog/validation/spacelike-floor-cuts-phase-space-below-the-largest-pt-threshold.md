---
type: Backlog Item
title: "The spacelike floor takes the largest single-leg pT² and can cut away phase space next to a softer jet"
description: "Cuts::spacelike_floor bounds every peripheral rung by the largest single-leg pT_min² (e.g. pta² = 1600 GeV²); with ptj 0.1 σ reads ~6% lower than with the smallest threshold."
area: validation
state: open
priority: high
closes_when: "Each peripheral rung's spacelike bound is proved from the thresholds of the legs it can reach (or is the smallest one), and σ at pta 40, ptj 0.1, draj 0.4 matches local MadGraph by the multi-seed criterion, with a test that fails on the largest-threshold rule."
blocked_by: []
opened: 2026-11-07
tags: [phase-space, cuts, t-channel, bias, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hb1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-B1-report.md", title: "H-B1 report, Found 2"}
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log: the CERN host's CKM agent"}
---
`Cuts::spacelike_floor` (`vibegraph-lib/src/cuts.rs`, about line 1031) takes
the largest single-leg `pT_min` over all final legs and applies its square as a
hard `t ≤ −floor` support bound on every peripheral rung of a diagram channel.
On `p p > a j j` with `pta = 40` and soft jet thresholds, that caps a rung next
to a jet whose own threshold is far lower. The docstring claims the bound always
holds for three-body final states; on this evidence it does not.

Measured in sprint H (one or two seeds each, so not yet calibrated):
- **The CERN host's CKM agent.** At `pta 40`, `ptj 0.1`, `draj 0.4`, stock
  vibegraph gives 52.64 and 51.96 (± 1.1). A diagnostic build using the smallest
  threshold gives 56.45 and 54.82 (± 1.4–1.5), about +6%. At `ptj ≥ 1` the shift
  is within errors (1.009, 1.012).
- **H-B1.** About −1.2% at `ptj ≤ 1`, from single seeds.

No MadGraph reference exists at `ptj 0.1` yet. To close: run local MadGraph at
`pta 40`, `ptj 0.1`, `draj 0.4`. Then derive the bound per rung (from the legs
adjacent to it), or fall back to the smallest threshold. Compare both against
MadGraph over ≥5 seeds.

[spacelike-floor-too-loose](../performance/spacelike-floor-too-loose.md) is the
opposite concern, a floor below where cut-surviving |t| starts, and is a
performance item.
