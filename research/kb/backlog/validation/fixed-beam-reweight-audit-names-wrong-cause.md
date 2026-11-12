---
type: Backlog Item
title: "Fixed-beam reweighting of an identical-particle decay chain fails its audit and names the wrong cause"
description: "On g g > z u u~, z > u u~ at fixed energy the reweighter's |M|² differs from the integrand's by up to 19% on 22 of 50 events; the audit refuses the file, blaming the subprocess assignment."
area: validation
state: open
priority: medium
closes_when: "Per-event dumps of both |M|²'s and the leg assignment locate the difference, fixed-beam reweighting of such a card either agrees with the integrand or is refused naming the real cause, and a test pins it."
blocked_by: []
opened: 2026-11-12
tags: [reweighting, decay-chains, identical-particles, fixed-beam, error-messages]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-12}
sources:
  - {id: jw1, resource: "../../sprints/v021-fixes/sessions/J-W1-report.md", title: "J-W1 report, Found 6"}
---
Found by J-W1 (Found 6). On `g g > z u u~, z > u u~` at fixed energy, the
reweighter's card-point |M|² differs from the integrand's by up to 17–19% on
21–22 of 50 events. That happens at `cut_decays = F` and `T` alike. The audit
then refuses the file with "reweighting evaluated 22 of 50 events in another
subprocess than the integrand drew them in".

The failure is loud, not silent, but the message names the wrong cause. The
likely cause is how the reweighter assigns the record's identical quarks to legs
on a decay-chain set. That was not diagnosed.

The pairing-weighted case (`cut_decays = F` with the core/decay overlap) is
refused earlier, by name. This item covers the `T` case and other
identical-particle chains. v0.2.1's release notes list it under Known issues.
