---
type: Design Decision
title: Propagators stay separate from vertices
description: "The amplitude-closing vertex never propagates, so unfused vertices are needed anyway; separate propagators give N+M routines instead of ALOHA's N×M, at a typed seam."
status: draft
tags: [propagators, evaluator, aloha, intertwiners]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n13-prop, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/13-typed-repr-conventions-design.md#L223-L243", title: "Note 13 §4 (propagator stays separate from vertex)"}
  - {id: code-op, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/op.rs", title: "Op::Propagate"}
  - {id: code-kernel, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/kernel.rs", title: "propagate_core and the per-type propagator kernels"}
  - {id: code-slot, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/waveform_slot.rs", title: "WaveformSlot (every vector slot contravariant)"}
---

# Propagators stay separate from vertices

HELAS and ALOHA fuse the propagator into the off-shell current: ALOHA's
`FFV2_3` (and HELAS's `jioxxx`) bake `1/(q² − m² + imΓ)` into the routine that
builds leg 3's current. Vibegraph does not. A vertex kernel produces the bare
off-shell current; a separate `Propagate` op applies the propagator to
it[^n13-prop].

## Why

The deciding fact is an asymmetry, not combinatorics:

> The amplitude-closing vertex does not propagate its output. It contracts
> onto an external wavefunction, so the unfused vertex is needed whatever
> else is done.

Since the unfused form must exist anyway, fusing would mean carrying both a
fused (internal) and an unfused (closing) version of every vertex: ALOHA's
N × M routine family (one routine per structure × output leg × propagator
form). Keeping them apart gives N vertex kernels plus M propagator kernels.

It also keeps the peephole layer simple: a fused vertex kernel
([intertwiner-basis-and-peephole](../amplitudes/intertwiner-basis-and-peephole.md))
never has to know which propagator follows it.

## The seam is typed

Separation is clean only if the boundary between vertex output and propagator
numerator is typed. The propagator numerator raises and lowers indices
(fermion `q̸ + m`; massive vector `−g^{μν} + q^μq^ν/m²`), and in an untyped
evaluator that seam is where variance bugs live[^n13-prop].

As built, the seam is typed by the register, not by a node tag:

- Every vector slot is the physical contravariant current `ε^μ`: external
  legs, momenta, vertex producers (`GammaVout`, `MetricVout`) and propagated
  currents alike (`WaveformSlot::Vector`). There is no covariant vector
  slot[^code-slot].
- Fermion currents carry their adjoint side on the slot
  (`FermionIn` = ket, `FermionOut` = bra).
- `Op::Propagate` has children `[current, Mass, Width]` and
  `propagate_core` dispatches on the slot variant: Dirac on a ket or a bra,
  vector, scalar. A `Multivector` (the Clifford element of a cyclic
  four-fermion contact) never leaves its vertex, and propagating one
  panics[^code-op][^code-kernel].
- In the compiled helicity program the four cases are separate instructions
  (`PropagateFin`, `PropagateFout`, `PropagateVector`, `PropagateScalar` in
  `helas/eval/layout.rs`).

The current already carries its routed momentum, as HELAS's off-shell routines
output it (`fvixxx` q = fi − vc, `fvoxxx` q = fo + vc, `jioxxx` jmom = fo − fi),
so the propagator reads `q` from its input and needs no topology
information.

## Consequences

- The vertex i lives in the UFO coupling, and the propagator carries its own
  `−i`. ALOHA's routines fold these differently, so a vibegraph vertex current
  and the matching ALOHA routine differ by a fixed phase. An example is the
  VVS current, which is `+i × VVS1P1N_1` (ALOHA's no-propagator variant). The
  ledger of these factors is
  [global-phase-i-counting](../amplitudes/global-phase-i-counting.md).
- The propagator forms themselves (the Weyl-basis Dirac numerator, unitary-gauge
  massive vector, Feynman-gauge massless vector, scalar, all with fixed-width
  denominators) are in
  [wavefunctions-and-propagators](../amplitudes/wavefunctions-and-propagators.md).

[^n13-prop]: Note 13 §4. Its caveat named the then-untyped `MetricVout` / `LowerVout` / `PropagateLowered` boundary; that boundary is now typed by the contravariant-only vector slot, and `PropagateLowered` and `LowerVout` no longer exist.
[^code-op]: `Op::Propagate` in `vibegraph-lib/src/helas/eval/op.rs`. Its doc comment still describes a covariant `MetricVout` current being raised back; that describes no current code path.
[^code-kernel]: `propagate_core` in `vibegraph-lib/src/helas/eval/kernel.rs`.
[^code-slot]: `WaveformSlot::Vector` in `vibegraph-lib/src/helas/eval/waveform_slot.rs`.
