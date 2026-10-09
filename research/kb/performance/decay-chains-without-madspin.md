---
type: Design Decision
title: Decay chains are sampled in-process, with no MadSpin step
description: "The decay-chain cost ladder: |M|² cost stays near its core's while unweighting efficiency falls 2.6–16× from flat decay angles; correctness and convergence need no decay-after-generation step."
status: draft
tags: [performance, decay-chains, unweighting, madspin, phase-space]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n38-d3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/38-process-grammar-sprint-plan.md#L689-L801", title: "Note 38 §D3 (decay-chain phase space, σ and the sampler ladder)"}
  - {id: ladder-test, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/vibegraph-lib/tests/decay_chain_ladder.rs#L1-L60", title: "vibegraph-lib/tests/decay_chain_ladder.rs"}
measured:
  - {host: "4 shared cores (host not recorded)", command: "cargo test -p vibegraph-lib --profile release-debug --features extended-validation --test decay_chain_ladder -- --ignored --nocapture"}
---

# Decay chains are sampled in-process, with no MadSpin step

**Decision.** A decay-chain card (`p p > t t~, t > b e+ ve, …`) is integrated and
unweighted with the full decayed |M|² inside the ordinary multichannel sampler.
No MadSpin-style decay-after-generation step exists, and none is needed for
correctness or convergence. How the chain's diagrams are enumerated is in
[decay-chain enumeration](../process/decay-chains.md); how each forced resonance
is drawn inside its Breit–Wigner window is in
[resonance and pole maps](../phase-space/resonance-and-pole-maps.md).

The hypothesis it rests on: each forced resonance is one Breit–Wigner-mapped
invariant, the decay subtrees are shared by every channel, and what remains are
smooth decay angles.

## The ladder

`tests/decay_chain_ladder.rs` (ignored; a measurement, not a gate) integrates
each rung to a fixed relative accuracy, scans its channel maxima on the trained
grids and draws unweighted events by the same accept/reject pass `generate`
runs. Each decayed rung sits beside its undecayed core. One seed, integration to
5e-3, scan at the integration share, 5 000 unweighted events, `MaxRule` default,
4 shared cores; "pt" is one integrand point (cut rejections included), "ME" the
summed |M|² alone:[^n38-d3][^ladder-test]

| rung | legs | channels | pt (µs) | ME (µs) | integration | ε_unw | ms/event |
|---|--:|--:|--:|--:|---|--:|--:|
| `e+ e- > z z` | 2 | 2 | 4.7 | 5.4 | 0.72M pts, 2.2 s | 0.511 | 0.013 |
| `…, z > e+ e-, z > mu+ mu-` | 4 | 2 | 3.7 | 4.0 | 0.72M, 2.2 s | 0.194 | 0.018 |
| `p p > t t~` | 2 | 4 | 9.1 | 11.4 | 0.72M, 5.1 s | 0.355 | 0.029 |
| `…, t > b e+ ve, t~ > b~ mu- vm~` | 6 | 4 | 13.2 | 8.8 | 0.72M, 7.2 s | 0.063 | 0.234 |
| `…, (t > w+ b, w+ > l+ vl), t~ > b~ j j` | 6 | 4 | 15.3 | 10.6 | 0.72M, 7.1 s | 0.065 | 0.235 |
| `p p > t t~ h h` | 4 | 46 | 67.8 | 58.8 | 0.72M, 35 s | 0.036 | 1.9 |
| `…, (t > w+ b, w+ > j j), (t~ > w- b~, w- > l- vl~)` | 8 | 46 | 15.7 | 70.2 | 8.3M, 467 s | 0.0022 | 40.5 |

## Reading

- **Leg count does not overwhelm the sampler.** Every rung converges, and the
  gated decay-chain σ rows agree with MadEvent.
- **The decayed |M|² costs 0.7–1.2× its core's.** The per-event cost grows
  *beyond* that because unweighting efficiency falls: 2.6× (`z z`), 5.7×
  (`t t~`) and 16× (`t t~ h h`), and the heaviest rung needed 11× its core's
  integration points.
- **The cause is the decay angles.** The forced invariants are Breit–Wigner
  mapped exactly; the channels leave the decay angles flat. Their V−A and
  spin-correlation shapes have max/mean ratios of a few per decay (`Z → ℓℓ` 1.5,
  so 2.25 for two, against the measured 2.6), which a factorised VEGAS grid only
  partly learns.

## What would buy the efficiency back

At eight legs a MadSpin-style step would buy roughly an order of magnitude per
event (core 1.9 ms, plus decay unweighting at the full |M|² price). Shaped
decay-angle maps in the channels are the in-sampler alternative. Both are a
performance item, not a correctness one, and neither reopens MadSpin now:
[decay-angles-drawn-flat-in-decay-chains](../backlog/performance/decay-angles-drawn-flat-in-decay-chains.md).[^n38-d3]

## Caveats

- One seed per rung; the integration and event counts are fixed, so ms/event is
  a budget-specific figure, read for its ratios against the core.
- The host is recorded only as "4 shared cores".

## Reproduce

```
cargo test -p vibegraph-lib --profile release-debug --features extended-validation \
    --test decay_chain_ladder -- --ignored --nocapture
```

`LADDER_ROWS=zz,zz_emu` runs a subset; `LADDER_EVENTS` sets the event
count.[^ladder-test]

[^n38-d3]: Note 38 §D3, the sampler ladder and its conclusion.
[^ladder-test]: Module documentation and constants of `vibegraph-lib/tests/decay_chain_ladder.rs`.
