---
type: Backlog Item
title: "The AMP2 read still pays a weight dispatch and two bounds checks per amplitude"
description: "filled_amp2 is 4.4% of integrate on p p > e+ e- j: each amplitude pays an AmpWeight enum dispatch, a constant-pool index and two bounds checks, about 24 instructions for 10 of arithmetic."
area: performance
state: open
priority: low
closes_when: "Each BoundAmplitude reads amplitude weights from a resolved table refreshed whenever its pools change, a real weight stays real, and the saving is measured by instruction count on p p > e+ e- j."
blocked_by: []
opened: 2026-11-10
tags: [performance, evaluator, configuration-shares, amp2]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares, with the share-accounting performance commits (found there)"}
---
After the share-accounting optimisation (nsmith-/vibegraph#124), the
remaining overhead on `p p > e+ e- j` is +5.74% of instructions, of which
`filled_amp2` is +4.44%. Disassembly shows about 24 instructions per
complex-weight amplitude, about 10 of them arithmetic: an `AmpWeight` enum
dispatch, an index into the constant pool and two bounds checks. A table of
resolved weights per `BoundAmplitude` would remove them, but it must be
refreshed whenever `pools_mut`/`set_pools` rewrite the pools (the bound
amplitude is shared across threads), and a real weight must not become a
complex one, which would change non-finite and signed-zero behaviour.
