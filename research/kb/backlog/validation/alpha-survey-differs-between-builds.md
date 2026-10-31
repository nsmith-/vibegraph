---
type: Backlog Item
title: One source built twice can give different α vectors
description: "The same library source built twice gave an α survey differing at the last digit on some seeds (bbx seed 55 at 1.2M: 2.6e-5 in σ), so a per-channel trace replays only on its own binary."
area: validation
state: open
priority: medium
closes_when: "The cause is named (codegen or link order, float contraction, parallel reduction order, or other) and either the survey is made reproducible across builds of one source at the ulp scale its tolerance allows, or the record states why it cannot be and the ladders' replay rule says so."
blocked_by: []
opened: 2026-10-31
tags: [reproducibility, alpha-survey, budget-ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-31}
sources:
  - {id: t1, resource: "../../sprints/alpha-share-tails/sessions/T1-report.md", title: "alpha-share-tails T1 report, Brief corrections and Found 4"}
---
S1 (scaling-floor) saw the same drift across trees and read it as unrelated
code changing the arithmetic order. T1's case had identical library source
apart from a removed rule, and seeds 20260719 (1.2M) and 55 (600k) matched to
every digit while seed 55 at 1.2M did not. AGENTS.md does not ask for
bit-reproducibility across platforms, but a ladder's paired control assumes
one source gives one survey on one host.
