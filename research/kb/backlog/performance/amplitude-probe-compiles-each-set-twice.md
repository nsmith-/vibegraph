---
type: Backlog Item
title: "The amplitude probe compiles every diagram set a second time in integrate and generate"
description: "amplitude_probe compiles each set itself (compile_class) independently of the evaluators the integrand already built; on the largest cards that is seconds per run."
area: performance
state: open
priority: low
closes_when: "The probe evaluates through the evaluators the run already compiled (fixed-energy and proton paths), and the saving is measured on p p > e+ e- j j j."
blocked_by: []
opened: 2026-11-10
tags: [amplitude-probe, compile, integrate, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr121, resource: "https://github.com/nsmith-/vibegraph/pull/121", title: "Self-contained integrate artifact (found there)"}
---
The probe (nsmith-/vibegraph#121) is computed at integrate and recomputed at
generate by compiling every set with `compile_class`, independent of how
flavour groups are formed. By the enumerate design's measurements that is up
to 2–3 s on the largest cards. The fixed-energy path already holds an
evaluator per subprocess; the proton path compiles inside
`derive_flavor_groups` and does not expose them.
