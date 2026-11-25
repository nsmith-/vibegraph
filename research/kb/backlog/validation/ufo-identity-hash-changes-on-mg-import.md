---
type: Backlog Item
title: "MadGraph 3.7.1 rewrites a py2 UFO in place on import, so a UFO's identity hash depends on whether MG has loaded it"
description: "auto_convert_model rewrites __init__.py, object_library.py and write_param_card.py; two copies of VLQ_v4_4FNS_UFO identical in amplitude content gave different identity hashes."
area: validation
state: open
priority: low
closes_when: "The UFO identity recipe used by gpdb and the standalone banks hashes only the amplitude-bearing files (parameters, couplings, vertices, lorentz, particles, propagators, coupling_orders, decays, function_library, and the restriction card), or hashes before MG's first import. The recipe is documented, and a test shows a converted and an unconverted copy hash the same."
blocked_by: []
opened: 2026-11-25
tags: [provenance, ufo, madgraph, identity-hash]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
sources:
  - {id: l-i1, resource: "../../sprints/v023-orders-majorana-timeouts/sessions/L-I1-report.md", title: "L-I1 report, Found"}
---
Found by sprint L's L-I1 probe. The validator's gpdb-staged
`VLQ_v4_4FNS_UFO` hashed to `52d46852…` and the physics session's local copy to
`83bdef2c…`. Under the recipe, a sha256 over the sorted "relpath\tsha256" lines of
`*.py`, 12 of the 15 files were byte-identical. The other three are
infrastructure: `__init__.py`, `object_library.py` and `write_param_card.py`.
MG 3.7.1's `auto_convert_model` rewrites exactly those when it converts a py2
UFO on import.

A provenance check on the current recipe therefore fails on a copy MG has
touched, even though the amplitudes are the same.
