---
type: Backlog Item
title: "The result record's process echo drops `/` exclusions"
description: "Card 6 ran `... $$ w+ w- / h ...` and the record echoes it without `/ h`, though the census shows the exclusion is applied. A reader cannot tell what ran."
area: hygiene
state: open
priority: low
closes_when: "The process echo in result records reproduces `/` exclusions (and every other process-line constraint) as applied, with a test on a process that uses `/`."
blocked_by: []
opened: 2026-11-26
tags: [provenance, process-syntax, result-record, display]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
---
Found by `vg-validator` on card 6 (tZq_EFT_CPV_point_1). Sprint L's L-D1 census
confirmed that `/ h` does reach the diagram set, so this is display only. It
cost a diagnosis step, because the record looked as if the exclusion had been
lost.
