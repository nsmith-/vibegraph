---
type: Backlog Item
title: MadGraph-compatibility choices are unconditional
description: Several sites reproduce a MadGraph choice a clean design would not make; with no switch, the cleaner behaviour is unreachable and its cost unmeasured.
area: feature
state: open
priority: medium
closes_when: A --madgraph-compat flag (default on, recorded in the integrate artifact and LHEF header) exists; with it on every banked byte and sigma gate is unchanged, and with it off a documented per-site delta table exists.
blocked_by: []
opened: 2026-09-06
tags: [cli, madgraph-parity, lhef, alphas, kt-clustering, user-request]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L643-L667", title: "TODO.md entry T058"}
---
A user request on PR #4. One flag, default on (every banked gate depends on
the compatible behaviour), recorded in the artifact and LHEF header. Each site
gets a "MadGraph compatibility" admonition in the docs. Sites:

- (a) `vibegraph-lib/src/coupling/cluster/kt.rs`: the `1 + 1e-6` crossed
  beam–leg inflation (`TIE_BREAK`), which does not cancel when every admissible
  candidate is crossed and so leaks a part in 1e6 into `SCALUP`, and the
  first-pair-in-visit-order tie-break. Off: an inflation-free measure and a
  tie rule that cannot enter the value.
- (b) `vibegraph-lib/src/coupling/alphas.rs`: `CMASS = 1.42`, `BMASS = 4.7`,
  `ZMASS`, the `TOL = 5e-4` Newton stop (a specific iterate, not the root) and
  the transcribed β-function constants. Off: the model's quark masses, computed
  constants, and an ODE solver run to convergence.
- (c) `lhef/mod.rs` + `lhef/write.rs`: the Python post-processor column layout
  and the two-dialect re-emission keeping a Fortran-dialect file's seven
  significant digits on the scale and coupling columns. Off: one layout at
  full precision.

Not sites, leave alone: `AQCDUP` is already untruncated, the jet-count memo
is not carried across events, `SCALUP = max(μF)` is the accord's definition,
and the truncated `w_max` rule is an improvement. No validation gate runs in
the off mode.

The flag also carries the MadGraph defects that are easy to reproduce
([defect policy](../../validation/madgraph-defect-policy.md)): on, the defect
is reproduced bug-for-bug; off, the more robust calculation runs. A defect
upstream fixes leaves the flag's scope when the MadGraph pin moves past the
fix.

- (d) identical particles across decays
  ([concept](../../process/identical-particles-across-decays.md)): on, one
  pairing per MadGraph block and the `identical_decay_chain_factor` division,
  with the interference between pairings dropped; off, every pairing and its
  interference. Cheap to reproduce: the stitching already marks which stitched
  diagrams lead to MadGraph's blocks. This site is an approximation rather than
  a defect, and with the flag on by default it changes today's output on such
  cards; their σ row is informational, so no gate moves.
