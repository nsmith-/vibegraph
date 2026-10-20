---
type: Design
title: "Multi-tier VEGAS, absolute grid coordinates and sharp cut edges (draft)"
description: "Which cut edges each channel's grid can see, what absolute coordinates, a shared (τ, y) tier, cut-aware conditional windows and a pluggable sampler would each buy, and what to measure before choosing."
status: draft
tags: [vegas, multichannel, grid-coordinates, cuts, phase-space-map, sampler, design]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-20}
sources:
  - {id: per-channel, resource: "per-channel-vegas-grids.md", title: "One VEGAS grid per multichannel term"}
  - {id: seam, resource: "importance-sampler-seam.md", title: "A pluggable importance sampler (proposed)"}
  - {id: coords-item, resource: "../backlog/performance/vegas-grid-coords-not-absolute.md", title: "VEGAS grid coordinates are relative to each point's window"}
---

Draft for a design session. Code is cited at `origin/main` = `2ad97207` unless a
line says otherwise. The paused absolute-coordinate work is cited at
`origin/vegas-absolute-grid-coords-p1-wip` = `3b23d875` (code) and its report
`a7361630` (`sessions/P1-report.md`).

Labels: **[code]** read in the source at the cited line; **[kb]** stated in the
knowledge bundle; **[paper]** a paper or external codebase read for this note;
**[inferred]** my own derivation or judgement; **[P1, unverified]** a claim from
the paused session's report that I did not confirm in code.

---

## 0. Summary

- [inferred] On the banked rows, almost no cut edge is both inside a
  coordinate's window and at a fixed absolute value of that coordinate. Those
  are the only edges absolute grid coordinates make sharper. The edges that are
  aligned with an axis are mostly already map edges (`τ_min`, the `m_ll` floor,
  the floors xqcut implies). The jet `pT` edge is aligned, but its position
  depends on the earlier coordinates in both relative and absolute coordinates.
  The lepton `η`, `ΔR` and `pT` cuts and the MLM kT veto have no axis in any
  channel.
- [inferred] A shared outer tier over `(τ, y)` is cheap and unbiased **because
  the grid density never enters the mixture `g`**. It sharpens no edge, though.
  What it buys is pooled statistics for the beam variables.
- [inferred] The lever with the most promise for edges that are aligned but
  drift is a fourth option, not on the original list: **cut-aware conditional
  windows**. Each coordinate's window is narrowed to the exact (or provable)
  accepted interval, given the coordinates drawn before it. That generalises
  the timelike floor, the spacelike floor and the planned t-rapidity bound. For
  `pp_to_jj` it could remove every rejection.
- [inferred] Measure before building anything. One extension of an existing
  probe (§5) tells, per channel, which edge carries the weight tail, and
  whether that edge is fixed in relative coordinates, in absolute coordinates,
  or in neither.

---

## 1. The problem, precisely

### 1.1 Where `(τ, y)` is drawn today

- [code] **Per channel, inside each channel's VEGAS grid.** A hadronic
  channel's grid is over `OUTER_NDIM + channel_ndim` coordinates, with `(τ, y)`
  prepended (`proton.rs:1071-1075`, `:1908-1910`). `value_in_channel` maps
  `grid_u[0..2]` through `map_point` and passes the rest to the channel
  (`proton.rs:2534-2546`). `integrate_channels` builds one `VegasGrid` per
  channel over that dimension (`budget.rs:669-698`).
- [code] The analytic `τ` map is the same for every channel of a run.
  `TauMap::Log` or `TauMap::InverseSquare` is decided per process, not per
  channel (`maps.rs:44-55`; the rule is in `MapOptions::resolve`, `maps.rs:248`).
  `y` is uniform in `[−y_max(τ), y_max(τ)]` (`proton.rs:1982-2001`).
- [code] The τ support `[τ_min, 1]` is fixed for each `ProtonIntegrand`, with
  `τ_min = shat_min/s` (`proton.rs:1587`). The τ coordinate is therefore
  already absolute within one multiplicity. y's window moves with τ.
- [code] **The mixture density does not contain `(τ, y)`.**
  `ScaledMultiChannel::density_at(√ŝ, p)` sums `αⱼ gⱼ` at the point's own `√ŝ`
  (`channel.rs:822-857`), and the term weight is `αⱼ/g` at that energy
  (`channel.rs:889`). The outer Jacobian is a common factor inside `shape`
  (`proton.rs`, `m.jac`, `:2001`, `:2046`).
- [code] The α survey draws `(τ, y)` **flat** with a selection coordinate
  (`survey_variance`, `proton.rs:2866-2905`). No grid is involved.
- [code] A matched (MLM) run is a `MultiplicitySum` of one `ProtonIntegrand`
  per multiplicity (`multiplicity.rs:90-110`). Each part has its own `τ_min`,
  so `u[0]` means a different τ in each part.

### 1.2 Which edges are aligned, in which channels

"Map edge" means the window boundary of a drawn coordinate, at `x = 0` or
`x = 1`, which no grid needs to learn.

| row / cut | `τ`, `y` (outer) | `m_ll` invariant | rung `t` / split angle | decay angles of the ℓℓ system | verdict |
|---|---|---|---|---|---|
| `τ_min = ŝ_min/s` (all rows) | **map edge** [code `proton.rs:1587`, `cuts.rs:642`] | — | — | — | already sharp |
| `pp_to_llj`, `mmll = 0` [kb `reference-row-rationale.md`] | — | floor from `ptl`, `drll` (`2·10·10·(1−cos 0.4)` ≈ 16 GeV²) is a **map edge** in every channel [code `cuts.rs:751-813`]; the true boundary below is curved | — | — | floor sharp; residual curved |
| `pp_to_llj_fixed`/`_dyn`/`_mlm`, `mmll = 50` | — | **map edge** (2500 GeV², attained within 1.0002) [kb `cut-implied-timelike-floors.md`] | — | — | already sharp |
| `ptj = 20` (llj rows) | — | — | aligned with `t` (or the q* decay angle) **at fixed `(ŝ, m_ll)`**; the map bounds only `|t| ≥ ptj²` [code `cuts.rs:676-684`] | — | aligned, but drifts |
| `etaj = 5`, `etal = 2.5` | couples `y`… | | …with the angles | | non-aligned everywhere |
| `ptl`, `drll`, `drjl` | | | | functions of `(m_ll, t, φ, θ*, φ*, y)` | non-aligned everywhere |
| `pp_to_jj`: `ptj = 20` | — | — | `|t|·|u|/ŝ ≥ ptj²` is a `t` window `[t₋(ŝ), ŝ − t₋(ŝ)]` | — | aligned, drifts; exact window derivable |
| `pp_to_jj`: `etaj = 5` | needs `|y| < 5` | — | `|cos θ*| < tanh(5 − |y|)` given `y` | — | aligned **conditionally on `y`**; exact window derivable |
| `pp_to_jj`: `drjj = 0.4` | | | | | inert: back-to-back jets have `ΔR ≥ π` [inferred] |
| MLM: `ptj → xqcut`, `mmjj → xqcut` | `τ` floor implied [kb] | jet-pair floor `xqcut²` is a **map edge** | spacelike regulator `xqcut²` | — | already sharp |
| MLM: kT veto at `xqcut` | | | | | per flavour group, not a per-point edge; "cannot become a floor" [kb `cut-implied-timelike-floors.md` §xqcut] |
| `htjmin` and the other CMS-survey `ht*` cuts | | | | | refused today: `UNIMPLEMENTED_CUTS` [code `cuts.rs:352-400`, `htjmin` at `:379`]. With one jet `HT = pT_j` (aligned, like `ptj`); with two or more jets it is non-aligned [inferred] |

Two corrections to the framing I was given:

- [inferred] In `p p > l+ l- j`, **every** channel draws `m_ll` as an
  s-channel invariant. At LO the lepton pair always comes from γ*/Z, and the
  KB counts 6 merged channels [kb `channel-budget-allocation.md`]. "`m_ll` is
  an axis in the Z/γ* channel but curved in the t-channel one" holds for
  processes like `e+ e- > e+ e-`, not for the banked llj rows.
- [kb] `pp_to_llj` itself runs at `mmll = 0`. The `mmll = 50` cut belongs to
  `_fixed`, `_dyn` and `_mlm`.

### 1.3 Where the weight tail actually sits

- [kb] On `pp_to_ll_0j2j_mlm` the heaviest weights sat in single wide bins of
  **the first two coordinates**, which are `(τ, y)`. `@0`'s edge bins were 17×
  the average width, at large lepton-pair rapidity, "where `etal = 2.5` keeps
  only a narrow slice of decay angles" (`vegas-grid-weight-tail.md`). This was
  measured before the `K = 6` bound.
- [inferred] That is the signature of a **non-aligned** edge: an outer `y`
  correlated with an inner decay angle. A product grid can only widen the `y`
  bin. No outer tier, shared or not, and no absolute coordinate fixes this.
  Either a window conditional on `y` (§3 D) or a non-separable sampler (§3 C)
  can.
- [kb] On `pp_to_llj` the residual concentrates where the `m_ll` map's lower
  edge meets the curved `ΔR`/`pT` boundary (`m_ll ∈ [0, 5)` var/σ 24.5 → 55.7,
  about 9% of the row's variance; `llj-map-lower-edge-on-cut-edge`).

---

## 2. Absolute coordinates on their own

What they buy:

- [kb] MadEvent's `sample_get_x` bins an invariant on `s/s_tot` or
  `−t/s_tot` and restricts the draw to the point's window. A feature at a fixed
  absolute invariant value is then a fixed grid location
  (`madevent-phase-space-maps.md` §Grid coordinates).
- [inferred] This helps only a feature **strictly inside** the window **at a
  fixed absolute value**. An edge at the window boundary is already at `x = 0`
  or `x = 1` in relative coordinates. Candidates on today's rows:
  - a Z peak drawn inside a γ* channel's log map;
  - an upper mass cut such as `mmllmax`, which no banked row sets;
  - a residual shape tied to a mass scale, for example the Z interference in a
    γ* channel.
- [inferred] **It does not pin a `pT` edge.** For a massless jet recoiling
  against a system of mass `M`, `pT² = t·u/ŝ` with `t + u = −(ŝ − M²)`. The
  edge is therefore `|t|₋ ≈ ŝ·ptj²/(ŝ − M²)`, which moves with `ŝ` and `M` on
  the absolute `−t` axis. On the map's log axis it sits at
  `x = ln(ŝ/(ŝ−M²)) / ln((ŝ−M²)/ptj²)`, which also moves. Neither coordinate
  system holds it still.
- [kb] The KB already reaches the same conclusion for poles: "For a
  Breit–Wigner or log-mapped invariant the analytic map already pins the
  feature, so the difference matters for flat-drawn invariants and residual
  shape."

What they cost:

- [P1, confirmed in code] The plumbing exists in WIP form. `GridPoint` draws a
  point's `ndim` uniforms up front and maps each dimension through its bins
  when the walk asks for it (WIP `vegas.rs:347-445`). The draw count per point
  is unchanged, so `adapt_blocks_iteration`'s `point·ndim` seek holds.
  `draw_through` inverts a window through a fixed transform and pays
  `T′(y)·(y_hi − y_lo)` (WIP `diagram_channel.rs:1986-2003`). Nothing has been
  tested yet; the report says only `cargo check` ran.
- [code] **The log map is window-shaped.** Its floor is
  `min(10 GeV², t_hi/50)` (`diagram_channel.rs:1792`, `:1845`). On an absolute
  support it becomes a different map. P1 reached the same finding, so absolute
  mode is a physics change for every zero-width pole, not only a change of
  coordinates.
- [inferred] **In-window resolution collapses for flat transforms.** A flat
  invariant on `[thr², s_tot]` at `√ŝ ≈ 200 GeV` occupies a window of about
  `ŝ/s ≈ 2·10⁻⁴` of the axis, so a 64-bin grid has no bin inside it. A log or
  Breit–Wigner transform keeps a sizeable fraction; for example
  `ln(ŝ/thr²)/ln(s/thr²) ≈ 0.4` for a log map. Absolute mode therefore needs
  MadEvent's pre-warped transforms (its `setgrid`) on every invariant it
  touches.
- [P1, unverified] K on an absolute axis caps absolute bin width but no longer
  guarantees in-window resolution. The acceptance rescale would then need a
  coverage-aware denominator. I agree from the arithmetic of
  `bound_bin_width` (`vegas.rs:297-322`), which bounds widths on `[0, 1]` and
  knows nothing about windows.
- [P1, unverified] Replay must store the in-window `y` verbatim, which needs a
  new `SliceCoords` variant. The mode goes into the artifact and `MapChoices`.
  This is consistent with WIP `coords.rs:43-70`: `SliceCoords::within` returns
  `lo + (hi − lo)·u` and so would re-window a stored value.

With tiering:

- [inferred] τ is already absolute within a part. Absolute `y` (binned on
  `[−½ ln(1/τ_min), ½ ln(1/τ_min)]` with a τ-dependent window) is the one outer
  change absolute mode would add.
- [inferred] Absolute τ matters only for an outer tier **shared across
  multiplicities**, since each part has its own `τ_min`. That case could use
  P1's windowed draw directly.

---

## 3. The design space

Notation: term `j` is `I_j = ∫ h_j`, with `h_j = αⱼ f/g · J_map` on channel
`j`'s unit cube. Outer coordinates are `u_o = (u_τ, u_y)` and inner
coordinates are `u_i`. VEGAS densities are `q(u_o)` (outer) and `p_j(u_i)`
(inner).

### A. A shared outer grid × per-channel inner grids

**A1: a shared grid, with points still per channel (recommended form).**

```text
Î_j = (1/N_j) Σ_n h_j(u_o,n, u_i,n) / (q(u_o,n) · p_j(u_i,n)),
      u_o ~ q (shared),  u_i ~ p_j,  independent streams per channel
Var(Î_j) = (1/N_j) [ ∫ h_j² / (q p_j) − I_j² ]
```

- Unbiasedness [inferred]: holds for any `q > 0` because the VEGAS bins cover
  `[0, 1]` [code `vegas.rs` `from_raw` endpoint checks]. The density `q`
  appears in no other channel's weight, because `g` contains no grid density
  [code `channel.rs:822-857`, `:889`].
- α adaptation [code + inferred]: unchanged. The survey runs before any grid
  and draws flat (`proton.rs:2866-2905`); `g` does not see `q`.
- Per-channel error [inferred]: unchanged. Streams are still per block, so the
  terms remain independent and the quadrature sum stays valid.
- Per-channel `w_maxⱼ` [code + inferred]: still a frozen scan of channel `j`
  (`unweight.rs:401-450`), now drawn through `q × p_j`. The channel is still
  drawn ∝ `w_maxⱼ` (`unweight.rs:9-28`).
- Refinement of `q` [inferred]: the optimum given the `p_j` minimises
  `Σ_j (1/N_j) ∫ H_j/q`, with `H_j(u_o) = ∫ h_j²/p_j du_i`. That gives
  `q* ∝ √(Σ_j H_j/N_j)`. In VEGAS terms, the outer histogram is
  `Σ_j hist_j[d][k] / N_j²` over each block's existing per-bin sums. Each
  block already returns its own `hist` in point order
  [code `BlockIteration::hist`, `vegas.rs:994`]. Summing the blocks in block
  index order is deterministic and independent of `-j`. P1's concern about a
  cross-block reduction [P1, unverified] is real but cheap.
- Cost [inferred]: small.
  - The grid for dimensions 0 and 1 is shared and the rest stays per channel.
    Because VEGAS is a product density, this needs **no** contract inversion:
    the eager draw just reads two edge tables.
  - The artifact can stay at today's format by copying the shared edges into
    each `ChannelGrid::grid` (`artifact.rs:294-306`). `generate` is then
    unchanged.
- Risk to the eager path [inferred]: none behind a flag. Default off means the
  grids are built exactly as now.
- What the per-channel split's measured gains become [kb + inferred]:
  - The split's measured wins (Δσ 1.16–1.37×, `w_max` efficiency 1.7–2.9×)
    came from the coordinate semantics differing per channel
    (`per-channel-vegas-grids.md`), and all were on fixed-beam rows with no
    outer coordinates.
  - `(τ, y)` mean the same thing in every channel, so argument 1 of that note
    does not apply to them. Argument 2 (correlations a product density cannot
    learn) applies only weakly.
  - The expected effect is a gain on floor-bound, starved channels, which get
    `Σ N` points for their outer marginal instead of about 512 accepted
    [kb `channel-budget-allocation.md`]. Expect a loss where channels' `(τ, y)`
    marginals differ. §5 M4 measures that difference for free.
- K per tier [inferred]: natural. A1 gives the outer dimensions their own
  `GridRule`. The tail evidence in §1.3 points at the outer dimensions.
  However, a per-dimension K is a strict generalisation that needs no tier.
- Sampler trait [kb + inferred]:
  - The seam proposal keeps "one trained sampler per channel"
    (`importance-sampler-seam.md`). A1 needs either a sampler whose state is
    split into shared and per-channel parts, or the driver composing two
    samplers per channel (outer shared, inner owned).
  - Composition keeps the trait unchanged: `draws_per_point = 2 + d_j`, and
    the records are routed to two accumulators.
  - Generic `F: Real` is unaffected.
- Lanes [inferred]: identical to today. A1 changes which edges a dimension
  reads, not how it is drawn.

**A2: a shared outer *point* (nested), rejected.**

- [inferred] Draw `u_o` once, then one inner point per channel (or a channel
  draw):
  `Î = Σ_j h_j(u_o, u_i^j)/(q(u_o) p_j(u_i^j))`.
- It is unbiased, but:
  - the per-channel terms become correlated through `u_o`, so the quadrature
    error is wrong and a covariance is needed;
  - `N_j` is forced equal (or the scheme degenerates to the selection-coordinate
    mixture that the split replaced);
  - the cost of an outer point scales with the channel count.
- It gives up the hard split's allocation and floors
  [kb `channel-budget-allocation.md`].

**A3: the outer tier as its own multichannel (Sherpa's arrangement).**

- [paper] Sherpa multiplies independent beam, ISR and FSR multichannel weights
  (`PHASIC++/Main/Phase_Space_Point.C:294-303` at Sherpa master `a26184ad`,
  fetched for this note). It feeds every tier the same point value for its α
  update (`Phase_Space_Point.H:112-121`).
- [inferred] Here that would be
  `g = G_o(τ, y) · g(Φ | ŝ)` with `G_o = Σ_a β_a q_a`, and Kleiss–Pittau on `β`.
  The `q_a` could be a 1/τ² channel and one Breit–Wigner in ŝ per whole-state
  resonance.
- It answers the open "τ map decided per process" item
  (`tau-rule-reads-whole-process`) without a τ map per channel.
- It is orthogonal to A1 and costs an outer density sweep per point, which is
  cheap: two dimensions and a handful of `q_a`.

### B. Per-channel grids with absolute coordinates (the paused plan)

This is MadEvent's choice: one grid per configuration covering all dimensions,
absolute invariants and windowed draws [kb `madevent-phase-space-maps.md`].

- Cost [P1 + inferred]: large.
  - Contract inversion: WIP diff of 17 files, +1192/−178.
  - Fixed transforms for four map families, each with an inverse.
  - A new replay variant and artifact field.
  - Pre-warped transforms (§2) and a coverage-aware acceptance rescale.
- Payoff on the banked rows [inferred]: small (§2). It pins no edge from §1.2
  that is not already a map edge.
- Eager path [P1, unverified]: P1 argues bit-identity in relative mode, which
  is untested. The log map's window dependence (§2) means absolute mode cannot
  be bit-identical, and need not be.
- K [P1]: needs a per-axis K and a coverage-aware rescale.
- Sampler trait [inferred]:
  - The windowed per-coordinate draw is natural only for a **separable**
    sampler. VEGAS and VEGAS+ (which keeps the separable grid) qualify.
  - A flow or a tree sampler draws `u` jointly and cannot honour a window on
    coordinate `k` chosen after coordinates `< k`, except by rejection. B and
    C therefore compose poorly.
  - The seam's "fill `u`, return `1/p(u)`" signature (`importance-sampler-seam.md`)
    and P1's driven `Coordinates::within` are two different contracts. The
    seam would have to choose.
- Lanes [kb + inferred]: the in-window bin restriction is a per-lane bin
  search over window edges, which waits on the `Real` lane extension
  (`real-has-no-lane-masks-or-bin-search`). In relative mode, `within` is pure
  arithmetic.

### C. Per-channel grids plus a cut-aware stage on the non-aligned variables

- [kb] VEGAS+ adds stratification within the grid, which reallocates points to
  high-variance hypercubes. It is unmeasured here, and it is blocked on the
  sampler seam (`vegas-plus-stratification-unmeasured`). Flows learn the
  correlated density, need an ML-dependency decision, and also wait on the seam
  (`normalizing-flow-sampler-untried`).
- [inferred] This is the only option that can learn the lepton `η`/`ΔR`/`pT`
  surfaces and the kT veto in §1.2. Stratification does not sharpen an edge,
  but it stops a wide bin's few accepted points carrying the whole bin, which
  is the tail mechanism [kb `vegas-grid-weight-tail.md`].
- [paper] MadNIS learns the channel weights jointly with one flow per channel
  (`references/papers/madnis.md`). That is the learned form of C combined with
  α adaptation.
- Eager path [kb]: untouched, since a new kind sits behind `--sampler`.
  Statistical gates only.
- K [inferred]: a VEGAS-kind knob. Other kinds ignore it, and the seam
  proposal already moves `GridRule` into a VEGAS config.
- Lanes [kb]: a flow's forward pass is the natural batch consumer. VEGAS+
  needs the same per-lane bin search as plain VEGAS.

### D. Cut-aware conditional windows (proposed here)

- [inferred] Narrow each drawn coordinate's window to the exact or provable
  accepted interval **given the coordinates drawn before it**. Use the same
  window in the draw and in the density, the pattern `draw_lo` already follows
  [code `diagram_channel.rs:1875-1878`; kb `cut-implied-timelike-floors.md`].
  An aligned edge that drifts then becomes a map edge at `x = 0` or `x = 1`,
  in relative coordinates, for every `ŝ`. That is sharper than absolute
  coordinates can make it.
- Examples:
  - `pp_to_jj`, `ptj` [inferred]: at fixed `ŝ`, `|t| ∈ [t₋, ŝ − t₋]` with
    `t₋ = (ŝ/2)(1 − √(1 − 4ptj²/ŝ))`.
  - `pp_to_jj`, `etaj` [inferred]: `|y| < η_max`, and
    `|cos θ*| < tanh(η_max − |y|)` given `y`. With `ptj` this makes the 2 → 2
    jet cuts an exact window, so in principle there is no rejection. The `y`
    bound is an **outer-tier** window.
  - General `y` bound [inferred]: when every final-state leg carries an `η`
    cut, `|y_sys| ≤ max_i η_max,i`, because
    `e^{2y} = Σ pT_i e^{η_i} / Σ pT_i e^{−η_i}`. This is a provable bound, not
    an exact one, and it needs the bias oracle.
  - `pp_to_llj` [kb + inferred]: the t-rapidity bound already designed in
    `t-channel-map-ignores-rapidity-cut` is one instance. The "windowed" soft
    split angle, which confines the lepton-pair decay angle to the energy
    window, is another, and it halved llj's evaluations [kb
    `llj-map-lower-edge-on-cut-edge`]. The lepton cuts in 2 → 3 give bounds,
    not exact windows.
- Unbiasedness [kb]: holds only while the window is provable. The test is
  `no_accepted_configuration_sits_below_a_subsystem_floor`
  (`cut-implied-timelike-floors.md`). An over-tight window biases σ silently.
- Mixture [inferred]: each channel's density uses the window computed from the
  point's own earlier coordinates, as `draw_lo` does today, so `Σ αⱼ gⱼ` stays
  defined at a foreign point.
- Cost [inferred]: per cut and map family. Each window is a small derivation
  plus a test; there is no grid or artifact-format change. The maps change, so
  the σ calibrations follow.
- Eager path [inferred]: default off until measured.
- K [inferred]: unaffected. Windows remove the low-acceptance bins that the K
  bound exists to tame.
- Sampler trait [inferred]: orthogonal, since this is the map side of the seam.
  It composes with A1 and C.
- Lanes [kb]: needs lane-wise `min`/`max`/select, which is listed in
  `real-has-no-lane-masks-or-bin-search`. It does not need a bin search.

### Other arrangements checked

- [kb] MadEvent: B with single-diagram enhancement in place of a mixture, and
  pre-warped grids (`setgrid`). It has no shared tier: each `G<config>` job's
  grid includes the beam variables.
- [paper] Sherpa: A3 (independent beam, ISR and FSR mixtures, each with its own
  α).
- [inferred, not verified at a pinned commit] I believe Sherpa also keeps a
  VEGAS grid per channel; this was not read in the fetched files.

---

## 4. What the paused plumbing (P1) is worth under each option

| P1 piece | A1 | A3 | B | C | D |
|---|---|---|---|---|---|
| Contract inversion (`Coordinates`, `GridPoint` with eager uniforms and lazy mapping, `value_in_channel_driven`) | not needed: a product grid can share two dimensions' edges eagerly. P1's "a tiered source answers τ, y from the outer grid" works but is optional [P1 claim confirmed: WIP `proton.rs:2543-2562` asks `coords.unit()` twice, then hands the source to the channel] | not needed | **needed** | not needed for a joint sampler. A per-coordinate source does not fit flows (§3 B) | not needed: relative windows already exist (`windowed`, `draw_lo`, `with_fiducial_t_max`, `diagram_channel.rs:865`) |
| Fixed-transform maps with the window inverted | not needed | not needed | **needed**, plus a decision on the log map's shape | not needed | not needed, but `FixedTransform::interior_coordinate` (WIP `diagram_channel.rs:1947-1965`) is a ready tool for mapping a cut-implied window into a map coordinate |
| Absolute `SliceCoords` replay variant | — | — | needed | — | — |
| Coverage-aware acceptance rescale | — | — | needed | — | — |
| K per axis | useful | — | needed | — | — |

- [inferred] Under every option except B, P1's code is optional. Its most
  reusable part is `GridPoint`'s split of eager uniforms and lazy mapping. That
  split is the right shape if the sampler seam ever needs driven draws.
- [code] At `origin/main` the seam proposal assumes eager draws (`unweight.rs:422`, `:664`).
- [inferred] Keep the WIP branch parked. Do not merge it without a consumer.

---

## 5. Measurement plan (before choosing)

Base: `probe_llj_weight_tail_regions` (`validate_hadronic.rs:4288`, helper at
`:4302`). It already trains per-channel grids, draws 200k points per channel
through `event_in_channel` (with lab momenta), and bins the second moment by
`m_ll`, `pT_j` and `√ŝ` per channel [code].

- **M1, edge proximity per channel** (`pp_to_llj`, `pp_to_llj_dyn`,
  `pp_to_jj`; then `pp_to_llj_mlm`).
  - Per point, record each active cut's signed margin in its own variable:
    `pT/pT_min − 1`, `η_max − |η|`, `ΔR/R − 1`, and `m_ll` against the floor.
    For rejected points, record which cut failed first.
  - Report per channel, for `ε ∈ {1%, 5%, 10%}`, the share of `Σw`, of `Σw²`,
    of the top-0.1% second moment, and of the top-20 weights (the Hill
    window, `unweight.rs:288`) carried by points within `ε` of each edge.
  - Add the same shares for the frozen `w_max` scan's top weights.
- **M2, is the edge fixed in relative or absolute coordinates, or neither?**
  - For the near-edge points of each cut, record every grid coordinate `x_k`
    **and** its absolute value (`s/s_tot`, `−t/s_tot`, `y`).
  - An edge that is a window edge clusters at `x_k ∈ {0, 1}`. One fixed in
    absolute terms clusters in absolute value across `√ŝ` bins. One that
    clusters in neither, but tightly at fixed earlier coordinates, is a D
    candidate. One that clusters in no coordinate is C territory.
  - This directly decides between B and D.
- **M3, per-dimension bin widths of trained grids.**
  - Read the `grid.bin.zst` of existing runs: which dimension holds the widest
    bins per channel, and at which `x`.
  - This is the offline study `bin-width-bound-best-k-per-row` already asks
    for. It decides per-dimension K versus per-tier K.
  - Cost: minutes, no integration.
- **M4, outer-marginal similarity across channels** (decides A1).
  - From the same artifacts, compute the pairwise L1 distance between channels'
    trained `(τ, y)` bin densities (the dimension 0 and 1 edges), and
    `N_total/N_j` per channel.
  - Similar marginals plus starved channels favour A1; dissimilar marginals
    mean A1 trades the split's gain away.
  - Cost: minutes.
- **M5, what removing rejection could buy at most** (a ceiling, not a design).
  - Per channel, measure the trained-grid acceptance `Q` (already counted:
    `BlockIteration::accepted`, `vegas.rs:991`), and ns/eval for accepted
    and rejected points separately.
  - [inferred] A perfectly sharp window with the shape unchanged gives
    variance per point `Q·M₂ − σ²` against `M₂ − σ²`. Time per point goes from
    `Q·T_ME + T_ps` to `T_ME + T_ps`. When `T_ME ≫ T_ps` the variance×CPU gain
    tends to 1.
  - The payoff of sharpening is therefore in the **tail shape** near the edge
    (M1), not in the rejected volume. M5 checks whether `T_ps` is small enough
    for that to hold on these rows.
- **Statistics** [kb AGENTS.md]:
  - M1 and M2 are diagnostics: one seed for the table and five seeds for any
    share used in a decision, with the spread reported.
  - Any design's payoff, once built, is ≥5 seeds per arm, and 20+ seeds on the
    deciding pair, read against the measured seed spread. Report rel²·CPU,
    ε_unw and the Hill index together.
- **Cost** [inferred]: one session.
  - About 100–200 lines of probe code (ignored test, `--nocapture`).
  - Runs: llj with 6 channels × 200k draws is about 1.2M evaluations per row
    per seed; jj is smaller. Minutes each on 4 cores. M3 and M4 are free.
  - Existing helpers to reuse: `probe_fiducial_bound_on_llj_fixed`
    (`validate_hadronic.rs:841`, a two-arm map comparison at the same seeds),
    `probe_jj_channel_partition` (`:2391`), `probe_unweighting_weight_max`
    (`validate_sigma.rs:1552`) and `--grid-max-bin-width none` for an
    unbounded arm [kb].

---

## 6. Open questions for the design session

1. **Is the target "sharp edges" or "a lighter tail"?** Recommendation: the
   tail (Hill index, ε_unw, rel²·CPU). M5 suggests that rejected volume is
   nearly free when matrix elements dominate the cost.
2. **Build absolute coordinates now?** Recommendation: no. Park P1's WIP
   branch. Revisit only if M2 finds high-weight edges fixed in absolute
   invariants.
3. **Shared outer tier (A1)?** Recommendation: decide on M4. If built, use the
   A1 form (shared grid, per-channel points), not A2, and share per
   multiplicity part, not across parts.
4. **Make conditional windows (D) the next build?** Recommendation: yes,
   starting with `pp_to_jj`'s exact 2 → 2 window (`pT` and `η` given `y`).
   That row can be made rejection-free and has a clean oracle. Then extend the
   t-rapidity bound already designed for `ee_to_ee_nsz` to hadronic rungs.
5. **K per tier or per dimension?** Recommendation: per dimension, decided by
   M3. A tier is then just a set of dimensions.
6. **Driven or eager draws in the sampler seam?** Recommendation: eager (the
   seam as proposed). Driven draws serve only B, and they exclude flows.
7. **An outer multichannel (A3) for τ?** Recommendation: it is the clean
   answer to `tau-rule-reads-whole-process`, but it is low priority. It belongs
   after M4.
8. **Non-aligned lepton and kT-veto edges?** Recommendation: they go to C via
   the seam, VEGAS+ first, since it needs no ML dependency. In the meantime,
   provable D bounds (the `|y_sys|` bound, lepton-pair angular windows) take
   what they can.

---

## Found

- [code] `probe_llj_weight_tail_regions` calls the outer draw "the shared
  logarithmic `(τ, y)` draw" (`validate_hadronic.rs:4371`). For a run with no whole-state resonance (the
  llj rows) the default is `TauMap::InverseSquare` (`maps.rs:44-55`, `:248`),
  so the comment is stale.
- [code] Several KB line references are stale:
  - `vegas-integrator.md` cites `refine_grid` at `vegas.rs:684` (now `:886`),
    `adapt_blocks_iteration` at `:799` (now `:1036`) and `VEGAS_NBINS` at
    `hadronic.rs:69` (now `:72`).
  - `hadronic-tau-y-sampling.md` cites `map_point_with` at `proton.rs:2009`
    (now `:1982`) and `τ_min` at `:1614` (now `:1587`).
  - `cut-implied-timelike-floors.md` cites `timelike_floor` at `cuts.rs:720`
    (now `:751`).
- [inferred] No test pins the inverse transforms on interior sub-windows (P1's
  own Found, unchanged).
- [inferred] The `|y_sys| ≤ max η_max` outer bound (§3 D) is unfiled. It needs
  the bias oracle on rows where every leg is η-cut.
- [inferred] An exact 2 → 2 `pT`/`η` window for `pp_to_jj` is unfiled.
