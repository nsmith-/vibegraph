---
type: Backlog Item
title: MadGraph 3.7.1's reweight-module defects are documented only in generator docstrings
description: "A multi-launch reweight card keeps only the last hypothesis and mislabels its weight, and a compiled rwgt_dir cannot be reused; no note 07 entry or upstream draft exists."
area: validation
state: open
priority: low
closes_when: "Both defects have a reproducer and a drafted upstream report beside the other MadGraph defect drafts, ready for the user to file."
blocked_by: []
opened: 2026-10-09
tags: [madgraph-defect, reweight, upstream-report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: oracle, resource: "../../../../validation/madgraph/gen_reweight_oracle.py", title: "gen_reweight_oracle.py docstring (~:34-37)"}
  - {id: bench, resource: "../../../../validation/madgraph/bench_reweight.py", title: "bench_reweight.py docstring (~:17-20)"}
---
Two MadGraph 3.7.1 reweight-module defects are worked around and described
only in docstrings:

- **Multi-launch cards.** With several `launch` blocks in one card, each
  rewrites `events_out.lhe` from the unmodified input, so only the last
  hypothesis survives, and its weight is mislabelled
  (`ReweightInterface.import_command_file`). `gen_reweight_oracle.py` runs one
  hypothesis per work area because of it, and `bench_reweight.py` estimates
  setup + H × loop.
- **`rwgt_dir` reuse.** A compiled `rwgt_dir` cannot be reused:
  `setup_f2py_interface` reads an undefined `opts`.

Neither has a note 07 entry, a reproducer directory or a drafted report.
Filing upstream is the user's call
([madgraph-defect-reports-unfiled](madgraph-defect-reports-unfiled.md)).
Found by Phase 2 drafter D2.
