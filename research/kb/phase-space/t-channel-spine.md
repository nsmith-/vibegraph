---
type: Algorithm
title: The multi-rung t-channel spine
description: "Spacelike lines nest into an ordered rung chain of peripheral 2-body steps: the logarithmic t map and its Jacobian, beam anchoring, flat fallback, multi-body spines, and the ordering firing test."
status: draft
tags: [t-channel, spine, phase-space-map, firing-test, ladder]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n21-spine, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L168-L230", title: "Note 21, t-channel spine (single spacelike line)"}
  - {id: n21-closeout, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/21-resonance-sampling-and-events-plan.md#L231-L299", title: "Note 21, close-out: t-map firing tests"}
  - {id: n24-p0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L525-L618", title: "Note 24 P0, llj topology and the three-body spine"}
  - {id: n24-p2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L879-L1010", title: "Note 24 P2, per-energy channels, walk weight, channel coverage across groups"}
  - {id: n24-p2b, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/24-user-distribution-and-proton-events-plan.md#L1049-L1092", title: "Note 24 P2b, the floor applies to every spine"}
  - {id: n28-chain, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1376-L1538", title: "Note 28 §S2.1–S2.2, the rung chain and its types"}
  - {id: n28-ord, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1539-L1622", title: "Note 28 §S2.3, the ordering firing test and its negative controls"}
  - {id: n28-s4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e/research/notes/28-kt-spine-feature-sprint-plan.md#L1827-L1850", title: "Note 28 §S4, the spine in production"}
  - {id: dc-rs, resource: "vibegraph-lib/src/phasespace/diagram_channel.rs", title: "SpineRung, Spine, spine_chain, draw_t, t_measure, t_kinematics, peripheral_factor, sample_spine, spine_jacobian"}
  - {id: diagram-rs, resource: "vibegraph-lib/src/diagrams/diagram.rs#L130-L138", title: "Prop::is_spacelike"}
  - {id: dc-tests, resource: "vibegraph-lib/tests/diagram_channel.rs", title: "Chain, graph-cut, ordering and walk-vs-density tests"}
---

## Spacelike lines form a chain

A propagator is spacelike when exactly one beam flows through it
(`Prop::is_spacelike`, `diagrams/diagram.rs:136`)[^diagram-rs]. Each spacelike
line `k` cuts the externals in two; let `S_k` be the outgoing-leg slots on
beam 0's side, read by `spine_partition` from the nonzero pattern of the
stored momentum routing, complemented when the stored coefficients do not
touch beam 0 ([diagram channels](diagram-channels.md)).

For a tree diagram the sides are **totally ordered by strict inclusion**,
`S_1 ⊂ S_2 ⊂ … ⊂ S_r`, so sorting by size is sorting along the chain. Rung `i`
emits the blob `B_i = S_i \ S_{i−1}` (`S_0 = ∅`), the recoil is
`full \ S_r`, and the running transfer is

```text
q_i = p_a − Σ_{slot ∈ S_i} p_slot,    t_i = q_i²
```

That nesting is a claim, not a definition: two equal or incomparable sides
would mean the lines are not a path. `spine_chain` returns `None` then and the
diagram falls back to the all-timelike tree rather than picking an order
arbitrarily[^dc-rs]. `spacelike_lines_of_a_diagram_nest_into_an_ordered_rung_chain`
found zero violations over 89 diagrams of four processes (spacelike-line
counts `{0: 17, 1: 33, 2: 22, 3: 17}`) and refuses to pass unless a three-rung
ladder is present; an uncommitted sweep of all 3 024 diagrams of
`p p > e+ e- j j QCD=0` also found none.
`the_rung_chain_agrees_with_an_independent_graph_cut` re-derives each `S_k` by
removing the propagator and taking connected components, so a FeynGraph
routing change trips one derivation or the other[^n28-chain].

On `u d > e+ e- u d QCD=0` with slots `[e+, e-, u, d]`, a three-rung chain is
blobs `[[2], [0], [1]]`, recoil `[3]`: the multiperipheral topology where `e+`
and `e-` leave the chain at different vertices across a spacelike lepton line,
which a single-rung spine cannot express.

The types (`phasespace/diagram_channel.rs`):

```rust
struct SpineRung<F> { emitted: Node<F>, t_mass2: F, t_max_cap: Option<F>, rest_floor: F }
struct Spine<F>     { rungs: Vec<SpineRung<F>>, recoil: Node<F> }   // ordered away from beam 0
```

Blobs and the recoil hang off the chain as ordinary timelike subtrees, drawn
by the same `sample_branch`/`branch_jacobian` machinery with their own BW or
log maps ([resonance maps](resonance-and-pole-maps.md)). `spine_poles()` lists
the rungs' `t_mass2` in chain order after any floor; `t_channels()` is an
unordered `props`-order list kept only as metadata.

## One rung

With `Q_0 = p_a + p_b` (invariant `ŝ`), rung `i` splits the system `Q_{i−1}`
(invariant `ŝ_{i−1}`) into the blob `B_i` (invariant `s_i`) and the remainder
`Q_i = q_i + p_b` (invariant `ŝ_i`). It is the peripheral 2-body step
`t_kinematics(ŝ_{i−1}, ma2 = t_{i−1}, mb2 = m_b², s1 = s_i, s2 = ŝ_i)` with
`t_0 = m_a²`; the single-rung spine is `r = 1`. For `i > 1` the incoming line
is spacelike (`ma2 < 0`), which the Källén-based kinematics handles, and the
rung is built in the CM of `Q_{i−1}` with `q_{i−1}` rotated onto `+z`
(`the_inter_rung_rotation_is_a_rotation`,
`a_spacelike_incoming_line_has_a_frame_of_its_own`)[^n28-chain].

**The `t` map.** The rung importance-samples the propagator `1/(t − m²)` with
density `∝ 1/(m² − t)`:[^n21-spine]

```text
t = m² − (m² − t_min)·exp(−x·N),   N = ln[(m² − t_min)/(m² − t_max)],   dt/dx = N·(m² − t)
```

Both endpoints are `≤ 0`. When the pole cannot shape the draw — a massless line
whose window reaches the collinear edge `t_max = m²`, or a degenerate
threshold window — `draw_t` falls back to flat in `t`, the analogue of the BW
map's zero-width fallback, and the rung reduces to an isotropic 2-body split.
The polar angle is fixed by `t`
(`t = m_a² + s₁ − 2E_aE₁ + 2k·p*·cos θ`), only `φ` is free, and the 2-body
LIPS reparametrised from `(cos θ, φ)` to `(t, φ)` gives the rung factor
`π·(dt/dx)/(4√s·k)` with `p*` cancelling — not the timelike
`R_2 = π|p*|/√s` (`peripheral_factor`). Firing tests:
`t_map_is_measure_preserving`, `t_map_zero_variance_on_propagator`,
`t_bounds_include_initial_state_mass` (a massive initial state pushes
`t_max < 0`), `t_channel_threshold_window_collapses`[^n21-closeout].

**Where the collinear edge is.** With a massless spectator beam and a massless
emitted blob, exactly

```text
t_max^(i) = t_{i−1} · ŝ_i / ŝ_{i−1}
```

(`a_spacelike_incoming_line_pushes_the_transfer_edge_off_the_pole`). Rung 1
(`t_0 = 0`) sits on the edge; interior rungs are pushed off it in proportion
to `t_{i−1}`, which is itself regulated rather than large, so they still need
the regulator; the last rung is back on the edge when the recoil is a single
massless leg. The regulator — bound `t_max ≤ −floor` and a pole floor, from
the cuts — is [the spacelike floor](spacelike-floor.md). Without a positive
floor no spine is built past two outgoing legs[^n24-p2b].

**Degrees of freedom.** With blobs of sizes `k_1 … k_r` and the remainders
`R_i = B_{i+1} ∪ … ∪ B_{r+1}`, a chain consumes `2r` (one `t_i`, one `φ_i` per
rung) `+ #{composite blobs}` (each blob's invariant) `+ #{|R_i| ≥ 2}` (each
remainder's invariant) `+ Σ(3k_i − 4)` over composite blobs. It always comes to
`3·n_out − 4`, which every derived chain is asserted to reach.

## Density, anchoring, and what no volume check can see

`Channel::density` recomputes each `t_i` from the final momenta as
`(beams[0] − Σ_{S_i} p)²`, with the beams rebuilt at the draw's energy from
the stored beam masses, so the density is well defined at foreign points and
on a hadronic run whose `ŝ` changes per event ([channel contract](channel-contract.md)).
`sample` accumulates its own path weight from the invariants it drew, and the
tests compare it with `1/density` at `WALK_DENSITY_TOL = 1e-7` (worst measured
7.1e-9 over every diagram channel, 1.2e-8 on a floored llj spine; an
unfloored spine reaches 4e4)[^n24-p2]. `sample` must not define its weight as
`1/density`: the reciprocity check would then be vacuous and blind to a
sampling/weighting mismatch.

**Anchor.** Rungs are ordered away from beam 0 along `+z`. Anchoring at beam 1
reads the same ladder from the other end; it is a different map (a different
blob becomes the recoil), not a relabelling. Pairing the emitted blob with the
wrong beam would read the crossed `u`-channel invariant
(`spine_transfer_pairs_emitted_with_beam0`, `spine_emitted_is_forward_biased`,
whose bias flips under a silent emitted/recoil swap)[^n21-closeout].

**The multiset `{t_i}` is anchor- and order-independent; the chain order is
not.** Every `t_i` is the square of a propagator momentum, the same read from
either side by momentum conservation. So `V_n`, σ and any histogram of the
*volume* in `t_i` are identical between a chain and its reverse. A wrong
ordering is not a wrong number; it is wrong importance sampling.

## The ordering firing test

Because of that, the ordering oracle is a **coverage test on a peaked
integrand**, not an agreement test on a volume
(`the_rung_ordering_test_fires_on_a_swapped_chain`, `tests/diagram_channel.rs`;
the methodology is [integrand and sampler oracles](../validation/integrand-and-sampler-oracles.md))[^n28-ord][^dc-tests].
On the reference process's asymmetric two-rung chain at fixed `√ŝ` with its
run card's cuts, the probe integrand is `Π_i 1/(m_i² − t_i)² · BW(s_pair)`
times the cut, with the `t_i` computed from the diagram's own `S_i` prefixes,
never asked of the channel:

- **per-rung coverage** — 400 000 raw draws binned by `ln|t_i|` in 12 bins
  over the fiducial window; every bin holds at least `T_ORD_MIN_BIN_SHARE =
  1.2%`. The derived order holds ≥ 1.80% in every bin of both rungs;
- **per-bin precision** — the integral restricted to each bin is estimated,
  and the whole holds its relative error under 10%;
- **seed stability** — five seeds agree within their claimed errors, the guard
  a scalar cannot be;
- **volume neutrality** — `V_n` against flat RAMBO
  (`ladder_chains_integrate_their_own_support_and_cover_the_fiducial_region`),
  recorded as ordering-blind so nobody reads it as confirmation.

The controls make it non-vacuous:

- **Swapped chain must fail.** The same channel with rungs reversed
  (`with_rung_order`, test-only) must fail, and only in rung 1's projection: a
  two-rung swap leaves `t_2 = (p_a − p_{B_1} − p_{B_2})²` unchanged and moves
  `t_1` to `(p_a − p_{B_2})²`, which is not a propagator of the diagram. The
  reversed order falls to 0.91% in rung 1 and stays at 2.62% in rung 2; both
  halves are asserted.
- **Anchor flip and distinguishability.**
  `a_swapped_chain_and_an_anchor_flip_are_different_maps` requires the
  reversed and beam-1-anchored densities to differ from the derived one by
  more than rounding on most points, at the floor a real run gives. If they
  coincided the ordering question would be moot and the test empty — which is
  what massless-propagator processes do at floor zero, where all four
  `g g > g g` channel maps collapse onto one.

What it provably cannot detect: a symmetric chain (equal poles and
interchangeable blobs make the reverse the same map — hence the reference
process's asymmetric blobs and mixed massless/`m_Z` poles); anything a positive
density cannot see (a phase, a sign, a colour index); an error common to both
orderings (a wrong per-rung Jacobian or anchor), which the walk-vs-density and
volume checks own; a wrong order whose coverage survives at a `√ŝ` where the
windows overlap heavily (hence a stated `√ŝ` and an asserted margin); and
whether the chain belongs to this diagram, which is the graph-cut test's job.

The reference row is `ud_to_epemud_qcd0` (`u d > e+ e- u d QCD=0` at fixed
scale; 35 diagrams split 12/14/9 over one, two and three spacelike lines), an
enforced σ gate at `rel_tol 0.01` (`validate_sigma.rs:438`). It was chosen over
`p p > e+ e- j j QCD=0`, whose 112 subprocesses and 1 608 pooled channels make
it ~60× costlier per point[^n28-s4]
([reference rows](../validation/reference-row-rationale.md)).

## Where spines appear in practice

On `p p > l+ l- j`, all four `u u~ > e+ e- g` diagrams are single-rung spines
(`emitted = {jet}` twice, `{ℓℓ}` twice) and two of four `g u > e+ e- u`
diagrams are (`{jet}`); the rest route the quark through the full `ŝ`. The
rung always separates the lepton pair, with its Z/γ* pole, from the
jet[^n24-p0]. The `g q` mirror peak at small `(p_b1 − p_jet)² = (p_b0 − p_ℓℓ)²`
is covered by the `q q̄` groups' `{ℓℓ}` spines in the pooled `g`, so no
mirrored channels are needed[^n24-p2].

`RungOrder::Derived` is the default; `Reversed` reads 1.01 ± 0.02 of its
evaluations on `u u~ > g g g` at twenty seeds ([map choices](map-choices.md)).
MadEvent's `tstrategy` ping-pong for ≥ 3-transfer ladders is not built:
[ping-pong for three-rung ladders](../backlog/performance/tchannel-pingpong-for-three-rung-ladders.md).
Sampler methodology: [seed sweeps and budget ladders](../validation/seed-sweeps-and-budget-ladders.md).

[^diagram-rs]: `vibegraph-lib/src/diagrams/diagram.rs`, `Prop::is_spacelike`.
[^dc-rs]: `vibegraph-lib/src/phasespace/diagram_channel.rs`, `Spine` doc and `spine_chain`.
[^n28-chain]: Note 28 §S2.1–S2.2.
[^n21-spine]: Note 21, the single-rung spine addendum.
[^n21-closeout]: Note 21 close-out, firing-test inventory.
[^n24-p2b]: Note 24 P2b, "correction worth carrying forward".
[^n24-p2]: Note 24 P2 and its design decisions.
[^n28-ord]: Note 28 §S2.3.
[^dc-tests]: `vibegraph-lib/tests/diagram_channel.rs`, the ordering test's doc and `T_ORD_*` constants.
[^n28-s4]: Note 28 §S4 and §6 D2; `validation/madgraph/scripts/ud_to_epemud_qcd0.mg5`.
[^n24-p0]: Note 24 P0, probe verdict points 1–3.
