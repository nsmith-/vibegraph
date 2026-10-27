---
type: Backlog Item
title: "A fixed-energy run card accepts a non-positive or non-finite beam energy"
description: "RunCard::parse refuses a proton beam below 0.938 GeV, but on lpp = 0 an ebeam of 0, a negative value or NaN parses; MadGraph clamps a beam below its leg mass up to the mass."
area: feature
state: open
priority: low
closes_when: "A fixed-energy card with ebeam at or below its leg's mass (or non-finite) is refused at parse with a test naming the field."
blocked_by: []
opened: 2026-10-26
tags: [run-card, fixed-beam, refusal, beams]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-26}
sources:
  - {id: v1, resource: "../../sprints/unequal-beams/sessions/V1-report.md", title: "unequal-beams V1 report, Found 5"}
  - {id: v2, resource: "../../sprints/unequal-beams/sessions/V2-report.md", title: "unequal-beams V2 report, Found 4"}
---
The unequal-beams PR added `RunCardError::ProtonBeamBelowMass`
(`vibegraph-lib/src/runcard.rs`), checked only when `(lpp1, lpp2) == (1, 1)`.
On `lpp = 0` nothing checks `ebeam1`/`ebeam2`: a test asserts 0.5 GeV parses,
and 0 was not tried. MadGraph clamps a beam energy below the leg mass up to it
(`genps.f:670-671` at `b7687064`). Which line to draw at fixed beams (the leg
mass, or the process threshold) is the choice to make
([V1](../../sprints/unequal-beams/sessions/V1-report.md)).
