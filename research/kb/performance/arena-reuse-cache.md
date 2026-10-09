---
type: Design
title: Arena reuse between helicity-summed read-outs
description: "A fill_token plus bit-compared momenta stamp on ScratchSpace lets AMP2, |M|² and event read-outs share one fill; any writer retires it; prefix sharing was measured a NO-GO."
status: draft
tags: [performance, evaluator, arena, amp2, cache]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n31-e3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/31-perf-sprint-3-plan.md#L704-L749", title: "Note 31 §E3 (chain-B draw work-sharing, measured results)"}
  - {id: run-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L55-L330", title: "ScratchSpace, fill_token and next_fill_token"}
  - {id: run-fill-for, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L699-L727", title: "BoundAmplitude::fill_for"}
measured:
  - {commit: f3d6e8b, host: "Apple M3 Max, macOS"}
---

# Arena reuse between helicity-summed read-outs

Several read-outs of one phase-space point run the same helicity-expanded
program on the same momenta: on a configuration-drawing integrand, `eval_amp2`
runs before `eval_m2`; after acceptance, event selection reads the JAMP and
helicity diagonals. `ScratchSpace` therefore remembers which fill its arenas
hold, and a read-out whose fill matches skips the forward pass and costs only
its read-out.[^run-fill-for] Per-diagram AMP2 and its weights are described in
[per-diagram AMP2](../amplitudes/per-diagram-amp2.md); the arenas themselves in
[the helicity program layout](../performance/evaluator-program-layout.md).

## The stamp

- `BoundAmplitude::fill_token` names the amplitude *and* the current contents of
  its constant pools. It is drawn from a process-wide monotone counter at
  construction, on `clone`, on `set_pools`, and on every `pools_mut` (which is
  how a per-event coupling move rewrites the pools). Tokens are never reused, so
  a stamp cannot outlive the state it was taken from, including across a drop
  and reallocation at the same address. `0` means "the arenas hold nothing a
  reader may trust".[^run-rs]
- Tokens are handed out in per-thread blocks of 2⁴⁰, so a parallel integrator
  drawing one token per point per thread does not contend on a shared cache
  line.
- `ScratchSpace` stores `fill_token` and the external momenta of its fill.
  `holds(token, momenta)` requires the same non-zero token and **bit-identical**
  momenta (`==` plus sign agreement, so `-0.0` ≠ `0.0` and `NaN` never matches).
  A hit therefore implies identical inputs and identical values: the reuse is
  exact, not approximate.
- `fill_for` is the only path that re-stamps. `fill_arenas` clears the stamp on
  entry, so any other writer of the arenas (the unexpanded single-helicity path,
  probes) leaves them unreusable.
- The workspace is per thread, so no thread can observe another's fill.

Every helicity-summed read-out goes through `fill_for`: `eval_m2`, `eval_jamp2`,
`eval_amp2`, `eval_hel_m2`, `eval_hel_jamps`, the per-combination JAMP dump and
the helicity-filter probe `mark_contributing_helicities`.

## What it bought, and what it costs

Order-preserving and bit-for-bit: 100 banked row files digest-identical before
and after, census unchanged. The reuse tests count forward passes (an
anti-vacuity fills counter in `cfg(test)`), so a test sees a pass *skipped*,
not merely an equal answer; one is a 4-thread cross-thread isolation test.
Measured on the M3 Max:[^n31-e3]

| effect | size |
|---|--:|
| event selection on `gu_to_epemu` / `gux_to_epemux` (4 passes → 1) | −37.5% / −36.2% event read-out |
| `pp_to_ll` configuration-draw cost | −83% (193.6 → 32.5 ns/point), −8.4% total |
| `pp_to_llj_dyn`, where the cache can never hit | +1.0–1.6%, consistently signed |

The `pp_to_llj_dyn` cost is the stamp check on a shape that never hits; it is
accepted, because removing it would trade away the exactness that makes the
reuse safe.

## Prefix sharing: measured NO-GO

The alternative was to compute `eval_amp2` as a prefix or by-product of
`eval_m2` when the two differ (a per-event strong coupling between them). Only
≤28% of the live-draw rows' program nodes are αs-invariant, and they are the
cheap ones; a stream partition would also perturb the op-blocked order, worth
−17.3% (see [execution order](../performance/execution-order.md)). On
strong-coupling drawing rows the clustered prescription exists precisely to
move the coupling between `AMP2` and `|M|²`, so those two evaluations share only
momenta; Drell–Yan is the only absorbable case.[^n31-e3]

[^n31-e3]: Note 31 §E3, measured results.
[^run-rs]: `ScratchSpace`, `BoundAmplitude::fill_token`, `next_fill_token`, `set_pools`/`pools_mut` in `run.rs`.
[^run-fill-for]: `BoundAmplitude::fill_for` and the `fill_token = 0` clear in `fill_arenas`.
