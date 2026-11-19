---
type: Backlog Item
title: "generate_sample and generate_proton_sample are about 230 lines each"
description: "generate.rs's generate_sample and generate_proton_sample (about 230 lines each) were already past 100 lines and grew again with the sampled-fault refusals."
area: hygiene
state: open
priority: low
closes_when: "Both are split so no function in generate.rs exceeds about 100 lines, with event files bit for bit on a fixed-beam and a hadronic card."
blocked_by: []
opened: 2026-11-19
tags: [maintainability, generate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H3 (hygiene pass on #143, #152 and #156, PR #159), Found 4. generate_sets_inner (240 lines) is tracked in [diagram-generation-functions-long](diagram-generation-functions-long.md).
