---
type: Backlog Item
title: "Every generate job re-runs the per-channel w_max scan that integrate could record once"
description: "generate scans every channel's frozen grid for its unweighting maximum before writing events, at about one integration iteration's cost (1/8 of integrate CPU in one measurement), in every job."
area: feature
state: open
priority: high
closes_when: "integrate records each channel's unweighting maximum in the artifact, generate uses it without scanning unless a scan option asks otherwise, a changed truncation is either re-derived or refused by name, and a gate shows stored-maxima samples agree with fresh-scan ones statistically (σ̂ pull and overweight share)."
blocked_by: []
opened: 2026-11-23
tags: [generate, integrate, unweighting, artifact, cms, gridpack, performance]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
sources:
  - {id: unweighting, resource: "../../events/unweighting.md", title: "Unweighting: the truncation ladder and the scan budget"}
  - {id: split, resource: "../../pipeline/integrate-generate-split.md", title: "Integrate and generate split by data dependency: the scan options in GenerateOptions"}
---
`generate` starts by scanning each channel's frozen grid for its maximum weight
`w_max_j`, using [the truncation ladder](../../events/unweighting.md). That
maximum sets both the acceptance threshold and the channel-selection
probabilities. By default (`--scan-points share`) the scan draws as many points
per channel as one integration iteration did, so its cost grows with the
integration. vg-batch measured the scan at 2.2 s against the integration's
16.9 s on `e+ e- > u u~ g g` (one measurement). A CMS production runs
`generate` once per job, so it pays this cost hundreds of times on the users'
grid allocation.

The maxima depend only on the trained grids, the scan budget, the truncation
and the scan seed. The integrate/generate split put the budget and the
truncation in `GenerateOptions`, and v0.2.2 derives the scan seed from
`generate --seed`. Nothing requires a per-job scan: the maxima are estimates,
and overweights keep a weight above 1, so one shared set of maxima leaves every
job's sample unbiased.

The design has to settle four things:

1. **A changed truncation at generate.** Under the default 1 % excess share,
   the maximum is a quantile of the scanned weights. Either store enough of
   each channel's weight tail to re-derive it at another `--max-truncation`, or
   re-scan or refuse by name. A changed `--scan-points` re-scans.
2. **The artifact version.** Older artifacts keep scanning in `generate`.
3. **Whether the final integration iteration's points can serve as the scan.**
   That holds only if the grid they were drawn from is the frozen grid
   `generate` samples. This is unchecked.
4. **The gate.** Compare stored and fresh maxima on gated rows, through the
   σ̂ pull and the overweight weight share, not just σ.

Until this lands, a job can lower the scan cost with `--scan-points N`. Below
about 10⁴ points per channel the ladder cannot step below the scan's top
weight, so keep N at or above that.
