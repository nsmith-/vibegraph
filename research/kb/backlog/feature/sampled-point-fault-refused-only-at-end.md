---
type: Backlog Item
title: "A sampled-point fault is refused only after the survey or the whole run"
description: "A SampledPointFault is refused only after the survey or run (generate: after the event file is written); raising the run's StopSignal would stop at the next iteration."
area: feature
state: open
priority: medium
closes_when: "A recorded SampledPointFault stops the run at the next iteration boundary through the StopSignal path, generate leaves no complete-looking event file, and a test drives a fault and checks both."
blocked_by: []
opened: 2026-11-19
tags: [refusal, batch, generate, integrate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-M1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-M1-report.md), Found 3).

K-N1 confirms the `generate` half on both paths: `emit_to` renames its
partial file to `--out` before `refuse_sampled_fault` runs
([K-N1 report](../../sprints/v022-smeft-and-refusals/sessions/K-N1-report.md), Found).
