---
type: Backlog Item
title: "The -full model restriction is not supported"
description: "MadGraph's import model X-full loads the model with no restriction card applied; vibegraph 0.2.1 does not accept the -full suffix."
area: feature
state: open
priority: medium
closes_when: "import model X-full loads the model with no restriction applied, as MadGraph does, with a test on a model that also ships restrict_default.dat."
blocked_by: []
opened: 2026-11-16
tags: [ufo, restriction, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store. In MadGraph, `X-full` means
"no restriction card", as opposed to a bare `X`, which applies
`restrict_default.dat` when present. Check what the loader does with the suffix
today, refusal or misreading, before choosing the fix.
