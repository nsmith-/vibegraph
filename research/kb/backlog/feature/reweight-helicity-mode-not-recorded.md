---
type: Backlog Item
title: "A sample's reweight ratio (per event helicity or summed) is recorded only in a log line"
description: "Neither the LHE header nor the generated record says which ratio the weights carry, and the per-event refusals cannot be checked before generate. simshop wants both, to label samples and refuse in its wizard."
area: feature
state: open
priority: medium
closes_when: "The generated record and the event file's header carry the ratio (e.g. reweight_helicity: per-event | summed). The polarized, signed and 2 → 1 / 1 → n per-event refusals can be raised before generate (by integrate --check with the reweight card, or a dedicated check), with a test for each."
blocked_by: []
opened: 2026-11-28
tags: [reweight, helicity, provenance, simshop, ldjson, lhef]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-28}
---
Found in simshop's v0.2.3 release-notes review.

Since #182 the default ratio is per event helicity, and simshop's cards have
no `change helicity` line. A downstream tool can therefore tell a sample's mode
only from the release that wrote it.

The per-event refusals are raised in `ReweightPlan::new`, called from
`generate` after the artifact loads (`reweight/engine.rs`
`check_event_helicity`). The identical-particle refusal needs the events, so it
stays a `generate`-time one.
