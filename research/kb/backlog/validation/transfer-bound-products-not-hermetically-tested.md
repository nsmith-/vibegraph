---
type: Backlog Item
title: "The transfer bound's products argument is guarded only by an extended-validation test"
description: "Only cli_reweight_proton under extended-validation fails if the products argument is wrong; CI's test job cannot see it."
area: validation
state: open
priority: medium
closes_when: "A hermetic test through MapChoices::channel fails when the transfer bound's products argument is dropped or swapped."
blocked_by: []
opened: 2026-11-24
tags: [multichannel, phase-space, tests, hermetic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-24}
---
Found by sprint L's hygiene pass L-Z4 on PR 5 (#184) and `ew_recombine.py`
(#170, #196).
