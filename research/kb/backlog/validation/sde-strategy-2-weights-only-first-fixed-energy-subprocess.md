---
type: Backlog Item
title: "At fixed energy under sde_strategy = 2, only the first subprocess has configuration weights"
description: "Fixed-energy runs derive channel-cut forests for subprocess 0 only; under sde_strategy = 2 later subprocesses get no shares and draw configurations from the wrong forest."
area: validation
state: open
priority: medium
closes_when: "The fixed-energy integrand derives one ChannelSet per subprocess (as the proton path does), the share and event-generation paths index it by subprocess, and a test on a fixed-energy sde_strategy = 2 card with two subprocesses of different diagram content pins both."
blocked_by: []
opened: 2026-11-10
tags: [phase-space, multichannel, sde-strategy, events]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-10}
sources:
  - {id: pr124, resource: "https://github.com/nsmith-/vibegraph/pull/124", title: "Configuration shares (found there)"}
  - {id: sde, resource: "../../phase-space/madevent-single-diagram-enhancement.md", title: "MadEvent single-diagram enhancement"}
---
`use_running_coupling` in `vibegraph-lib/src/hadronic.rs` calls
`compile_configuration_weights(&[(self.subs[0].evaluator(), diagrams)], …)`
and keeps the one `ChannelSet` it returns. Under `sde_strategy = 2`
([single-diagram enhancement](../../phase-space/madevent-single-diagram-enhancement.md))
MadEvent weights configurations by those channel-cut factors, one forest per
subprocess. The proton path passes every subprocess; the fixed-energy path
does not.

Two consequences, on a fixed-energy card with several subprocesses at
`sde_strategy = 2`:

- **Configuration shares** (nsmith-/vibegraph#124): `split_term` gives every
  subprocess after the first zero weight, so its whole term is banked as
  `configuration: None`. σ is unaffected; the ranking cannot rank those
  subprocesses.
- **Event generation** (on `main` before #124): the configuration and colour
  draw for an event of subprocess `k > 0` fills that subprocess's buffer with
  subprocess 0's channel cuts. `ChannelSet::channel_cuts` asserts the buffer
  length, so a subprocess with a different configuration count panics; one
  with the same count draws from the wrong forest silently. Kinematics and σ
  are unaffected; the configuration (and so the colour flow and clustering
  root) is not.

Fixed-energy beams already require every subprocess to have the same ordered
external particles (`HadronicError::InconsistentExternals`), so several
subprocesses arise only from polarizations (same diagrams: harmless by
coincidence) or decay-chain lines reaching the same final state through
different assignments (different diagrams: affected). MadGraph auto-selects
strategy 2 for single-colour-flow processes, so such a card can reach it
without asking.
