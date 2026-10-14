---
type: Backlog Item
title: "A few comments still cite plans or describe old code"
description: "run.rs and root_diagram.rs cite plans; a few docs name deleted items (member_luminosity) or misdescribe ParsedModel; ufo/lorentz.rs has a history comment; FIELD_CLASSES' alignment is stale."
area: hygiene
state: open
priority: low
closes_when: "Each listed comment describes the code in its own terms."
blocked_by: []
opened: 2026-10-10
tags: [comments, stale-comment]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-10}
sources:
  - {id: f-a-report, resource: "../../sprints/hygiene/sessions/F-A-report.md", title: "F-A report (hygiene sprint)"}
  - {id: f-b-report, resource: "../../sprints/hygiene/sessions/F-B-report.md", title: "F-B report (hygiene sprint)"}
  - {id: f-c-report, resource: "../../sprints/hygiene/sessions/F-C-report.md", title: "F-C report (hygiene sprint)"}
---
F-A Found 4, F-B Found 1, F-C Found 4, V1 Found 7. Also from the close-out kb pass: the doc of `a_members_share_of_the_draw_is_its_share_of_the_parton_luminosity` (`proton.rs` ~:5712) links the deleted `FlavorGroup::member_luminosity`, and the `ParsedModel` doc says it holds \"everything in `UFOModel` except `topo`\" though it also carries `propagators`.

Filed at the close-out of the [hygiene sprint](../../sprints/hygiene/sprint.md); evidence in [F-A](../../sprints/hygiene/sessions/F-A-report.md), [F-B](../../sprints/hygiene/sessions/F-B-report.md), [F-C](../../sprints/hygiene/sessions/F-C-report.md).
