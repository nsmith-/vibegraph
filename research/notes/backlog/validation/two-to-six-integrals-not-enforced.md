---
type: Backlog Item
title: The 2→6 σ rows are info, not enforced, pending the budget ladder
description: The uux/bbx_to_ccx_emmm_qcd0 integrals cells stay info because of a heavy multichannel tail; the budget ladder under the accepted-point floor is due.
area: validation
state: open
priority: high
closes_when: probe_2to6_budget_ladder is re-run under the accepted-point floor with 20+ seeds on the deciding rungs, and either single-seed swings shrink with budget (note 32 §5.4 falsifier) and both integrals cells gate, or the record says why not.
blocked_by: []
opened: 2026-08-05
tags: [two-to-six, multichannel, heavy-tail, gate-promotion, seed-sweep]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L1076-L1098", title: "TODO.md entry T100"}
---
`uux_to_ccx_emmm_qcd0` (579 channels) and `bbx_to_ccx_emmm_qcd0` (615) have
`integrals` cells in `mode = "info"` in `validation/manifest.toml`. Before the
accepted-point floor, single-seed pulls reached ±3.5–4.8 % at every rung of a
300k/600k/1.2M ladder, while five-seed means held within 1.1 % of a 0.30 %
reference. The swings did not shrink with budget. Any fix must make single-seed
swings **shrink as budget grows**; that is note 32 §5.4's falsifier. A
constant-factor reduction at fixed budget is a variance win, not a resolution.

The accepted-point floor moved over-seed χ²/dof from 2.10 to 0.81 (uux) and
13.46 to 0.33 (bbx), and the worst single-seed rel from +1.68 % to +0.89 % and
+3.82 % to +0.46 %. That was measured at one budget only, so the falsifier is not
claimed. Re-run `probe_2to6_budget_ladder` (`vibegraph-lib/tests/validate_sigma.rs`)
under the floor. Read rung differences against measured seed spread, with 20+
seeds on the rungs that matter (AGENTS.md). A wide-row rung now spends about
2×; that cost is the floor's and is not a regression.

Channel merging cannot help here. Fingerprinting every channel by its full map
determinant gives 579 → 411 and 615 → 447 classes, with the largest class two
members. Those counts are from floor-less channels; `probe_channel_dedup_census`
is a cheap re-run for post-floor numbers, and floors can only split classes.

Detail: [note 32 §5.4 and §7](../../32-perf-addendum-plan.md),
[note 34 §2 S3](../../34-draw-followup-plan.md); the manifest's cell notes.
