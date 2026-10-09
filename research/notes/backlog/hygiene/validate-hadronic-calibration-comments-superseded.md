---
type: Backlog Item
title: validate_hadronic.rs jj, MLM and recarded-llj rationale quotes superseded readings
description: "The JJ_*, MLM_SEEDS and RECARDED_ROWS pp_to_llj rationale, two of them written into the report, quote readings since replaced by the vector-vertex fix, channel merging and note 34's ensemble."
area: hygiene
state: open
priority: medium
closes_when: "The sites listed in the body quote the current recorded readings (the manifest's or the cited probe's), and the report strings they feed say the same."
blocked_by: []
opened: 2026-10-09
tags: [stale-comment, budget-ladder, calibration, pp-to-jj, mlm, pp-to-llj]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: test, resource: "../../../../vibegraph-lib/tests/validate_hadronic.rs", title: "tests/validate_hadronic.rs"}
  - {id: note34, resource: "../../34-draw-followup-plan.md", title: "Note 34 S2, the forty-seed pp_to_llj ensemble"}
---
Not covered by [llj-gate-comments-quote-pre-floor-ladders](llj-gate-comments-quote-pre-floor-ladders.md),
which names the `LLJ_*` family, nor by
[sigma-calibration-comments-stale](../validation/sigma-calibration-comments-stale.md),
which covers `validate_sigma.rs`. Paths under `vibegraph-lib/tests/validate_hadronic.rs`:

- `:1803-1856` (`JJ_SEEDS`, `JJ_NEVAL`, `JJ_MAX_REL`): quote the pre-vector-vertex-fix reading `+3.329e-3`, "`0.005` clears it by `1.5x`", and the ladder `+0.33 / +0.26 / +0.25 / +0.30 %`. The manifest's `pp_to_jj` integrals note records `+0.18%` after the fix (note 39), so the margin is about 2.8×. The report-note string at `:2476-2482` repeats the old ladder.
- `:4605-4606` (`MLM_SEEDS`): ten seeds because "five read χ²/dof 2.9 about their mean". Since channel merging the row reads χ²/dof 0.86 over ten seeds.
- `:3021-3036` (`RECARDED_ROWS`, `pp_to_llj`), whose `ladder_note` is written into the report cell, and `:4039`: the ladder "still climbs monotonically". Note 34 S2's forty-seed ensemble (`probe_llj_seed_ensemble`) found it flat from 150k (drift rejected at 7.3σ), as the row's own manifest note says.

These are fixable from recorded readings. Re-measure only where none exists.
Found by drafter D2 and verifiers V2 and V3.
