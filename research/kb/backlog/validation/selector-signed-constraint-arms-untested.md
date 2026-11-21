---
type: Backlog Item
title: "The selector tests assert nothing and two signed-constraint arms are untested"
description: "diagrams/selector.rs:136-177 tests only build a selector (let _ = …), and the == and > arms of apply_signed_constraint have no test anywhere."
area: validation
state: open
priority: medium
closes_when: "Each apply_signed_constraint arm (==, >, <=) has a test asserting diagram counts on a toy row (e.g. QED==0, NP>0) checked against MadGraph's counts, and a mutation of each arm fails a test."
blocked_by: []
opened: 2026-11-19
tags: [diagrams, coupling-orders, non-vacuity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-19}
---
Found by K-H1 (hygiene pass on #141, #144 and #145, PR #157), Found 3 (finding H14).
