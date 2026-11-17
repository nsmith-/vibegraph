---
type: Backlog Item
title: "A restriction card with no CKMlambda value is refused"
description: "22 SMEFTsim_U35_MwScheme_UFO_b_massless restrictions give no CKMBLOCK 2 (CKMlambda) value; vibegraph refuses them as MadGraph's restriction needing it. What MadGraph does is unchecked."
area: feature
state: open
priority: medium
closes_when: "MadGraph's handling of a parameter absent from a restriction card is read and reproduced (kept at its UFO default and treated as free, or otherwise), so the 22 restrictions load with the restricted model MadGraph builds, with a test; or the refusal is kept with a recorded reason."
blocked_by: []
opened: 2026-11-17
tags: [ufo, restriction, refusal, smeftsim, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Reported by simshop's SMEFT probe of the CMS model store (vibegraph 0.2.1).
22 restrictions of `SMEFTsim_U35_MwScheme_UFO_b_massless` are refused at model
load:

> the restriction card gives no value for CKMlambda (block CKMBLOCK, code 2),
> which MadGraph's restriction requires

Whether MadGraph does require it is the open question. Its restriction code
may fall back to the UFO default for an absent entry. Read
`models/import_ufo.py`'s restriction path before changing the loader. The same
model's near-one values are
[`restriction-card-near-one-values-refused`](restriction-card-near-one-values-refused.md).
