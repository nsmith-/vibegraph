---
type: Backlog Item
title: "SMEFTsim's charged contacts with CP-odd coefficients bind differently since #214, and no MadGraph row covers them"
description: "V_587, V_593, V_627 and V_633 (t b W γ/Z) and V_290/V_329 (t b W H) now bind their partner's coupling wherever ctWIm or cbWIm is nonzero. Only toy rows with the same structures pin this."
area: validation
state: open
priority: medium
closes_when: "A committed per-diagram x per-helicity standalone row in SMEFTsim, with ctWIm or cbWIm nonzero and reaching a charged t b W γ/Z contact (e.g. b g > t W- a under restrict_vg_ctWIm), matches MadGraph 3.7.1."
blocked_by: []
opened: 2026-11-26
tags: [smeftsim, cp-violation, couplings, standalone, oracle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-26}
sources:
  - {id: l-d2, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-D2-report.md", title: "L-D2 report, Found 3"}
---
Found by sprint L's L-D2 (#214). On a real-coefficient card the bound value
equals the vertex's own, which a unit test checks. With CP-odd coefficients the
binding changed, and SMEFTsim is where CMS users set them. Before #214, and at
v0.2.2, these contacts bound the vertex's own coupling, as dim6top's did.
