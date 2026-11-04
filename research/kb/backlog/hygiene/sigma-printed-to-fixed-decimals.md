---
type: Backlog Item
title: integrate prints σ to six fixed decimals, so a small σ loses its digits
description: "The integrate result line prints σ and its error as {:.6} pb, so σ below about 1e-3 pb keeps one to three significant figures, and readers of the line lose the rest."
area: hygiene
state: open
priority: medium
closes_when: "The integrate result line and the per-multiplicity summary print σ and its error to a fixed number of significant figures (e.g. {:.6e}, or digits set by the error), a CLI test pins a σ of order 1e-5 pb printed with its full precision, and the survey tool's parser reads the new form."
blocked_by: []
opened: 2026-11-03
tags: [cli, output, cms-survey]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
sources:
  - {id: integrate, resource: "../../../../vibegraph-cli/src/integrate.rs", title: "vibegraph-cli/src/integrate.rs, the result line (~:837) and the per-multiplicity line (~:1104)"}
  - {id: pr71, resource: "https://github.com/nsmith-/vibegraph/pull/71#issuecomment-6605096410", title: "PR #71: campaign main-23d3a76-200-pc, Found"}
---
`integrate` prints its result as `σ = {sigma_pb:.6} ± {sigma_err_pb:.6} pb`
(`vibegraph-cli/src/integrate.rs` ~:837), and the per-multiplicity summary uses
the same `{:.6}` (~:1104). A σ of 1e-4 pb prints as `0.000100 ± 0.000002`, so
anything that reads the line keeps one to three significant figures. The CMS
gridpack survey (#71) parses this line into its database, so its small-σ BSM
rows carry that rounding.

The artifact keeps σ at full precision; only the printed line is short.
Print a fixed number of significant figures instead, and update
`validation/cms_gridpacks/` to parse the new form.
