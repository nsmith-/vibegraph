---
type: Backlog Item
title: "Ignored probes are unmaintained: several fail on their own stale premises"
description: "Four ignored probes fail on premises the code no longer meets (pre-sprint-I), two ignore doctests always fail under --ignored, and two probe names collide across test binaries."
area: hygiene
state: open
priority: medium
closes_when: "Every ignored test is classed as gate, probe or tool; each defunct probe is fixed or deleted; ignore doctests become text or no_run; no two ignored tests share a name; and an --ignored run fails only on documented pending gates."
blocked_by: []
opened: 2026-11-09
tags: [tests, probes, hygiene, validate-deep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: iz2, resource: "../../sprints/v02-last-refusals/sessions/I-Z2-report.md", title: "I-Z2 report"}
---
I-Z2's classification of all 110 ignored tests is in its report §5. These
fail identically on d87ad3ae and 2b26c109:
- `validate_hadronic::probe_cluster_scale_spread_over_configurations`: it
  assumes the clustered configuration is the drawn channel's. The integrand
  draws it ∝ AMP2 from the trailing uniform.
- `validate_hadronic::probe_scale_draw_cost`: it assumes `sde_strategy = 2`
  turns the configuration draw off.
- `validate_samples::probe_pta_windows_against_madgraph`: it builds points
  without the scale-draw uniform.
- `validate_scales::probe_first_channel_cost_in_alpha_s`: it assumes every
  clustered run has SMINPUTS 3. Six toy-model runs don't.

The `validate_sigma` namesakes of the first two were not run. Check them for
the same premise.

Also:
- the `ignore` doctests in `diagrams/mod.rs:17` and `helas/repr/vectorspace.rs:17`;
- the duplicate names, which stop libtest's `--skip`/`--exact` from selecting
  one of a pair;
- the five scatter-guard calibrations and 30 `validate_sigma` probes not run
  in sprint I, on the user's call that most probes are defunct;
- a doc block in `proton.rs` (~:7071) that sits on the wrong test.
