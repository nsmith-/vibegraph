---
type: Backlog Item
title: "mlm_match.py's mean_of reports the quoted error, not the seed policy's"
description: "validation/pythia/mlm_match.py's mean_of uses the unweighted mean but reports error = quoted instead of max(quoted, spread/√n)."
area: validation
state: open
priority: low
closes_when: "mean_of reports max(quoted, spread/√n) with the quoted and spread parts beside it, or its doc says why the comparison it feeds needs the quoted part alone."
blocked_by: []
opened: 2026-10-14
tags: [seed-policy, pythia, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-14}
sources:
  - {id: seed-policy, resource: "../../validation/madevent-reference-seed-policy.md", title: "MadEvent reference seed policy"}
---
The Rust gates read seeds through one helper, `combine` in
`vibegraph-lib/tests/common/seeds.rs`; this Python reader cannot use it and
already takes the unweighted mean, but its error is the quoted part alone.
