# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: `validation-3` (validation) ✅ **closed + merged to
`main`** 2026-07-31; `refdata-2` published and pinned, CI's `banked` job gates
merges. The suite has three declared dependency layers (`hermetic` / `banked` /
`oracle`), one manifest that says which check may assume what, and a
per-process × per-category report the banked layer renders and asserts:
`pixi run validate` ends by writing `target/validation-report/report.md` and
failing if the cells measured are not the cells `validation/manifest.toml`
declares. 26 rows × 4 categories = 104 cells, 72 of them measured in that layer
(68 gated green, 4 informational), 4 oracle-layer, 18 blocked on a named feature,
10 covered-by or admitted gaps. The full record — what each session landed, the
rendered table, the findings register and the recommended order for the follow-up
work — is note 25 §10.

**Next**: the **`v3-backlog` sprint** — **active (launched 2026-08-01), note
27**. B1–B3 ✅ merged to the `v3-backlog` integration branch: the h→ττ pole is
**MadGraph 3.5.7's defect** (fixed upstream in `286feb8e6`, first in 3.6.2),
the general ŝ floor gates `pp_to_bb_fixed` σ at −0.011%, and the
colour-selection premise is falsified in favour of a per-diagram `AMP2_d` draw
(note 27 §B3.2). Decisions D1–D4 in note 27 §6 (D3 = oracle layer moves to
3.7.1; D4 = `AMP2_d` is session B6). Remaining: **B4** (DY event banking at
3.7.1) ∥ **B6** (`AMP2_d`), then **B5** (3.7.1 re-bank + hygiene +
`refdata-3` re-cut + close-out).

Unrun until the user pushes a first tag: `release.yml` and `acceptance.yml`.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. Deferred: multi-rung t-channel ladders (note 21) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 11 partonic GATE rows incl. the 3 QCD 2→2s, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.284 ± 0.537** vs MG 933.110 ± 0.447; **643.765 ± 0.367** vs 644.420 ± 0.315), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **423.048 ± 0.248 pb** over three seeds vs MG 422.840 ± 1.805 (Δ = 0.11σ). Deferred: `dynamical_scale_choice = -1` (needs `kt-clustering`), which also blocks the four llj partonic σ rows |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) + colour-flow (`∝ JAMP2`) selection with the flow→`ICOLUP` dictionary checked against MG's `leshouche.inc` (30/30 subprocesses), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 26 banked MG runs byte-for-byte (258 747 events). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

## Closed-sprint history

One line each; the note is the full record. Earlier sprints
(`helas-generalize`, `mg-validation-coverage`, `cleanup-refactor`,
`performance-sprint`) live in git history and notes 12/13.

- **`color-flow`** (feature, merged 2026-07-12) — multi-flow JAMPs + exact CF |M|²; note 16 (incl. the VVVV phase-bug root cause and the fermion-flow slot-swap debrief).
- **`validation-sprint`** (validation, closed 2026-07-13) — `gg_to_gg` NCOLOR=6 enforced, VVVV −i fixed; notes 12/16.
- **Eval performance program** (performance, closed 2026-07-17) — layout/folding/SoA + helicity expansion + helicity filtering; vs-MG gap 8.6×–110× → **1.2×–3.5×**; note 15. Ratios are single-host (M3 Max); rerun kit `scripts/mg_perf_compare.sh` + note 15 §2.4. Contract: pruned evaluators need partonic-CM beams-along-±z momenta.
- **`hadronic-xsec`** (feature, closed 2026-07-19) — PDF convolution + run-card cuts + (τ,y) VEGAS; σ(pp→e⁺e⁻) vs MG 0.14%/0.07%; `integrate` CLI + artifact; note 18.
- **`validation-2`** (validation, closed 2026-07-21) — V1–V6: NHEL pinning 14/14, proc-card `integrate`, σ-level gate, PDF seam, rooting-soundness (all rooting signs lifted to `fermi_sign`, 0/133), convention-channel guards; V7 deferred (below); note 19.
- **`eval-perf-2`** (performance, closed + merged 2026-07-21) — mul-split, one-shot DAG validation, ZEROAMP skipping, fewest-ext-leg rooting; `forward` **1.18×–2.19×** every process, ≤1e-12 vs MG; note 20.
- **`resonance-sampling`** (feature, closed + merged 2026-07-26) — MadGraph-style multichannel in production; 2 resonant σ rows SKIP→GATE. Transferable lesson: a fixed-seed pull cannot validate a sampler — VEGAS's 1/σ² combination makes a missed region *confidently wrong*, so seed sweeps are part of the gate; note 21.
- **`dynamical-scales`** (feature, closed + merged 2026-07-27) — MG's `αs` RGE + per-event `μR`/per-beam `μF` through the constant pools; 3 QCD σ rows GATE, DY unmoved; found MG's `AQCDUP` π-truncation and `SCALUP` ≠ μR defects (note 07) and the missing `gg→gg` symmetry factor; note 22.
- **`event-output-lhef`** (feature, closed + merged 2026-07-28) — JAMP2 flow selection + `leshouche.inc`-checked `ICOLUP` dictionary, accept/reject unweighting, byte-pinned LHEF writer/reader, `vibegraph generate`, model identity in the artifact (fv3); two plan corrections recorded (channel draw `∝ w_maxⱼ`; `IDWTUP=-4` not required for overweights); note 23.
- **`user-distribution` + `proton-events`** (feature, two tracks, closed + merged 2026-07-31) — Track P: llj amplitude rows 14→18 plus a per-diagram `AMP()` oracle, measured flavour groups, `ProtonIntegrand`, σ(pp→ℓ⁺ℓ⁻j) and `generate` gated at `lpp = 1` (artifact fv3→fv4). Track U: release/CI/acceptance workflows, `~/.vibegraph` cache, consent-gated pinned PDF fetch, `check-events`. Transferable lesson: **a seed sweep is necessary and not sufficient** — five mutually-consistent seeds were collectively 1.0% low, so budget convergence is a second axis. Also `[profile.dev] opt-level = 2` cut `cargo test` 3m16s → 1m05s with nothing weakened; note 24.
- **`validation-3`** (validation, closed 2026-07-31) — three declared dependency layers (`hermetic`/`banked`/`oracle`) with `validation/manifest.toml` as the single per-process source of truth; the `amplitudes` category made hermetic on MadGraph's own banked events; every hadronic σ moved onto the general `ProtonIntegrand`; the new `samples` category (KS + χ² against MadGraph's event samples); Pythia consumption; and one asserted report table over 26 rows × 4 categories. Transferable lesson: **a report is only evidence if every green cell is a recorded measurement** — inferring a cell from "the suite passed" is the same failure as a vacuous check. Findings register in note 25 §10.

---

## 🔎 Validation backlog

### The cells the report cannot fill yet

The rendered table's non-green cells, in the order note 25 §10 recommends taking
them. Every one of them is a measurement that exists, not a suspicion.

- **No banked Drell-Yan event sample** — the `samples` cell of `pp_to_ll` (and so
  of `pp_to_ll_qcd0`, which pointed at it) is `uncovered`, not measured: the
  Drell-Yan reference banks the two `dy13` cards' cross sections and not their
  events, and the row's own banked MadGraph run takes the MG-internal `nn23lo1`
  set at a dynamical scale, which this crate cannot reproduce. Filled by an
  oracle-layer run banking events for the committed `dy13` cards — after which the
  general path's `dσ/dm_ll` at low `m_ll` becomes measurable against MadGraph, the
  thing the deleted `dy_dsigma_dmll.md` table was standing in for. That table had
  lost its regeneration path with the bespoke integrand and was deleted rather
  than reproduced: the `samples` binning that replaces it runs on
  `ee_to_mumu_tata_qcd0`, not on Drell-Yan, so nothing regenerates a Drell-Yan
  spectrum today. (L4.)

### Standing discrepancies to resolve (never a loosened tolerance)

- **`higgs-pole-in-m-tautau` — DIAGNOSED, the defect is MadGraph's** (B1,
  2026-08-01; full evidence in note 27 §B1). Asked for the *same window* directly,
  the two sides agree: MadGraph's own `σ(m(ττ) ∈ [124.9, 125.1])` is
  `7.2077e-5 ± 2.9e-7` pb and ours `7.2065e-5 ± 3.2e-8` pb. MadGraph's window plus
  its complement (`1.2965e-3 ± 3.4e-6`) sums to `1.36858e-3` pb against the
  `1.3373e-3` pb its own unwindowed run reports — **MadEvent fails to close
  against itself by 7.2σ on its own quoted errors**, and our `1.367e-3` pb sits
  0.35σ from its sum. Root cause, in MadGraph's *integration*: **3.5.7** (the
  version that produced every banked run — the banked banner says
  `VERSION 3.5.7 2024-11-29`) computes the `sde_strategy=2` channel weight in
  `get_channel_cut` (`genps.f`) from `(t-Mass)*(t+Mass)` where `t` is already `p²`,
  so it never vanishes on a pole; the Higgs channel gets **α = 1.9e-3** instead of
  `1 − 1.2e-7` at the pole (recomputed from MadGraph's own `configs.inc`/`props.inc`
  at its own on-pole event, and confirmed by the windowed run's realised 0.198%
  channel share), leaving 24 non-resonant channels to find a 6.4 MeV structure in
  a 500 GeV range. The pinned submodule **3.7.1** has `t - Mass**2`; re-running
  3.5.7 with `sde_strategy = 1` gives `1.3742e-3 ± 3.9e-6` pb, i.e. our number.
  Nothing on this side needed fixing, and no tolerance moved.
  Standing evidence: `validation/madgraph/gen_higgs_window.sh` +
  `validation/madgraph/higgs_window_reference.json` (committed), measured live by
  `validate_samples.rs` `the_higgs_pole_window_is_measured_against_madgraph`.
  **Decided (D3, 2026-08-01): the oracle layer moves to 3.7.1** — B4 banks with
  it, B5 re-banks the rest and re-measures every banked gate (note 27 §B4/§B5);
  this row's `integrals` cell becomes gateable for the first time then. Upstream
  fix identified: mg5amcnlo `286feb8e6` ("change sde_strategy2 to avoid negative
  weights", 2025-01-27), first released in 3.6.2, never backported to 3.5.x
  (3.5.16 still carries the bug); provenance detail in note 27 §B1.
- **`uux_to_uux` colour-flow frequencies — needs a per-diagram `AMP2`
  accumulator.** Every kinematic observable agrees (min KS p `6.7e-3` over three
  seeds) and so do the helicity frequencies, but the realised `ICOLUP` frequencies
  do not: MadGraph writes the flow whose lines join each incoming pair on
  **99.96%** of its events where we write it on **90.4%** (`∝ JAMP2` over every
  flow, which is the banked `|JAMP1|²/|JAMP2|² = 8.5…9.0`), χ² 1015 on one degree
  of freedom, stable across seeds. MadEvent's rule is now read end to end and
  reproduced (note 27 §B3.1): `SELECT_COLOR` masks `JAMP2` with the integration
  configuration's `ICOLAMP` row and keeps `∝ JAMP2` inside the mask. The table is
  implemented as `LeadingColorFlows` and matches MadGraph's own generated
  `coloramps.inc` row for row on `u u~ > u u~`, `g g > t t~` and `g g > g g`
  (`color_cf.rs::leading_color_flows_match_madgraphs_coloramps`), and the masked
  draw is `select_flow_reached_by`.
  What is missing is the *conditioning variable*. MadEvent's configuration is an
  amplitude share, `AMP2_j(x)/Σ AMP2(x)`; our sampling channel is a density share,
  `α_j g_j(x)/g(x)`. Conditioning on ours was implemented and measured: χ² 1015 →
  **7268**, our flow-1 share 90.4% → 51.0%. The reason is that our per-diagram
  channels for a massless-propagator process are the *same map* — worst pairwise
  relative density difference `0.000e0` over 2000 accepted points on both
  `u u~ > u u~` (2 channels) and `g g > g g` (4 channels), α frozen at uniform,
  per-channel σ 49.6/50.4 and 25/25/25/25 against MadGraph's 0.055/99.945. So the
  channel index carries no information about which diagram produced the point.
  Wanted: `AMP2_d`, the helicity-summed squared modulus of each diagram's coherent
  amplitude, as a second folded root beside `Op::Flows` (the per-diagram
  counterpart of `eval_jamp2`, accumulated only over diagrams that would carry a
  MadGraph config — no four-point vertex, per `get_amp2_lines`), then the
  per-event configuration drawn `∝ AMP2_d` and the `ICOLAMP` mask applied. That is
  sampler-independent and reproduces MadGraph's marginal by construction. `samples`
  cell informational until it lands. (`validate_samples.rs`, note 27 §B3.)
- **The per-diagram multichannel builds degenerate maps for massless-propagator
  processes** (exposed by the above, not chased). On `u u~ > u u~` the two
  `DiagramChannel` densities are bit-identical at every probed point, and on
  `g g > g g` all four are; the Kleiss–Pittau α-adaptation therefore never moves
  off uniform and the multichannel buys nothing over flat RAMBO for those rows.
  `g g > t t~`, whose t/u maps carry a `173 GeV` top pole, is not degenerate
  (worst pairwise density difference `0.84`, α converging to
  `[0.267, 0.364, 0.369]`). Both σ cells gate today, so this costs variance rather
  than correctness; it is the same "multi-rung spine" gap the `uux_to_uux`
  `integrals` note already names.
- ~~**`hadronic-shat-floor`**~~ — ✅ **closed** (`v3-backlog` B2). `Cuts::shat_min`
  now derives the two general bounds `setcuts.f` derives: `√ŝ ≥ Σᵢ pTᵢ^min` over
  the legs a single-leg cut holds above a threshold, and `√ŝ ≥ Σᵢ mᵢ` over the
  final-state masses. Both read off `√ŝ = Σᵢ Eᵢ` in the partonic centre of mass
  with `Eᵢ ≥ max(mᵢ, pTᵢ)`, so neither needs a back-to-back or two-body argument.
  `pp_to_bb_fixed` gets `shat_min = 1600 GeV²`, the value MadGraph's own `smin`
  takes for that card, and no other row moves (`ptl` still gives dy13 `(2·ptl)²`;
  llj's `mmll² = 2500` still dominates its `(2·ptl + ptj)² = 1600`). σ measured
  for the first time at **2 145 255 ± 961 pb** against MG
  **2 145 500 ± 3 414 pb** (rel −0.011%, pull −0.07, χ²/dof 0.51 over three
  seeds), flat across a 75k–1.2M budget ladder. `integrals` cell ⛔ → **GATE**;
  `samples` cell ⛔ → **info**, for the new finding below.
- **`pp_to_bb_fixed` colour-flow frequencies** — the `uux_to_uux` finding above on
  a second process, exposed by the first `samples` measurement this row could
  take. Everything else agrees: kinematics at min KS p `9.7e-3`, helicity
  frequencies at χ² p `0.57…0.78`, flavour-group frequencies at p `0.31…0.46`,
  over three seeds. The realised `ICOLUP` frequencies do not: χ² `23…31` on five
  degrees of freedom, p `1.0e-5…3.0e-4`, seed-stable. The excess is entirely in
  the two sub-percent flows — MadGraph writes `0.07%` and `0.08%` of its events
  there against our `0.23%` and `0.25%`, a factor `3.1…3.2` — while the two
  dominant flows agree to about a percent of themselves. Note 27 §B3.2 explains
  this shape too: the s-channel `g → b b̄` configuration admits *both* flows at
  leading colour, so only the t/u configurations discriminate — which is why the
  effect is 3× here against `uux_to_uux`'s 240×. The per-diagram `AMP2`
  accumulator (the `uux_to_uux` entry above) settles both rows at once, and this
  row is the sharper acceptance check for it: a mask that is merely *on*
  reproduces a 99.96% split, but only correct per-configuration weights
  reproduce a 3×. Not an
  integration defect: this row's σ agrees at `−0.01%` and the ŝ floor cannot reach
  the colour draw. `samples` cell informational.
  (`validate_samples_proton.rs`
  `generated_b_quark_events_agree_with_madgraphs_banked_ones`.)
- **Four llj partonic σ rows are unreachable, not merely ungated** — `uux_to_epemg`,
  `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` are banked with cross sections
  and cost seconds to integrate, so L3 was to promote them to GATE. They cannot
  run at all: all four run cards leave both scales free at
  `dynamical_scale_choice = -1`, and their topology — a t-channel propagator into
  a three-leg final state — is exactly the case whose cluster scale depends on
  the merge order, which `coupling::scales` refuses rather than approximates. No
  scale on this side reproduces MadGraph's number, and a fixed-scale re-run would
  be a different cross section. Their `integrals` cells are `blocked` on
  `kt-clustering` in the manifest and named in `validate_sigma`'s `plan_for`;
  their `samples` cells are blocked on the same blocker, and L4 measured the
  refusal in generation rather than assuming it.
  Fixed by `kt-clustering` (feature backlog), which grows four ready-to-flip σ
  rows on top of the six asserted-refused scale rows it already owns.
- **`uux_to_uux` residual bias** — hard σ GATE, but the five-seed mean is
  **~−0.30%** since per-channel grids (was ~−0.17% shared-grid) and does not
  shrink with budget. Sharper per-channel grids cover the spacelike collinear
  tail *less* — the region a single-rung t-channel spine under-resolves. Evidence
  for the multi-rung spine (feature backlog), not a new defect.
  (`validate_sigma.rs` `probe_qcd_seed_stability`.)
- **The mirror term's visibility is unmeasured below the electroweak scale** —
  `the_mirrored_beam_ordering_needs_the_reflected_matrix_element` is the control
  that makes the mirrored beam ordering's *identity* check meaningful: it asserts
  that evaluating the representative unreflected — what dropping the mirror
  amounts to — moves `|M|²` by more than `1e-3` at every probe point. Its ladder
  starts at 220 GeV. Extended down to `√ŝ = 25` GeV it finds `p p > l+ l- j`
  configurations where the mirror term is worth only `8.4e-4`, so the control
  fails there on the present bound. The identity itself is not in question — it
  holds to `5.4e-13` at every point measured — and no gate moved, but "a dropped
  mirror would be visible" is a claim currently supported only above 220 GeV.
  Wanted: measure how the weakest mirror term scales with `ŝ` and state the bound
  as a function of it, rather than widening the ladder and lowering the number.
  (`proton.rs` `fresh_points`.)

### Deferred coverage

- **V7 per-flavor diagram matching** — deferred from `validation-2`: Python
  extractor + Rust sorted-PDG matching + JSON regen, with a real-finding risk
  (whether vibegraph enumerates MG's exact concrete-subprocess union). Design
  preserved in note 19 §3 / §V7.
- ~~**`MG_VALIDATED_PROCESSES` is 14 of the gate's 18 process strings**~~ ✅
  **resolved.** All four `p p > l+ l- j` subprocess rows are in the list, so the
  library-level sweeps reach a coloured 2→3 amplitude. Re-verified on the extended
  list: rooting soundness **165 re-rootings, 0 failures** (was 133); op coverage
  unchanged, so `Hels` and `IdentityAmp` are still the only `KNOWN_UNCOVERED`
  entries and the new rows add no op; egglog round-trip and extraction identity
  both hold on them; the four default-suite sweeps (op coverage, binary
  add/mul, forward finiteness, lane-vs-scalar) stay inside the hermetic budget.
- **`IdentityAmp` process-level coverage** — the last `KNOWN_UNCOVERED` op; needs
  an `Identity` scalar bilinear the SM lacks, so it rides with `non-sm-ufo`
  (feature backlog).
- **Flavour-group probe coverage** — `derive_flavor_groups` partitions on sampled
  `|M|²`, which is complete but unsound whatever the probe set: two subprocesses
  differing only where the probe does not look are merged silently. The interim
  hardening is **done** — the ladder now runs a fifth of the base energy (20 GeV
  for a massless final state), the model's `Z` mass, and the three original rungs,
  each clamped above the final state's own threshold and collapsed where the clamp
  makes two coincide, so it reaches both below the electroweak scale and onto a
  resonance. `p p > l+ l- j` partitions identically under it (6 groups of 4), and
  the margin is measured rather than assumed: the closest pair of groups separates
  by **0.74** at points the partition was not fitted on, six orders above
  `GROUP_SEPARATION_MIN`, asserted to stay above 0.1. What remains is the
  replacement of the sampled criterion by the sound s-expression one (feature
  backlog). (`proton.rs`, note 24 §P2c.)
- **Pythia consumption gate — what it cannot see.** The gate reads both emitted
  samples n/n and its negative control proves it is not colour-blind, but four
  things stay outside it. (a) Only the `Buffer` strategy (`IDWTUP = -4`) is fed
  to Pythia; `StochasticRounding` (`+3`) writes a different `<init>` header and
  is unexercised. (b) The reconstruction check compares the *multiset of outgoing
  PDG codes* against the file, so a permuted or corrupted momentum would be
  consumed silently — comparing Pythia's `process` four-momenta against the
  record's would close that. (c) The negative control mutates `ICOLUP(1)` only,
  on one event, so an error confined to `ICOLUP(2)` or to the beam-side
  connectivity is not shown to be detectable. (d) Nothing checks Pythia's
  interpretation of `SCALUP`, `AQCDUP` or the `<init>` cross section — the file
  is proven *readable*, not proven to mean what we intended.
- ~~**Minor pinned discrepancies**~~ ✅ **resolved** (note 22 close-out). The
  `ee_to_wpwm` topology mask was a real error, and an inert one: `validate_scales`
  declared the transpose of what `coupling::topology` derives, and the derivation
  is the right one — the charged current pairs each beam with the `W` of its own
  charge, so `e⁺ → W⁺` exactly as Bhabha's `e⁺ → e⁺`. The declaration is
  corrected, the derivation is now asserted on this process too, and the per-event
  replay is unmoved (500 000 comparisons over 160 000 events), which is the
  measurement behind "the tie-break never reaches the scale". The
  `run_card_dy.dat` `fixed_ren_scale` half rested on a false premise about that
  file — see the hygiene row below.

### Gate + tooling hygiene

Small, independent, each one a gate that is weaker than it looks.

- ~~**One work-area `matrix1_orig.f` is hand-patched**~~ ✅ **resolved in the
  `refdata-2` re-cut.** The `COMMON/DBG_AMP/` block an old debugging session added
  to the `ee_to_mumu_tata_qcd0` subprocess is excised. Regenerating the process
  with `mg5_aMC` was tried first and is *not* the way to do this: a fresh
  `output` of the same generate line reproduces the file except for the order
  MadGraph emits its `FK_*` declarations in, which moves run to run, so
  regenerating would have introduced a gratuitous diff into the bundle for a
  three-line removal. The excised file is byte-identical to the fresh generation
  but for that permutation, and the `Events/` tree was not touched.
- ~~**`run_card_dy.dat` is a verbatim MG5 copy**~~ ✅ **resolved — the premise was
  wrong.** It is a hand-written fixture in MadGraph's run-card syntax, not
  a copy of any MadGraph file: the template it resembles is a Python `%(...)s`
  template, and the fixture's values are chosen to exercise the parser (`lhaid`
  230000, a free `μR` against a fixed `μF`, a cut block that stops short so the
  defaults have to fill it in). It is also what `include_str!` hands the *hermetic*
  parser test, so there was nothing to delete and no submodule read to put in its
  place. Renamed `run_card_parser_fixture.dat`, given a header saying what it is,
  and its one copied banner block replaced; the scales test that recorded its
  `fixed_ren_scale` as a disagreement now records it as the only committed card
  covering the free-scale branch.
- ~~**Silent soft-skip tests**~~ ✅ **resolved structurally.** Every test that
  needed the submodule, a fetched PDF set or a banked MadGraph run is registered
  behind `required-features` and absent from the default build, so the category
  "runs by default but quietly needs data" no longer exists: `cargo test` on a
  bare clone is complete with zero skips. The banked layer's remaining skips go
  through `vibegraph::validation::require`, which fails naming the input — the
  15-entry `EXPECTED_SKIPS` table it replaces was audited entry by entry and every
  one was dead, because the reference bundle carries every run the gates iterate
  over, the two hadronic reference σ are committed, and the PDF sets come from a
  fetch task that fails when it cannot acquire them. So the banked layer now takes
  **no** runtime skips at all, and CI's non-gating `banked` job fails naming the
  bundle instead of recording nine expected skips. The `ufo` and
  `ufo::sm` members moved to `tests/ufo.rs` and `tests/sm_interned_blob.rs`,
  where a missing submodule is now a failure, not a skip.
- ~~**Two reference files are still generated rather than committed**~~ ✅
  **resolved.** `validation/pdf/oracle{,_multigrid}.json` (51 + 71 KB) and
  `validation/helas/reference.{csv,npz}` (27 KB) are committed, both
  `EXPECTED_SKIPS` entries are deleted, and the two gates read the committed
  files. `validate_helas` needs nothing external any more and moved to the
  hermetic layer (0.25 s); `validate_pdf_grid`'s 12 tests are live and stay
  banked because the *other* side of the comparison is the two fetched PDF sets,
  which `pixi run validate` now acquires (`fetch-pdf-multigrid` joined its
  dependencies).
- ~~**Publish the `refdata-2` release asset and make the CI banked job
  gating**~~ ✅ **done.** The `refdata-2` release carries
  `vibegraph-refdata-2.tar.zst` (65 066 838 bytes, sha256 `4495d6df…f40e736c`,
  matching the manifest pin), `[refdata].published = true`, and the `banked`
  job fetches all three inputs and gates merges — `continue-on-error` is gone.
  The repository being private means the plain release URL serves 404 to
  unauthenticated clients; `vg_ensure_refdata` falls back to an authenticated
  `gh release download` of the same asset (`GITHUB_TOKEN` in CI), and the
  plain URL becomes live if the repository ever goes public.
- ~~**The reference bundle double-compresses its event files**~~ ✅ **taken in
  the `refdata-2` re-cut.** Event files travel as plain Les Houches text and
  `vg_ensure_refdata` gzips them back as it unpacks, so no consumer changed:
  **65 066 838 bytes against 90 597 923** while carrying one more run. The
  byte-for-byte round-trip gate keeps its meaning — it compares Les Houches text,
  gzip is lossless, and the archive now holds exactly the bytes it asserts on
  instead of a container around them. Side effect worth having: a work area
  unpacked from a bundle re-assembles to that same bundle, which an archive of
  gzipped members could not promise. Measured: all 26 runs' decompressed text
  unchanged sha256 for sha256 through pack and unpack.
- **`validation/madgraph/compact_events.py` has no consumer** — the projection
  L1b measured (note 26) is committed with its `lhe-compact` pixi environment so
  the verdict's numbers are reproducible, but nothing runs it: no gate reads its
  output and `generate-references` does not call it. The bundle *was* re-cut
  (`refdata-2`) and deliberately not around Parquet, so the "wire it in" branch
  has now been declined once; what remains is to delete it or to keep it as
  reproducible evidence for the verdict, and to say which. A committed generator
  that nothing exercises is exactly the shape `validate-pdf-grid` had while it
  covered nothing for four sessions.
- ~~**`g g > g g` diagram count: 6 against 4**~~ ✅ **decided (L7): report our
  count in our own convention, mark the cell informational.** MadGraph writes the
  four-gluon contact term as three graphs, one per colour structure
  (`VVVV1_0`/`VVVV3_0`/`VVVV4_0` into `AMP(1..3)`); we write one diagram whose
  vertex carries all three, so 3 exchange + 3 contact against 3 + 1. Re-splitting
  the enumeration to match a counting convention would change the thing being
  validated in order to make a number match, and the same process is pinned at
  8.25e-14 per flow — far below what any difference in diagram *content* could
  survive. The cell renders `⚠️ 4/6` with that reason attached.
- ~~**`validate_sigma` writes a note its own binning falsified**~~ — DONE (B1,
  2026-08-01): the `ee_to_mumu_tata_qcd0` `Plan::Info` reason and the module
  header now say what the windowed measurement established.
- **The `diagrams` gate could be hermetic** — it reads the committed
  `diagrams.json`, the committed `.mg5` scripts and nothing else, and runs in
  1.5 s including the two 2→6 enumerations, but it is registered
  `required-features = ["extended-validation"]` on the enumeration-cost argument.
  L7 made the manifest tiers say `banked` to match the registration rather than
  move the binary mid-close-out. Moving it would put the whole `diagrams` column
  on a bare clone for ~1.5 s of the 3-minute hermetic budget.
- **`init-sm-submodule` fails outside a git checkout** — the pixi task runs
  `git submodule update --init` unconditionally, where `vg_ensure_submodule` in
  `fetch_common.sh` checks for `models/sm/particles.py` first and is a no-op when
  the source is already there. Bites a `git archive` export (how the clean-tree
  gate is run), not CI. The task should call the shared function.
- **`Process`'s `Display` drops coupling-order constraints** — the report's
  measurement detail prints `p p > b b~` for a row whose generate line is
  `p p > b b~ QCD=2`. The enumeration honours the constraint (that row counts 6
  diagrams where the default-order row counts 4), so this is a printing loss
  only, but it makes two report lines read as if they measured the same process.
- **`diagrams.json` carries counts only, not the per-flavour union** — the
  committed reference is what the existing extractor produces, so the
  multi-channel `diagrams` cells assert a summed count and not the concrete
  subprocess list the manifest describes. Filling that in is the deferred V7
  design (note 19 §3/§V7) reaching `extract_diagrams.py`; until then the
  manifest's "includes the per-flavour concrete-subprocess union" notes describe
  the intent, not the current assertion.
- ~~**`clippy::approx_constant` deny at `coupling/alphas.rs:224`**~~ ✅
  **resolved.** The constant carries a targeted `#[allow]` with its
  MG-source-tracking rationale, and `cargo clippy --workspace --all-targets` is
  now **clean** — the 52 further warnings behind that error are fixed in place,
  except two lints allowed workspace-wide with the reason in `Cargo.toml`:
  `neg_cmp_op_on_partial_ord` (every site is the `!(x > 0.0)` guard that routes a
  NaN to the rejecting branch) and `unusual_byte_groupings` (hex seed words like
  `0x5EED_1`).
- **Weekly `schedule` trigger on `acceptance.yml`** — left off because it can only
  fail until a first release exists. Turn it on once one does: it is also the
  second detector for the "CERN repackages the PDF archive" risk, whose only
  other detector is an `#[ignore]`d test nobody runs on a timer. (Note 24 §U2.)

---

## 🧩 Feature backlog

- **`identical-particle-permutation`** — make the symmetry factor a property of
  the phase-space map. `dΦ_n` over-counts a final state with identical particles
  by `Π_s n_s!`; `dynamical-scales` added `final_state_symmetry_factor`
  (`hadronic.rs`) but as a per-integrand scalar — the wrong home. Two latent
  consequences, both smooth factor-of-`n!` σ errors: `FixedBeamIntegrand::new`
  derives the factor from `amps[0]` and applies it to every subprocess, but in
  `p p > j j` the factor differs between subprocesses whose mass lists agree
  (`gg→gg` needs 1/2, `qq̄→qq̄` needs 1, both `[0,0]`). `ProtonIntegrand`
  deliberately did not extend the `amps[0]` pattern — it *asserts* the factor is
  1 for every group member and would refuse a group where it is not, which is
  the right shape but only defers the question. The map knows its own
  outgoing multiset, so deriving it there makes every consumer right by
  construction, and settles whether multichannel treats permutations as distinct
  channels or one channel with the factor folded in. Pair with a gate process
  with a repeated outgoing particle — `g g → g g` is currently the only one,
  which is exactly why the factor of 2 survived.
- **Multi-rung t-channel spine** — ladder topologies (VBF/DIS, ≥2 spacelike
  lines). The ordering Jacobian cannot be pinned by `Vₙ`/σ in-session, so it was
  deferred rather than committed unvalidated; hand-off design written up
  (`Spine → rungs: Vec`, running `q_i = p_a − Σp`, note-07 §2.9.0 ordering firing
  test). Also where the `uux_to_uux` bias evidence points. (Note 21.)
- **`kt-clustering`** — general kT clustering for `dynamical_scale_choice = -1`
  (sprint sketch; also what MLM matching needs). 6 banked runs are asserted as
  refused; **hard prerequisite for gating any QCD process at MadGraph's default
  scale choice** — the no-strong-coupling short-circuit stops covering it the
  moment the matrix element carries `G`. (Multiplicity is not the barrier:
  `p p > l+ l- j` is gated at a *fixed* scale.) Note 22 §1.3 pins the degenerate
  closed-form cases;
  this sprint builds the general path. Sessions:
  1. **Design note** — read MG's `cluster.f`/`setscales.f`/`reweight.f` path
     end to end and pin the algorithm: the `djb`/kT measure, which merges are
     admissible (graph-guided — only vertices the process's diagrams contain),
     the tie-break order (including the `1 + 1e-6` beam-crossing inflation
     note 22 §1.3 caught in `uux_to_uux`), and how the cluster sequence maps
     to μR (the geometric-mean prescription) and per-beam μF.
  2. **Clustering engine** — diagram-guided kT clustering of an event's
     external momenta down to a 2→2 core, building on the `ClusterTopology`
     derivation from the `dynamical-scales` sprint.
  3. **Scale synthesis + wiring** — replace the closed-form-only `-1` branches
     in `ScaleChoice` with the general path; the degenerate cases become
     consistency checks (the general code must reproduce them exactly on the
     already-gated runs).
  4. **Gate** — flip the 6 asserted-refused rows in `validate_scales` to
     enforced per-event replays (`SCALUP`/`<rscale>`/`<pdfrwt>`), flip the four
     llj partonic σ rows (`uux_to_epemg`, `ddx_to_epemg`, `gu_to_epemu`,
     `gux_to_epemux`) from `blocked` to GATE — they are banked, cheap and
     waiting on nothing else — then re-gate σ(pp→ℓ⁺ℓ⁻j) against a
     *dynamical*-scale MG run: the fixed-scale row is already enforced, so the
     whole rest of that chain is held fixed and only the scale moves.
  (`coupling/scales.rs`, `validate_scales.rs`, note 22 §1.3/§5.)
- **s-expression program identity for flavour grouping** — a dedicated future
  sprint, user-scoped. Today's `derive_flavor_groups` partitions subprocesses by
  sampled `|M|²` agreement: **complete but unsound** — two programs that differ
  only where the probe does not look are merged, and the merge is silent.
  Replace it with a sound-but-conservative criterion: two subprocesses share a
  group iff their compiled programs are *identical as s-expressions*. Three
  prerequisites, in order:
  1. **Universal constant ids.** Compare UFO-stable coupling/particle
     identities, never per-compilation pool slot indices — flavour-dependent
     couplings can share a slot, so slot-index equality would be **unsound**,
     the exact failure the new criterion exists to remove.
  2. **Canonicalization of the un-optimized s-expression.** Lowering carries a
     ±1-CSE-node nondeterminism (note 15 §4–5) and diagram order is unstable
     (cf. `MG_DIAGRAM_ORDER`), so the comparison must run on a canonical form,
     before optimization, with a deterministic diagram ordering.
  3. **Colour folded into the s-expr language**, so the colour basis is part of
     the compared term rather than a side condition checked separately.
  Being conservative, it can only *split* groups that are genuinely equal —
  costing compiled programs, never correctness. Keep the sampled criterion as an
  independent cross-check when it lands: they should agree, and a disagreement
  is a finding. (`proton.rs`, note 24 §P2c.)
- **VEGAS first-iteration convergence bias** — `VegasGrid::adapt` feeds *every*
  iteration into `combine_iterations`' `1/σ²` weighted mean, including the first
  ones on an unadapted grid. An early iteration that undersamples the peak
  returns a low integral **and** a low variance, so it is weighted *up*. Measured
  on llj (five seeds each): −1.03% at 30k/iter, −0.28% at 150k, +0.002% at 300k,
  +0.16% at 600k — steps halving as the budget doubles, the `O(1/N)` signature.
  Not hadronic-specific; the same combination runs the fixed-beam path, and llj
  exposes it because 24 pooled channels × a 7-dim grid each buys every channel
  far fewer points than a partonic 2→2 does. Independent confirmation: the
  accept/reject pass is a *single* pass over frozen grids, does not go through
  `combine_iterations`, and converges to the true σ — at 100k the emitted
  sample's own σ sits +1.25% above the banked integral, and at 300k they agree.
  Fix: discard the first `k` iterations (or an unweighted final pass over the
  trained grids). Would let the llj gate run at a quarter of its budget.
  (`vegas.rs`, note 24 §P3.)
- **`w_max` scan budget decoupled from the integration budget** — a frozen scan
  estimates each channel's maximum on that channel's share of the *integration*
  budget, so it inherits the same undersampled small channels. On llj the share
  of σ above the maxima runs 3.2e-2 → 1.5e-2 → 8.4e-3 → 5.3e-3 over
  30k → 100k → 300k → 600k and is **still falling**, against 3.04e-4 for a
  fixed-beam process — 20–100× worse, and nowhere near converged at any budget
  the gate can afford. The estimator stays unbiased (overweights are kept at
  weight > 1), so what this costs is unweighting efficiency and sample
  lumpiness under `IDWTUP = +3`, not correctness. Wanted: a scan budget set
  independently of `neval`. Note the largest `w/w_max` moves non-monotonically
  (23.5, 9.4, 15.0, 11.1) because it is an extremum estimate — do not read it as
  a convergence measure. (`unweight`, note 24 §P4.)
- **Streaming `IDWTUP = -4`** by deterministic two-pass replay — the interface
  hook (`EventSource::restart`) is in place and contract-tested; not needed while
  100k-event runs buffer in ~42 MB. (Note 23 close-out.)
- **`μF ≥ 2 GeV` event veto** — `reweight.f:1185` *vetoes* the point below it;
  `coupling::scales` reports the scale only. Bites nothing today; a hadronic run
  with a dynamic μF reaching below 2 GeV will disagree with MG without it.
  (Note 22 §4 + close-out.)
- **Massless-t-channel fiducial cut** — a massless beam pins `t_max = 0`
  (collinear edge) where the t-map falls back to flat; whether a fiducial cut is
  wanted instead is unresolved for a physical massless-initial-state t-channel.
  (Note 21 close-out.)
- **`typed-units`** — research `uom`/`dimensioned`/`units` crates for typed
  four-momenta and cross sections.

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

The UFO surface is deliberately model-generic, but "generic" currently ends at
the SM's feature set. None of these block anything; collected so a future
BSM-model task scopes against a checklist instead of rediscovering each wall one
hard error at a time. A small dedicated test model (or a public BSM UFO) would be
the natural vehicle for several at once.

- **Color sextets and baryonic epsilons**: the color engine handles
  Singlet/Triplet/AntiTriplet/Octet only (`helas/repr/color.rs`); sextet tensors
  `K6`/`K6Bar`/`T6` (diquark models) and the baryon-number-violating
  `Epsilon`/`EpsilonBar` (e.g. RPV SUSY) are deliberate hard errors
  (`ufo/color.rs::SextetUnsupported`, `helas/color/tensor.rs`). Note the two
  distinct "6"s: NCOLOR=6 (flow-basis dimension) is fully supported; the sextet
  *representation* is not. MG's reference algebra lives in `color_algebra.py`;
  support means new `ColorTensor` atoms + trace-basis reduction rules + CF
  products, validated the color-flow way (CF oracle vs MG's DATA CF, then the
  JAMP-weighted |M|² gate).
- **Spin codes beyond {1, 2, 3}**: `helicity_states_for_spin` (`eval/compile.rs`)
  future-proofs the spin-2 helicity list (code 5), but nothing downstream builds
  tensor external wavefunctions or propagators; spin-3/2 (code 4) is an
  `UnsupportedSpin` error. Ghost codes stay irrelevant at LO.
- **Majorana fermions** (MSSM neutralinos, gluinos): fermion-flow handling
  assumes Dirac-continuous lines end to end — no flow-flip/charge-conjugation
  machinery. Classically subtle sign territory; the `color-flow` fermion-flow
  slot-swap bug shows how delicate the flow conventions are even pure-Dirac.
- **`IdentityAmp` process-level coverage**: needs an `Identity` scalar bilinear
  the SM lacks — a natural rider on whichever small test model lands first.
- **Loop-level UFOs** (`loop_sm`, NLO models): out of the LO charter (parser
  history in note 04).

---

## ⚡ Performance backlog

- **Per-flow α tuning — offline gain measurement first** (user, 2026-08-01;
  sequenced after B6, which provides the shares). Stratify the integrand by
  leading-colour share `s_i = |JAMP_i|²CF_ii / Σ_k |JAMP_k|²CF_kk` (positive,
  partition of unity, interference apportioned pro rata) and tune a separate
  channel-mixture α per stratum. **Stage 1 is a measurement, not a sampler**:
  the Kleiss–Pittau optimal α and its variance are computable offline from
  recorded `g_j(x)`, `f(x)` and `s_i(x)` on existing samples — report the
  achievable variance reduction against the ×(strata) evaluation overhead
  before building anything; a small number dies here like note 26's parquet.
  Known limits, measured this sprint: on `uux_to_uux`/`gg_to_gg` the channel
  maps are **bit-identical** (note 27 §B3.2), so per-flow α is a no-op there
  until the multi-rung spine differentiates the maps; flows overlap heavily, so
  the gain is the inter-stratum covariance term, expected modest. **Guardrail:
  split the tuning, never the coverage** — every stratum keeps every channel
  with an α floor, or the `sde_strategy`-class fragility (note 27 §B1) is
  rebuilt on our side.
- **Stratified-parallel integration axes** (user, 2026-08-01) — the iterative
  VEGAS+α loop needs an embarrassingly parallel axis for SIMD/multi-thread
  promotion. Catalogued, exact-first (no partition function, no fragility):
  (a) **channel-block stratification** — allocate `N_j = α_j·N` points per
  mixture component deterministically instead of drawing the label per event;
  unbiased, removes the multinomial label noise (a small free variance win),
  and each block is one map = one code path = SIMD-clean lanes with no branch
  divergence; (b) **helicity strata** — `Σ_hel |M_hel|²` is an exact orthogonal
  decomposition (no interference for unpolarized beams), so helicity classes
  (parity-folded, zero-classes dropped) can carry their own budgets/grids;
  first real consumer for `mg-single-helicity-bench`; (c) **flavour groups ×
  beam orderings** (hadronic) — already independent integrals, blocked only by
  the `RefCell` scratch (the DY-parallelism item below); (d) **frozen-pass
  bulk** — `sample_frozen` is already embarrassingly parallel; keep the
  sequential adapt phase short and put the budget in frozen passes (synergises
  with the VEGAS first-iteration item: discard/shorten adaptation, bulk-sample
  frozen); (e) **batch-size vs iteration-count** — measure whether α/grid
  adaptation converges in fewer sequential iterations with larger parallel
  batches per iteration (adaptation signal-to-noise grows ~√batch, so the
  sequential critical path should shrink until the update is
  quasi-deterministic; the measurement is a batch-size sweep at fixed total
  budget). Partition-based axes (per-diagram AMP2 shares à la MadEvent
  G-directories, per-diagram-class = per *distinct* map) are second tier:
  real cluster-scale precedent, but they carry the routing fragility and need
  the same coverage guardrail as the per-flow item above.
- **Compiled-program cache in the artifact** — designed in note 23, deliberately
  not built: compilation costs 0.05–0.29 s against ~13 s for a 20k-event
  `generate`. **Trigger:** setup climbing to a noticeable share of a generation
  run (richer diagram enumeration, or e-graph extraction joining compilation).
  Key `(model digest, process, compiler schema version)` is already derivable
  from banked fields — no schema bump needed. Three obstacles recorded in note 23
  (no serde in `helas::eval`; `folded_hel` is a lazy `OnceLock`, the expanded
  arena is the large part; `prune_zero_helicities`' kinematic contract must be
  rechecked on load).
- **Per-event scale hot-path cost** — ~100 ns/point on top of a 0.5–1.7 µs
  matrix element (+6% `gg_to_gg`, +21% `uux_to_uux`). `ScaleChoice::clustered`
  heap-allocates its beam–leg candidate list per event; that is the obvious
  first cut. (`coupling/scales.rs`; `validate_sigma.rs` `probe_scale_cost`.)
- **2→6 σ rows** — `uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0` stay
  `Plan::Skip`: ~1 ms/eval over a 24-dim map is too slow to gate — a cost issue,
  not a sampling one. (`validate_sigma.rs`.)
- **DY integrand parallelism** — `hadronic.rs`'s per-flavor-class scratch is
  `RefCell`-based `FnMut`, so the `Fn + Sync` parallel VEGAS paths can't be used;
  per-thread scratch unlocks both multi-threaded runs and distributed sharding
  (note 18 §2.4). Whole default-cut run is ~2 s single-threaded, so unhurried.
- **`feyngraph-perf`** — `AssignWorkspace::assign()` (`workspace.rs:L122`) calls
  itertools `.counts()` (a fresh `HashMap`) per candidate vertex per topology per
  subprocess — ~340M allocations for pp→qq̃4l. Fix: pre-compute per-vertex counts
  in `AssignWorkspace::new()`. Submodule change, dedicated session. Vibegraph-side
  mitigations already applied: topology caching per `(n_ext, n_loops)` and the
  charge-conservation pre-filter (~86% of candidates eliminated).
- **`egraph-rewrite`** (blocked) — remaining rule families are *sharing* rewrites
  invisible to tree-cost extraction; path to yes needs a global/ILP extractor +
  compute-aware `WorkCost` + a ≥3-consumer demo process. Substrate on `main`:
  egglog round-trip skeleton (`egraph.rs`, parked) + the DAG-cost extractor.
  (Notes 14, 15 §4–5; known ±1-CSE-node lowering nondeterminism noted there.)
- **`mg-single-helicity-bench`** — still no consumer. A6 verdict: the fair
  comparison needs an MG single-config timing, which means editing the generated
  Fortran driver + `gen_amplitude.py` and regenerating reference data. E2
  outcome: accept/reject selects helicity off the `eval_hel_m2` diagonal (one
  helicity-summed evaluation per accepted event), so single-helicity evaluation
  never became the hot path. Re-sequence under whatever first needs a single
  fixed helicity in a loop. (Note 23 §E2.)
- **Per-lane scales** — `eval_m2_lanes` can only batch points sharing one `αs`;
  a SIMD-batched dynamic-scale integrator would need the scaling fused into the
  constant loads. Nothing needs it today. (`helas/eval/rescale.rs`.)
- **`generate-stream` Part B** — lazy `generate_*` iterator (long-tail, from
  `cleanup-refactor`).
- **`Coeff(f64)` → `CoeffRat`** — optional cleanup now that `Op::CoeffRat` exists
  for color; the remaining `f64` leaves (Lorentz-structure and fermi-sign
  coefficients) could migrate. No consumer blocked. (Note 16 §5.)
