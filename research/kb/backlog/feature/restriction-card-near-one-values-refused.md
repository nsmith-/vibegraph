---
type: Backlog Item
title: "Restriction cards that set a free parameter to 9.999999e-1 are refused"
description: "Several SMEFTsim and X53 restriction cards use 9.999999e-1 to fix a parameter; the loader refuses them with a clear message, but MadGraph uses these cards as they are. 38+ refusals."
area: feature
state: open
priority: medium
closes_when: "MadGraph's restriction semantics for a value of 9.999999e-1 are read and reproduced, so the cards below load with the same restricted model, with a test; or the refusal is kept with a recorded reason."
blocked_by: []
opened: 2026-11-16
tags: [ufo, restriction, refusal, smeftsim, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-16}
---
Reported by simshop's scan of the CMS model store. The cards are in
SMEFTsim_U35_MwScheme_UFO_b_massless, X53_*_diagonalCKM_5FS_UFO and
SMEFTsim_topU3l. MadGraph treats 1 (and values written to equal it) as a
parameter to fix at that value in the restricted model. Read its rule,
including how it tells 9.999999e-1 from 1, before changing the loader.
