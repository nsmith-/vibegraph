---
type: Backlog Item
title: make_anti negates the colour of singlets and octets
description: The model's antiparticle entries carry color = -1/-8 for singlets and octets, where UFO's anti() leaves self-conjugate representations unchanged.
area: hygiene
state: open
priority: medium
closes_when: An antiparticle's `Particle::color` equals UFO's `anti()` (1 and 8 unchanged, 3 and 6 negated), with a test pinning it, and local workarounds such as `member_line_pdg`'s are removed.
blocked_by: []
opened: 2026-09-26
tags: [ufo, model, colour, trap]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L333-L336", title: "TODO.md entry T020"}
---
`make_anti` (`vibegraph-lib/src/ufo/particles.rs:111`) sets
`color: -self.color` for every particle. UFO's `anti()` negates only
non-self-conjugate representations (3, 6) and leaves 1 and 8 alone. So the
model's antiparticle entries carry `color = -1` and `-8`.

Decay-chain event records work around it locally (`member_line_pdg`,
[note 38 §4 E1](../../38-process-grammar-sprint-plan.md)). Any new reader of
`Particle::color` on an antiparticle meets the quirk, and a test that
compares colour to 1 or 8 is wrong for antiparticles. Before changing it, list
every existing reader of `Particle::color` (LSP references), since some may
rely on the sign to tell a particle from its antiparticle.
