# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: the **`v3-backlog` sprint** (validation follow-up) ✅
**closed + merged to `main`** 2026-08-01; `refdata-3` published and pinned
(`[refdata].published = true`), CI's `banked` job gates merges against it.
Census over the 26-row × 4-category report: **75 measured / 74 ✅ / 1 ⚠️**, the
one ⚠️ being the decided `gg_to_gg` 4/6 diagram-counting convention. Full
record in note 27 §7; the `validation-3` findings register it worked from is
note 25 §10. Standing caveat: a partonic σ quoted from `refdata-2` is **not
comparable** to one from `refdata-3` (MadGraph 3.5.7 applied the PDF set's
`αs(M_Z) = 0.130` to `lpp = 0` runs; 3.7.1 keeps the model's `0.118` — note 27
§B5).

**Current sprint**: the **`kt-spine` feature sprint** — note 28, approved and
launched 2026-08-01 (D1–D3 decided on the recommended options). Two tracks
(K: kt-clustering; S: permutation factor + multi-rung spine + massless-t-cut)
converging on a `p p > j j` default-dynamical-scale capstone; freezes the
channel/map structure ahead of the integration-focused performance sprint
(VEGAS first-iteration bias + `w_max` scan decoupling, performance backlog
below).

Unrun until the user pushes a first tag: `release.yml` and `acceptance.yml`.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. Deferred: multi-rung t-channel ladders (note 21) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 12 partonic GATE rows incl. the 3 QCD 2→2s and `pp_to_bb_fixed`, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.284 ± 0.537** vs MG 933.110 ± 0.447; **643.765 ± 0.367** vs 644.420 ± 0.315), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **423.048 ± 0.248 pb** over three seeds vs MG 422.840 ± 1.805 (Δ = 0.11σ). Deferred: `dynamical_scale_choice = -1` (needs `kt-clustering`), which also blocks the four llj partonic σ rows |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) selection, colour selection via MadEvent's `SELECT_COLOR` rule (configuration `∝ AMP2_d`, flow `∝ JAMP2` inside its `ICOLAMP` row) with the flow→`ICOLUP` dictionary checked against MG's `leshouche.inc` (30/30 subprocesses), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 34 banked MG runs byte-for-byte (714 759 events, both of MadGraph's serialisation dialects, source-text pass-through by construction). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

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

---

## 🔎 Validation backlog

### Standing discrepancies to resolve (never a loosened tolerance)

- ~~**The per-diagram multichannel builds degenerate maps for massless-propagator
  processes**~~ — **resolved in `kt-spine` S4**, and not by the multi-rung spine.
  The cause was that `FixedBeamIntegrand::use_multichannel` supplied no fiducial
  scale, so a massless spacelike line sat on the collinear edge and its transfer
  was drawn *flat*: every peripheral fixed-beam channel collapsed to the same
  isotropic 2-body split. Passing the cuts' own `spacelike_floor()` — what
  `ProtonIntegrand` always did — makes the draw `1/|t|` over the fiducial window.
  Re-measured (`validate_sigma.rs` `probe_channel_map_degeneracy`): worst pairwise
  density difference `1.000` on both `u u~ > u u~` (0 of 2000 pairs coincident,
  α `[8.5e-6, 0.99999]`) and `g g > g g` (α `[3.2e-5, 3.2e-5, 0.496, 0.504]`),
  against the unchanged `g g > t t~` control (`0.84`, `[0.267, 0.364, 0.369]`).
  `g g > g g`'s two non-peripheral channels stay bit-identical to each other,
  which is expected — neither has a spacelike line for the floor to act on.
  (note 27 §B3.2, note 28 §S4.)
- **Four llj partonic σ rows are unreachable, not merely ungated** — `uux_to_epemg`,
  `ddx_to_epemg`, `gu_to_epemu`, `gux_to_epemux` are banked with cross sections
  and cost seconds to integrate. They cannot run at all: all four run cards leave
  both scales free at `dynamical_scale_choice = -1`, and their topology — a
  t-channel propagator into a three-leg final state — is exactly the case whose
  cluster scale depends on the merge order, which `coupling::scales` refuses
  rather than approximates. No scale on this side reproduces MadGraph's number,
  and a fixed-scale re-run would be a different cross section. Their `integrals`
  cells are `blocked` on `kt-clustering` in the manifest and named in
  `validate_sigma`'s `plan_for`; their `samples` cells are blocked on the same
  blocker, and the refusal in generation is measured rather than assumed.
  Fixed by `kt-clustering` (feature backlog), which grows four ready-to-flip σ
  rows on top of the six asserted-refused scale rows it already owns.
- ~~**`uux_to_uux` residual bias**~~ — **resolved in `kt-spine` S4**. The −0.30%
  five-seed mean was the flat transfer draw above, not a missing rung: with the
  jet cut's floor supplied, the five-seed mean is **+0.019%** at the gate budget
  and **+0.015%** at four times it, worst |pull| `0.93` and worst |rel| `1.1e-3`
  (was `2.69` / `6.4e-3`). The quoted error at the gate budget fell 2.4×, and
  `g g > g g`'s fell 2.6×. (`validate_sigma.rs` `probe_qcd_seed_stability`,
  note 28 §S4.)
- **`ud_to_epemud_qcd0` carries a relative sign between diagrams** — localised in
  `kt-spine` S5; the σ cell stays informational at `1.0860e-1 pb` against
  MadGraph's `1.4107e-2 ± 3.4e-5 pb`. The row is now a registered amplitude
  process with a committed table, compared on every run as a `hermetic` / `info`
  cell (`KNOWN_LINEAR_DISAGREEMENT` in `amplitude_oracle`). Every one of the 35
  diagrams reproduces MadGraph's own `AMP()` to rounding under a unit phase — the
  pairing is banked and exact at overlap `1` over 48 (point, helicity) rows — but
  **eleven of them carry the opposite sign to the other 24**: MadGraph graphs
  `1–8` (the three-rung ladders whose middle rung is a spacelike lepton), `17`
  (the same with a spacelike neutrino between two `W`s) and `18, 19`
  (`W+W- → γ*/Z* → e+e-`). Flipping exactly those eleven takes the worst |M|²
  deviation over all 74 banked points from `5.1e+1` to `4.1e-14`. The predicate
  that selects them is *the beam-to-beam spine carries an even number of boson
  rungs*; three candidate correction rules were each falsified by a gated control
  (`ee_to_mumu_tata_qcd0`, `ee_to_wpwm`, `u u~ > u u~`), so the fix is **not** a
  missing multiplicative factor of those forms. Rooting soundness passes on the
  process (270 re-rootings, 0 failures) and the CKM is diagonal on both sides.
  **Next step**: derive the crossing sign for a diagram whose spine passes through
  a crossed fermion line, and for a triple-gauge vertex whose legs are all
  internal — the required sign vector is known exactly, so a candidate rule can be
  checked against it and the 19 gated tables in one run. (note 28 §S5.)
- **`ee_to_mumua` drifted when the references moved to 3.7.1** — the one row
  where 3.7.1 disagrees with us *more* than 3.5.7 did. Our σ is unchanged
  (`1.007660e-1` pb, same integration); MadGraph's moved `1.00630e-1 ± 3.865e-4`
  → `9.980100e-2 ± 2.335e-4` pb, so the `integrals` pull went **+0.31 → +3.12**
  (rel +0.135% → +0.97%) against a `PULL_LIMIT` of 3.5, and about half of that
  growth is the tighter reference error rather than the −0.83% shift itself. Its
  `samples` cell followed: minimum KS p **2.14e-3 → 2.74e-4** against a `1e-4`
  floor, worst observable `y(a)` → `pt(a)`. Both cells still gate, and both are
  now the tightest in their category. The photon here is soft/collinear-regulated
  by the run card's cuts, which is the region MadGraph's channel-weight change
  reallocates; whether the remaining 1% is that or ours is not established.
  Wanted: a windowed comparison over `pt(a)` of the kind note 27 §B1 used on the
  Higgs pole, which is the measurement that decides which side owns it.

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
  sound replacement is the s-expression criterion (feature backlog).
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
- **`pp_to_llj_qcd2_qed2` is a duplicate of `pp_to_llj`** — the banked event
  payloads are sha-identical (the `QCD=2 QED=2` restriction coincides with the
  default orders for this process), so the census double-counts one
  measurement. Decision (user, 2026-08-01): keep `pp_to_llj`, prune the
  duplicate at the kt-spine close-out alongside the refdata-4 re-cut; until
  then the pair counts as one independent row. (Note 28 §6/D4.)
- **`pp_to_jj`'s banked event sample is not reproducible across MG re-runs** —
  σ is identical to all printed digits and single-group runs regenerate
  bit-identically, but `pp_to_jj`'s five subprocess groups make the unweighting
  draw sensitive to job scheduling, so a re-run yields a different (equally
  valid) event sample. The banked sample is the reference; any "regenerate the
  bank byte-for-byte" claim must exempt multi-group runs, and C's `samples`
  gate compares distributions, not bytes. (Sb, note 28.)
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
- **`cargo fmt --check` is red on `main`** for four files
  (`validate_samples_proton.rs`, `validation/samples.rs`,
  `amplitude_oracle.rs`, `color_cf_oracle.rs`) — pre-existing; wants a
  formatting-only commit at a quiet moment.

---

## 🧩 Feature backlog

- **`identical-particle-permutation`** — ✅ **done in `kt-spine` S1** (note 28
  §"S1 — channel-enumeration decision"). `phasespace::identical_particle_factor`
  is the single definition of `1/Π_s n_s!`; every consumer derives it from the
  outgoing legs it owns, and neither latent shape survives —
  `FixedBeamIntegrand`'s integrand-level factor field (the `amps[0]` derivation)
  and `ProtonError::IdenticalFinalState` (the assert-the-factor-is-1 refusal) are
  both gone, and a flavour group applies each *member*'s own factor. The
  channel-enumeration question is decided with the decision recorded: one channel
  per diagram, permutations not enumerated because the diagram set is already
  closed under them — pinned by a test with a control that refuses a
  configuration where the closure cannot be seen. Still owed by the sprint: the
  capstone `p p > j j`, the only process that exercises unequal factors against
  MadGraph.
- ~~**Multi-rung t-channel spine**~~ (sprint plan: note 28 §S2–S4) — ladder
  topologies (VBF/DIS, ≥2 spacelike lines). **Landed in production in `kt-spine`
  S3+S4**: `Spine → rungs: Vec<SpineRung>` + terminal recoil, each rung emitting
  one blob against the running `q_i = p_a − Σ_{j≤i} p_{B_j}`, built in the CM of
  what the previous rung left behind with the previous (spacelike) transfer as its
  incoming line. `from_diagram_regulated` derives the chain; `from_diagram_capped`
  is the truncated map the informational arm measures against. The fiducial `t_max`
  bound is per rung (D3), and the fixed-beam path now supplies its own cuts'
  `spacelike_floor()` the way the proton path always did — which is what actually
  moved anything, since no production process is both a ladder and regulated
  without it. Gates green on `u d > e+ e- u d QCD=0` — graph-cut cross-check of the
  chain, per-chain volume against its own support, the §S2.4 density contract,
  walk-vs-density 6.8e-9, the §S2.3 ordering oracle with NEG-A/B/C firing, and a
  per-process union-coverage gate over every bounded channel set. Still owed by the
  sprint: the capstone rides the chain, and `ud_to_epemud_qcd0`'s σ stays
  informational on the amplitude disagreement above. (Notes 21, 28.)
- **`kt-clustering`** (sprint plan: note 28 §K1–K5, superseding the sketch
  below) — general kT clustering for `dynamical_scale_choice = -1`
  (also what MLM matching needs). 6 banked runs are asserted as
  refused; **hard prerequisite for gating any QCD process at MadGraph's default
  scale choice** — the no-strong-coupling short-circuit stops covering it the
  moment the matrix element carries `G`. (Multiplicity is not the barrier:
  `p p > l+ l- j` is gated at a *fixed* scale.) Note 22 §1.3 pins the degenerate
  closed-form cases;
  this sprint builds the general path. Sessions:
  1. ✅ **Design note** — the binding spec is note 28 §K1: the `djb`/kT measure,
     which merges are admissible (graph-guided — only vertices the process's
     diagrams contain), the tie-break order (including the `1 + 1e-6`
     beam-crossing inflation note 22 §1.3 caught in `uux_to_uux`), and how the
     cluster sequence maps to μR (the geometric-mean prescription) and per-beam
     μF, each claim with its falsifier.
  1b. ✅ **Oracle** — an instrumented 3.7.1 dumps every intermediate per banked
     event, for 9 runs × 10k events (`validation/madgraph/wrappers/ktdump*`,
     `kt_cluster_dump_manifest.json`).
  2. ✅ **Clustering engine** — `coupling/cluster/` (merge graph, clustering,
     scale walk), informational only; `ScaleChoice`'s `-1` still takes the
     closed-form-or-refuse path. **All 90 000 dumped events reproduce, merge
     sequence and scales, at zero observed deviation**
     (`validate_kt_cluster.rs`, 2.4M candidate pairs, 120 merge tables). Two
     counted exceptions and one open thread in note 28 §K3: MadGraph's on-shell
     flag array is stale across events (244 events, both 2→6 runs), the dump
     cannot name a process directory (7 flavour assignments across 3 runs), and
     a single external leg's `ipdgcl` keeps its flavour where the source reading
     says the complement registration overwrites it.
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
- **Streaming `IDWTUP = -4`** by deterministic two-pass replay — the interface
  hook (`EventSource::restart`) is in place and contract-tested; not needed while
  100k-event runs buffer in ~42 MB. (Note 23 close-out.)
- **`μF ≥ 2 GeV` event veto** — `reweight.f:1185` *vetoes* the point below it;
  `coupling::scales` reports the scale only. Bites nothing today; a hadronic run
  with a dynamic μF reaching below 2 GeV will disagree with MG without it.
  (Note 22 §4 + close-out.)
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
