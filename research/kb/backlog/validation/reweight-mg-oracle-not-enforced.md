---
type: Backlog Item
title: The MadGraph reweight oracle is informational, not enforced
description: reweight_mg_oracle agrees with MadGraph's reweight module but is not a gate; it lacks a hadronic polynomial row and a second SMEFT process.
area: validation
state: open
priority: medium
closes_when: The reweight oracle carries a hadronic row on the polynomial path (hypothesis side included) and a second SMEFT process, and it runs as an enforced gate.
blocked_by: []
opened: 2026-10-05
tags: [reweight, madgraph-oracle, smeft, gate]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
---
`validation/madgraph/gen_reweight_oracle.py` runs MadGraph's reweight module on
vibegraph's own events (`p p > l+ l- j`, `e+ e- > t t~ h`, a `ta+ ta- > t t~ h`
3×3 grid, SMEFTsim `e+ e- > t t~`). All 5000 events × 27 hypotheses agree to
the files' printed precision, and `vibegraph-cli/tests/reweight_mg_oracle.rs`
replays the banked 200 per row through both paths. It stays informational.

Two coverage gaps keep it from being enforced: both hadronic MadGraph rows are
on the exact path, so the hypothesis side of a hadronic polynomial group
(`--reweight-couplings`) has no MadGraph comparison; and SMEFT is covered by
one process. Add a hadronic polynomial row and a second SMEFT process, then
enforce. The `extended-validation` skill names the regeneration task.
