---
type: Backlog Item
title: Polarized intermediate resonances are refused
description: A polarization on a decayed leg (p p > w+{0} w-, w+ > e+ ve) or a propagator code {A}/{G}/{H}/{Q}/{W}/{S} is refused by check_supported.
area: feature
state: open
priority: medium
closes_when: A polarized decayed resonance and the propagator-only codes evaluate with a helicity-projected propagator numerator and match MadGraph per diagram and in sigma on a banked row.
blocked_by: []
opened: 2026-09-25
tags: [process-grammar, polarization, helas, madgraph-parity]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L610-L613", title: "TODO.md entry T055"}
---
Polarized external legs landed ([note 38 P1](../../38-process-grammar-sprint-plan.md));
polarized intermediate resonances did not. `check_supported` refuses them as
`Unsupported::DecayedPolarization` and `Unsupported::PropagatorPolarization`
(`vibegraph-lib/src/diagrams/check.rs`).

The work is a helicity-projected propagator numerator in `helas/eval`: the
resonance's propagator numerator replaced by the projection on the named
helicities. P1 deferred it because evaluator performance PRs were editing that
area; there are no open PRs on the repository now (checked 2026-10-06).

The `check.rs` module table marks the propagator-only codes "(not planned)",
while the backlog lists them as in scope; one of the two needs correcting.
