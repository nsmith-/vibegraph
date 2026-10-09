---
type: Backlog Item
title: validate_hadronic.rs budget rationale quotes pre-timelike-floor ladders
description: The doc comments justifying the ℓℓj σ gate's budget and tolerances quote five-seed ladders measured before the timelike floors landed.
area: hygiene
state: open
priority: medium
closes_when: The four comment sites quote ladders re-measured on the current sampler, each five-seed rung figure alongside the measured per-seed spread.
blocked_by: []
opened: 2026-08-07
tags: [vegas, budget-ladder, stale-comment, pp-to-llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L552-L561", title: "TODO.md entry T044"}
---
Four load-bearing doc comments in `vibegraph-lib/tests/validate_hadronic.rs` quote
five-seed budget ladders from before the cut-implied timelike floors
([note 34 §1.2](../../history/notes/34-draw-followup-plan.md)). They are the `LLJ_NEVAL`
rationale (lines ~88–119, e.g. `423.94` pb and χ²/dof `1.66`/`1.03`/`1.11`/`0.40`),
`LLJ_DYN_MAX_REL` (~130–146), the `LLJ_MAX_CHI2_PER_DOF` calibration family
(~147–155) and the doc comment on `sigma_llj_dynamical_scale_vs_mg` (~1290–1321; `TODO.md` calls it `measure_llj_dyn_sigma`, a name no longer in the file). The constants and gates are
sound; only the rationale numbers are stale.

Fix: re-run `probe_llj_fixed_budget_ladder` and `probe_llj_dyn_budget_ladder` on a
quiet host (about 357 s each on 16 cores; long commands must run detached), then
rewrite the four sites. Quote the measured per-seed spread next to every
five-seed rung figure, because five-seed scatter understates this row's spread
by 2–5× (see `AGENTS.md`, "Samplers gate statistically").
