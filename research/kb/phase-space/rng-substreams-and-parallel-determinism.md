---
type: Design Decision
title: Counter-based RNG substreams and thread-count-independent integration
description: "ChaCha8 addressed by (stream, position) replaces RANMAR; each chunk seeks its substream by global point index and reduces in point order, so -j changes wall time, never a bit of the artifact."
status: draft
tags: [rng, determinism, parallelism, vegas, reproducibility]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n18-rng, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L133-L162", title: "Note 18 §1.4, RNG: splittable and modern, not RANMAR"}
  - {id: n18-dec, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/18-hadronic-xsec-design.md#L479-L911", title: "Note 18 §5, decision records H3 (SubStream, bits→uniform) and H5 (parallel VEGAS)"}
  - {id: n31-i3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/31-perf-sprint-3-plan.md#L152-L242", title: "Note 31 I3, parallel integrate: Sync hadronic integrand and -j"}
  - {id: n31-jcol, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/31-perf-sprint-3-plan.md#L1195-L1241", title: "Note 31 §6.7, the -j column at the CLI"}
  - {id: n32-s3, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/32-perf-addendum-plan.md#L478-L597", title: "Note 32 §5.1, survey_variance parallelised (S3)"}
  - {id: rng-rs, resource: "vibegraph-lib/src/phasespace/rng.rs", title: "SubStream, u64_to_uniform, SCALE_DRAW_STREAM_BASE"}
  - {id: vegas-rs, resource: "vibegraph-lib/src/vegas.rs#L474-L560", title: "adapt_parallel, adapt_parallel_seeded, adapt_blocks_iteration"}
  - {id: parallel-rs, resource: "vibegraph-cli/src/parallel.rs", title: "The -j/--parallel flag"}
  - {id: proton-survey, resource: "vibegraph-lib/src/proton.rs#L2870-L2960", title: "ProtonIntegrand::survey_variance, chunked parallel survey"}
  - {id: lanes-rs, resource: "vibegraph-lib/src/helas/eval/lanes.rs", title: "Lane-uniformity contract"}
measured:
  - {commit: b612253, landed_in: 17fd612, command: "vibegraph integrate on partonic, dy13_default and pp_to_llj cards at -j 1/4/8/16, artifact md5 compared, 1× and 4× budget"}
  - {commit: 1c15cf4, landed_in: 5ff1de5, command: "survey_variance chunked; artifact digest asserted at -j {1, 4, 16}"}
---

## Decision

Every random draw in integration and event generation comes from
`rand_chacha::ChaCha8Rng`, addressed as a `SubStream { seed, stream,
position }` (`phasespace/rng.rs`)[^rng-rs]. RANMAR, the CERNLIB generator
MadEvent uses, was considered and rejected: its only unique asset is
cross-toolchain seed compatibility, which nothing here needs, because the
RAMBO oracle replays *uniforms*, not generator streams
([RAMBO](rambo.md))[^n18-rng]. `rand_pcg`'s `Pcg64Mcg` is the fallback if the
generator ever shows in a profile; it is faster but its stream selection is
weaker, and the matrix element dominates phase-space cost by orders of
magnitude.

ChaCha8 is counter-based, so substreams are structurally independent rather
than independent by hash mixing: 2⁶⁴ streams per seed (`set_stream`) and a
settable position in each (`set_word_pos`). A 64-bit draw consumes two 32-bit
words, so `SubStream::new(seed, stream, position)` sets the word position to
`2 · position`. Re-creating a substream with the same triple replays the same
draws.

**Bits to uniform.** `u64_to_uniform` takes the top 53 bits,
`(bits >> 11) as f64 / 2^53`, then casts to the scalar field `F`. The result
is in `[0, 1)` and never reaches 1. Because the rule is a function of the
integer bits alone, a lane-batched `f64` pack fed the same draw reproduces
scalar `f64` bit for bit, and an `f32` rule would be a one-line variant.
Conversion goldens (0 → 0, `u64::MAX` → (2⁵³−1)/2⁵³, low 11 bits discarded)
and an end-to-end seeded draw golden pin it[^n18-dec].

## Stream families

Each consumer opens its own family, so adding a draw never shifts another
sequence by a bit:

| constant | where | what draws on it |
|---|---|---|
| `MULTICHANNEL_ADAPT_STREAM = 0xA1FA_5EED` | `hadronic.rs:92` | fixed-beam α survey |
| `ADAPT_STREAM = 0xA1FA_9110` (+ iteration) | `proton.rs:1107` | hadronic α survey |
| `CHANNEL_STREAM_BASE = 0xC7A0_0000` (+ channel) | `hadronic.rs:98` | per-channel VEGAS integration |
| `SCALE_DRAW_STREAM_BASE = 0x5CA1_0000` (+ offset) | `rng.rs:38` | per-point scale-configuration draw |
| `SCAN_STREAM_BASE = 0x0057_4D41` | `unweight.rs:56` | per-channel `w_max` scans |

The scale draw sits on its own family because its coordinates are appended to
a point *after* the phase-space map has taken its own; with a separate stream
the channel grids, the channel selection and the acceptance test draw exactly
what they drew without it[^rng-rs].

## The determinism contract

The thread count (`-j`) is a scheduling knob: `-j 1` and `-j 16` write
byte-identical artifacts[^parallel-rs]. Two properties make that hold rather
than usually hold:[^vegas-rs]

1. **Addressing by global point index.** One point consumes exactly `ndim`
   draws, so the point at global index `p` (counting across iterations) starts
   at draw `p · ndim`. A chunk seeks straight to its first point instead of
   inheriting a predecessor's generator state, so chunks can run in any order.
2. **Reduction in global point order.** Chunks return per-point values and bin
   indices; sums and the refinement histogram are formed in point order on one
   thread. Floating-point addition is not associative: a per-chunk partial sum
   would give a different, equally valid grid at the next refinement and from
   there a different point sequence.

`VegasGrid::adapt_parallel_seeded` holds both and is bit-for-bit the
sequential `adapt` driven by `SubStream::new(seed, stream, 0)`;
`adapt_blocks_iteration` holds both per block while scheduling every
channel's chunks in one rayon region. The older `adapt_parallel` /
`sample_frozen_parallel` key substreams by `(iteration, chunk)` at position 0
and reduce per chunk: thread-count invariant, but not equal to `adapt`, and
unused in production[^n31-i3].

The hadronic α survey (`ProtonIntegrand::survey_variance`) takes a weaker
form of the contract. Both its substreams are addressed by the point's index
within the survey, but it sums per chunk of `SURVEY_CHUNK = 128` points and
reduces the partials in chunk order, because carrying every point's whole
`n_channels` density row to one sequential reduction would cost hundreds of
megabytes on a several-hundred-channel process. So the chunk size is part of
the answer and the thread count is not[^proton-survey]. The fixed-beam survey
(`MultiChannel::adapt_alphas`) runs serially.

Making the integrand shareable was the other half: per-thread scratch through
`ThreadLocal` in place of `RefCell`, a `SubprocessProto`/`BoundSubprocess`
split, a per-thread subsystem memo inside `MultiChannel`, and `Send + Sync`
on `Real`, `Channel` and `ScaledChannel`. The per-point scale draw is a pure
function of the point index, never of the thread[^n31-i3].

## CLI behaviour

`-j/--parallel N` on `integrate` and `generate` sizes rayon's global pool once,
at the top of the command, because the pool is immutable after first use. A
count the platform cannot honour is refused, not silently replaced, since a run
asked for a thread count is usually being timed. `generate`'s accept/reject
replay is serial, so there the flag only sizes the pool. Diagram enumeration
runs on one thread unless `--parallel-diagrams` is given; a 2→3 process
enumerates slower on sixteen threads than on one[^parallel-rs].

## Evidence and its limits

- Artifact digests identical across `-j 1/4/8/16`, and against the serial
  binary before the change, on a partonic card, `dy13_default` and
  `pp_to_llj`, repeated at 4× budget; the full validation layer was unchanged
  to the census character[^n31-i3]. At the CLI, four rounds × two thread
  counts gave one digest per card[^n31-jcol].
- Code pins: `test_adapt_parallel_thread_count_invariant` and
  `test_adapt_parallel_seeded_is_the_sequential_adapt` (`vegas.rs`),
  `adapt_grids_reproduces_a_sequential_integration` (`hadronic.rs:4004`) and
  `the_parallel_integration_reproduces_a_sequential_one` (`proton.rs:4660`).
- One check written for this contract was vacuous and caught by its own
  negative control: a fixed-beam trailing uniform turned out inert (40 of 40
  probes unmoved), and was replaced by a live-draw reference on the proton
  path.
- The contract is about thread count, not about code changes. Re-associating
  a sum is a one-time byte change; parallelising the survey was one, recorded
  in advance and asserted at `-j {1, 4, 16}` afterwards[^n32-s3].

Because of it, a validation row run single-threaded measures the same numbers
the parallel CLI produces. The validation layer never forced
`--test-threads=1`; thread-count independence is what makes its numbers the
CLI's.

What it does not cover: lane-batched evaluation has its own uniformity
contract (every data-dependent branch on `F` must be lane-uniform), stated in
the `helas/eval/lanes.rs` module doc[^lanes-rs] and in
[SIMD lane evaluation](../performance/simd-lane-evaluation.md). Wall-time
scaling and the serial floor are in
[integrate thread scaling](../performance/integrate-thread-scaling.md);
the statistical side of reproducibility (a pinned seed is not evidence) is in
[seed sweeps and budget ladders](../validation/seed-sweeps-and-budget-ladders.md)
and [integrand and sampler oracles](../validation/integrand-and-sampler-oracles.md).
See also [VEGAS integrator](vegas-integrator.md).

[^rng-rs]: `vibegraph-lib/src/phasespace/rng.rs`, module and item docs.
[^n18-rng]: Note 18 §1.4.
[^n18-dec]: Note 18 §5, H3 records.
[^vegas-rs]: `vibegraph-lib/src/vegas.rs`, `adapt_parallel_seeded` and `adapt_blocks_iteration` docs.
[^n31-i3]: Note 31 I3, plan, brief corrections and measured result.
[^proton-survey]: `vibegraph-lib/src/proton.rs`, `survey_variance` doc.
[^parallel-rs]: `vibegraph-cli/src/parallel.rs`, module doc.
[^n31-jcol]: Note 31 §6.7.
[^n32-s3]: Note 32 §5.1, the survey parallelisation.
[^lanes-rs]: `vibegraph-lib/src/helas/eval/lanes.rs`, "Lane-uniformity contract".
