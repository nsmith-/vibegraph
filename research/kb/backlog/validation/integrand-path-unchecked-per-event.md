---
type: Backlog Item
title: No gate replays MadEvent's events through the integrand's own path
description: "validate_mlm_dumps feeds MadEvent's CM momenta, configuration and flavour into this side's factors, so the integrand's own frames, draw and member average go unchecked."
area: validation
state: open
priority: medium
closes_when: A gate rebuilds each dumped MLM event from (x1, x2, CM finals) through the integrand's own path (build_frames, the configuration draw, per_group_sum, term_at) and compares per factor, with a mutation that the lab-frame clustering would have failed.
blocked_by: []
opened: 2026-10-15
tags: [mlm, oracle-blind-spot, integrand]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
sources:
  - {id: v1, resource: "../../sprints/sigma-ttx-mlm-at1/sessions/V1-report.md", title: "sigma-ttx-mlm-at1 V1, Found 2"}
  - {id: v2, resource: "../../sprints/sigma-ttx-mlm-at1/sessions/V2-report.md", title: "sigma-ttx-mlm-at1 V2, Found 3"}
---
[The MLM dump oracle](../../validation/mlm-dump-oracle.md) agrees factor by factor
on every event MadEvent writes, yet the integrand clustered in the lab frame
for months. The dump replay hands this side MadEvent's own CM momenta,
`iconfig` and flavour, so the steps the integrand does differently (frames, a
configuration drawn ∝ `AMP2`, the luminosity-weighted member average) are
never exercised. A per-point `DSIG1` driver exists in the V2 scratch.
