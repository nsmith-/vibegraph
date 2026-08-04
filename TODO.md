# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: the **note-29 validation sprint** ✅ **closed** 2026-08-03
on branch `val4`. All three standing discrepancies resolved (conjugate colour
tags per-member; the `AMP2_c` scale-channel draw retired the channel-partition
tolerances to 0.005; `ee_to_mumua` adjudicated — the reference owns it), every
descoped card surface a hard error, `refdata-5` **published and pinned**
(release `refdata-5`, asset digest round-trip-verified), and the 8
`nn23lo1`-blocked cells enforced. Census over the 29-row × 4-category report:
**98 measured / 96 ✅ / 2 ⚠️** (one ⚠️ is the pre-existing `gg_to_gg`
diagrams 4/6 annotation; the other is a new finding, below). Full record:
note 29 close-out.
Standing caveats: a partonic σ quoted from `refdata-2` is **not comparable** to
one from `refdata-3`/`refdata-4`/`refdata-5` (MadGraph 3.5.7 applied the PDF
set's `αs(M_Z) = 0.130` to `lpp = 0` runs; 3.7.1 keeps the model's `0.118` —
note 27 §B5); and the four re-carded runs' σ are **not comparable** across the
`refdata-4`→`refdata-5` boundary (MG-internal `nn23lo1` vs LHAPDF NNPDF2.3-QED
are different densities — `p p > b b~` moves −9.8%).

**Scope decision (user, 2026-08-02)**: the release goal is restricted to
**arbitrary fixed-order Standard Model processes** over unpolarized
proton–proton or fixed-energy partonic beams, without decay-chain syntax.
Every extension beyond that — BSM UFO support, other beam configurations,
polarization, decay chains — is explicitly descoped to the feature backlog
(see "Descoped from v1" below), and every descoped surface a card can still
reach must be a **hard error**, never a silent acceptance; the fixes closing
the remaining silent acceptances are validation-sprint items.

**Awaiting the user**: the first release tag is **`v0.1`**, decided
2026-08-02 and re-affirmed 2026-08-03 to follow the **performance sprint**
(next below) — a 0.x line because no global backwards-compatibility promise
is made yet; a future "quality sprint" tightening the `pub` API surface
(backlog below) precedes any 1.0. Tagging runs `release.yml` and
`acceptance.yml` for the first time.

**Next sprint**: the **performance sprint** — the integration-focused pass
(VEGAS first-iteration bias + `w_max` scan decoupling + stratified-parallel
axes, performance backlog below). `kt-spine` froze the channel/map structure
it measures against; the note-29 sprint hardened the gate it optimizes
against. One note: chain B added one `eval_amp2` + one `set_alpha_s` per
point on live-draw rows (cost unmeasured — a baseline timing before
optimizing is the first task).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. The multi-rung t-channel spine and the per-subprocess identical-particle factor are in production (`kt-spine` Track S, note 28) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 17 partonic GATE rows incl. the 3 QCD 2→2s, `pp_to_bb_fixed` and all 4 llj subprocesses at the kT-clustered per-event scale, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.284 ± 0.537** vs MG 933.110 ± 0.447; **643.765 ± 0.367** vs 644.420 ± 0.315), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **423.048 ± 0.248 pb** over three seeds vs MG 422.840 ± 1.805 (Δ = 0.11σ). At a *dynamical* scale each point's cluster scale is taken in the integration configuration drawn from the point's own squared amplitudes (`∝ AMP2_c/Σ AMP2`, MadEvent's enhancement-weight conditional, note 29 chain B): `gu_to_epemu` **+0.004%** (pull +0.02) / `gux_to_epemux` **−0.11%** (pull −0.49) and σ(pp→ℓ⁺ℓ⁻j) **−0.01%** (pull −0.02), all GATE at `rel_tol` 0.005 set by the references' own errors. The four `refdata-5` re-carded rows gate on the same path (`pp_to_bb` +0.02%, `pp_to_bb_qcd2` +0.00%, `pp_to_llj` +0.04% at 600k, `pp_to_ll_scalefact2` −0.01%). The `p p > j j` capstone runs the same path on the canonical QCD process and is **GATE**: **6.803009e8 ± 2.511e5 pb** over three seeds vs MG 6.788500e8 ± 1.4726e6, rel **+0.21%** at pull **+0.97**, at `rel_tol` 0.005 — the reference's own 0.22% with headroom, pull asserted, since its channel-partition ambiguity is only `1.0e-3` (its own Monte-Carlo error, because a 2 → 2 gives the clustering no merge to choose). It sums over MadGraph's own 65 concrete assignments, pinned entry for entry against the run's `leshouche.inc` |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) selection, colour selection via MadEvent's `SELECT_COLOR` rule (configuration `∝ AMP2_d`, flow `∝ JAMP2` inside its `ICOLAMP` row) with **per-member colour-flow tables** — each flavour member's tags derived under the structurally-determined flow permutation, refuse-on-ambiguity — checked against MG's `leshouche.inc` (73/73 concrete subprocesses over 47 files; note 29 chain A), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 37 banked MG runs byte-for-byte (744 759 events, both of MadGraph's serialisation dialects, source-text pass-through by construction). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

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
- **note-29 validation sprint** (validation, closed 2026-08-03, branch `val4`) — seven design→implement→review chains + the §G re-bank. A: conjugate colour tags fixed by **per-member colour-flow tables** under a structurally-determined permutation (dijet `ICOLUP` χ² p 0 → 0.105–0.263, T5 0/80 000; the design's premise falsified twice by measurement en route). B: **MadEvent's per-point `AMP2_c` scale-channel draw** in production (pure function of `(channel, u)`, zero bits from existing streams) — partition gaps collapsed to MC noise, `gu`/`gux`/llj_dyn tolerances retired 0.02/0.015 → **0.005**, σ(pp→ℓ⁺ℓ⁻j) rel −0.68% → **−0.01%**; reviewer derived the missing `this_config` reconciliation from MG source. C1+C2: every descoped card surface a **hard error** (polarization, decay chains, `propagators.py`, 209-field audit with 23 refused, μF ≥ 2 GeV veto as zero-weight, `dynamical_scale_choice` 1–5 refused). D: `ee_to_mumua` drift adjudicated **D1 — the reference owns it** (MG's own partitions disagree at ≥15σ; our total matches its m(μμ) re-integration at 0.16σ; tolerances unchanged). E: ForcePositive with LHAPDF's own clamp semantics; `validate_kt_cluster` a declared oracle tier; +2 cells. F: U(1) charge-flow phase — pre-registered negative result. §G: **`refdata-5` pinned** (four runs re-carded onto lhaid 247000, member list identical name-for-name; publication pending), all 8 ⛔ cells enforced. Census 87/85✅/2⚠️ → **98/96✅/2⚠️**. Transferable lesson: **pre-register the may-move set** — chain B's escalation diff landing byte-exactly on its five predicted cells is what made the sprint's biggest σ change auditable at a glance; note 29 close-out.
- **`kt-spine`** (feature, two tracks, closed 2026-08-02) — Track K: MadGraph's general kT clustering reproduced merge for merge against an instrumented 3.7.1 (90 000 dumped events, zero observed deviation), the closed forms deleted so `dynamical_scale_choice = -1` takes one path, `GridAlphaS` made LHAPDF's own `AlphaS_Ipol` and the density grid continued past its edges — then the flips: 6 asserted-refused scale rows became per-event replays, the 4 llj partonic σ rows and their `samples` cells left `blocked`, σ(pp→ℓ⁺ℓ⁻j) re-gated at the dynamical scale, and the capstone **`p p > j j`** gated on MadGraph's shipped run-card defaults (**6.803009e8 ± 2.511e5 pb** vs MG 6.788500e8 ± 1.4726e6, rel +0.21%, pull +0.97). Track S: the identical-particle factor moved into the phase-space map per subprocess, and the multi-rung t-channel spine landed in production. Two bugs the sprint found rather than assumed: the **fixed-beam path was never regulated** (every prior "what is the spine worth" measurement was taken on flat transfer draws), and `p p > j j`'s σ was **36% high** because a repeated final-state label enumerated `g u > g u` and `g u > u g` as two subprocesses. Transferable lesson: **a per-event field is a finer oracle than a cross section, and it exists more often than it looks** — the clustering was pinned by an instrumented replay of MadGraph's own intermediates long before any σ moved, which is why every σ flip that followed had a diagnosis attached. Census 75/74/1 → **87/85/2** over 29 rows; note 28.

---

## 🔎 Validation backlog

### Standing findings to diagnose (from the note-29 sprint; never a loosened tolerance)

- **`ud_to_epemud_qcd0`'s event sample fails its `ICOLUP` χ² at ≈650 on 1 dof**
  (p ≈ 0, seed-stable, 60 000 events over 6 seeds) while kinematics and
  `SPINUP` clear their floors — measured the moment chain E wrote the
  comparison. This is the **fixed-beam** record path (`SubprocessRecord::new`),
  not chain A's relabelled-member mechanism (that fix is in and gated on the
  hadronic rows): a colour-flow convention gap in the mixed-line topology's
  flow→`ICOLUP` dictionary is the standing hypothesis. The row is `info` until
  diagnosed; the diagnosis session should start from the banked run's
  `leshouche.inc` against `color_flow_tags` for this process class.
- **`ee_to_mumua` residual ~1% ours-high excess in the radiative-return
  `pt(γ)` windows** — the one thing chain D's D1 verdict left unattributed,
  localised to MadGraph's `pt(γ)/η(γ)` coverage rather than either matrix
  element or the Z propagator. The named next probe: the 2D
  `[39.4, 77) × [86, 96)` `(pt(γ), m(μμ))` cell. Related watch item: the
  row's `samples` KS floor headroom is **1.3×** (measured `1.292e-4` against
  the `1e-4` floor), half what the stale doc claimed — if the cell flaps, the
  D record (MG's sample contradicts MG's own integrals at ≥15σ) is the
  diagnosis context, and the floor does not move.
- **`p p > j j`'s across-group scale spread is `4.999999e-7`, not zero**,
  while every within-group spread is exactly `0.0` (chain B-0's census).
  Nothing in production reads the group axis for scales, so it moves no cell —
  but it is far too large to be rounding on a 2→2 whose scale ought to be
  group-independent, and it is the size of the effect a future group-axis
  change would expose. Worth one look at where the group enters.

### Sharper oracles the sprint named but did not build

- **A `SCALUP` column in the `samples` category** — the sharpest missing
  oracle (chain B review): no samples cell compares `SCALUP`, though
  MadGraph's banked LHEs carry it and `validate_scales` already replays it
  for MG's own events. Two findings wait on it: rows that compile no scale
  prescription emit the run-card `SCALUP` and `AQCDUP = 0`
  (`vibegraph-cli/src/generate.rs:349`) while MadGraph's own `ee_to_mumua`
  events carry a clustered channel-dependent `SCALUP` — σ is right (nothing
  reads the scale) but the *records* differ, and no gate sees it; and a
  per-event scale distribution check is the one oracle that would catch an
  `AMP2_c`-share error that preserves σ.
- **Expose the drawn scale configuration and close chain B's two accepted
  gaps**: assert the `∝ AMP2_c/Σ AMP2` frequency law end-to-end (today it is
  factored into four independently-gated pieces — `select.rs`'s binomial
  test, the order pin, the colour-draw χ², and σ), and promote the
  zero-spread census to a banked assertion on the cheapest inert rows so a
  future change that makes a scale configuration-dependent on a declared-inert
  row fails a standing gate instead of a one-time manual diff. Blocked on
  nothing but runtime cost for the latter.
- **`scale_draw_fallbacks()` is counted and read by nothing** — a NaN `AMP2`
  falls back to the sampler's channel silently (`select_index` returns `None`
  on a non-finite total). One assertion that the counter is zero on the gated
  rows makes the silent path loud.
- **`RunningAlphaS::eval` returns NaN silently below ~0.5 GeV** (the two-loop
  `newton1` seed takes `ln` of a negative argument). The μF ≥ 2 GeV veto and
  the μR floor bound today's exposure, but the surface is a silent-NaN class:
  one guard (error or clamp, matching MG's own behaviour) closes it.
- **`k`/`G` measured exactly ±1 (and the per-process route-sign patterns)** —
  chain F's settled leads: one `|Im(k/G)|` assertion converts free phases to
  pinned bits (no reference data, no tolerance move); `run_config_amps()[i]`
  disagrees in sign with the single-diagram compile on 3 processes (exactly
  ±1, spread 0, production evaluators, no production consumer — eval_amp2 is
  sign-blind). Any future assertion must pin per-process sign patterns, not
  uniformity.

### Deferred coverage

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
- **Small hygiene the note-29 sprint left named**: the `blocked` tier is now a
  documented manifest schema slot used by nothing (keep or retire — a schema
  decision); `probe_recarded_budget_ladder` (~26 min, oracle layer) has no
  pixi task and `validate-deep`'s long-tier text does not name it;
  `pp_to_llj`'s integrals gate runs 600k points/iteration (~+2 min per banked
  run) because its ladder was still climbing at 300k — the alternative is
  `info` at 300k, never a wider tolerance; the direct-vs-mirror ordering is a
  third partition axis chain B named with a falsifier but nothing measures;
  chain B's draw raises low-budget seed scatter (χ²/dof 6.38 at 75k,
  clean ≥150k) — a future budget reduction on `pp_to_llj_dyn` would bite;
  the `Opaque` run-card default payload fix (note 28 §C2.5).
- **`pp_to_jj`'s banked event sample is not reproducible across MG re-runs** —
  σ is identical to all printed digits and single-group runs regenerate
  bit-identically, but `pp_to_jj`'s five subprocess groups make the unweighting
  draw sensitive to job scheduling, so a re-run yields a different (equally
  valid) event sample. The banked sample is the reference; any "regenerate the
  bank byte-for-byte" claim must exempt multi-group runs, and C's `samples`
  gate compares distributions, not bytes. (Sb, note 28.)
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
