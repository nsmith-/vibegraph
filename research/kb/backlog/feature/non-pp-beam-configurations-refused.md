---
type: Backlog Item
title: Beam configurations other than p p and fixed-energy partonic are refused
description: RunCard::parse admits only lpp = (0,0) and (1,1); antiproton, mixed, lepton-PDF and photon beams are hard errors.
area: feature
state: open
priority: low
closes_when: At least antiproton (lpp = -1) and mixed beam configurations integrate and generate against a MadGraph sigma reference; any configuration still refused names its reason.
blocked_by: []
opened: 2026-08-02
tags: [descoped-v1, beams, pdf, runcard]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-06}
sources:
  - {id: todo, resource: "https://github.com/nsmith-/vibegraph/blob/466a60f/TODO.md#L584-L586", title: "TODO.md entry T049"}
---
Descoped from the v1 release goal (user, 2026-08-02). `RunCard::parse`
returns `RunCardError::UnsupportedLpp` for anything but `(0,0)` and `(1,1)`
(`vibegraph-lib/src/runcard.rs:433`).

Missing configurations: antiproton beams (`lpp = -1`, Tevatron), mixed
configurations (one hadron beam, one fixed-energy parton), lepton-PDF beams
and photon beams. Antiproton is the cheapest (a charge-conjugated PDF lookup
and the matching flavour bookkeeping in `hadronic.rs` / `proton.rs`);
lepton-PDF and photon beams need PDF sets the fetch path does not handle yet.
Each needs a MadGraph sigma row before it is accepted.
