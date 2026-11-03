---
type: Backlog Item
title: A run whose cross section is exactly zero finishes as a success
description: "integrate reports σ = 0 ± 0 and exits 0 when every sampled point has zero weight; it should refuse, or at least fail loudly, naming why the weight vanished."
area: feature
state: open
priority: medium
closes_when: "An integrate run in which every sampled point carries zero weight exits non-zero with an error that names the likely cause (cuts reject every point, or the matrix element vanishes at the parameters in force), pinned by a test, and generate refuses an artifact recorded with σ = 0."
blocked_by: []
opened: 2026-10-27
tags: [refusal, sigma, cli, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-27}
sources:
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6432085527", title: "PR #71: main-6cb9756-200 re-run, H2ToH1H3To2Mu MH2-500 / MH2-250 at σ = 0 ± 0, outcome ok"}
---
In the CMS-gridpack re-run `main-6cb9756-200` (#71), both
`H2ToH1H3To2Mu` gridpacks (MH2-500 and MH2-250) report σ = 0 ± 0 exactly, and
the campaign counts them as `ok`: the CLI exited 0. MadGraph's σ is non-zero.

The zero itself may be legitimate for what was run. The campaign has no param
card option, so BSM gridpacks run on their models' default parameters
([cli-has-no-param-card-option](cli-has-no-param-card-option.md)), where a
coupling of the process may be zero. A run in which no point has weight is
still never a usable result:
- unweighting has no maximum weight;
- a zero σ hides cuts that reject everything, a restriction that zeroes the
  process, or a bug.

So it should not pass silently.

**What to do.** At the end of integrate, if every evaluated point has zero
weight, refuse. Name what can be told apart cheaply:
- how many points passed the cuts;
- whether |M|² was zero on the points that passed.

`generate` should refuse an artifact recorded at σ = 0.

**First,** check whether these two gridpacks are zero because of the default
parameters, or because of something in vibegraph. Re-run them once with the
gridpack's own param card applied (the survey tool's `--bake`). Record which;
if vibegraph is at fault, that is a separate bug.
