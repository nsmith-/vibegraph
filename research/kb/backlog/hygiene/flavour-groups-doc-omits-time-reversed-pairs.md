---
type: Backlog Item
title: "flavour-groups.md does not say that time-reversed subprocesses share a group"
description: "g g > q q~ and q q~ > g g share one flavour group on every p p > j j card (one spin-summed |M|², weighted apart by their averaging factors); the concept states only the conjugate-pair rule."
area: hygiene
state: open
priority: low
closes_when: "hadronic/flavour-groups.md states that time-reversed subprocesses with one spin-summed |M|² share a group and why their weights stay right, next to the conjugate-pair rule."
blocked_by: []
opened: 2026-11-07
tags: [flavour-groups, docs, knowledge-bundle]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-07}
sources:
  - {id: hn1, resource: "../../sprints/cms-gridpack-coverage/sessions/H-N1-report.md", title: "H-N1 report, Found 4"}
---
Found by H-N1. On every `p p > j j` card, `g g > q q~` and `q q~ > g g` share one
group (rounding 5.7e-13, f32 ratio 2.7e6): time-reversed processes with one
spin-summed `|M|²`, weighted apart by their own averaging factors. H-N1 found no
bug (it predates the sprint and is gated bit for bit in `pp_to_jj`), but
[flavour-groups](../../hadronic/flavour-groups.md) describes only the
charge-conjugate pairing, so a reader cannot tell this join is intended.
