---
type: Backlog Item
title: gen_reweight_oracle.py reads an empty VG_MG5_ROOT as a path
description: "MG_ROOT = os.environ.get('VG_MG5_ROOT', default) returns '' when the variable is set but empty, where mg5_pinned.sh's ${VG_MG5_ROOT:-default} falls back to the submodule."
area: hygiene
state: open
priority: low
closes_when: "Every Python reader of VG_MG5_ROOT treats an empty value as unset, as mg5_pinned.sh does."
blocked_by: []
opened: 2026-10-20
tags: [validation, madgraph, scripts]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: hy1, resource: "../../sprints/ckm-no-b-mass/sessions/HY1-report.md", title: "ckm-no-b-mass HY1 report, Found 2"}
---
`validation/madgraph/gen_reweight_oracle.py` ~:61. `gen_couplings.py` was
fixed the same way (`os.environ.get("VG_MG5_ROOT") or <default>`) in the
[ckm-no-b-mass sprint](../../sprints/ckm-no-b-mass/sprint.md); grep the other
Python readers at the same time.
