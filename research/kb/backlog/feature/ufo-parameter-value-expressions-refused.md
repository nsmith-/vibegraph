---
type: Backlog Item
title: "UFO parameter values using conditional expressions or functions are refused"
description: "Values such as '0.0 if aEW0M1==0. else 1./aEW0M1' or max(1e-5, re(cmath.sqrt(...))) are refused at load; MadGraph evaluates them. Found in 4 CMS-store models."
area: feature
state: open
priority: medium
closes_when: "The parameter-expression evaluator accepts Python conditional expressions and the functions UFO models use in values (max, min, re, im, abs, cmath.sqrt and the cmath set MadGraph's parser allows), with tests, and the four store models below load."
blocked_by: []
opened: 2026-11-16
tags: [ufo, parameters, refusal, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store
(`https://cms-project-generators.web.cern.ch/cms-project-generators/`):

- `loop_qcd_qed_sm_Gmu.tgz`: `aEW0` has the value
  `'0.0 if aEW0M1==0. else 1./aEW0M1'`;
- `DMPseudoScalar_MonoB.tgz`: `WPhiee` has the value
  `max(1e-5, re(... cmath.sqrt(...)))`, and DMScalar and DMsimp_s_spin0_4f do
  the same.

simshop counts this item,
[`negative-ufo-coupling-orders-refused`](negative-ufo-coupling-orders-refused.md)
and [`ufo-vertex-coupling-lists-refused`](ufo-vertex-coupling-lists-refused.md)
as the cheapest wins: about 180 of 337 refused loads together.
