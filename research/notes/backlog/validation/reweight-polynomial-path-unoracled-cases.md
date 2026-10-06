---
type: Backlog Item
title: Two polynomial-reweighting cases have no physics oracle
description: A coupling of the form a + b*c and a derivative all-scalar vertex are unchecked against MadGraph on the polynomial reweighting path.
area: validation
state: open
priority: low
closes_when: A reweight-oracle row exercises a coupling spanning two monomial classes, and a row with a derivative all-scalar vertex pins its sign, both agreeing with MadGraph event by event.
blocked_by: []
opened: 2026-10-05
tags: [reweight, polynomial, oracle-gap, smeft]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L668-L730", title: "TODO.md entry T059"}
---
Two cases on the coupling-polynomial reweighting path (`vibegraph-lib/src/reweight/poly.rs`)
lack an oracle outside the polynomial algebra's own unit tests:

- (f) a coupling that is itself `a + b·c`, so one diagram spans two monomial
  classes. SMEFTsim splits every coefficient into its own coupling order, so
  no MadGraph row reaches this; a model or restriction that combines them is
  needed.
- (g) a derivative all-scalar vertex (`P` operators on scalar legs) has no
  sign oracle. The relative-sign defects found on `HHH`/`HHHH` and
  Yukawa-only lines were in this family of vertex, which is why a sign check
  matters here.

Either can be closed separately; the item closes when both are.
