---
type: Backlog Item
title: The run summary prints χ²/dof = 0.000 when only one iteration produced a result
description: "integrate's VEGAS summary line prints the iteration χ²/dof as 0.000 on --fixed-budget --niter 3 runs, where only the last iteration enters the result; it should print n/a."
area: hygiene
state: open
priority: low
closes_when: "The integrate summary prints n/a (not a number) for the iteration χ²/dof whenever fewer than two iterations enter the result, pinned by a CLI test on a --fixed-budget --niter 3 run, and the artifact or report field carrying it distinguishes 'undefined' from zero."
blocked_by: []
opened: 2026-10-28
tags: [cli, output, statistics, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-28}
sources:
  - {id: pr71-sweep, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6438701885", title: "PR #71: VBF/EWK Drell-Yan sweep, Found"}
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs, the 'VEGAS: … (χ²/dof = {:.3})' summary line (~:669)"}
---
The CERN host's VBF/EWK Drell-Yan sweep (#71) ran 35 integrations at
`--niter 3 --fixed-budget`. Every one printed `χ²/dof = 0.000` on the summary
line in `vibegraph-cli/src/integrate.rs` (~:669), because only iteration 3
produced a result line, and a χ² over one value has no degrees of freedom.

A reader takes 0.000 as perfect agreement. It is an undefined statistic, so it
should read n/a. Check where `result.chi2_per_dof` gets its zero for a single
entering iteration, and whether the artifact or the validation report record
the same zero.
