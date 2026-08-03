# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: the **`kt-spine` feature sprint** ✅ **closed** 2026-08-02.
General kT clustering, the multi-rung t-channel spine and the per-subprocess
identical-particle factor are in production, and `p p > j j` is gated at
MadGraph's own default dynamical scale. `refdata-4` is published and pinned.
Census over the 29-row × 4-category report: **87 measured / 85 ✅ / 2 ⚠️**.
Full record in note 28 §Z; the standing discrepancies it left are below.
Standing caveat: a partonic σ quoted from `refdata-2` is **not comparable** to
one from `refdata-3`/`refdata-4` (MadGraph 3.5.7 applied the PDF set's
`αs(M_Z) = 0.130` to `lpp = 0` runs; 3.7.1 keeps the model's `0.118` — note 27
§B5).

**Scope decision (user, 2026-08-02)**: the release goal is restricted to
**arbitrary fixed-order Standard Model processes** over unpolarized
proton–proton or fixed-energy partonic beams, without decay-chain syntax.
Every extension beyond that — BSM UFO support, other beam configurations,
polarization, decay chains — is explicitly descoped to the feature backlog
(see "Descoped from v1" below), and every descoped surface a card can still
reach must be a **hard error**, never a silent acceptance; the fixes closing
the remaining silent acceptances are validation-sprint items.

**Awaiting the user**: `main` is pushed; the first release tag is **`v0.1`**
(decided 2026-08-02) — a 0.x line because no global backwards-compatibility
promise is made yet; a future "quality sprint" tightening the `pub` API
surface (backlog below) precedes any 1.0. Tagging runs `release.yml` and
`acceptance.yml` for the first time.

**Next sprints** — one more round before the first release tag:
1. **Validation sprint** over the restricted scope: the sprint slate below
   (standing correctness items + the hard-error closures). **Planned as note
   29** — design→implement→review chains: A conjugate colour tags, B
   `AMP2_c` scale-channel draw, C1/C2 hard errors + card audit, D
   `ee_to_mumua` ownership, E ForcePositive + hygiene, F the research-only
   U(1)-charge-flow phase sidecar; plus the decided `nn23lo1` re-bank (§G:
   `refdata-5` **replaces** the four superseded runs, bundle size flat).
2. **Performance sprint**: the integration-focused pass (VEGAS
   first-iteration bias + `w_max` scan decoupling + stratified-parallel axes,
   performance backlog below). `kt-spine` froze the channel/map structure it
   measures against, which was its precondition.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. The multi-rung t-channel spine and the per-subprocess identical-particle factor are in production (`kt-spine` Track S, note 28) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 17 partonic GATE rows incl. the 3 QCD 2→2s, `pp_to_bb_fixed` and all 4 llj subprocesses at the kT-clustered per-event scale, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.284 ± 0.537** vs MG 933.110 ± 0.447; **643.765 ± 0.367** vs 644.420 ± 0.315), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **423.048 ± 0.248 pb** over three seeds vs MG 422.840 ± 1.805 (Δ = 0.11σ). At a *dynamical* scale each point is clustered in the channel its own sampling channel names, per flavour group: `gu_to_epemu` **+1.07%** / `gux_to_epemux` **+0.97%** and σ(pp→ℓ⁺ℓ⁻j) **−0.68%**, all GATE at tolerances set by the channel-partition ambiguity (backlog below). The `p p > j j` capstone runs the same path on the canonical QCD process and is **GATE**: **6.803009e8 ± 2.511e5 pb** over three seeds vs MG 6.788500e8 ± 1.4726e6, rel **+0.21%** at pull **+0.97**, at `rel_tol` 0.005 — the reference's own 0.22% with headroom, pull asserted, since its channel-partition ambiguity is only `1.0e-3` (its own Monte-Carlo error, because a 2 → 2 gives the clustering no merge to choose). It sums over MadGraph's own 65 concrete assignments, pinned entry for entry against the run's `leshouche.inc` |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) selection, colour selection via MadEvent's `SELECT_COLOR` rule (configuration `∝ AMP2_d`, flow `∝ JAMP2` inside its `ICOLAMP` row) with the flow→`ICOLUP` dictionary checked against MG's `leshouche.inc` (30/30 subprocesses), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 37 banked MG runs byte-for-byte (744 759 events, both of MadGraph's serialisation dialects, source-text pass-through by construction). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

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
- **`v3-backlog`** (validation follow-up, closed + merged 2026-08-01) — every register finding resolved rather than tolerated: the h→ττ pole was **MadGraph 3.5.7's `get_channel_cut` defect** (`(t-Mass)*(t+Mass)` on `t = p²`; upstream fix `286feb8e6`, first in 3.6.2) and both cells now GATE against 3.7.1; the colour draw reproduces MadEvent's `SELECT_COLOR` via per-diagram `AMP2_d` (both χ² targets hit); `Cuts::shat_min` derives `setcuts.f`'s general bounds (`pp_to_bb_fixed` σ GATE); DY events banked with a live `dσ/dm_ll` gate; references re-banked on **3.7.1** into `refdata-3` (finding: 3.5.7 ran every `lpp = 0` process at `αs(M_Z) = 0.130`, so refdata-2/3 partonic σ are not comparable); the LHE writer round-trips **both** MG serialisation dialects by construction (34/34 byte-for-byte, 14/34 still reproduced with source dropped); latent `IDWTUP = -3` σ-misread fixed en route. Census 72/68/4 → **75/74/1**; note 27.
- **`kt-spine`** (feature, two tracks, closed 2026-08-02) — Track K: MadGraph's general kT clustering reproduced merge for merge against an instrumented 3.7.1 (90 000 dumped events, zero observed deviation), the closed forms deleted so `dynamical_scale_choice = -1` takes one path, `GridAlphaS` made LHAPDF's own `AlphaS_Ipol` and the density grid continued past its edges — then the flips: 6 asserted-refused scale rows became per-event replays, the 4 llj partonic σ rows and their `samples` cells left `blocked`, σ(pp→ℓ⁺ℓ⁻j) re-gated at the dynamical scale, and the capstone **`p p > j j`** gated on MadGraph's shipped run-card defaults (**6.803009e8 ± 2.511e5 pb** vs MG 6.788500e8 ± 1.4726e6, rel +0.21%, pull +0.97). Track S: the identical-particle factor moved into the phase-space map per subprocess, and the multi-rung t-channel spine landed in production. Two bugs the sprint found rather than assumed: the **fixed-beam path was never regulated** (every prior "what is the spine worth" measurement was taken on flat transfer draws), and `p p > j j`'s σ was **36% high** because a repeated final-state label enumerated `g u > g u` and `g u > u g` as two subprocesses. Transferable lesson: **a per-event field is a finer oracle than a cross section, and it exists more often than it looks** — the clustering was pinned by an instrumented replay of MadGraph's own intermediates long before any σ moved, which is why every σ flip that followed had a diagnosis attached. Census 75/74/1 → **87/85/2** over 29 rows; note 28.

---

## 🔎 Validation backlog

### Validation-sprint slate (restricted scope, decided 2026-08-02)

The next validation sprint hardens the restricted scope: resolve the standing
discrepancies, and close every place a card can reach outside the scope without
hitting a hard error. The slate:

1. **Conjugate-rep colour-flow tags** — the standing discrepancy below; the
   only thing between `p p > j j` and a gated event sample.
2. **Per-point `AMP2_c` scale-channel draw** — the fix the channel-partition
   discrepancy below names; removes the partition-ambiguity tolerances
   (0.015–0.02) on the dynamical-scale σ rows.
3. **`ee_to_mumua` windowed `pt(a)` comparison** — the measurement that
   decides which side owns the 3.7.1 drift (standing discrepancy below).
4. **Hard-error closures** — every one of these is a card surface that today
   is accepted and silently ignored, against the project's hard-error rule:
   - **Nonzero `polbeam1`/`polbeam2`**: parsed as known run-card fields
     (`runcard.rs` defaults) and read by nothing — a card asking for polarized
     beams runs unpolarized. Reject with an explicit unsupported-polarization
     error. (Beam configurations other than `lpp` (0,0)/(1,1) already
     hard-error via `UnsupportedLpp` — nothing to do there.)
   - **Decay-chain syntax** (`generate p p > t t~, t > w+ b`): the comma
     survives tokenization, so the chain is misparsed as required-s-channel
     syntax and dies with a misleading unknown-particle error
     (`diagrams/parse.rs::parse_process_body`). Detect the comma and reject
     with an explicit decay-chains-unsupported error.
   - **`propagators.py` in a UFO directory**: the loader reads exactly
     particles/lorentz/couplings/parameters/vertices (+ optional
     `coupling_orders.py`, `ufo/mod.rs`); a UFO 2.0 model defining custom
     propagators would silently get default propagators. Presence of the file
     must be a hard error until it is implemented.
   - **Run-card ignored-field audit**: enumerate every field the parser
     accepts (typo protection means unknown keys already error) but nothing
     consumes, classify each as physics-relevant or not, and hard-error on the
     physics-relevant ones when set away from their default. `polbeam` is the
     known instance; the audit is what proves it is the only one.
5. **`μF ≥ 2 GeV` event veto** (moved from the feature backlog) —
   `reweight.f:1185` *vetoes* the point below 2 GeV; `coupling::scales`
   reports the scale only. Bites nothing gated today, but within the restricted
   scope a hadronic run whose dynamic μF dips below 2 GeV silently disagrees
   with MG — implement the veto (or hard-error until it exists). (Note 22 §4.)
6. **`ForcePositive`** — the ~5-line implementation plus the
   `FORCE_POSITIVE_FLOOR` screen re-read (Gate + tooling hygiene below); an
   arbitrary-LHAPDF-set run card is in scope, and NNPDF31 is where it bites.
7. **`nn23lo1` decision** — re-bank the four blocked runs at `lhaid = 247000`
   vs implement MG's internal parameterisation (Gate + tooling hygiene below);
   a decision item for the sprint, not necessarily an implementation.
8. **Gate + tooling hygiene as budget allows** — priority order:
   `validate_kt_cluster`'s silent skip becomes a declared tier; the
   `release-debug` `#[should_panic]` contract tests; the three `uncovered`
   cells the `kt-spine` runs earned (Deferred coverage below).

### Standing discrepancies to resolve (never a loosened tolerance)

- **A flavour group's colour flows are the representative's, and are reused for
  members whose legs carry conjugate colour reps** — the defect that came out from
  behind the enumeration surplus once the surplus was gone, and the only thing
  between `p p > j j` and a gated event sample. `SubprocessRecord::relabelled`
  (`lhef/build.rs`) carries a group's `ColorFlowTags` from the representative
  subprocess to every member, on the stated premise that "the flavours sharing an
  amplitude are the ones whose legs carry the same masses". `u` and `u~` share an
  amplitude and a mass list and carry **conjugate SU(3) reps**, so their `ICOLUP`
  slots must be swapped, and they are not: `color_flow_tags` derives each flow's
  slots from the leg's own rep and checks them, so tags legal for an antiquark leg
  provably are not what was applied. Measured on the record alone, with no
  reference in it: of 20 000 generated `p p > j j` events, **2309** carry an
  antiquark whose colour line sits in `ICOLUP(1)` where Les Houches puts it in
  `ICOLUP(2)` — **4758 of 80 000 legs**, every one an antiquark, always both
  antiquark legs of the same event. The `samples` `ICOLUP` χ² reads it as
  `≈2470 / 25 dof` at `p 0` on every seed while every other column of that cell
  clears the `1e-4` floor. **It reaches no gated row**: `p p > l+ l- j`
  **0/100 000** legs, `p p > b b~` **0/80 000**, MadGraph's own banked `pp_to_jj`
  **0/40 000**. `p p > j j` is the first row whose groups mix the two — `g q > g q`
  and `g q~ > g q~` share a pointwise `|M|²`, mass list, cut filter and colour
  basis. **Why the net did not have it**: `color_flow_tags_oracle` checks the
  derived table against `leshouche.inc` for the **first** subprocess of each
  `SubProcesses/P*` directory — the same representative whose tags are then reused
  — so it validates exactly the member that is right and never a conjugate one.
  **Fix**: conjugate the tags when a member's leg reps are the representative's
  conjugates (or derive them per member), and widen the oracle past each
  directory's first subprocess so the repair is gated rather than asserted.
  (note 28 §C2.5.)
- **σ at a channel-dependent scale is only defined up to the channel partition** —
  the residual `kt-spine` K6 left, measured rather than tolerated. Once the scale
  reads the integration channel, the channel-split estimator
  `σ = Σⱼ ∫ dΦ f(p, j)·αⱼgⱼ/g` stops being independent of `αⱼ`: the selection
  weights decide *which scale* a region of phase space is evaluated at, not only
  how often it is visited. Integrating the same row at the converged α and at
  uniform α (`validate_sigma.rs` `probe_channel_partition_moves_sigma`) moves
  `gu_to_epemu` by **−1.48e-2** and `gux_to_epemux` by **−1.53e-2** against a
  Monte-Carlo error of `1.6e-3`, while it moves `uux_to_epemg` and `ddx_to_epemg`
  by `+1.0e-3` / `+1.9e-3` — their own noise, since their `μR` does not depend on
  the channel at all. **MadGraph's σ lies inside the interval the two partitions
  span** (`+1.08e-2` adapted, `−4.24e-3` uniform on `gu_to_epemu`), and MadEvent
  partitions by a third rule: single-diagram enhancement weights channel `c` by
  `AMP2_c/Σ AMP2`, a function of the point rather than a constant. That is why
  the two rows gate at `rel_tol` 0.02 and σ(pp→ℓ⁺ℓ⁻j) at 0.015 — the algorithm's
  own ambiguity, not the reference's error. **Fix**: draw the scale's channel
  `∝ AMP2_c(p)` per point instead of taking the phase-space channel, which
  reproduces MadEvent's own `iconfig` distribution and stops the scale riding on
  this crate's sampler. A second per-point draw inside the integrand, with
  reproducibility and artifact consequences, so it is a design decision rather
  than an improvisation. It does **not** set `p p > j j`'s tolerance: measured
  there, the gap is `+1.03e-3` against its own `9.6e-4` Monte Carlo, because a
  `2 → 2` final state gives the clustering no merge to choose, so that row gates
  at the reference's own error instead. (note 28 §K6.5/§K6.8/§C.3.)
- **`ee_to_mumua` drifted when the references moved to 3.7.1** — the one row
  where 3.7.1 disagrees with us *more* than 3.5.7 did, and the widest σ row in
  the set. Our σ is `1.006000e-1 ± 1.665e-4` pb (it was `1.007660e-1` before
  `kt-spine` S4 regulated the fixed-beam transfer draw, which moved it toward
  MadGraph and shrank its error); MadGraph's moved `1.00630e-1 ± 3.865e-4`
  → `9.980100e-2 ± 2.335e-4` pb, so the `integrals` pull reads **+2.79**
  (rel **+0.80%**) against a `PULL_LIMIT` of 3.5, and about half of that
  growth is the tighter reference error rather than the −0.83% shift itself. Its
  `samples` cell followed: minimum KS p **2.14e-3 → 2.74e-4** against a `1e-4`
  floor, worst observable `y(a)` → `pt(a)`. Both cells still gate, and both are
  now the tightest in their category. The photon here is soft/collinear-regulated
  by the run card's cuts, which is the region MadGraph's channel-weight change
  reallocates; whether the remaining 1% is that or ours is not established.
  Wanted: a windowed comparison over `pt(a)` of the kind note 27 §B1 used on the
  Higgs pole, which is the measurement that decides which side owns it.

### Deferred coverage

- **Three `uncovered` cells the `kt-spine` runs earned and nobody wrote** — each
  is unwritten rather than refused, and the run that would feed it is banked.
  (a) `ud_to_epemud_qcd0` `samples`: the spine reference row banks an unweighted
  event file and its σ now gates, so nothing blocks the comparison. (b)
  `pp_to_llj_dyn` `samples`: `validate_samples_proton` integrates and generates
  the *fixed*-scale llj card only, so the dynamical one needs its own artifact
  and generation pass — worth taking together with whatever closes the
  integration-channel deficit that cell's `integrals` note records. (c)
  `ud_to_epemud_qcd0` `diagrams`: the run's counts are not in `diagrams.json`,
  and enumerated against them this side finds 35 topologies to MadGraph's 35, so
  banking the counts makes it a passing hermetic gate with no allowance needed.
  (Note 28 §S6, §K5b.6.)
- **V7 per-flavor diagram matching** — deferred from `validation-2`: Python
  extractor + Rust sorted-PDG matching + JSON regen, with a real-finding risk
  (whether vibegraph enumerates MG's exact concrete-subprocess union). Design
  preserved in note 19 §3 / §V7.
- **`diagrams.json` carries counts only, not the per-flavour union** — the
  committed reference is what the existing extractor produces, so the
  multi-channel `diagrams` cells assert a summed count and not the concrete
  subprocess list the manifest describes. Filling that in is the deferred V7
  design (above) reaching `extract_diagrams.py`; until then the manifest's
  "includes the per-flavour concrete-subprocess union" notes describe the
  intent, not the current assertion.
- **`IdentityAmp` process-level coverage** — the last `KNOWN_UNCOVERED` op; needs
  an `Identity` scalar bilinear the SM lacks, so it rides with `non-sm-ufo`
  (feature backlog).
- **Flavour-group probe coverage** — `derive_flavor_groups` partitions on sampled
  `|M|²`, which is complete but unsound whatever the probe set: two subprocesses
  differing only where the probe does not look are merged silently. The probe
  ladder is hardened (five rungs down to a fifth of the base energy and onto the
  `Z` mass, closest-pair separation measured at **0.74**, asserted > 0.1); the
  sound replacement is the s-expression criterion (feature backlog). Accepted
  for v1 on the MG-helicity-filtering precedent (see the feature-backlog entry).
  (`proton.rs`, note 24 §P2c.)
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

### Gate + tooling hygiene

- **Weekly `schedule` trigger on `acceptance.yml`** — left off because it can only
  fail until a first release exists. Turn it on once one does: it is also the
  second detector for the "CERN repackages the PDF archive" risk, whose only
  other detector is an `#[ignore]`d test nobody runs on a timer. (Note 24 §U2.)
- **`validate_kt_cluster` is a banked gate the manifest does not know about, and
  it skips in silence** — the finest oracle the sprint built (all 90 000 dumped
  events, every candidate pair and merge against an instrumented MadGraph) has no
  `[[standalone]]` row, and `the_clustering_engine_reproduces_madgraphs_own`
  returns early with a `println!` when `output/ktdump/dumps/` is absent. The
  dumps are **75 MB** and deliberately not bundled
  (`validation/madgraph/README.md` §"The kT-clustering dumps": work-area sized,
  only their checksums committed), so on every fetching checkout — CI's `banked`
  job included — that gate is green and vacuous, which is exactly the failure
  mode `validation-3`'s lesson names. Two ways out, and choosing is a size
  decision: bundle the dumps (`refdata-4` is 118 MB, so it would roughly double)
  and register the row `banked`, or register it `oracle` and let the banked layer
  require nothing. Either way the silent early return should become a declared
  tier. (Note 28 §Z.)
- **Four banked runs are blocked on `nn23lo1` — decided: re-bank** (user,
  2026-08-02; note 29 §G) — `pp_to_bb`, `pp_to_bb_qcd2`, `pp_to_llj` and
  `pp_to_ll_scalefact2` carry `pdlabel = nn23lo1`, MadGraph's internal
  parton-density parameterisation rather than an LHAPDF6 grid the `pdf/` layer
  can load: 8 ⛔ cells. The re-bank at `lhaid = 247000` runs as an
  oracle-layer background task in the note-29 sprint, and **`refdata-5`
  replaces the four superseded runs rather than adding a bundle** (the bundle
  is ~100 MB and does not grow by superseded runs; retired runs go to the
  local retired area per the note 27 D4 precedent). Their scale fields are
  already gated — `validate_scales` replays all four runs event by event.
  (Note 28 §Z.)
- **`pp_to_jj`'s banked event sample is not reproducible across MG re-runs** —
  σ is identical to all printed digits and single-group runs regenerate
  bit-identically, but `pp_to_jj`'s five subprocess groups make the unweighting
  draw sensitive to job scheduling, so a re-run yields a different (equally
  valid) event sample. The banked sample is the reference; any "regenerate the
  bank byte-for-byte" claim must exempt multi-group runs, and C's `samples`
  gate compares distributions, not bytes. (Sb, note 28.)
- **`ForcePositive` is unimplemented, and out of grid it stops being
  negligible** — on NNPDF31 (`ForcePositive: 2`) the clamp fires on 205 of 935
  extrapolated probes, replacing a continued value of magnitude up to 25.7
  with `1e-10`: MadGraph reads `1e-10`, we read `−25.7`. No production impact
  today (every `pdlabel = lhapdf` run carries lhaid 247000, where it fires on
  0/1190 probes), and the relationship is checked data
  (`the_only_difference_from_madgraphs_own_value_is_the_positivity_clamp`),
  not an assumption. Closing it is ~5 lines plus re-reading the interpolation
  gates' `FORCE_POSITIVE_FLOOR` screen. (Note 28 §K5a2.)
- **Tie the clustering-computed μR to the grid coupling in one gate** — 
  `validate_scales`'s `banked_events_reproduce_aqcdup_from_the_computed_scale`
  steps over the `pdlabel = lhapdf` runs because its second oracle was the
  beta-function solve; with the faithful `AlphaS_Ipol` grid reading landed,
  those four runs could join it, checking cluster-scale → μR → αs(μR) in a
  single per-event comparison. (Note 28 §K5a, "available strengthening".)
- **`pp_to_jj`'s 9 tie-break events want a K2-style clustering dump** — K4
  enforces them by signature (the `√(1+1e-6)` beam-crossing inflation is the
  only difference, `<rscale>`'s printed digits pin it) and asserts the count,
  but only an instrumented dump of a `p p > j j` run would show the merge
  sequence directly. The sprint banked the run without a dump; a future
  oracle-layer pass can add one via `gen_kt_cluster_dumps.sh`. (Note 28 §K4.)
- **The K2 clustering-dump format should key its per-directory tables by
  process-directory name** — the writer already has the name (`SHARD` records)
  but the extraction drops it, which forced K3 to disambiguate merged tables
  by forest-row length plus a per-event candidate-list consult on 2 of 9 runs
  (`pp_to_bb_qcd2`, `pp_to_llj*`, with an outright `NQCD` collision). A
  re-extraction keyed by directory removes that whole exception class.
  (Note 28 §K3.4.)
- **`pixi run -e madgraph extract-diagrams` silently re-runs MadGraph** for any
  script whose output directory is missing (`depends-on = ["build-diagrams"]`
  → `build.sh` regenerates). Anyone holding a run directory aside must invoke
  only `--skip-deps` tasks, or the held-out run comes back as a fresh MG job.
- **`release-debug` profile cannot run the `#[should_panic]` contract tests** —
  the profile inherits `release`, so `debug_assertions` are off and
  `eval_m2_pruned_rejects_boosted_frame` (a `#[should_panic]` guarded by
  `debug_assert!`) fails under `cargo test --profile release-debug`. Either gate
  such tests on `cfg(debug_assertions)` or promote the guard to a hard assert.

---

## 🧩 Feature backlog

### Descoped from v1 (user, 2026-08-02)

Each of these is out of the release goal's restricted scope. The validation
sprint makes every one a hard error where a card can ask for it (slate item 4
above); the entries here are the eventual features.

- **Beam polarization** (`polbeam1`/`polbeam2`) — polarized matrix-element
  sums and the per-event `SPINUP` consequences.
- **Beam configurations beyond unpolarized `p p` and fixed-energy partonic** —
  antiproton beams (`lpp = -1`, Tevatron), mixed configurations, lepton-PDF /
  photon beams. `RunCard::parse` admits exactly (0,0) and (1,1) today.
- **Decay-chain process syntax** (`p p > t t~, t > w+ b`) and 1→n
  single-particle decay processes — the grammar and the phase space both
  assume a 2→n hard process.
- **Custom UFO propagators** (`propagators.py`, UFO 2.0) — parse the file and
  thread the propagator forms through the HELAS compiler.
- **Non-SM UFO models** — the `non-sm-ufo` checklist below; the README's
  scope section points at it as the natural next scope step.

### In-scope features

- **s-expression program identity for flavour grouping** — a dedicated future
  sprint, user-scoped. Today's `derive_flavor_groups` partitions subprocesses by
  sampled `|M|²` agreement: **complete but unsound** — two programs that differ
  only where the probe does not look are merged, and the merge is silent.
  **Accepted for v1** (user, 2026-08-02): probe-based judgment has MadGraph
  precedent — MG's own helicity filtering drops vanishing helicity
  configurations on the same sampled-probe basis — and the probe ladder is
  hardened (below); the sound criterion remains the right eventual replacement.
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
- **Streaming `IDWTUP = -4`** by deterministic two-pass replay — the interface
  hook (`EventSource::restart`) is in place and contract-tested; not needed while
  100k-event runs buffer in ~42 MB. (Note 23 close-out.)
- **Massless-t-channel fiducial cut** (sprint plan: note 28 §S2/D3) — a
  massless beam pins `t_max = 0` (collinear edge) where the t-map falls back to
  flat; whether a fiducial cut is wanted instead is unresolved for a physical
  massless-initial-state t-channel. (Note 21 close-out.)
- **Re-examine the "no spine without a scale past two outgoing legs" policy** —
  the policy predates the peripheral-kinematics conditioning fixes (grouped
  Källén, `γ = E/√s`), which removed most of the unregulated-spine defect it
  guarded against: with the grouped form the massless transfer edge is the
  exact analytic zero whenever the emitted subsystem carries a fixed
  invariant, and only composite emitted sides still exhibit the defect. The
  conservative fallback is kept; whether it is still the right default is an
  open measurement. (Note 28 §S3 deviations.)
- **`typed-units`** — research `uom`/`dimensioned`/`units` crates for typed
  four-momenta and cross sections.
- **Self-contained `generate` artifact** (user, 2026-08-02; post-v0.1) — one
  file a clean worker machine can sample from. Today a proton-beam worker
  needs the binary + artifact + both cards + the PDF set (unweighting reads
  densities and grid-αs per trial point; the README documents the
  copy-to-working-dir workaround), and a non-SM run needs its UFO directory
  too. Three pieces, taken together as one feature:
  1. **Bundle the compiled program** (design in note 23; absorbed from the
     performance backlog, whose trigger — setup climbing to a noticeable
     share of a generation run — still applies: compilation is 0.05–0.29 s
     against ~13 s for a 20k-event `generate` today). Key
     `(model digest, process, compiler schema version)` is derivable from
     banked fields, no schema bump needed. Note 23's recorded obstacles: no
     serde in `helas::eval`; `folded_hel` is a lazy `OnceLock` and the
     expanded arena is the large part; `prune_zero_helicities`' kinematic
     contract must be rechecked on load.
  2. **Bundle the PDF data the run reads** — the member's grid file verbatim,
     or a subgrid slice pinned to the run's (x, Q²) support; which, is part of
     the design. Keeps the artifact's refuse-on-mismatch property: the banked
     set name/member already gate, the data would too.
  3. **Investigate compactifying the VEGAS grids** — long-term: per-channel
     grids dominate artifact size on multichannel processes; quantization,
     sparser binning, or shared axes are unexplored.
- **Quality sprint: tighten the `pub` API surface** (user, 2026-08-02) —
  before any backwards-compatibility promise (i.e. before 1.0): audit what
  `vibegraph-lib` exports, demote what only the CLI and the validation crates
  consume, and decide what the supported library surface actually is. Until
  then releases stay on the 0.x line (first tag `v0.1`).

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

**Explicitly descoped from v1** (user, 2026-08-02): the release goal is the SM
UFO, and the README's scope section says so and points here. The UFO surface is
deliberately model-generic, but "generic" currently ends at the SM's feature
set. None of these block anything; collected so a future BSM-model task scopes
against a checklist instead of rediscovering each wall one hard error at a
time. A small dedicated test model (or a public BSM UFO) would be the natural
vehicle for several at once — and would also retire the standing gap that no
non-SM model has ever been loaded end to end, so "model-generic" is currently
exercised on SM evidence alone.

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
  The blocker that stood here is gone: `uux_to_uux`/`gg_to_gg`'s channel maps are
  no longer bit-identical and their α no longer sits at uniform (note 28 §S4 B2),
  so per-flow α is no longer a no-op on those rows. Flows still overlap heavily,
  so the gain is the inter-stratum covariance term, expected modest. **Guardrail:
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
  One half of the integration-sprint pair with the `w_max` item below: both are
  "how the budget splits across adapt / scan / frozen phases", one holistic
  redesign. (`vegas.rs`, note 24 §P3.)
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
- **Per-stage timing capture** (user, 2026-08-01; deliberately deferred from
  the B5 re-bank). Neither side records wall times today: the banked runs
  carry no timing at any stage (the bundled `run_*_log.txt` are job-wrapper
  logs; MadEvent's `run1_app.log` iteration logs are excluded), our report row
  JSONs have no duration fields, and the only timing instrument is
  `scripts/mg_perf_compare.sh` (matrix-element stage only). Wanted, when
  taken: (a) a host-labelled `timings.json` sidecar for MG's stages
  (generate / output / compile / integrate / events, per process) captured
  during an oracle-layer regeneration pass; (b) duration fields in the
  collator's row files so our diagrams/amplitudes/integrals/samples stages
  are timed per run. **Hard requirement: every timing record carries its
  machine identity in full — architecture, core count and which cores were
  used, nominal/boost frequency, memory, OS, compiler/toolchain versions, and
  build settings (profile, flags, `RUSTFLAGS`, MG's Fortran flags) — or it is
  noise**; cross-host comparison of absolute times stays out of scope (note
  15's single-host-ratio position stands). Keep timings out of the refdata
  bundle — they are measurements about a machine, not references. The
  host-independent efficiency layer (points-to-precision from `results.dat`,
  unweighting efficiency and `w_max` shares from the artifact's subsampler
  summary) is already reconstructable from banked data and needs no capture.
- **Per-event scale hot-path cost** — ~100 ns/point on top of a 0.5–1.7 µs
  matrix element (+6% `gg_to_gg`, +21% `uux_to_uux`). `ScaleChoice::clustered`
  heap-allocates its beam–leg candidate list per event; that is the obvious
  first cut. (`coupling/scales.rs`; `validate_sigma.rs` `probe_scale_cost`.)
- **Tighter spacelike floor** — `Cuts::spacelike_floor() = pT_min²` is provable
  but 10–100× looser than the true fiducial floor: S2's D3 measurement found the
  cut-surviving region above `|t| ≈ 4 000–40 000 GeV²` where the floor sits at
  400. A tighter derived bound scales the bounded-`t_max` variance win (measured
  1.67–1.83×) with it. (Note 28 §S2.5.)
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
