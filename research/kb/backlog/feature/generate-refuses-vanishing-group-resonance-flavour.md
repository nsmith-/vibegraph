---
type: Backlog Item
title: "generate refuses a run whose vanishing flavour group carries a massless s-channel quark line"
description: "On g2HDM card 894, integrate succeeds and sets aside a group whose matrix element vanishes, but generate refuses the whole run before any event: the group's s-channel quark line has no shared flavour."
area: feature
state: open
priority: high
closes_when: "generate samples card 894 and the five-line reproduction below; a group integrate found vanishing never blocks generation; whether a live group with such a line is refused is measured, and fixed if it is; a test pins the reproduction."
blocked_by: []
opened: 2026-11-23
tags: [generate, refusal, flavour-groups, resonance, g2hdm, bsm, user-facing]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
On main b4e284a3, `vibegraph generate` exits before drawing an event on an
artifact of g2HDM card 894 (M1000). It refuses by name, without a panic:

```
ERROR the resonance 2 of g u > e+ ve b b~ d has no flavour in the subprocess
with outgoing [-11, 12, 5, -5, -2] that shares its matrix element
```

A reproduction that runs in seconds:

```
import model g2HDM_UFO
define q = u d~
define qq = d u~
generate g q > h+ qq QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
add process c b~ > h+ QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
```

`integrate` gives σ = 2.37 ± 0.20 fb and logs that the one flavour group's
matrix element vanishes at the card. `generate` then refuses.

**Mechanism** (sprint L's g2HDM session, report addendum a5060cfc):

- `g u > h+ d` and `g d~ > h+ u~` share a flavour group because both matrix
  elements vanish at the card: H± has no u or d coupling in g2HDM.
- Their diagrams carry a massless s-channel quark line, u in one subprocess
  and d~ in the other. `member_line_pdg` (`lhef/resonance.rs`) finds no
  flavour common to the group.
- `group_resonances` (`generate.rs`) then refuses the whole run, although
  integrate already set the group aside as carrying no σ.

**Not yet measured:** whether a group with a non-zero matrix element and such
a line is refused too. If it is, the defect reaches live groups and more
models. Every v0.2.2 g2HDM card with dead groups of this shape integrates but
cannot generate, and other multi-flavour BSM cards likely can't either.
MadGraph generates these cards.

The likely fix has two parts: skip resonance resolution for groups that
integrate found vanishing, and for a live group, take the line's flavour from
the subprocess's own legs.
