---
type: Backlog Item
title: No CI job runs the long-tier σ gates
description: "Every long-tier gate is an ignored test that no workflow runs, so a regression in a 2→6 row, an MLM σ, a ladder or the no-pT-cut EWK e+ e- j j row shows only when someone runs validate-deep by hand."
area: validation
state: open
priority: medium
closes_when: "A scheduled or dispatch workflow runs the long-tier tasks (validate-deep) and the validation-report collator on the pinned refdata, and its first run is green; or, for a row too costly for that, a reduced-budget CI variant gates it with a tolerance calibrated on its own measured seed spread."
blocked_by: []
opened: 2026-11-02
tags: [validation, ci, long-tier, seed-policy]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-02}
---
CI's `banked` job runs `pixi run --skip-deps validate`, and
`validation/validate.sh` calls `cargo test … --no-fail-fast` without
`--ignored`. Every long-tier gate is `#[ignore = "long tier: …"]`, and no
workflow calls `validate-deep` or any single long-tier task. That covers the
2→6 σ rows, the MLM σ and dump gates, the ladders, and
`validate_ewk_lljj_sigma`, the σ gate for `pp_to_epemjj_qcd0` (the
electroweak e+ e- j j row with no pT cut). That gate's four non-ignored tests
do run in CI, but they check only its record reader and verdict, not
vibegraph's integration.

Cost sets the shape of a fix. `validate_ewk_lljj_sigma` alone is five seeds
at `--neval 40000 --niter 12`, about 600 s a seed on four cores, so roughly
an hour of a four-core runner. A scheduled job covers the whole tier at once.
A reduced-budget variant suits a single row, but its tolerance must come from
that budget's own measured seed spread, not from the long-tier gate's bounds
([AGENTS.md](../../../../AGENTS.md), "Samplers gate statistically").

Closely related: [long-tier-gates-write-no-receipt](long-tier-gates-write-no-receipt.md).
That item is about recording what a hand-run produced; this one is about
CI running the tier at all.
