---
type: Design Decision
title: "CMS PDF memory: lossless knot storage first, up to 2× time, aim below 1.5 GB, guarded in CI"
description: "Members are stored as their knots with the cubic built on use, bit-identical to today's interpolation; generate may slow by up to 2× on the CMS request; the target is below 1.5 GB peak; a default test pins bytes per loaded member."
decided: 2026-11-03
decided_by: human:nsmith-
status: stable
tags: [pdf, memory, cms, sprint-f]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-03}
verified: [{by: "human:nsmith-", at: 2026-11-03}]
sources:
  - {id: log, resource: "../log.md", title: "cms-pdf-memory log, 2026-11-03: the user's answers to D1–D4"}
---
The user's answers to the sprint's questions D1–D4, each the recommended option.

- **D1. Technique.** Each member is stored as its knots, and the x-direction
  cubic is built on use from the same arithmetic as today's precomputation, so
  every interpolated value is bit-identical. Two alternatives come back to the
  user only if this misses the target: set-at-a-time passes over buffered
  per-event records (a design change of `generate`) and `f32` storage.
- **D2. Time.** Up to 2× wall time for `generate` on the CMS request on
  `pp_to_ll`, against the current code on the same machine.
- **D3. Target.** Below 1.5 GB peak RSS at `-j 1` and `-j 4`. That leaves
  headroom under the 2 GB in the item's `closes_when`.
- **D4. Guard.** A default (CI) test pins heap bytes per loaded member against
  its knot count. The RSS figure is a recorded measurement. A script fetches
  the 880 members, so the long tier can run the CMS gate.
