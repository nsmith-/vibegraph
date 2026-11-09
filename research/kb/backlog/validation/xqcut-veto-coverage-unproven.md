---
type: Backlog Item
title: "The xqcut veto's coverage of coloured jet splittings is pinned on three topologies, not proven"
description: "The guard counts MLM's kT veto as a jet-pair regulator; the iqjets bookkeeping across a later colourless vertex is not derived, and colourless-line splittings stay conservatively refused."
area: validation
state: open
priority: medium
closes_when: "The iqjets invariant is derived (or the per-event veto test is extended over the CMS census processes), and whether a coloured pair also split off a γ* line is regulated is settled by a per-event probe."
blocked_by: []
opened: 2026-11-09
tags: [cuts, mlm, xqcut, ir-safety]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-09}
sources:
  - {id: ic2, resource: "../../sprints/v02-last-refusals/sessions/I-C2-report.md", title: "I-C2 report and addenda"}
---
From I-C2's second addendum (aee3812e):
- `final_state_vertex` can clear a quark's jet tag at a later colourless
  vertex. It is unproven that the veto still fires for a `g* → q q̄` pair whose
  line meets one, as in `p p > j j j j` with an s-channel Z/γ*.
- A coloured jet pair that another diagram also splits off a colourless line
  (QED ≥ 4) still needs `mmjj` or `drjj` on an xqcut card. That may be a false
  refusal.
