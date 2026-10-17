---
type: Backlog Item
title: "mlm_match.py hand-writes its χ² tail, crashes on a one-bin histogram, and caches across Pythia builds"
description: "The MLM Pythia driver's chi2_sf is a second incomplete-gamma copy, jet_rate_legs raises on dof ≤ 0, and its run cache is keyed without the Pythia library, so a build change is mislabelled."
area: hygiene
state: open
priority: low
closes_when: "chi2_sf is replaced by a library tail (or the collator computes p), a histogram with dof ≤ 0 counts as a failed or excluded leg with a test, and the Pythia build is part of the run cache's key or printed per run by the driver."
blocked_by: []
opened: 2026-10-17
tags: [mlm, pythia, statistics, cache, duplication]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-17}
sources:
  - {id: hy1, resource: "../../sprints/mlm-pythia-tolerance/sessions/HY1-report.md", title: "mlm-pythia-tolerance HY1 report, Found 1–3"}
---
From [HY1](../../sprints/mlm-pythia-tolerance/sessions/HY1-report.md):

1. **A second incomplete gamma.** `chi2_sf` in `validation/pythia/mlm_match.py`
   is hand-written. It is now pinned by scipy values and closed forms, so it is
   correct, but it is a second copy beside the Rust crates' tail. Either add
   `scipy` to the `pythia` environment, or emit χ² and dof and let the
   collator compute p.
2. **A one-bin histogram raises.** `jet_rate_legs` throws `TypeError` on a
   histogram with dof ≤ 0, which can happen with fewer than two bins after the
   ≥100-event merging. The failure is loud, not green.
3. **The cache ignores the Pythia build.** The cache key is the inputs' and
   driver's sha, not the Pythia library's. Results differ across builds, so a
   cache read after an environment change is labelled with the new build.
   Adding the build to the key invalidates the 410-run cache, about 5.6 h on
   a 4-core host. `mlm_match.cc` also prints the version as a float, which
   the driver now formats in Python.
