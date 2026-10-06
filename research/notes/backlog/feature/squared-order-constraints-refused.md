---
type: Backlog Item
title: Squared-order constraints are refused
description: "Squared-order constraints (`NP^2==1`, `QCD^2<=4`) are a hard error, so no SMEFT interference-only or pure-BSM-squared |M|² can be generated."
area: feature
state: open
priority: low
closes_when: "`e+ e- > t t~ NP^2==1` (interference only) and `NP^2==2` match MadGraph's |M|² and σ on banked SMEFTsim rows, and `Unsupported::SquaredOrder` is removed."
blocked_by: []
opened: 2026-09-05
tags: [non-sm-ufo, descoped-v1, smeft, coupling-orders, process-grammar]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L924-L925", title: "TODO.md entry T079"}
  - {id: todo-t067, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L822-L826", title: "TODO.md entry T067"}
  - {id: todo-t054, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L603-L609", title: "TODO.md entry T054"}
---
A squared-order constraint is refused in the card check
(`Unsupported::SquaredOrder`, `vibegraph-lib/src/diagrams/check.rs:256`). It is
also refused in decay-chain resolution (`vibegraph-lib/src/diagrams/resolve.rs:361`).
The test `a_squared_order_constraint_is_a_hard_error` pins the refusal.

Descoped by user decision D4 ([note 35 §7](../../35-ufo-lorentz-sprint-plan.md)).
Every SMEFT row compares the full |M|² at `NP<=1`.

The check module's table lists this as "amplitudes split by coupling order (not
planned)". The decision record files it as tracked backlog, not as refused for
good.

**What's needed.** Split the amplitude by coupling-order class and form only the
cross terms the constraint selects. `reweight::poly` already proves each
diagram's coupling monomials. A graded evaluator that carries each current as
its monomial components would also serve one-evaluation-per-event reweighting.
`WEIGHTED==`/`>` (`Unsupported::WeightedOrder`) needs the same split.
