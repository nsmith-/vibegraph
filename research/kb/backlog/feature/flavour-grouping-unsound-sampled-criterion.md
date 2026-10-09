---
type: Backlog Item
title: Flavour grouping merges subprocesses on sampled |M|² agreement rather than program identity
description: derive_flavor_groups partitions subprocesses by sampled |M|² agreement, which can silently merge programs that differ only where the probe does not look.
area: feature
state: open
priority: low
closes_when: Two subprocesses share a flavour group iff their canonicalized compiled programs (with UFO-stable constant ids and colour) are identical as s-expressions, and the sampled criterion runs beside it as a cross-check.
blocked_by: []
opened: 2026-07-30
tags: [flavour-groups, proton, s-expression, soundness]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L788-L806", title: "TODO.md entry T063"}
  - {id: todo-t036, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L488-L495", title: "TODO.md entry T036"}
---
`derive_flavor_groups` (`vibegraph-lib/src/proton.rs:758`) groups subprocesses whose
`|M|²` agrees within `GROUP_REL_TOL` on a shared probe ladder
(`probe_energies` / `probe_momenta`, same file). The criterion is complete but
unsound: two programs that differ only where the probe does not look are merged
silently. The user accepted it for v1 (2026-08-02) on the MadGraph precedent, since
MadGraph's helicity filter drops vanishing configurations on the same sampled-probe
basis, with the probe ladder hardened.

The sound replacement is that two subprocesses share a group iff their compiled
programs are identical as s-expressions. It has three prerequisites, in order:

1. **Universal constant ids.** Compare UFO-stable coupling and particle identities,
   never per-compilation pool slot indices. Flavour-dependent couplings can share a
   slot, so slot equality would reintroduce the unsoundness.
2. **Canonicalization of the un-optimized s-expression.** Lowering has a ±1 CSE-node
   nondeterminism, and diagram order is unstable.
3. **Colour folded into the s-expression language**, so the colour basis is part of
   the compared term.

The criterion is conservative: it can only split genuinely equal groups, which costs
extra compiled programs and never correctness. Keep the sampled criterion running as
an independent cross-check; a disagreement between the two is a finding.
Background: [note 24 §P2c](../../history/notes/24-user-distribution-and-proton-events-plan.md).
