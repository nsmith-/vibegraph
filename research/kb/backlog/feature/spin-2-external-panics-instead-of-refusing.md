---
type: Backlog Item
title: A spin-2 external leg may reach a panic instead of a refusal
description: "helicity_states lists five states for spin code 5, so compile does not refuse it, and build_external_core panics on 'unsupported external spin code' where spin 3/2 gets EvalError."
area: feature
state: open
priority: low
closes_when: "A process with a spin-2 external leg is refused with an error naming the spin (as spin 3/2 is), by a test, or shown to be refused earlier, with the panic arm unreachable."
blocked_by: []
opened: 2026-10-09
tags: [spin, refusal, non-sm-ufo, panic]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: particles, resource: "../../../../vibegraph-lib/src/ufo/particles.rs", title: "ufo/particles.rs helicity_states (~:91-101)"}
  - {id: run, resource: "../../../../vibegraph-lib/src/helas/eval/run.rs", title: "helas/eval/run.rs build_external_core's panic arm (~:1941)"}
---
`Particle::helicity_states` (`vibegraph-lib/src/ufo/particles.rs` ~:91-101)
returns five helicities for spin code 5. So `AmplitudeEvaluator::compile`
(`compile.rs` ~:206), which refuses a spin only when that list is `None`, lets
spin 2 through. `build_external_core` (`helas/eval/run.rs` ~:1941) then panics
with "unsupported external spin code". Spin 3/2 (code 4) gets
`EvalError::UnsupportedSpin` instead. A crash where the release scope asks for
a hard error is the gap.

Unchecked: whether every spin-2 vertex already fails Lorentz parsing or rooting
before evaluation, which would make the panic unreachable in practice. Settle
that first. The support itself is
[spin-2-and-spin-3-2-particles-unsupported](spin-2-and-spin-3-2-particles-unsupported.md).
Found by Phase 2 drafter D9.
