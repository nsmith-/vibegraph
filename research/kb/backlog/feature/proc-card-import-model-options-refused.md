---
type: Backlog Item
title: "A proc card's import model --modelname is refused, and gpdb's model rewrite mangles it"
description: "'import model --modelname X' is refused as model options; gpdb also rewrites it to 'import model gp_<hash> X', turning the path into a restriction name."
area: feature
state: open
priority: high
closes_when: "vibegraph accepts import model --modelname <model> (with a stated reading of what the flag changes), gpdb's model rewrite keeps the flag and replaces only the model path, and both HAHM gridpacks get past proc-card parsing."
blocked_by: []
opened: 2026-11-07
tags: [proc-card, gpdb, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
---
MadGraph's `--modelname` keeps the model's own particle names instead of
MadGraph's default renaming. vibegraph already uses the model's names, so the
flag should be a no-op to verify, not a feature.

Two defects stack: the CLI refuses any option on `import model` ("model
options and model merging are not supported"), and gpdb's rewrite treats
`--modelname` as the model token, producing `import model gp_bc19359d
HAHM_MG5model_v3/HAHM_variableMW_v3_UFO`.

Refused in `main-202a540-200-pc` (2): hToaaTo2gamma2b_ma25GeV_MLM_4f_max1j,
Zh_M125_ToAA_35_Tobbgg. Census: [refusals at 202a540](../../validation/cms-gridpack-refusals-202a540.md).
