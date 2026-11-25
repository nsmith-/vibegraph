---
type: Backlog Item
title: "Diagram and integrate artifacts cannot see a UFO-reader change, and no test forces ENUMERATION_VERSION to move"
description: "Both artifacts embed the parsed model, so a reader fix (#178's GoldstoneBoson spelling) leaves every digest and the probe agreeing with a stale file. v0.2.2's Bprime artifact gave 17.70 pb under HEAD, against 2.527 fresh."
area: validation
state: open
priority: high
closes_when: "A change in how the UFO reader interprets a model either invalidates existing artifacts automatically (a model-reader version folded into the artifact's compatibility check) or is caught by a known-answer test on an edited toy UFO that fails until the version moves. A mutation that reverts #178's spelling fix without a bump makes the test fail."
blocked_by: []
opened: 2026-11-25
tags: [artifacts, ufo, versioning, silent-wrong-answer, release]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-25}
---
Found by sprint L's release-prep PR (#211). v0.2.2 and HEAD were built and
cross-loaded on 5 cases:
- **Bprime `t t~ > z h`:** v0.2.2 integrates to 17.703 pb and HEAD to 2.527 pb.
  HEAD read v0.2.2's integrate artifact silently, and wrote events with XSECUP
  17.70.
- **`c s~ > t b~`:** the same silent read, 105.34 against 104.71 fb fresh.
- The other cases were caught: the MSSM chirality case by the probe, and
  `p p > t t~ j` by the map digest.

The cause is that both artifacts embed the model as the writing release read it,
and nothing re-reads the UFO. #211 bumped `ENUMERATION_VERSION` to 4, so every
v0.2.2 artifact is refused. Main-built artifacts from between #178 and the bump
were exposed the same way, and v4 refuses them too. Nothing forces the next
reader fix to bump. This is the same lesson as sprint K's #161, one layer down.
