---
type: Backlog Item
title: "Decay-chain process strings drop the / X forbidden list, so refusals can quote two lines identically"
description: "SupportedProcess's Display omits forbidden particles by design; e+ e- > z j j / z, z > j j prints as e+ e- > z j j, z > j j, so lines that differ only in / read the same in messages."
area: hygiene
state: open
priority: low
closes_when: "Process strings in refusal and log messages print the forbidden list (or the refusals that quote two lines note it), with a test on two lines differing only in /."
blocked_by: []
opened: 2026-11-07
tags: [cli-output, decay-chains, messages, refusals]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hd2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-D2-report.md", title: "H-D2 report, Found: decay-chain process strings omit / X"}
---
Found by H-D2. The log of `e+ e- > z j j / z, z > j j` prints
`'e+ e- > z j j, z > j j'` (`H-D2_vg_ee.log`). This is the documented behaviour
of `SupportedProcess`'s `Display` ("the forbidden particles are not printed"),
but two process lines that differ only in `/` then print identically in refusal
messages, such as the cross-line duplicate refusal. Print the forbidden list, or
say in the refusals that quote process strings that it is omitted.
