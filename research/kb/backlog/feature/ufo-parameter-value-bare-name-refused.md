---
type: Backlog Item
title: "A UFO parameter whose value is a bare Python name is refused"
description: "Models such as DMAxial_monow012j write value = MED (a Python name, not a string); the loader requires a string. Also VPrime_Allanach_* WZp."
area: feature
state: open
priority: low
closes_when: "A parameter value given as a bare name resolves to that name's value in the UFO module (or is refused naming the name and file), with a test, and DMAxial_monow012j loads or is refused by name."
blocked_by: []
opened: 2026-11-16
tags: [ufo, parameters, refusal, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store: `DMAxial_monow012j.tgz`
(axial and vector `monoW` models) has `MZp ... value = MED`, and the
VPrime_Allanach_* models' `WZp` is refused with "`value` must be a string".
Find what `MED` is bound to in the module before choosing between resolving it
and a better refusal.
