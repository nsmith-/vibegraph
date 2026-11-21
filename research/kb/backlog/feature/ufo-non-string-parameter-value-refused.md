---
type: Backlog Item
title: "A UFO Parameter whose value is not a string is refused (SMEFTsim topU3l ctGpatched)"
description: "All 11 restrictions of SMEFTsim_topU3l_MwScheme_UFO_ctGpatched are refused with \"parameter 'Gstrong': `value` must be a string\"; whether MadGraph loads it is unchecked."
area: feature
state: open
priority: medium
closes_when: "MadGraph 3.7.1's handling of the ctGpatched model is read and recorded; if MadGraph loads it, vibegraph accepts the same non-string value forms as import_ufo.py does and the 11 restrictions enumerate the store probe, with a test on a minimal UFO; if MadGraph refuses it, the refusal names that and the item closes on the reason."
blocked_by: []
opened: 2026-11-21
tags: [ufo, model-loading, smeftsim, cms-model-store, simshop]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
---
Found by simshop's v0.2.2 SMEFT re-scan (2026-11-21). The user asked for it in
the next release.

Every restriction of the CMS store's `SMEFTsim_topU3l_MwScheme_UFO_ctGpatched`
is refused at model load:

- bare, SMlimit_massless, massless, massless_2t, massless_2tn, massless_4t,
  massless_4t_2t2b, massless_4t_2t2b_cQQ1, massless_VBS, massless_VH,
  massless_Wtb;
- message: `Parameter parse error: parameter 'Gstrong': value must be a string`.

The patched UFO gives the parameter `Gstrong` a value that is not a Python
string. Whether MadGraph's `import_ufo.py` accepts that form is unread.
simshop's lookup of the file timed out, so the exact value form is not yet
recorded: reading the model's `parameters.py` is the first step.

This class is not one of the named out-of-scope refusals of the v0.2.2 exit
criterion (CKMlambda, squared orders, Majorana, NLO).
