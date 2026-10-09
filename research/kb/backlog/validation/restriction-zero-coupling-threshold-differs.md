---
type: Backlog Item
title: The restrict-card zero-coupling threshold differs from MadGraph's
description: "The loader prunes couplings with |g| < 1e-20; MadGraph prunes |g| < 1e-13 and reruns with strict zeros when any coupling lies in [1e-13, 1e-10), so the two can keep different vertices."
area: validation
state: open
priority: medium
closes_when: "The restriction pruning applies MadGraph's rule (1e-13, with the strict-zero rerun), pinned by a test with a coupling between the two thresholds, or the difference is refused by name."
blocked_by: []
opened: 2026-10-09
tags: [ufo, restrict-card, madgraph-parity, model]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: code, resource: "../../../../vibegraph-lib/src/ufo/mod.rs", title: "ufo/mod.rs is_zero_coupling (~:710-715)"}
  - {id: mg, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/models/import_ufo.py#L2549-L2558", title: "MadGraph import_ufo.py detect_identical_couplings at the pin"}
---
`is_zero_coupling` (`vibegraph-lib/src/ufo/mod.rs` ~:710-715) treats a coupling
as zero under the restrict parameters when `|g| < 1e-20`. MadGraph's
`detect_identical_couplings` (`import_ufo.py:2549-2558` at the pin) does this:

- it treats `|g| < 1e-13` as zero;
- if any coupling lies in `[1e-13, 1e-10)`, it reruns with `strict_zero=True`,
  which prunes only exact zeros.

A coupling between `1e-20` and `1e-13` therefore survives here and is pruned
by MadGraph, unless some other coupling triggers the strict rerun. Then the
two sides have different vertex sets, diagram counts and amplitudes. No banked
restrict card is known to sit in that window: every gated row passes. A user's
card with a small Wilson coefficient could.

MadGraph also rounds coupling values to merge identical ones (`limit_to_6_digit`).
Check whether that matters here before porting the threshold. Found by
Phase 3 verifier V13; both thresholds confirmed at the pin.
