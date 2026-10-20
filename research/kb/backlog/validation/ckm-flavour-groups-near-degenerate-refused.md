---
type: Backlog Item
title: "CKM-model flavour groups separated by 1e-6 to 1e-8 are refused as rounding"
description: "derive_flavor_groups refuses groups separating below GROUP_SEPARATION_MIN = 1e-6; four CMS CKM cards hit it at 9.08e-7 and 3.88e-8, which may be a real CKM difference."
area: validation
state: open
priority: medium
closes_when: "The 9.08e-7 and 3.88e-8 separations on the CMS CKM cards are shown to be either a physical CKM difference (and the groups are kept apart or merged soundly) or rounding (and the floor or probe is fixed), with a test on one of those processes."
blocked_by: []
opened: 2026-10-20
tags: [flavour-groups, ckm, proton, triage, cms]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: survey, resource: "../../validation/cms-gridpack-survey-2026-10.md", title: "CMS gridpack survey, 88 production cards (e3f543f)"}
---
`derive_flavor_groups` refuses a partition whose closest groups separate by
less than `GROUP_SEPARATION_MIN = 1e-6` at their best probe point
(`ProtonError::DegenerateGroups`, `vibegraph-lib/src/proton.rs:151`, raised at
`:833`), on the reasoning that such a split is rounding.

In the [CMS gridpack survey](../../validation/cms-gridpack-survey-2026-10.md)
cards 006, 010, 020 and 089, all with a CKM model, are refused at 9.08e-7 and
3.88e-8. Products of CKM elements can differ at that level, so the split may be
physical. **Inferred**, not measured: compare the groups' couplings
symbolically, or their |M|² at higher precision, before choosing between a
tolerance change and a fix.

The sound grouping criterion is
[flavour-grouping-unsound-sampled-criterion](../feature/flavour-grouping-unsound-sampled-criterion.md).
