---
type: Algorithm
title: Cut-implied timelike floors, xqcut and the τ bound
description: "Cuts::timelike_floor's provable bounds on subsystem invariants, how they enter the maps, the bias oracle, why xqcut floors are already in the maps, and why a tighter τ floor was dropped."
status: draft
tags: [phase-space, cuts, floors, xqcut, mlm]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n34-tf, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L56-L136", title: "Note 34 §1.2 (timelike-floor)"}
  - {id: n34-s5, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L326-L337", title: "Note 34 deferred S5 (the map's lower edge on the cut edge)"}
  - {id: n41-14, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L146-L190", title: "Note 41 §1.4 (MadGraph cuts and setup under xqcut)"}
  - {id: n41-m1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L531-L757", title: "Note 41 M1 (the τ-minimum audit)"}
  - {id: n41-m6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L2702-L2915", title: "Note 41 M6 (xqcut floors already in the maps; τ floor dropped)"}
  - {id: mg-setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L156-L189", title: "MadGraph setcuts.f, the xqcut rewrite of ptj/mmjj"}
  - {id: mg-myamp, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/myamp.f#L337-L551", title: "MadGraph myamp.f set_peaks (xe, xm, the τ minimum)"}
measured:
  - {landed_in: 7664ff9, command: "two-arm five-seed ladders on pp_to_llj and pp_to_llj_dyn"}
  - {commit: 069a951, host: "4-core container shared with another session", command: "τ floor probe on pp_to_llj_fixed, pp_to_llj_mlm and pp_to_ll_0j2j_mlm"}
---

# Cut-implied timelike floors, xqcut and the τ bound

## What a floor is

`Cuts::timelike_floor(slots)` (`vibegraph-lib/src/cuts.rs:720`) returns a
**provable** lower bound on the invariant mass² of the final-state subsystem
`slots` (bit `k` = the `k`-th final-state leg in `Cuts::compile`'s order),
holding at every configuration `Cuts::pass` accepts; zero when the cuts imply
none. A channel that draws that subsystem's invariant from the floor upward
loses only configurations the cuts reject, so the estimator stays unbiased,
**but only while the bound is provable**. An over-tight bound cuts into the
accepted region and biases σ silently. The derivation lives in the function's
doc comment:

- **Monotonicity.** Every bound is read off a sub-multiset `T ⊆ S`, and
  `m²(S) ≥ m²(T)` because a sum of on-shell final-state momenta is
  future-pointing. A bound on a pair inside `S` bounds `S`.
- **Pair-mass windows** (`mmll` and the like): the lower edge of a *normal*
  window is the bound. An inverted window (`m2_max < m2_min`) is a veto band
  and implies nothing; that case is pinned.
- **`pT` and `ΔR` thresholds:**
  `(p_i + p_j)² ≥ 2 p_Ti p_Tj (cosh Δy − cos Δφ)`, minimised over the accepted
  separations `Δy² + Δφ² ≥ R²` at `Δy = 0, Δφ = R`, giving
  `2 p_Ti^min p_Tj^min (1 − cos R)` for `R ≤ π` (and
  `cosh√(R² − π²) + 1` beyond). `R` is taken a hair below nominal
  (`DELTA_PHI_CLAMP_SLACK`) because `DELTA_PHI` clamps its cosine.
- **`mmnl`** bounds its own member set.

The spacelike counterpart, a *regulator* scale rather than a kinematic bound,
is [phase-space/spacelike-floor](spacelike-floor.md). The ŝ floor of a hadronic
run (`Cuts::shat_min`) is [phase-space/hadronic-tau-y-sampling](hadronic-tau-y-sampling.md).

## How the maps use it

`DiagramChannel::with_timelike_floors` installs the floor as the lower end of
every drawn timelike invariant through one shared `draw_lo`
(`max(μ², min(floor, hi))`), used identically by the draw and the density, so
reciprocity is structural ([phase-space/channel-contract](channel-contract.md)).
The production site is `MapChoices::channel` (`phasespace/maps.rs:156`), the
one place a channel is built for integration or replay, and the floors are part
of `map_key`/`map_identity`.

A floor moves where the map puts its density; it does not narrow the support.
A configuration below a floor keeps a positive density. That is consistent:
the bound that makes the floor admissible says every such configuration fails
the cuts and so carries integrand zero. Its main use is a zero-width pole, whose
logarithmic map ([phase-space/resonance-and-pole-maps](resonance-and-pole-maps.md))
otherwise starts at a fixed absolute floor and spends a tenth of its draws
reaching an edge no accepted point comes near, mixing zero-weight and
largest-weight draws in one grid cell.

A `2 → 2` final state draws no timelike invariant, so the floors are inert
there (pinned by test; `a_two_body_final_state_is_bit_identical_under_any_timelike_floor`).
Only `2 → 3` and up move.

### The bias oracle

`no_accepted_configuration_sits_below_a_subsystem_floor` (`cuts.rs:2102`) draws
40k flat-RAMBO configurations and checks every subsystem mask of every accepted
one against its floor. It is the test any future floor must pass, because the
failure it catches (an over-tight bound) is invisible in a σ that happens to
land inside its tolerance. On the `mmll = 50` card the floor is 2500 GeV² and is
attained within a factor 1.0002, so the bound is tight, not merely valid.[^n34-tf]

### What the floors bought, and what they did not

On `pp_to_llj_dyn`, the `m_ll ∈ [40, 70)` bin's variance/σ fell from 16.86 to
4.36, the top-0.1% share of the second moment from 80–92% to 30–51%, trained
χ²/dof from 4.02 to 1.80, and acceptance rose from 22.5% to 33.8%; `pp_to_llj`'s
error² fell 50%. Two-arm five-seed ladders show no bias (eight rungs within
0.8 sd, scatter shrinking with budget).

One residual got worse: with a small floor the map's lower edge lands on the
cut edge, and the leftover `ΔR`/`pT` boundary concentrates there
(`pp_to_llj` `m_ll ∈ [0, 5)` variance/σ 24.5 → 55.7). Fixing it needs bounds in
the other cut coordinates or a softened lower edge, a map-shape change
([backlog](../backlog/performance/llj-map-lower-edge-on-cut-edge.md)); it
carries about 9% of the row's variance.[^n34-s5]

## xqcut: MadEvent's floors are already in the maps

Under `xqcut > 0` MadEvent rewrites the jet cuts before integrating
(`setcuts.f:156-189`):[^mg-setcuts] with `auto_ptj_mjj` (default true),
`ptj ≥ 0` and `ktscheme = 1`, **`ptj = xqcut`** (else `ptj > xqcut` becomes 0);
likewise `mmjj = xqcut`; `drjj = drjl = 0`. Legs with `do_cuts = .false.`
(decay products under `cut_decays = F`, masses above 20 GeV, neutrinos) are
exempt.[^n41-14] vibegraph ports the rewrite (`runcard::matching`); the semantics of the
card fields are [run-card/matching-parameters](../run-card/matching-parameters.md).

MadEvent's `setxqcuts` (`setcuts.f:892`) and `set_peaks` (`myamp.f:337-551`)
then derive phase-space hints:[^mg-myamp] a jet leg's energy floor
`xe = max(…, √(xqcut² − m²))`, an s-channel jet-pair mass floor `xm = xqcut`,
and a τ minimum `(Σ xe)²/s`. `xm` only presets a grid (`setgrid` keeps 10% of
the bins below it); the τ minimum is the one hard limit. After the rewrite the
compiled cuts carry both floors, so the maps already start there:[^n41-m6]

| MadEvent hint | vibegraph, from the rewritten cuts |
|---|---|
| jet energy floor | `Cuts::energy_floor` = `xqcut` on a massless jet |
| jet-pair s-channel minimum | `timelike_floor` = `xqcut²` (from `mmjj`) |
| — | `spacelike_floor` = `xqcut²` (from `ptj`) |

`cuts::madevents_xqcut_floors_reach_the_maps_through_the_rewritten_cuts` pins
all three. The kT veto itself cannot become a floor: it is decided per flavour
group in that group's clustering while every point sums every group; no point
loses all its groups to it (0 of 1.77M one- and two-jet draws); and
`d_ij = m² p_T,min/p_T,max ≥ xqcut²` only when `m ≥ xqcut`, so no invariant
floor tighter than `mmjj` follows.

### The τ minimum is implied, or the card is refused

With the resolved `ptj = xqcut`, MadEvent's τ minimum is implied by the cuts: a
leg's energy is at least its `pT`, and any pair holding a cut jet already has at
least `xqcut` of energy. It cuts nothing;
`madevents_xqcut_tau_floor_is_implied_by_the_rewritten_cuts` (`cuts.rs:1701`)
pins that on 200k sampled points.[^n41-m1] With a resolved `ptj < xqcut`
(`auto_ptj_mjj = F`, or `ptj < 0`) the τ minimum is a real cut that changes σ
and differs between integration channels. vibegraph does not build a
channel-dependent cut; the card is refused with
`RunCardError::XqcutAboveJetThreshold` (`runcard/matching.rs:84`), a scoped
refusal ([backlog](../backlog/feature/mlm-ptj-below-xqcut-refused.md)).

### A tighter τ floor was built and dropped

The partition bound `√ŝ ≥ max over partitions Σ max(Σ p_T,min, √timelike_floor)`
is `50 + 20·n_j` GeV on the matched llj cards, against the current
`max(mmll, Σ p_T,min)`. It is provably implied (0 accepted points below it). It
was measured and reverted:

- waste removed: 6–7.5% of draws in the survey and first iteration, then
  0.8–1.1% per iteration once VEGAS adapts;
- `pp_to_llj_mlm`: rel²·CPU 516 ± 95 → 460 ± 99 µs, not significant;
- `pp_to_llj_fixed` (banked, `xqcut = 0`): summed `w_max` rose 7.80e3 → 1.33e4
  and predicted unweighting efficiency fell 5.45% → 3.18%; the
  sample-against-integration bound failed at −1.80% (bound 1.5%), and the
  five-seed headroom spread widened, while σ itself held.

A tighter *exact* floor made the weight tail heavier, and why was not
diagnosed ([backlog](../backlog/performance/tau-floor-fattens-weight-tail-undiagnosed.md)).
The lesson stands on its own: a provable floor is safe for σ but not free for
the grid; measure unweighting efficiency, not only σ and variance.

[^n34-tf]: Note 34 §1.2: the floors, their proofs, the oracle and the measured payoff.
[^n34-s5]: Note 34, deferred S5: the lower-edge residual.
[^mg-setcuts]: `setcuts.f` at `b7687064`, the `xqcut` rewrite.
[^mg-myamp]: `myamp.f` `set_peaks` at `b7687064`.
[^n41-m6]: Note 41 M6: xqcut floors already in the maps; the τ floor measured and dropped.
[^n41-m1]: Note 41 M1: the τ-minimum audit and the refusal.
[^n41-14]: Note 41 §1.4: MadGraph's cut setup and hints under `xqcut`.
