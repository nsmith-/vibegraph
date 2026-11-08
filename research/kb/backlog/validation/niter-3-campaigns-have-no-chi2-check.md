---
type: Backlog Item
title: "A niter-3 gridpack campaign has no in-run χ² consistency check"
description: "With DEFAULT_WARMUP_ITERS = 2, niter 3 leaves one iteration in the result, so χ²/dof is None on every run; gpdb's policy docs do not say a campaign wanting the check needs niter ≥ 4."
area: validation
state: open
priority: low
closes_when: "gpdb's policy docs (vgrun.py's docstring and the campaign concept) state that niter 3 leaves one post-warm-up iteration and no χ²/dof, and that a campaign wanting an in-run consistency check runs niter ≥ 4; or vgrun warns when niter − warm-up < 2."
blocked_by: []
opened: 2026-11-08
tags: [gpdb, cms, vegas, chi2, docs]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: log, resource: "../../sprints/cms-gridpack-coverage/log.md", title: "Sprint H log, 2026-11-07: \"χ²/dof = n/a at niter 3\" is by design"}
  - {id: vegas, resource: "../../../../vibegraph-lib/src/vegas.rs", title: "vegas.rs: DEFAULT_WARMUP_ITERS"}
  - {id: vgrun, resource: "../../../../validation/cms_gridpacks/vgrun.py", title: "vgrun.py: --niter default 3"}
---
The CMS campaigns run `integrate --niter 3` (`vgrun.py`'s default).
`DEFAULT_WARMUP_ITERS = 2` (`vegas.rs:114`) drops the first two iterations,
so the result holds one iteration and χ²/dof is `None` (`artifact.rs:66–70`):
every run prints "χ²/dof = n/a". This is by design (read by the CERN host at
202a540f).

The campaign therefore has no in-run check that iterations agree. On VBF_HH
and TCP_m55 the single-iteration errors overstated the seed spread by about
2.4×, and only a multi-seed sweep showed it. Nothing in gpdb's docs says that
a campaign wanting the check needs niter ≥ 4. Closing is a docs change, or a
warning; raising the campaign's niter is a budget decision for the campaign.
