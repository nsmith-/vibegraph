---
type: Backlog Item
title: "The IR-unsafe exemption guard checks massless coloured legs only"
description: "refuse_ir_unsafe_exemption looks at massless coloured core legs; a massless photon or lepton exempt in one pairing and core in another has the same uncut-collinear failure once the window rule lands."
area: validation
state: blocked
priority: medium
closes_when: "When the cut_decays window rule lands, the guard covers every massless leg with a gauge charge (or is restated over the exempting pairing's |M|² support), with a test that exempts a massless colourless core leg and is refused."
blocked_by: [cut-decays-identical-quarks-cut-a-stricter-observable]
opened: 2026-11-07
tags: [cuts, decay-chains, ir-safety, guards]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hc3, resource: "../../sprints/cms-gridpack-coverage/sessions/H-C3-report.md", title: "H-C3 report, Found 1: the guard sees only coloured legs"}
---
Found by H-C3. The guard added for `cut_decays` exemptions
(`massless_coloured_core`, `CutError::IrUnsafeExemption`) refuses exempting a
massless coloured leg that is a core particle in some pairing. A massless
colourless leg (a photon, or a lepton under a massless-lepton restriction) that
is a core particle in one pairing and a decay product in another fails the same
way: an uncut γ or ℓ collinear to a charged leg, so σ is finite and wrong.

Nothing exempts such a leg today, because the every-pairing rule never does. The
kinematic window rule of
[cut-decays-identical-quarks-cut-a-stricter-observable](cut-decays-identical-quarks-cut-a-stricter-observable.md)
would exempt by pairing, which is why this is blocked on it. That item already
carries H-C3's other guard finding (rebuild the guard over the exempting
pairing's diagrams).
