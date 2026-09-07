# 36 — MadEvent phase-space map survey, and a soft-shaped 2-body angle (2026-09-07)

Two things, from one user observation: the 1→2 splitting in the phase-space
map draws its decay angle isotropically, while a splitting kernel goes like
`1/(z(1−z))` in the daughters' energy fractions — so (§2) does a sampling map
with that structure improve `p p > l+ l- j`'s convergence, and (§1) which of
MadEvent's maps are we not using yet. §1 is read off `genps.f`, `myamp.f`
(`set_peaks`), `dsample.f` (`sample_get_x`, `setgrid`) and `transpole.f` in
the pinned `mg5amcnlo` submodule; §3 is the measurement; §4 the decisions.

## 1. MadEvent's maps against ours

MadEvent's phase-space generator (`one_tree`) is the same recursive
2-body decomposition as `DiagramChannel` — s-channel invariants outermost-in,
then a t-channel chain, then the s-channel decays — and it importance-samples
in two places: **analytic transforms** (`transpole`) applied inside
`sample_get_x`, and **pre-warped VEGAS grids** (`setgrid`) that take the place
of an analytic map. The table lists every map it applies, what vibegraph does
for the same variable, and whether adopting the MadEvent form is worth a
measurement.

| Variable | MadEvent | vibegraph | Verdict |
|---|---|---|---|
| `τ = ŝ/s` (hadronic) | `transpole(pole=−2, width=xo)`: `y = xo/z`, density `∝ 1/τ²` above `xo`, flat below (`xo` = the cut-implied `ŝ_min/s`, or a BW `tan` map when the whole final state sits on one resonance) | `τ = τ_min^(1−u)`, density `∝ 1/τ` above the cut-implied `τ_min` (`ProtonIntegrand`) | **Candidate.** A steeper map on a luminosity that falls faster than `1/τ`. VEGAS's grid absorbs the difference in one dimension, so the payoff is bounded; measure on `dy13`/`pp_to_llj` once a PDF set is reachable (§3 records why it was not here). |
| `y` (rapidity) | flat in `[½ ln τ, −½ ln τ]` (`GENCMS`) | identical | Same. |
| s-channel invariant with a finite width | `transpole(pole>0)`: `s = m² + mΓ tan θ`, `θ` uniform; width floored at `mass × small_width_treatment`; applied only when `m + 5Γ ≥ xm` (the pole is reachable) and the line is not an identical-particle radiation (`iden_part`) | the same `tan` map (`draw_invariant`), no width floor, applied whenever the line has a width | Same map. The two conditions are worth noting: MG drops the BW when the cut-implied floor `xm` sits more than `5Γ` above the pole, where ours still stretches a tan map over the far tail. Low priority — the affected region carries little σ. |
| s-channel invariant with **no** width (massless or zero-width line) | `setgrid(itype=1, xo)`: the VEGAS grid is pre-warped with 90% of its bins log-spaced from `1` down to `xo` (`grid = xo^(1−i/ngu)`) and 10% linear below `xo`; `xo` = `max(xm², s_min cuts, 0.8·ET₁ET₂ΔR², xqcut)/s`, an invented `min(10/s, s/50, ½)` when none applies. `gen_s` itself is flat for `spole = 0` — the shape lives in the grid | the same two-piece shape as an analytic map (`log_scale`: log over `[t₀, t_hi]`, linear below), floored by the cut-implied `timelike_floor` (note 34 §1.2) | Same shape, already adopted (note 21). |
| s-channel invariant with no pole at all (an auxiliary invariant, or a massive line MG chooses not to BW) | `setgrid(itype=1)` **always** — every s-channel invariant without a BW gets the `1/x` pre-warp, offset `a = m²/s` ignored by `itype=1` | flat | **Candidate, small.** A massive off-shell line drawn flat (e.g. the `q*` in `g q → q* → q Z` when its invariant is drawn) would get a `1/s` shape. In this code that case is rare: a massive s-channel line has a width and takes the BW map; the flat draw is left only for auxiliary invariants of vertices with more than two subsystems. |
| t-channel transfer | `setgrid(itype=1, xo)` on `−t/s` with `xo = min(ET₁, ET₂)²/s` from the two sides' cut-implied energies, plus a `1/10000` invented floor; the grid variable is the absolute `−t/s`, restricted per point to `[−t_max, −t_min]/s` | `draw_t`: `t = m² − (m² − t_min)·exp(−xN)`, density `∝ 1/(m² − t)`, the pole floored at a fraction of the fiducial scale and the window capped at `t ≤ −scale` (`from_diagram_regulated`) | Same shape, already adopted (note 28). |
| t-channel chain ordering | `tstrategy` per configuration (`reorder_tchannels`, `export_v4.py`): **one side eats all** (`±2`, every rung measured against beam 2) or **ping-pong** (`±1`/`−2`, alternate beams), chosen from which of the outermost exchanged lines are massless; `ping-pong` when both ends are massless and the ladder has three or more transfers | one side, rungs ordered from beam 0 (`spine_chain`; `with_rung_order` exists to measure a deliberately wrong order) | **Candidate for ≥ 3-rung ladders** (`p p > j j j j`-class). Inert for every gated row: llj and the 2→3 QCD rows have at most two transfers, and MG itself uses one-side for fewer than three. |
| s-channel decay angles | flat `cos θ` and `φ` (`costh = 2x−1`, `jac·4π`), built against the frame axes by `mom2cx` and boosted along the parent — nothing shaped; VEGAS's per-dimension grid is what adapts | identical (before §2) | The question this note is about. **MadEvent has no such map either**; §2 builds one and §3 measures it. |
| t-channel rung azimuth | flat | flat | Same. |
| Grid coordinate semantics | `sample_get_x` draws each invariant on a grid over the **absolute** dimensionless invariant (`s/s_tot`, `−t/s_tot`, over `[xgmin, xgmax] = [−1, 1]`), restricting the draw to the point's own `[xmin, xmax]` window by bin index and scaling the weight by the window's bin count | each invariant's coordinate is the **fractional** position in its own `[lo, hi]` window, so a feature at a fixed absolute invariant moves in the unit cube as the other draws move `lo`/`hi` | **Candidate, structural.** For a BW or log-mapped invariant the analytic map already puts the feature at a fixed coordinate, so this matters only for flat-drawn invariants and for the residual shape the map leaves. The cost is the whole grid layer (a window-restricted draw needs `xbin` inversion and per-point bin-range weights). Not for this session. |
| Channel weight | `sde_strategy = 1`: `|A_c|²/Σ|A_d|²`; `sde_strategy = 2`: the product of the configuration's propagator denominators (`get_channel_cut`), plus the `tmin_for_channel` damping `exp((t − t_min)/(t + 1))` of a t-channel configuration's weight below `t_min` | the Kleiss–Pittau mixture `Σ αⱼ gⱼ` with variance-minimising `α` (note 21) | Known, recorded in note 29 chain B (the `AMP2` scale-channel draw) and TODO's channel-set migration item; not a phase-space map. |
| One grid per channel | one VEGAS grid **and one integration** per configuration (`G<config>/`), summed | one grid per channel over the mixture (note 21 addendum) | Known; the addendum records why MG's arrangement is stronger. |
| Zoom (`nzoom`) | re-draw within the last bin during unweighting refinement | none | Unweighting-side, not a map. |

Two MadEvent behaviours the survey found that are *not* maps but bear on how
the maps are read:

- `set_peaks` computes every floor from the run card's cuts — `ptj`, `ptl`,
  `etmin`, the `ΔR·ET` product, `xqcut`, `mmjj` — exactly the derivation
  `Cuts::timelike_floor` and `spacelike_floor` do here. MG's floors are the
  grid's *pre-warp only*: the grid can still move below them, which is what
  the reserved 10% of bins is for. Ours are the draw's support edge for the
  log map (with the same linear tail below), and the transfer bound is a
  hard edge.
- `small_width_treatment` floors every width at `mass × 1e-6` before the map
  is built, so a zero-width massive line (a UFO with `WZ = 0`) still gets a
  BW map of that width rather than a `1/(s − m²)²` log map with no
  regulator. Ours takes the log-map branch for `mΓ ≤ 0`. Both are unbiased;
  the difference is the map, and only for widths a model has set to zero.

## 2. The soft-shaped 2-body angle

### 2.1 Where the `z(1−z)` structure lives

A 2-body split of a parent with CM energy `E`, momentum `P`, mass `M`
(`γ = E/M`, `β = P/E`) into daughters of rest-frame energies `a`, `a′` and
momentum `p*` gives, with `θ*` the rest-frame angle from the parent's
direction of flight,

```
E₁ = γ(a + βp* cos θ*),   E₂ = γ(a′ − βp* cos θ*),   z = E₁/E,   1 − z = E₂/E.
```

So `E₁E₂ = E² z(1−z)`, and a massless emission's splitting kernel — `P_gg ∝
1/z + 1/(1−z)`, `P_qq ∝ 1/(1−z_q) = 1/z_g` — puts a `1/z` or `1/(z(1−z))`
into `|M|²` that the isotropic draw leaves in the weight. `dz = (βp*/E) d cos θ*`,
so a density `∝ 1/(E₁E₂)` in `cos θ*` is `dz/(z(1−z))` over the accessible
`[z_min, z_max] = [γ(a − βp*)/E, γ(a + βp*)/E]`: a two-sided logarithmic map
in the energy fraction, regulated at both ends by the pair's own mass through
`β < 1`. For a parent at rest (`β = 0`) it is the isotropic map.

The **isotropic angle is not measured from the parent's flight direction**
today: `sample_branch` builds the rest-frame vector against the collision-CM
axes and boosts it, and `mom2cx`/`boostm` in MadEvent do the same. The energy
fraction is then a function of `(cos θ, φ)` *and* the boost, which is exactly
the correlation a product-form VEGAS grid cannot learn — the reason to shape
the map rather than leave the angle to the grid.

### 2.2 The map

`SoftSplit` (`diagram_channel.rs`): `w = ln((a + b c)/(a′ − b c))` uniform,
`b = βp*`, `c = cos θ*`; `dw/dc = b(a + a′)/((a + b c)(a′ − b c))`, so the
density is `∝ 1/(E₁E₂)` and the measure replacing the flat `2` is
`Δw (a + b c)(a′ − b c)/(b(a + a′)) = Δw E₁E₂ M/(b E²)`. Written against
`w − ln(a/a′)` in `log1p`/`expm1`, it reaches the flat map continuously as
`b → 0` (`Δw → 2b(a + a′)/(aa′)`), so the root of an all-timelike tree — at
rest at the draw, at rest to rounding when the density re-sums its daughters
— reads the same either way (pinned at `1e-12`). `b < min(a, a′)` always,
so every logarithm is finite.

The density side rebuilds `E₁`, `E₂`, `E` from the momenta through the
subsystem memo, as invariants of the collision CM — no frame the sampler
worked in leaks in — and the walk accumulates the measure from the drawn
`c`, so reciprocity remains a real check (`soft_split_angles_stay_reciprocal_
and_cover_the_same_volume`, worst `1e-9` over the topology spread, with the
massless `V_n` reproduced). A rule selecting no split leaves the channel
bit-identical, key included; a selected split marks the key with its floors (`S(E₁ᵐⁱⁿ,E₂ᵐⁱⁿ)`).

Two rules exist: **every split** (`|_, _| true`), and **soft emission**
(`soft_emission_rule`: a daughter *is* a single massless vector — a gluon or
photon — read off the diagram's legs by `massless_vector_slots`). The
mechanism is symmetric, `1/(E₁E₂)`; a `q* → q g` split has its soft
singularity on one side only, and a one-sided `1/E_g` map is the obvious
refinement if the symmetric one measures well.

### 2.3 Where it can fire in `p p > l+ l- j` — nowhere

Read off the channel set (24 channels; `phase space: 4 channels, 2
peripheral, 2 all-timelike` per subprocess):

- `q q̄ → l+ l- g`: both diagrams' quark propagators are spacelike, so both
  channels are spines with the gluon as the rung's leaf. Its angle is fixed
  by the `1/t` transfer draw, and its energy by the remainder invariant
  `ŝ_rest = s_{ll}`, drawn BW (Z) or log (γ*). No 2-body split has the gluon
  as a daughter.
- `g q → l+ l- q`: the t-channel diagram is a spine (blob `l+ l-`, recoil
  `q`). The s-channel diagram is an all-timelike tree whose root `q* → q +
  (l+ l-)` **is at rest** in the collision CM — the shaped map is the
  isotropic one there by construction.
- The one boosted composite split in every channel is `Z*/γ* → l+ l-`, whose
  decay has no soft enhancement; shaping it costs variance.

So on llj the soft-emission rule selects nothing, and the every-split rule
selects only the lepton pair. The soft-gluon structure of llj — `|M|² ∝
1/(tu) = (1/t + 1/u)/(ŝ − s_{ll})` at massless `g` — sits in the **spine's
remainder-invariant draw**, `E_g = (ŝ − ŝ_rest)/(2√ŝ)`: the soft edge is
`ŝ_rest → ŝ`, the *upper* edge of a BW map centred far below it, and in the
`τ` draw when `s_{ll}` is on the Z pole. The `1/(z(1−z))` of the
initial-state splitting `q → q* g` is a `1/(ŝ − ŝ_rest)` shape on the
remainder invariant, not an angle. That is a different map (§4).

The analysis note 34 §3 left is consistent with this: llj's residual is
9× MadGraph's points at its cut edges (`ptj`, `ΔR`, the `m_ll` floor), and
the "soft/collinear jet" hypothesis lost to cut-edge variance by an order of
magnitude in the probe's own ratios (TODO, `convergence-and-2to3-abort`).

The splits the map is *for* are boosted composite subsystems with a
massless-vector daughter: `g* → g g` and `q* → q g` inside 2→3 QCD and
beyond. `u ū → g g g` (`g* → g g` under the root and on rungs, `P_gg`) and
`g g → g u ū` (`q* → q g`, `P_qq`; and `g* → u ū`, which the soft rule skips)
are the positive controls; `g u → e+ e- u` is the negative one — the shaped
lepton pair.

## 3. Measurement

Host: the session's 4-core Linux container (release build, `-j 4`; the
artifact is thread-count independent, wall times are not comparable to the
M3 Max figures elsewhere). Protocol: `vibegraph integrate <proc> --run-card
<gu_to_epemu's banked run card> --target-rel 0.001 --seed s`, seeds
20260719/20/21, fixed beams at `ebeam = 250` (`lpp = 0`), MG's default cuts
and the kT-clustered dynamical scale. The figure of merit is the evaluations
the convergence stop needs to reach 0.1% (χ²-scaled), which is what the map
changes; σ is the consistency check across arms.

**`p p > l+ l- j` itself could not be run here**: the PDF set's hosts
(`lhapdfsets.web.cern.ch`, `data.nnpdf.science`) are refused by the
session's egress policy (403 on CONNECT), and no other input carries the
grids. §2.3 shows the soft-emission rule selects no split on llj, so its
hadronic measurement would be the isotropic run bit for bit; the every-split
rule on llj is the `g u → e+ e- u` row's lepton-pair effect below, at
hadronic luminosity.

### 3.1 Evaluations to 0.1% (χ²-scaled), three seeds

`base` is the isotropic map; `soft` the soft-emission rule (a daughter is a
single gluon or photon); `every` shapes every composite split; `unfloored`
runs the map down to the kinematic edge instead of the cut-implied energy
floors. Each cell is one recorded run's evaluation count in millions, seeds
20260719 / 20260720 / 20260721, with the σ the runs agree on.

| process | base | soft, floored | every, floored | every, unfloored |
|---|---|---|---|---|
| `u u~ > g g g` (16 ch) | 5.04 / 4.80 / 4.80 | **3.48 / 3.36 / 3.12** (−31% / −30% / −35%) | identical to `soft` (every composite split has a gluon daughter) | 6.24 / 6.96 / 6.60 (+24% / +45% / +38%) |
| `g g > g u u~` (16 ch) | 5.11 / 3.49 / 2.75 | 5.36 / 3.49 / 2.74 (+5% / 0% / 0%) | 3.12 / 5.74 / 3.87 | 6.99 / — / — (series stopped) |
| `g u > e+ e- u` (4 ch) | 1.20 / 0.96 / 1.08 | **bit-identical** (same counts, same σ to the last digit) | 1.20 / 0.72 / 0.72 | 1.44 / 0.84 / 0.84 |

σ across arms: `u u~ > g g g` 1031.6 / 1030.4 / 1029.7 (base) against
1029.9 / 1030.0 / 1029.6 (soft), each ± 0.95 pb; `g g > g u u~` 8350 / 8350 /
8355 against 8349 / 8348 / 8352, each ± 7 pb; `g u → e+ e- u` 0.10867 /
0.10875 / 0.10878 ± 0.00010 on every arm. No arm moved a mean by more than
its own error.

Read:

- **The map does what it was built for, once regulated**: `u u~ > g g g`,
  where `g* → g g` is the `P_gg` case, spends a third fewer evaluations for
  the same χ²-scaled 0.1% on all three seeds.
- **Unregulated, it is a loss**: the `1/(E₁E₂)` shape runs down to
  `z_min ~ m₁₂²/(4E²)`, orders below the `ptj = 20` GeV threshold, and spends
  most of its angular draws on rejected points. This is the same lesson note
  34 §1.2 drew for the invariant draw — the cut-implied floor is what makes
  a `1/x` map pay — and `Cuts::energy_floor` is that floor for the angle.
- **`g g > g u u~` is neutral under the soft rule** (two seeds within 0.1%
  of the base count, one +5%): its `q* → q g` splits sit on channels that
  carry little of the mixture, and the symmetric map spends half its
  attention on the quark's soft end, which `P_qq` does not have. A
  one-sided `1/E_g` map is the refinement to try there.
- **Inert where it should be**: `g u > e+ e- u` has no split with a
  gluon daughter, and the soft-rule run reproduces the base run bit for
  bit. `p p > l+ l- j` is the same channel set at hadronic luminosity, so
  the production rule cannot change it — the user's question has an answer
  by construction, not only by argument: **the isotropic 1→2 angle is not
  where llj's convergence residual lives.**
- **Shaping the lepton pair** (`every` on `g u > e+ e- u`) reads 1.20 /
  0.72 / 0.72 against 1.20 / 0.96 / 1.08, with the two improved seeds sitting
  on the `--min-iters 6` floor, and the unfloored `every` arm reads mixed.
  The shape is wrong for `Z*/γ* → l+ l-`, so what helped is the **floor**:
  confining the lepton pair's angle to `E_l ≥ ptl`. Three seeds, two of
  them capped by the iteration floor, is a hint and not a measurement; it
  is filed as a follow-up (§4), because the same window applies to llj's
  lepton pair on an isotropic shape.

### 3.2 The one gated row the rule touches: `ee_to_mumua`

`e+ e- > mu+ mu- a` has `μ* → μ γ` timelike splits — the only Standard-Model
gated row with a boosted composite split carrying a massless vector (the
others put their gluon on a spine rung or have a two-body final state).
`probe_resonant_seed_stability` (five seeds, 80 000 × 8) before and after,
this host:

| arm | seed 20260719 | 11 | 22 | 33 | 44 | budget ×2 |
|---|---|---|---|---|---|---|
| base | 1.00898e-1 ± 2.01e-4 | 1.00827e-1 ± 2.04e-4 | 1.00686e-1 ± 2.05e-4 | 1.00623e-1 ± 2.03e-4 | 1.00707e-1 ± 2.05e-4 | 1.00781e-1 ± 1.49e-4 |
| soft | 1.00699e-1 ± 1.77e-4 | 1.00854e-1 ± 1.75e-4 | 1.00691e-1 ± 1.79e-4 | 1.00535e-1 ± 1.61e-4 | 1.00705e-1 ± 1.73e-4 | 1.00760e-1 ± 1.19e-4 |

The row's known +1.0% offset against MadGraph (reference-adjudicated, note
29 chain D; `PULL_REPORTED_NOT_ASSERTED`, `rel_tol 0.03` enforced) is
unmoved: rel +0.74% to +1.05% on both arms. The quoted error at the same
budget is 12–21% smaller under the rule (variance −22% to −37%), χ²/dof
0.64–1.35. The gate's `rel_tol 0.03` holds with the same margin.

### 3.3 What ran and what did not

Recorded: the hermetic suite (`cargo test --workspace`, exit 0, with the
rule as the production default), `cargo clippy --workspace --all-targets
-- -D warnings` and `cargo fmt --all --check` clean, the four new unit tests
(reciprocity + `V_n` with every split shaped; bit-identity with the rule
off and root-isotropy with it on; the floors confining the draw; a 4.7×
variance reduction on a `1/(E₁E₂)` toy integrand), `Cuts::energy_floor`'s
unit test, and the runs above. **Not run**: `validate-sigma` and
`validate-hadronic` as gates — the hadronic rows need the PDF set this
session cannot fetch. The partonic gated rows are inert under the rule by
the §2.3 argument (no gluon-daughter split: the llj subprocess rows put
their gluon on a rung, the 2→2 rows have a root at rest), verified bit for
bit on `g u > e+ e- u`, and the one row that is not inert is §3.2. The next
host with the PDF set should run `pixi run --skip-deps validate` before
this is called validated at the layer level.


## 4. Decisions

1. **The soft-emission rule with cut-implied energy floors is the production
   map** (`hadronic::use_multichannel`, `proton.rs`'s channel construction):
   a 2-body split with a single gluon or photon daughter draws its angle
   from the parent's flight direction with density `∝ 1/(E₁E₂)` inside the
   window `E_i ≥ Cuts::energy_floor`. Every other split is isotropic and
   bit-identical to before, which covers every gated row but `ee_to_mumua`
   (§3.2). The map key marks shaped splits, so an artifact trained under the
   old map is refused by `generate` for a process the rule touches.
2. **Not adopted**: shaping every split (wrong for `V → l+ l-`, and the
   `g g > g u u~` seeds disagree on it), and the unregulated map.
3. **Filed** (TODO, performance backlog): a one-sided `1/E_g` variant for
   `q* → q g`; an energy-floored *isotropic* window for every composite
   split (the §3.1 lepton-pair hint, the one item that can reach llj);
   MadEvent's `1/τ²` map and its `tstrategy` ping-pong for ≥ 3-rung
   ladders (§1); the `p p > l+ l- j` and `dy13` measurements themselves,
   blocked here on the PDF host.
4. **For llj specifically**, the residual is not the angle. The
   `1/(z(1−z))` of the initial-state `q → q* g` splitting is a
   `1/(ŝ − ŝ_rest)` shape on the spine's *remainder invariant* (§2.3), which
   competes with the Z/γ* pole on the same variable; if that is to be
   sampled, it is a second channel per spine (a soft-remainder map beside
   the resonant one), not a change to any existing draw. That is a design
   item, not a measurement, and it should be preceded by the
   `probe_llj_weight_tail_regions` decomposition binned in `ŝ − ŝ_rest`
   — the variable the tail would live in if this is right.


## 5. The map choices as configuration (2026-09-07, second session)

Every open choice of §1 that is implemented is now a flag on `vibegraph
integrate`, with `auto` a rule that reads the process, and the settled
choices are banked in the artifact (schema 8) so `generate` rebuilds its
channels and its `τ` draw from what the grids were trained on. The types
live in `phasespace::maps`: `MapOptions` (what was asked, `None` = auto),
`MapChoices` (what was settled, serialised), `ProcessShape` (what the rule
reads: how many splits the soft-emission rule would select, the longest
chain), and `MapChoices::channel`, the one place a channel is built for
integration — the previous commit had `integrate` and the fixed-beam
`generate` building channels through two different code paths, which the
shared builder closes.

| flag | values | `auto` |
|---|---|---|
| `--map-split-angle` | `isotropic`, `windowed`, `soft-emission`, `soft-all` | `soft-emission` where `ProcessShape::soft_emission_splits > 0`, else `isotropic` |
| `--map-tau` | `log`, `inverse-square` | `log` (the alternative is unmeasured: PDF host) |
| `--map-rung-order` | `derived`, `reversed` | `derived` |

`windowed` is new: the isotropic density confined to the cut-implied energy
window on every moving split — the floor without the shape, so the two
effects the first session's `every, floored` column mixed can be read apart.
A file older than schema 8 reads back under `MapChoices::LEGACY` (isotropic,
log, derived), which is what every such run integrated under; that also
retires the silent mismatch the previous commit introduced for a pre-existing
artifact of a process the rule touches.

### 5.1 Measurements, same protocol as §3.1

Evaluations to a χ²-scaled 0.1% in millions, seeds 20260719 / 20260720 /
20260721; `base` is §3.1's isotropic column. `auto` reproduced §3.1's `soft,
floored` column bit for bit (same counts, same σ to the last digit), which is
the check that the flag path builds the same channels the hard-wired rule did.

| process | base | `auto` (= `soft-emission`) | `windowed` | `soft-all` | `reversed` (with `auto` angles) |
|---|---|---|---|---|---|
| `u u~ > g g g` | 5.04 / 4.80 / 4.80 | 3.48 / 3.36 / 3.12 | 3.84 / 3.60 / 3.12 | identical to `auto` | **3.00 / 3.24 / 3.00** (−14% / −4% / −4% vs `auto`) |
| `g g > g u u~` | 5.11 / 3.49 / 2.75 | 5.36 / 3.49 / 2.74 | 5.61 / 4.74 / 2.99 | 3.12 / 5.74 / 3.87 | 5.24 / 2.87 / 2.87 |
| `g u > e+ e- u` | 1.20 / 0.96 / 1.08 | bit-identical to base | 1.44 / 0.84 / 1.80 | 1.20 / 0.72 / 0.72 | bit-identical to base (one rung) |

Readings:

- **The window carries most of the `u u~ > g g g` gain** (−24% / −25% / −35%
  from the window alone; the soft shape on top adds −9% / −7% / 0%). The
  first session attributed the whole effect to the shape; the split is now
  measured.
- **`reversed` beats `derived` on `u u~ > g g g` on all three seeds** (its
  two-rung ladders: the chain then draws the outer transfer first). On
  `g g > g u u~` it sits inside the row's own 2.7–5.4M seed spread, and on a
  one-rung process it is the identity. Three seeds at 4–14% do not move a
  rule (AGENTS.md: a rung-to-rung difference is read against a 20-seed
  spread); filed as the first thing to measure at that size. MadEvent's own
  `reorder_tchannels` flips the chain's side for a two-transfer ladder with
  massless lines at both ends by leg-number order, i.e. arbitrarily — this
  is the same question asked of a different chain semantic.
- **On the lepton pair, the shape helps and the bare window hurts**:
  `soft-all` 1.20 / 0.72 / 0.72 against `windowed` 1.44 / 0.84 / 1.80 on
  `g u > e+ e- u`. That is the opposite of the §4 guess (that the floor was
  the lever). Three seeds of a row whose two better seeds sit on the
  `--min-iters` floor; suggestive, not a rule.
- `g g > g u u~` distinguishes nothing at three seeds.

### 5.2 Absolute grid coordinates — the design, not yet the code

The §1 candidate the user asked to have as an option. What it is: MadEvent's
`sample_get_x` bins each invariant's VEGAS coordinate as the *absolute*
dimensionless invariant (`s/s_tot`, `−t/s_tot`) and restricts each draw to
the point's own `[x_min, x_max]` window by bin index, scaling the weight by
the window's bin count. A cut edge or a pole is then a fixed location in
grid space whatever the other coordinates did. Our coordinate is the
fractional position in the window, so the same feature drifts through the
unit cube as `ŝ` and the earlier invariants move.

What it takes here, and why it did not fit this session:

1. **The VEGAS↔channel contract inverts.** Today VEGAS draws the whole point
   (`VegasGrid::draw`) and hands the channel a slice; a windowed draw needs
   the channel to *drive* the grid one coordinate at a time, since the
   window of coordinate `k` depends on coordinates `< k`. That is a
   `Coordinates` source threaded through `sample_branch`/`sample_spine` and
   the hadronic outer draw, a per-dimension `draw_in_window(dim, lo, hi)` on
   `VegasGrid` that also reports the bin for refinement, and driven variants
   of `adapt`, `sample_frozen`, the `w_max` scan and the unweighting replay.
   The eager path must stay bit-identical (it draws the same uniforms in the
   same order, so it can).
2. **The analytic maps become window-independent.** A BW/log/`t` map must be
   a fixed transform of the absolute coordinate over the full range, with
   the window inverted through it (`T⁻¹(lo)`, `T⁻¹(hi)`), so the channel's
   analytic density stays grid-free — which is what keeps the Kleiss–Pittau
   mixture `Σ αⱼ gⱼ` well defined at a foreign point. MadEvent needs no such
   care because its single-diagram-enhanced weight never evaluates one
   channel's density at another's point. Four map families, each with an
   inverse.
3. **A rejection-form shortcut is not worth building**: interpreting the
   eager coordinate as absolute and zero-weighting draws outside the window
   is unbiased but, for a hadronic run whose windows sit at `ŝ ≪ s`, wastes
   most draws; it would measure worse for a reason unrelated to the idea.

Filed in TODO as its own item with these three parts; the flag is not
exposed until the code exists, so no artifact can claim a map that was not
run.
