---
type: Design
title: "The importance sampler and the phase-space map, designed together"
description: "The map owns every coordinate window, cut-aware ones included, and the sampler stays a density on the unit cube; no shared (τ, y) tier (replayed, 5 seeds); K per coordinate role; MadSpin as an opt-in decay stage after the map."
status: draft
tags: [sampler, phase-space-map, plugin, vegas, cuts, madspin, artifact, design, user-requested]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-22}
sources:
  - {id: seam, resource: "importance-sampler-seam.md", title: "A pluggable importance sampler (proposed)"}
  - {id: tiers, resource: "multi-tier-vegas-and-cut-edges.md", title: "Multi-tier VEGAS, absolute grid coordinates and sharp cut edges (draft)"}
  - {id: m1, resource: "../sprints/vegas-absolute-grid-coords/sessions/M1-report.md", title: "M1: where the weight tail meets the cut edges, per channel"}
  - {id: probe, resource: "vibegraph-lib/src/proton/edge_probe.rs", title: "edge_probe.rs: probe_shared_outer_tier_replay"}
  - {id: h1, resource: "https://github.com/nsmith-/vibegraph/pull/69", title: "PR #69: gridpack generate, the integrate/generate split's H1 (artifact version 13)"}
  - {id: h2, resource: "https://github.com/nsmith-/vibegraph/pull/60", title: "PR #60: the PDF from the run card, H2 (artifact version 14)"}
measured:
  - {host: "4-core Linux container", command: "VG_EDGE_ROWS=<row> VG_EDGE_SEEDS=5 VG_EDGE_DRAWS=200000 <lib test binary> --ignored --nocapture --test-threads=1 proton::edge_probe::probe_shared_outer_tier_replay"}
---

Designed with the user on 2026-10-22, from M1's measurements[^m1] and the two
drafts it answers: the [sampler seam](importance-sampler-seam.md) and
[multi-tier VEGAS](multi-tier-vegas-and-cut-edges.md). Where this concept and
either draft disagree, this concept holds. Code is cited at `5bc4a19`; the
artifact section is checked against `cfd9f7a`, the main that carries the
integrate/generate split.

## Decisions

| | decision | evidence |
|---|---|---|
| D1 | **The map owns every coordinate window**, cut-aware conditional ones included, in window-relative coordinates. The sampler is a density on the unit cube and never sees a window. (User, 2026-10-22.) | M2: no high-weight edge is fixed in an absolute invariant; the jet `pT` and `η` edges are pinned by earlier coordinates (`R` 0.03–0.11) |
| D2 | **Absolute grid coordinates (option B) are refused.** The driven, per-coordinate draw they need is not built. | D1; M2 |
| D3 | **No shared `(τ, y)` tier.** The seam keeps mergeable accumulators and coordinate roles, which a tier would need; no product sampler type is built. | the replay below: a shared `τ` costs the starved γ* channels 1.35–1.39× variance; no row gains more than 1.5% |
| D4 | **The bin-width bound is per coordinate role**, with today's `K = 6` on every role as the default. | no measurement favours a per-role value yet; `bin-width-bound-best-k-per-row` is that study |
| D5 | **MadSpin-kind decays are a decay stage after the map**, opt-in. The default for a decay request stays the in-process decay chain, trained with the rest of the point. (User, 2026-10-22.) | [decay chains without MadSpin](../performance/decay-chains-without-madspin.md) |
| D6 | The sampler's draw, weight and accumulate stay generic over `F: Real`, and so do the map's windows. The default path stays byte-identical until a window or kind is switched on. | the [seam](importance-sampler-seam.md)'s contract |
| D7 | The artifact records the sampler kind, the window set, the per-channel coordinate layout, any non-uniform `K` and the decay stage. It follows the [artifact versioning](../pipeline/artifact-format-versioning.md) the integrate/generate split left (PRs #69 and #60, merged). | [coordination with the split](#coordination-with-prs-69-and-60) |

## The point, end to end

```text
u_a ∈ [0,1]^d_a  ──sampler──▶  1/p(u_a)                      (adapted coordinates)
u_a ──outer map──▶ (τ, y), J_o
    ──channel j's map, windows inside──▶ momenta, αⱼ/g        (mixture)
u_t ∈ [0,1]^d_t  (own substreams, never adapted)
    ──trailing stages──▶ scale configuration; decay stage (D5)
weight = (1/p) · J_o · αⱼ/g · f
```

Three properties make this work, and each is already true for the `(τ, y)` draw
and the scale-configuration draw:

- The sampler's density appears in no other channel's weight: `g` sums the
  channels' map densities at the point's `√ŝ` (`channel.rs:822-857`, `:889`).
- A density shared by every channel, such as the outer Jacobian or a decay
  stage's, is a common factor outside `Σ αⱼ gⱼ`.
- Trailing draws come from substreams addressed by point index
  (`SCALE_DRAW_STREAM_BASE`), so `-j` does not move them.

## D1: the map owns windows

**Contract.**

1. The sampler sees only `[0,1]^d_a`. It fills `u`, returns `1/p(u)`, and never
   sees a window, a momentum or a cut.
2. A channel's map takes the cube onto a region at least as large as the
   cut-accepted region. A window narrows a coordinate's range from the
   coordinates drawn before it, and the window's width enters the map's own
   weight. `windowed`, `draw_lo` and `apply_fiducial_t_max`
   (`diagram_channel.rs:1875`, `:1889`, `:2448`) already do this.
3. At a foreign point, `density_at` recomputes every window from that point's
   own earlier coordinates. `Σ αⱼ gⱼ` therefore stays defined, and the
   walk–density reciprocity test (`WALK_DENSITY_TOL`,
   [channel contract](channel-contract.md)) covers windowed draws unchanged.
4. A window may be a union of disjoint intervals. The coordinate is mapped onto
   them in order, each getting a share of `u` proportional to its length in the
   map's base variable, and the Jacobian is the total length. The ℓℓj jet `pT`
   window in the all-timelike channels is two intervals (`±cos θ`, M2).
5. Coordinates stay relative to their windows. A trained grid's coordinate `k`
   therefore means "the fraction of the allowed range", which moves with the
   earlier coordinates. M2 found no edge that absolute coordinates would hold
   still, and conditional windows put the drifting edges at `x = 0` or `x = 1`.

**Why this side.** A whole-point sampler (flow, FOAM, tree) draws `u` jointly
and cannot honour a window on coordinate `k` chosen after coordinates `< k`,
except by rejection. With windows in the map, every candidate in the
[seam's table](importance-sampler-seam.md#what-the-candidates-need-of-it)
composes with every window, and the seam's draw stays eager (draft question
6).

**What windows can and cannot reach**, on the M1 rows:

| edge | class | route | ceiling (M5) |
|---|---|---|---|
| `pp_to_jj` jet `pT` (65–68% of the top-0.1% M₂ in the `t`/`u` channels) | exact window given `τ` | `|t| ∈ [t₋, ŝ − t₋]`, `t₋ = (ŝ/2)(1 − √(1 − 4ptj²/ŝ))` | 1.30× (jj), with `η` |
| `pp_to_jj` jet `η` | exact window given `(τ, y)` | `|y| < η_max`, `|cos θ*| < tanh(η_max − |y|)` | (in the above) |
| ℓℓj jet `pT` (9–17% of the tail, ~42% of rejections) | window given `(τ, m_ll²)`; two intervals in the all-timelike channels | derived per map family | ≤ 1.5× |
| ℓℓj lepton `η` (~half of rejections, 5–14% of the tail) | class C, aligned with no coordinate | the sampler: VEGAS+ first, then flows | — |
| ℓℓj interior tail (53–76% of the top-0.1% M₂) | no edge | the sampler; where it sits is unmeasured | — |

**Unbiasedness is the window's to prove.** An over-tight window biases σ
silently, so every window carries two tests:

- **Point level.** Draw with the window off, and assert that every accepted
  point lies inside the window computed from its own earlier coordinates. Zero
  violations in at least 10⁶ points per channel. This is the finer oracle, the
  pattern of `no_accepted_configuration_sits_below_a_subsystem_floor`
  ([cut-implied floors](cut-implied-timelike-floors.md)).
- **σ level.** Window on against window off, ≥5 seeds per arm, read by spread
  and χ²/dof. 20+ seeds on the pair a default flip rests on.

Windows are a map option, off by default and recorded in `MapChoices`, beside
the split angle, `τ` map and rung order ([map choices](map-choices.md)).
Turning one on by default is its own PR, with the σ calibrations re-recorded.

## The map side of the seam

Two additions, both inert on the default path:

- **Coordinate roles.** Each channel reports a role for each adapted
  coordinate: `Tau`, `Rapidity`, `Invariant`, `Polar`, `Azimuth`,
  `DecayPolar`, `DecayAzimuth` or `Select`. Configurations address
  coordinates by role, not index. That covers the per-role `K` (D4), a
  tier if one is ever built (D3), and the probes, which today hard-code the M1 layout
  (`x0 = τ` … `x6 = φ*_ll`). The artifact records each channel's role list as
  its layout (D7). They go in with the sampler trait, since its VEGAS config is
  their first consumer.
- **Trailing stages.** `ChannelIntegrand::scale_draw_ndim` (`unweight.rs`)
  becomes a list of trailing stages, each with its own substream base and draw
  count: the scale-configuration draw now, and the decay stage (D5) later. The
  unweighting pass already fills them from a stream of its own and carries them
  in the accepted point, so a reconstruction replays them. That is what the
  decay stage needs. They go in with the decay stage.

## The sampler side of the seam

The [seam draft](importance-sampler-seam.md#the-proposed-seam) stands: draw
(with a record and a fixed draws-per-point), accumulate in point order, refine,
freeze, serialise, and a uniform second implementor. Agreed with the user on
2026-10-26:

- **The scalar is a parameter of the trait, not of the method**, for example
  `FrozenSampler<F: Real>`. The refactor implements it for `F = f64` only. A
  lane implementation is added later with whatever bound the lane-primitives
  design settles, so this seam commits to none of that design.
- **Draw takes uniforms, not a random-number generator**:
  `draw(&self, r: &[F], u: &mut [F], rec: &mut Self::Record) -> F`, with `r`
  holding `draws_per_point()` uniforms. The caller fills `r` from the point's
  substream, so the seek by point index that keeps `-j` from moving bits stays
  with the caller. A lane batch packs N points' uniforms. The record is an
  associated type of each `F` implementation (VEGAS at `f64`: one bin index per
  dimension).
- **Accumulate is scalar `f64`, in point order, always.** A lane batch unpacks
  its records and weights and adds them point by point, so determinism holds by
  construction and the lane design needs no order-preserving scatter-add.
  Refine and freeze are plain `f64`.
- **Mergeable accumulators.** `merge(&mut self, other)` in block-index order.
  `adapt_blocks_iteration` already reduces the blocks' histograms in point
  order. A tier's shared part would need the same operation.
- **No product sampler type.** D3 leaves it without a user; the roles and merge
  are what a tier would build on.
- **A config per kind.** `VegasConfig { nbins, alpha, rule, max_bin_width_by_role }`.
  The per-role overrides (D4) sit beside `GridRule`, not inside it: `GridRule`
  is the artifact's `grid_rule` wire format, so a new field there would change
  the artifact's shape. The overrides enter the artifact with the sampler kind.
  `generate` never reads a config: it rebuilds the frozen state from the
  artifact.

## D3: the shared `(τ, y)` tier, replayed

M4 found the starved channels' `τ` grids 0.22–0.35 (L1) from the pooled grid,
1.8–2.8× their seed noise. It could not say whether that distance costs
variance. The replay measures that cost.

**Method** (`probe_shared_outer_tier_replay`). Train each row as its gate
does, then draw 200 000 points per channel from the frozen grids on the scan's
streams. With `u ~ q_j p_j` and `w = h_j/(q_j p_j)`,
`∫ h_j²/(q p_j) = E[w² q_j/q]`, so the variance under another outer density `q`
needs no new matrix element. The shared optimum given the inner grids is
`q* ∝ √(Σ_j G_j/N_j)`, with `G_j = ∫ h_j²/p_j du_i`. It is estimated per outer
dimension on 128 fine bins from the even-indexed draws, bounded at `K = 6`, and
the variance is evaluated on the odd-indexed draws. The arms:

- `own`: the same estimator built from channel `j` alone, so that sharing is
  compared like for like;
- `shared`: `q*`;
- `pool`: the `N_j`-weighted mixture of the trained grids, M4's pooled density;
- `vegas`: the trained grid itself.

Each arm is applied to `τ`, to `y`, or to both. The row figure is
`Σ_j Var_j/N_j` at the production allocation. Five seeds,
`20261020`–`20261024`. The trained σ̂ reproduce M1's seed for seed.

Variance ratios, mean ± sd over five seeds. The row columns are `Σ_j Var_j/N_j`
of `shared` over `own`, the like-for-like effect of sharing. The starved columns
are per channel (`N_tot/N_j` ≈ 47–60): `shared/own`, and in brackets `pool`
against the trained grid.

| row | row, `τ` shared | row, `y` shared | row, both | starved channels, `τ` shared | starved channels, `y` shared |
|---|---|---|---|---|---|
| `pp_to_jj` | 0.997 ± 0.001 | 1.007 ± 0.002 | 1.000 ± 0.002 | s-channel: 1.02 ± 0.05 (pool 0.76 ± 0.25) | 0.98 ± 0.02 |
| `pp_to_llj_fixed` | 1.015 ± 0.008 | 0.990 ± 0.003 | 1.005 ± 0.011 | γ* 0 / 2 / 4: 1.38 ± 0.05 / 1.37 ± 0.16 / 1.36 ± 0.05 (pool 1.47–1.61) | 0.89–0.98 |
| `pp_to_llj_mlm` | 1.001 ± 0.019 | 0.996 ± 0.002 | 0.995 ± 0.018 | γ* 0 / 2 / 4: 1.39 ± 0.05 / 1.37 ± 0.14 / 1.35 ± 0.06 (pool 1.53–1.56) | 0.95–0.96 |

The large channels (`N_tot/N_j` 2–3.6) move by at most 0.97–1.01 under any
arm. The second-moment estimate's own standard error is at most 0.20 of the
variance (one ℓℓj γ* seed) and ≤ 0.09 elsewhere, below the seed spread it
would have to explain.

**Reading.**

- **Sharing `τ` costs the starved γ* channels 1.35–1.39× variance** on both
  ℓℓj rows, consistently over five seeds. M4's pooled grid costs them
  1.47–1.61×. Their `τ` marginal is pulled to the `mmll` floor, and the pooled
  optimum is shaped by the Z channels whatever the `1/N_j` up-weighting.
- **The row does not gain.** Every row figure is within 1.5% of 1, inside two
  seed spreads, because the starved channels carry a small share of the row's
  variance. A1's only promised beneficiary was those channels, and they lose.
- **Sharing `y` is neutral at the row and mildly helps starved channels**
  (0.89–0.98). That is too little to justify a tier.
- **On `pp_to_jj`, sharing is neutral everywhere.** The s-channel's figures
  scatter by ±0.25 across seeds.
- **Not a verdict on tiers in general.** A1 across multiplicities (MLM
  `MultiplicitySum`) was never on the table, since each part has its own
  `τ_min`. A3, an outer multichannel, is a different construction and is
  untouched.
- **Side finding:** the `own` arm beats the trained VEGAS grid, by 0.85 on
  jj's row and 0.95–0.96 on the ℓℓj rows, almost all of it in `τ`. A one-shot
  refit of the `τ` marginal from the frozen-grid points beats ten damped VEGAS
  iterations. The refit uses 100 000 extra points per channel, so this is not
  an equal-budget comparison, but it suggests the `τ` grid is not converged
  (see Found).

## D4: bin-width bounds per role

`GridRule::max_bin_width` keeps its meaning as the bound on every role. The VEGAS
config gains per-role overrides, empty by default, so every banked grid and
calibration stands. Two things would justify an override:

- `bin-width-bound-best-k-per-row`, the offline study of P2's grids by
  dimension. It now reads by role.
- A window that removes a role's low-acceptance bins. The bound exists to tame
  those bins, so the bound on that role can loosen.

An override is recorded in the artifact. A bound on a tier (draft question 5)
is just an override on that tier's roles.

## D5: MadSpin-kind decays

**Two modes.** A request to decay resonances, for example a CMS card's
`madspin_card.dat`, is read once, at integrate.

- **Chain (default).** The decays are rewritten into the equivalent decay-chain
  process and integrated in-process as today ([decay chains](../process/decay-chains.md)).
  Every decay coordinate is adapted and the full decayed |M|² is the
  integrand. Nothing in the seam changes.
- **Factorised (opt-in, matching MadSpin).** Production is integrated
  undecayed, so σ, the grids and the `w_max` scans are the production ones, and
  σ × BR uses the recorded widths. The decay stage runs at generate, per kept
  production event:
  1. draw the decay kinematics (Breit–Wigner masses inside the cutoff, then
     angles) from a trailing substream;
  2. form MadSpin's decay weight, the ratio of the decayed matrix element to
     the production one;
  3. accept against a per-event maximum estimated by trial decays, as MadSpin
     does.

  The production point is never re-weighted, so the production `w_max` scans
  and efficiencies are unchanged.

**Where the stage sits.** After the map, as a trailing stage. Its density is
the same in every channel, so it never enters `Σ αⱼ gⱼ`. It is never adapted.
Its draws come from substreams addressed by event index, so `-j` holds and a
reconstruction from the recorded uniforms replays an accepted event exactly.
The decayed and undecayed matrix elements come from the core's evaluators,
built for the decayed and the production process. The stage owns no matrix
element code.

**What the implementation must still settle against MadSpin's source**
(`research/refs/mg5amcnlo`, `MadSpin/`), not from memory:

- the exact decay weight and its normalisation;
- the trial count behind the per-event maximum;
- the Breit–Wigner cutoff and its default;
- which `spinmode` values exist and which are supported (`spinmode=none` is
  the plain BR-weighted decay).

**Oracles.**

- σ × BR and the decayed distributions against MadSpin on a banked CMS card,
  `tt̄` with leptonic decays.
- The two modes against each other on the same card: they differ by the
  narrow-width approximation, `O(Γ/M)`. Keep this comparison running,
  informational, while the factorised mode is under construction.

## D6: scalar genericity and lanes

The sampler seam keeps the draft's
[`F: Real` signatures](importance-sampler-seam.md#generic-over-the-scalar-batchable-over-lanes),
implemented and tested at `f64` first. Windows are written over `F` in the
maps, which are already generic. From the lane-primitives design
([real-has-no-lane-masks-or-bin-search](../backlog/performance/real-has-no-lane-masks-or-bin-search.md)),
a window needs lane-wise `min`, `max` and select, plus a select between
intervals for a union. It needs no bin search. That design owns the interface.

**Byte identity.** The seam refactor, the roles and the trailing-stage list
move no draw and no sum. They close on byte identity:

- `grid.bin.zst` at `-j 1` and `-j 4`;
- a fixed-seed `generate` LHE;
- an empty `pixi run --skip-deps validate` report diff.

A window, a non-VEGAS kind, a `K` override or the factorised decay mode is a
change of results by construction, and is gated statistically.

## D7: the artifact, and what `generate` refuses

**What is recorded.** Per channel: the sampler kind with its trained state,
and the coordinate role list. Per run: the window set (in `MapChoices`), any
per-role `K` override (beside `grid_rule`), and the decay request verbatim with
its mode.

**What `generate` refuses**, naming the field each time:

- a kind it does not know;
- a kind whose `ndim` disagrees with the channel;
- a role list that this build's map lays out differently (the map changed
  under the grid);
- a window set it does not implement;
- a decay mode it does not implement.

The kind comes from the artifact, never from a flag.

### Coordination with PRs #69 and #60

Sprint D's [integrate/generate split](../pipeline/integrate-generate-split.md)
has landed: H1 as PR #69 (`ef96721`, gridpack `generate`), then H2 as PR #60
(`cfd9f7a`, the PDF from the run card, with
[run inputs](../pipeline/run-inputs-in-the-artifact.md)). What this design rests
on, as merged:

- **Version numbers.** #69 wrote version 13 and #60 version 14.
  `FORMAT_VERSION` and `OLDEST_READABLE_VERSION` are both 14 (`artifact.rs:31`,
  `:34`).
- **Every artifact is written at the current version.** `version_for` and the
  threshold constants are gone, so "a VEGAS-only artifact keeps an older
  version" no longer holds.
- **The floor is a per-step choice.** [Keeping old readers](../pipeline/artifact-format-versioning.md#keeping-old-readers)
  says a version that shares a schema with a later one costs nothing to keep,
  and raising the floor breaks users' files. #69 and #60 raised it because no
  older file could be replayed.
- **Gridpack-style `generate`.** `generate` takes the artifact plus
  generate-time inputs only, and the generation driver lives in
  `vibegraph-lib/src/generate.rs`.

So:

- Each field above lands as one version step, from 15 on, taking the next free
  number at merge.
- A version-14 file holds only VEGAS grids. Under rule 5 ("upgrade only what
  the old file knows") it can upgrade to the VEGAS kind with its role list
  "not recorded", which skips the layout check. Whether to keep that reader or
  raise the floor is that PR's call.
- A VEGAS-only, window-free artifact is checked for identical replay, the same
  LHE from the same seed, not for identical file bytes, because the version
  number changes.
- The sampler trait refactor now starts from the library driver, so
  `Unweighter` and the generation path are refactored in one place.
- The decay request is an integrate input under the split's principle. It
  defines the process and σ × BR, as the `madspin_card` does inside a MadGraph
  gridpack.

## Landing order

1. This design. It closes `sampler-and-phase-space-seams-designed-apart`.
   `vegas-grid-coords-not-absolute` closes as refused (D2).
2. Sprint D's H1 (#69) and H2 (#60): merged.
3. [sampler-hardwired-to-vegas-grid](../backlog/feature/sampler-hardwired-to-vegas-grid.md):
   the trait, roles, mergeable accumulators and per-role `K`. Byte-identical.
4. [artifact-has-no-sampler-kind](../backlog/feature/artifact-has-no-sampler-kind.md),
   then [integrate-cannot-select-a-sampler](../backlog/feature/integrate-cannot-select-a-sampler.md).
5. The map windows, each its own item and PR:
   [jj-two-to-two-cut-window-missing](../backlog/performance/jj-two-to-two-cut-window-missing.md)
   first, then
   [llj-jet-pt-conditional-window-missing](../backlog/performance/llj-jet-pt-conditional-window-missing.md). They need only the
   `MapChoices` field, so they can run beside 3–4.
6. [madspin-decays-unsupported](../backlog/feature/madspin-decays-unsupported.md):
   the trailing-stage list, then the factorised mode.
7. The sampler candidates, unchanged: VEGAS+, FOAM, trees, flows.

## Answers to the drafts' open questions

- **Multi-tier draft §6:**
  1. The target is the tail. Windows take the aligned share; the sampler takes
     the rest.
  2. Absolute coordinates: no (D2).
  3. A shared tier: no (D3, measured).
  4. Conditional windows next, `pp_to_jj` first (D1, landing order 5).
  5. `K` per role, which subsumes per tier (D4).
  6. Eager draws (D1).
  7. An outer multichannel (A3): untouched. It remains the answer to
     `tau-rule-reads-whole-process`.
  8. Non-aligned edges go to the sampler, VEGAS+ first.
- **Seam draft:** everything stands except where the artifact versioning moved
  under the split, #69 and #60 (D7). This design adds the roles, mergeable accumulators, the
  trait's scalar parameter and the uniforms-in draw.

[^m1]: [M1 report](../sprints/vegas-absolute-grid-coords/sessions/M1-report.md),
    tables M1, M2, M4 and M5.

## Follow-up items

Filed from this session's findings (user, 2026-10-22, all `priority: low`):

- [jj-two-to-two-cut-window-missing](../backlog/performance/jj-two-to-two-cut-window-missing.md)
  and [llj-jet-pt-conditional-window-missing](../backlog/performance/llj-jet-pt-conditional-window-missing.md):
  the first two windows under D1.
- [vegas-tau-grid-unconverged](../backlog/performance/vegas-tau-grid-unconverged.md):
  the replay's `own` arm, a one-shot refit of the `τ` marginal from 100 000
  frozen-grid points per channel, lowers row variance to 0.85 (jj) and 0.95
  (ℓℓj) of the trained grid. Not at equal budget, so an equal-budget,
  ≥5-seed comparison comes before any default moves.
- [llj-interior-weight-tail-unlocated](../backlog/performance/llj-interior-weight-tail-unlocated.md)
  and [mlm-multiplicity-sum-edge-tail-unmeasured](../backlog/performance/mlm-multiplicity-sum-edge-tail-unmeasured.md):
  M1's two open measurements, which size the sampler route.
