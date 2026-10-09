---
type: Backlog Item
title: The fixed-beam α survey runs serially
description: "MultiChannel::adapt_alphas surveys one point at a time, while the hadronic survey is chunked over rayon; the 579- and 615-channel fixed-energy 2→6 rows go through the serial one."
area: performance
state: open
priority: low
closes_when: "The fixed-beam α survey's share of wall time on the 2→6 fixed-energy rows is measured, and the survey is parallelised as the hadronic one is or recorded as not worth it."
blocked_by: []
opened: 2026-10-09
tags: [multichannel, alpha-survey, parallelism, two-to-six, measure-first]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: channel, resource: "../../../../vibegraph-lib/src/phasespace/channel.rs", title: "phasespace/channel.rs MultiChannel::adapt_alphas (~:493-560)"}
  - {id: proton, resource: "../../../../vibegraph-lib/src/proton.rs", title: "proton.rs, the chunked parallel hadronic survey (~:2887-2960)"}
---
`MultiChannel::adapt_alphas` (`vibegraph-lib/src/phasespace/channel.rs`
~:493-560) evaluates its survey points serially. The proton-beam survey
(`proton.rs` ~:2887-2960) splits the same work into rayon chunks. The fixed-beam
2→6 rows, with 579 and 615 channels, take the serial path, so their survey
runs on one core while the integration after it uses all of them.

Measure the survey's share of those rows' wall time before changing anything.
Note that [alpha-survey-iteration-limited-on-wide-splits](alpha-survey-iteration-limited-on-wide-splits.md)
says six iterations bind there, not points. Found by Phase 2 drafter D7.
