---
type: Design
title: "Lane-batched evaluation of eval_m2"
description: "eval_m2 over LaneField<N>: broadcast_lanes, the lane-uniformity branch contract, the bit-identity gate, and why lanes are built but not in production."
status: draft
tags: [performance, simd, lanes, evaluator, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n18-13, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L100-L132", title: "Note 18 §1.3, the Real seam and lane divergence"}
  - {id: n18-h4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L622-L748", title: "Note 18 §5 decision record H4 (bounds, divergence inventory, gate, AVX-512 kit)"}
  - {id: n18-out, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L935-L1039", title: "Note 18 Outcome"}
  - {id: lanes-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/lanes.rs#L1-L41", title: "helas/eval/lanes.rs module doc (lane-uniformity contract)"}
  - {id: run-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/helas/eval/run.rs#L835-L910", title: "broadcast_lanes, eval_m2_lanes, pack_lane_points, eval_m2_lanes_packed"}
---

# Lane-batched evaluation of `eval_m2`

The evaluator and the representation layer are generic over the scalar field `F: Real`.
Choosing `F = LaneField<N>`, an `N`-wide packed `f64` vector, runs one `eval_m2` pass over
`N` phase-space points at once with no change to the evaluator: each floating-point
operation executes the scalar operation independently per lane, so each extracted lane is
the scalar result at that point, bit for bit. The field itself is
[`LaneField<N>`](lane-field-over-wide.md); its measured payoff per host is
[lane throughput](lane-throughput.md).

## The API

All in `vibegraph-lib/src/helas/eval/run.rs`:[^run-rs]

- `BoundAmplitude<f64>::broadcast_lanes::<N>()` rebinds an already-bound scalar amplitude
  onto lanes by splatting each resolved constant (`consts_c`, `consts_f`, the colour
  factors) across all lanes. The lane amplitude borrows the same compiled evaluator, so a
  caller builds it once, pairs it with its own `scratch_space()`, and the hot loop stays
  allocation-free.
- `eval_m2_lanes(&amp, &[&[LorentzVector<f64>]; N], &mut scratch) -> [f64; N]` is the
  colour- and helicity-summed `|M|²` for `N` points.
- It is `pack_lane_points` (the AoS→SoA transpose, one `Vec` per call) followed by
  `eval_m2_lanes_packed`. The split lets a caller that already holds lane-packed
  kinematics, or one measuring the transpose, hoist it out of the loop.

**Why splatting rather than binding per lane.** The lane field has no `FromPrimitive`, so
card resolution cannot run at `F = LaneField`; only `bind` keeps that bound, and the lane
amplitude starts from a resolved scalar one. The same reasoning relaxed `Real`'s zero and
one to the method forms `Zero`/`One` (inherited through `Float`) instead of
`ConstZero`/`ConstOne`: a runtime-built SIMD value has no `const` zero, the const traits were
redundant with `Float`'s `Num` supertrait, and for `f64` the method forms are bit-identical
to the constants.[^n18-h4]

**One αs per batch.** The constant pools are rescaled once per call, so every lane in a
batch shares one `αs`. A dynamic-scale integrator cannot batch its points until the
rescaling is applied per lane ([backlog](../backlog/performance/lane-eval-shares-one-alpha-s.md)).

## The lane-uniformity contract

The one way lane identity breaks is a *data-dependent branch on `F`*. Comparisons on a lane
pack reduce to a single `bool` (lexicographic `<`/`>`, all-lanes `==`), so an
`if predicate(F)` takes one branch for the whole pack. If two lanes want different branches,
one of them gets the wrong formula, silently.[^n18-13]

Every such branch on the `eval_m2` hot path is in the external-wavefunction builders
(`vxxxxx`, `weyl_ixxxxx`) and the vector propagator, and each is lane-uniform by
construction when all points in a batch share the process and partonic-CM kinematics
(beams exactly along ±z, external masses broadcast card constants):

- **mass forks** (`vmass == 0`, `mass != 0`): the mass is identical on every lane;
- **on-axis forks** (`pt == 0`, `pp3 > 0`, `px == 0 && py == 0 && pz < 0`): a leg is either a
  beam, on ±z for every lane, or a produced leg, off-axis for every lane except a
  measure-zero exactly collinear point;
- **at-rest forks** (`pp == 0`): only an exactly-at-rest produced particle, which phase-space
  sampling does not produce; threshold-adjacent points take the moving branch uniformly.

The enumerated inventory lives in the `helas::eval::lanes` module doc; that doc, not this
page, is the authority, and it must change with the code.[^lanes-rs] No branch needed a
branchless rewrite. `min`/`max`/`abs`/`signum`/`sqrt` are per lane and are not branches.
Callers must batch kinematically homogeneous points, which the partonic-CM integrand does
natively: the pruned evaluator already assumes ±z beams.

## The gate

- `eval_m2_lanes_match_scalar`: every extracted lane `assert_eq!` on `to_bits` against
  scalar `eval_m2`, at N = 2, over every MG-validated process, on three homogeneous regimes
  (partonic-CM z-beams, generic off-axis momenta, threshold-adjacent z-beams). A mixed-branch
  batch would break it, which is what it pins.
- `lanes4_lanes8_match_scalar` covers N = 4 and 8 on one small process.
- Bit identity holds on **every** target, with or without hardware FMA, because both paths
  route multiply-adds through `Real::mul_add_fast` and so round alike
  ([`mul_add_fast`](mul-add-fast.md)). It is exact only between builds that agree on
  `HARDWARE_FMA`; CI runs it under `x86-64-v3`.

## Not in production

Nothing outside tests, `benches/eval_strategies.rs` and `examples/eval_loop.rs` calls the
lane path; the integrators and the release assets evaluate scalar. The blockers are
consumers, not the evaluator: a batched integrator needs the per-point chain generic in `F`
and per-lane `αs`, and adoption needs a `lanes4` σ/event gate against scalar
([backlog](../backlog/performance/lane-batching-not-in-production.md)). The natural width is
host-dependent ([lane throughput](lane-throughput.md)): 4 on an `x86-64-v3` asset, 8 on Zen 4
and, after constant collection, on Emerald Rapids.

The order the program runs in is chosen once per evaluator and shared by every `F`, and its
fallback threshold (`SCHEDULE_BYTE_LIMIT`, `vibegraph-lib/src/helas/eval/layout.rs`) is
evaluated at `f64` bytes, so it is blind to the N× larger lane working set
([backlog](../backlog/performance/schedule-fallback-lane-blind.md)).

## The earlier negative verdict, and why it does not hold

The first lane field was `NumericArray<f64, N>`, chosen on the premise that LLVM would
auto-vectorise its elementwise operations. On the M3 Max (NEON, 2×f64) every width was
1.4–2.7× *slower* than scalar, and wider N only reduced the penalty, the signature of
overhead-bound code. The conclusion drawn then, that lane-batching this interpreter is not a
win and that a gain would need AVX-512 and a non-interpreter kernel, was wrong about the
cause: the field's arithmetic was not inlining (out-of-line calls into `generic-array` and
`num_complex`), on every host. With `LaneField` over `wide`, lanes beat scalar at every width
on x86 (median 0.57× / 0.32× / 0.25× per event at N = 2 / 4 / 8, Emerald Rapids). The
interpreter itself was not the obstacle.[^n18-out]

The `f32`-lanes idea was deferred on that same premise and has not been revisited.

## Re-measuring on a new host

The procedure (confirm the vector units, keep `RUSTFLAGS` identical for scalar and lanes,
run the lane gate first, then the bench and `scripts/dump_lane_asm.sh`, reading
`arith_calls` and the inlining verdict before any packed-op count) is in
[benchmarking the evaluator](microbenchmark-protocol.md).

[^run-rs]: `vibegraph-lib/src/helas/eval/run.rs`, lane entry points and their doc comments.
[^n18-h4]: Note 18 §5, H4 bounds decision.
[^n18-13]: Note 18 §1.3; contract text now in `lanes.rs`.
[^lanes-rs]: `vibegraph-lib/src/helas/eval/lanes.rs` module doc; note 18 H4 holds the original 10-row inventory.
[^n18-out]: Note 18 H4 and Outcome (NEON numbers); the reversal is measured in the x86 study's AVX-512 section.
