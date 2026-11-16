---
type: Backlog Item
title: "UFO Lorentz structures that call form-factor functions are refused"
description: "loop_hvv's HZZ structure calls HVVCOEF1(P(-1,1),P(-1,2),MZ,MH), a function of momenta from the model's form-factor file; 11 refusals in the CMS store, plus 2HDMtypeIIymbMSbar's VVV1."
area: feature
state: open
priority: low
closes_when: "Form-factor functions defined in the UFO (function_library or form_factors) are evaluated in Lorentz structures with momentum-dependent arguments, gated against MadGraph on one loop_hvv process; or the refusal names the function and stays, with a recorded reason."
blocked_by: []
opened: 2026-11-16
tags: [ufo, lorentz, refusal, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store: `loop_hvv.tar.gz`'s `HZZ`
structure is `HVVCOEF1(P(-1,1),P(-1,2),MZ,MH)*Metric(1,2)+...` (11 refused
loads), and 2HDMtypeIIymbMSbar's `VVV1` does the same.

This needs momentum-dependent scalar functions inside the Lorentz algebra,
which ALOHA supports. It is a larger change than the parameter-expression gaps.
