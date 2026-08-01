# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: `user-distribution` + `proton-events` (feature) ✅ closed +
merged 2026-07-31 — cards → `.lhe` for `p p > l+ l- j` from a clean environment,
release/CI/acceptance workflows in place, and the project dual-licensed
`MIT OR Apache-2.0`. **In progress: the validation pass** — `validation-3`,
planned in note 25 (three-layer suite delineation + per-process × per-category
report; shower consumption and event-sample statistics land as the `samples`
category). L0 has landed: `validation/manifest.toml` is the per-process source of
truth, `required-features` is the only mechanism deciding layer membership, and
`cargo test` on a bare clone is complete with zero skips. L1 has landed:
`pixi run generate-references` is one staged entry point over every generator and
reproduces every committed reference — the bundle archive included — byte-for-byte
from a populated work area; `validation/fetch_common.sh` is the one place that may
download; the diagram counts, the HELAS grid and the two LHAPDF oracles are now
committed; and the banked layer runs off a fetched `vibegraph-refdata-1.tar.zst`
on a machine with no MadGraph at all. `pixi run validate` is the banked layer;
`validate-deep` / `generate-references` are the oracle layer. L1b measured the
compact in-repo alternative to that bundle and **rejected it**: projecting the
banked events onto the fields the gates read is exact, but bottoms out at 27.5 MB
against a 5–10 MB target, so the fetched bundle stands and no reader changed
(note 26). L2 has landed: the `amplitudes` category is
hermetic for all 19 rows — one committed table per process (|M|² at every point,
per-diagram `AMP()` and per-flow `JAMP()` per helicity at six of them), evaluated
at MadGraph's own banked events projected exactly on shell as well as at the fixed
grid, gated by a single `amplitude_oracle` binary in 1.1 s. L3 has landed:
every hadronic cross section now runs through the general `ProtonIntegrand` —
the bespoke `DrellYanIntegrand` is deleted, `p p > e+ e-` gates through the
general path on both dy13 cards (pull +0.25 / −1.35 over three seeds), and DY is
generatable as a side effect; the integration artifact carries a per-channel
subsampler summary (fv5); and every `integrals` gate writes its measurement to
`target/validation-report/integrals/<row>.json` for the L7 collator. L4 has
landed: the `samples` category compares our unweighted events against MadGraph's
banked ones column by column — weighted-ECDF Kolmogorov–Smirnov on the named
kinematic observables and χ² homogeneity on `SPINUP`, `ICOLUP` and the flavour
assignment — over twelve fixed-beam rows in the library and `p p > l+ l- j`
through the shipped binary, three generation seeds each at 20 000 events against
MadGraph's 10 000, at a p-floor of `1e-4`. Eleven rows gate; two are
informational with their measurement recorded (`uux_to_uux` colour-flow
frequencies, `ee_to_mumu_tata_qcd0`), and the `pp_to_bb_fixed` rider banked a
purely QCD-initiated multi-channel run whose cross section the general path
cannot yet reach. L5 has
landed: `pixi run -e pythia validate-pythia` generates the llj and dy13 samples
from their own cards and reads every event back through Pythia 8.312 —
**2000/2000 consumed on both**, with a colour-mutation negative control that
Pythia rejects (`ProcessLevel::checkColours: unphysical colour flow`) so the
count is not consistent with a colour-blind reader. It is a standalone banked
gate in its own pixi environment, so `pixi run validate` still needs no Pythia.
L6 has cleared the sprint's
hygiene riders: `cargo clippy --workspace --all-targets` is clean, the
library-level sweeps reach the coloured 2→3 amplitudes (rooting soundness 165
re-rootings, 0 failures), the flavour-grouping probe ladder reaches below the
electroweak scale and onto the `Z` pole, and the banked layer's tolerated-skip
table is gone — every one of its 15 entries was dead, so a missing input now
fails naming itself.
Unrun until the user pushes a first tag: `release.yml` and `acceptance.yml`.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. Deferred: multi-rung t-channel ladders (note 21) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 11 partonic GATE rows incl. the 3 QCD 2→2s, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.284 ± 0.537** vs MG 933.110 ± 0.447; **643.765 ± 0.367** vs 644.420 ± 0.315), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **423.048 ± 0.248 pb** over three seeds vs MG 422.840 ± 1.805 (Δ = 0.11σ). Deferred: `dynamical_scale_choice = -1` (needs `kt-clustering`), which also blocks the four llj partonic σ rows |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) + colour-flow (`∝ JAMP2`) selection with the flow→`ICOLUP` dictionary checked against MG's `leshouche.inc` (30/30 subprocesses), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 25 banked MG `.lhe.gz` byte-for-byte (248 747 events). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Deferred: event-sample-vs-MG statistics |

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

---

## 🔎 Validation backlog

### Next validation pass — natural content

Both rows below now have **two** waiting processes, not one: fixed-energy
`e+ e- > mu+ mu-` and hadronic `p p > l+ l- j`. llj is the more informative
subject — coloured initial state, three-body final state, a jet cut, and colour
lines an `e+ e-` sample does not have.

- ~~**Downstream-shower validation of the emitted `.lhe` (Pythia via pixi)**~~ —
  **closed**: `pixi run -e pythia validate-pythia` reads both emitted samples end
  to end through Pythia 8.312 (`pythia-consumption` in `validation/manifest.toml`).
  What it does *not* yet cover is filed under **Pythia consumption gate — what it
  cannot see** in *Deferred coverage*.
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

- **`higgs-pole-in-m-tautau`** (replaces `low-mll-reconciliation`, whose premise
  L4 falsified) — `ee_to_mumu_tata_qcd0` sits **+2.2% above** banked MG, and
  binning `dσ/dm_ll` against MadGraph's own events down to threshold says the
  offset is **not** a low-`m_ll` effect: every bin below 20 GeV agrees within its
  errors on both pairs, and **159% of the offset sits in one 200 MeV bin at the
  `h → τ⁺τ⁻` pole** — `7.137e-5 ± 1.3e-6` pb against `2.260e-5 ± 1.7e-6` pb, a
  factor 3.16 at 22σ around a resonance 6.4 MeV wide. The rest of the spectrum
  sits ~1.4% *below* MadGraph. Which side mis-covers the resonance is open: the
  third estimate (flat RAMBO under a VEGAS grid) puts `1.0e-6` pb there, twenty
  times under MadGraph, because a map with no Breit–Wigner cannot find that peak
  at all, so it shows only the direction a poor map fails in; MadGraph's
  per-channel `results.dat` and its 10 000 banked events agree with each other, so
  its sample is not merely under-representing its own integral. The ratio 3.158 is
  within errors of **π**, which in a Breit–Wigner map (`∫ds/((s−m²)²+m²Γ²) =
  π/(mΓ)`) is the first thing to check on this side. Decisive next step: a
  dedicated MadGraph run of this process with an `m(τ⁺τ⁻)` window around 125 GeV,
  which measures the resonance directly on both sides. `integrals` and `samples`
  cells both informational.
  (`validate_samples.rs` `the_low_m_ll_region_is_binned_against_madgraph`.)
- **`uux_to_uux` colour-flow frequencies** — every kinematic observable agrees
  (min KS p `6.7e-3` over three seeds) and so do the helicity frequencies, but the
  realised `ICOLUP` frequencies do not: MadGraph writes the flow whose lines join
  each incoming pair on **99.96%** of its events where we write it on **90.4%**,
  χ² 1015 on one degree of freedom, stable across seeds. The banked per-flow
  JAMPs give `|JAMP1|²/|JAMP2|² = 8.5…9.0` at MadGraph's own points — which is our
  90/10 split, and is not MadGraph's — so the two sides are not applying the same
  colour-selection rule to the same numbers. Ours is `∝ JAMP2` (MadGraph's
  documented `SELECT_COLOR`); the candidate explanation is that MadEvent's
  selection is conditioned on the integration channel's own diagram
  (`ICOLAMP`), which for a t-channel-dominated process leaves one flow. Neither
  is wrong as an LO colour assignment, but they differ at order `1/N²` and the
  shower is handed the difference. `samples` cell informational until it is
  settled. (`validate_samples.rs`.)
- **`hadronic-shat-floor`** — the general hadronic path cannot integrate a process
  with no leptons in the final state. Every lower bound on `ŝ` the cut layer
  derives is a lepton bound (`dsqrt_shat`, the same-flavour dilepton mass cut, the
  back-to-back `2·ptl` bound for two leptons), so `p p > b b~` leaves
  `shat_min = 0`, the `(τ, y)` map's `ln(1/τ_min)` is infinite, and the first
  parton-density call is asked for `x = NaN`. The missing bound is the one the
  lepton branch already makes — two back-to-back b quarks each above `ptb` give
  `m_bb ≥ 2·ptb` — plus `ŝ ≥ (2·m_b)²` from the final-state masses. Blocks the
  `integrals` and `samples` cells of `pp_to_bb_fixed`, whose MadGraph run is
  banked and whose diagram row already gates.
  (`validate_hadronic.rs` `bb_fixed_has_no_shat_floor_for_the_general_path`.)
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

- **One work-area `matrix1_orig.f` is hand-patched** — the
  `ee_to_mumu_tata_qcd0` subprocess carries a `COMMON/DBG_AMP/` block added by an
  old debugging session, so that file is not what MadGraph would write. The probe
  build now detects an existing block instead of adding a second one (a duplicate
  `COMMON` member makes f2py emit uncompilable C), but the work area is the
  bundle's source, so regenerating that process directory is owed before the next
  `refdata` bump.
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
- **Publish the `refdata-1` release asset and make the CI banked job gating** —
  user step. `validation/madgraph/assemble_bundle.sh` builds
  `vibegraph-refdata-1.tar.zst` (1736 files, 90 597 923 bytes, sha256
  `1afeadfa…cc447e50`, pinned in `validation/manifest.toml`) reproducibly from
  the work area, and `validation/fetch_common.sh` fetches and verifies it; the
  URL it points at is a `refdata-1` tag that does not exist yet, so the path is
  exercised through `$VIBEGRAPH_REFDATA_SOURCE` meanwhile. Tag + upload the
  asset, flip `[refdata].published`, then drop `continue-on-error` from
  `ci.yml`'s `banked` job and add `pixi run fetch-refdata` to its fetch step —
  the reason it is non-gating is exactly that a fresh runner could not obtain the
  MadGraph runs. L1b confirmed the asset is still the right shape: the compact
  in-repo alternative was measured and rejected (note 26), so nothing about this
  item's contents changes. Consider folding the recompression item below into the
  same re-cut if the archive is rebuilt before publication.
- **The reference bundle double-compresses its event files** — measured by L1b,
  not taken. `assemble_bundle.sh` tars 25 already-gzipped `.lhe.gz` and runs
  zstd-19 over the result, which cannot compress them further. Carrying the same
  events as plain `.lhe` text under the same zstd-19 gives **58 629 865 bytes
  against ~90 100 000**, a 35% smaller fetch with no fidelity loss and no change
  to what any gate reads: the unpack step re-gzips, or the four consumers
  (`validate_lhef`, `validate_alphas`, `validate_scales`, `cli_generate_proton`)
  read `.lhe` directly. Costs a new archive and a new `[refdata]` pin, which is
  why it waits for the next bundle re-cut rather than being done for its own sake.
- **`validation/madgraph/compact_events.py` has no consumer** — the projection
  L1b measured (note 26) is committed with its `lhe-compact` pixi environment so
  the verdict's numbers are reproducible, but nothing runs it: no gate reads its
  output and `generate-references` does not call it. Either wire it in if the
  bundle is ever re-cut around Parquet, or delete it — a committed generator that
  nothing exercises is exactly the shape `validate-pdf-grid` had while it covered
  nothing for four sessions.
- **`g g > g g` diagram count: 6 against 4** — exposed by L1, informational, not
  chased. MadGraph writes the four-gluon contact term as three graphs, one per
  colour structure (`VVVV1_0`/`VVVV3_0`/`VVVV4_0` into `AMP(1..3)`); we write one
  diagram whose vertex carries all three. So 3 exchange + 3 contact against
  3 + 1. The physics is pinned far below a count — the per-flow amplitude gate on
  this process agrees with MadGraph at 8.25e-14 — so the question is only whether
  our count should be reported in MadGraph's per-structure convention. Decide
  that with the report driver, not by changing enumeration.
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
