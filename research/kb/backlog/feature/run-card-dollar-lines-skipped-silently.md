---
type: Backlog Item
title: "A run-card value left as a $ placeholder runs at the default instead of being refused"
description: "CMS InputCards lines such as '$DEFAULT_PDF_SETS = lhaid' are skipped; integrate then runs at the default lhaid without a word. Measured 15.4% low on the ZH SMEFTsim cards."
area: feature
state: open
priority: high
closes_when: "integrate, generate and --check refuse by name a run card whose value for any key is an unfilled $ placeholder (naming the key and the placeholder, and that CMS's gridpack script fills it at build time), the run-card report marks such a key refused rather than honoured, and a CLI test pins both on a CMS InputCards-style card."
blocked_by: []
opened: 2026-11-19
tags: [run-card, refusal, silent-wrong-answer, cms-gridpacks]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-21}
---
Found by K-C1 ([report](../../sprints/v022-smeft-and-refusals/sessions/K-C1-report.md), Found).
The user raised it to high, to be fixed in the next release (2026-11-21).

simshop's v0.2.2 SMEFT re-scan measured the effect. The 7 SMEFTsim_topU3l ZH
InputCards run cards contain `$DEFAULT_PDF_SETS = lhaid`, which CMS's gridpack
script replaces at build time. vibegraph skipped the line, resolved `lhaid` to
its default 247000 (NNPDF23_lo_as_0130_qed) and integrated without a warning:
every σ came out 15.4% low (pull ≈ −30) until the line was filled in as
`325300 = lhaid`. v0.2.1 and v0.2.2 give bit-identical σ on that card, so it is
not a regression.

`vibegraph run-card` already detects the line
(`{"name":"lhaid","value":"$DEFAULT_PDF_SETS","problem":"placeholder"}`) but
still reports `lhaid` as honoured at the default. A placeholder means the card
is a template, not a card: a named refusal is the right outcome, as a warning
would still let a batch job run at the wrong PDF. Other `$DEFAULT_PDF_MEMBERS`
style lines are covered by the same rule.

simshop and the CERN host's campaign are not affected: both fill the
placeholder (simshop's clone uses 325300 for five flavours and 325500 for
four) or use the gridpack's built cards.
