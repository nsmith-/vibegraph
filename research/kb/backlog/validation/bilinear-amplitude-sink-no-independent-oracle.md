---
type: Backlog Item
title: The FFV bilinears have no independent oracle at the amplitude sink, or for FFV1/FFV5
description: "jioxxx, iovxxx and the production kernel share their bilinear code, and the ALOHA cross-check covers only the FFV2/FFV4 currents, so a shared bug is invisible at the amplitude sink."
area: validation
state: open
priority: low
closes_when: "Either ALOHA FFV1_0/FFV2_0/FFV5_0 transcriptions are compared at the amplitude sink, or a MadGraph per-helicity table reaching a massive Z current (e.g. t t~ > ta+ ta-) is gated, pinning the reversed-line [gR, gL] convention against MadGraph too."
blocked_by: []
opened: 2026-10-16
tags: [helas, chirality, oracle-gap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-16}
sources:
  - {id: j1, resource: "https://github.com/nsmith-/vibegraph/pull/34", title: "PR #34, J1 report, Found 1"}
---
Found while giving test_eval_jioxxx its massive regime (PR #34).
