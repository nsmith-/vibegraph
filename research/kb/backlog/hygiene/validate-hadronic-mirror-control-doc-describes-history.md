---
type: Backlog Item
title: "A validate_hadronic.rs test doc describes the unrotated mirror reading as history"
description: "madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2's doc calls the unrotated reading the one 'the integrand used to take'; it should state the control in its own terms."
area: hygiene
state: open
priority: low
closes_when: "The doc near validate_hadronic.rs:3386 describes the unrotated reading as the control it is, with no reference to earlier code."
blocked_by: []
opened: 2026-11-08
tags: [comments, tests, mirror]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-08}
sources:
  - {id: hv2, resource: "../../sprints/cms-gridpack-coverage/sessions/H-V2-report.md", title: "H-V2 report, Found 5"}
  - {id: test, resource: "../../../../vibegraph-lib/tests/validate_hadronic.rs", title: "validate_hadronic.rs, about line 3386"}
---
After #107, the doc of
`madevents_scale_configuration_is_drawn_from_its_own_matrix_elements_amp2`
(`vibegraph-lib/tests/validate_hadronic.rs`, about line 3386) says the
unrotated readings are those "the integrand used to take for a group's mirrored
term". The comment guidelines in `AGENTS.md` ask for docs that describe the
code as it is now. A one-line rewording; it could be folded into
[validation-test-docs-glued-and-vegas-claim](validation-test-docs-glued-and-vegas-claim.md),
which fixes other docs in the same file.
