---
type: Backlog Item
title: UFO asin and acos evaluate as acsc and asec
description: "The UFO expression grammar maps asin and acos (bare and cmath.) to Func::ACsc and Func::ASec, so a model parameter written with asin(x) silently evaluates asin(1/x)."
area: hygiene
state: open
priority: medium
closes_when: "asin and acos parse to functions that evaluate asin(x) and acos(x), with a unit test on each spelling (bare and cmath.) against the std value."
blocked_by: []
opened: 2026-10-09
tags: [ufo, expression, latent-bug, model]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/ufo/expr.rs", title: "ufo/expr.rs cmath_func_name (~:236-237), bare_func_name (~:257-258), evaluation (~:125-126)"}
---
In `vibegraph-lib/src/ufo/expr.rs`, `cmath_func_name` maps `asin` to
`Func::ACsc` and `acos` to `Func::ASec` (~:236-237, commented "asin not used
directly, but consistent"), and `bare_func_name` does the same (~:257-258).
`Func::ACsc` evaluates `asin(1/x)` and `Func::ASec` evaluates `acos(1/x)`
(~:125-126). So `cmath.asin(x)` in a `parameters.py` value gives the wrong
number, with no error.

It is latent today. No value string in any model the repo loads calls them.
The only callers are the `function_library.py` definitions of `asec`/`acsc`
(SMEFTsim and the two toy UFOs), and the grammar implements `asec`/`acsc`
directly rather than through those definitions. A user model whose mixing
angle is written `cmath.asin(...)` would load and be wrong.

The fix needs `Func::ASin`/`Func::ACos` variants. Found by Phase 2 drafter D13;
confirmed against the code.
