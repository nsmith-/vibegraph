# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: `event-output-lhef` (feature) ✅ closed + merged 2026-07-28 —
the pipeline now runs end to end to an unweighted event file. **Next (planned
2026-07-30, note 24): the `user-distribution` + `proton-events` two-track feature
sprint** — hadronic multichannel + `lpp = 1` event generation gated on a
fixed-scale MG rebank of `p p > l+ l- j`, plus the packaging track (release
binaries, default PDF, `~/.vibegraph` cache, first-run UX); exit criterion is
cards → `.lhe` for llj from a clean environment. The validation pass
(shower consumption + event-sample statistics, top of the validation backlog)
queues behind it and gains a second waiting process.

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 14 processes agree with MadGraph (11 bit-identical ≤6.3e-13, incl. 2→6/VVV/massive externals, all NCOLOR=1; `uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation). Deferred: multi-rung t-channel ladders (note 21) |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv3, model identity). σ gate: 11 GATE rows incl. the 3 QCD 2→2s |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) + colour-flow (`∝ JAMP2`) selection with the flow→`ICOLUP` dictionary checked against MG's `leshouche.inc` (24/24 subprocesses), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 20 banked MG `.lhe.gz` byte-for-byte (198 747 events). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). Deferred: shower consumption, event-sample-vs-MG statistics, `lpp = 1` |

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

---

## 🔎 Validation backlog

### Next validation pass — natural content

- **Downstream-shower validation of the emitted `.lhe` (Pythia via pixi)** —
  deferred out of E4 by decision. The E4 gate reads `generate`'s output back with
  **our** `lhef::parse`, so a self-consistently wrong format is invisible to it;
  E3's byte-for-byte MG round trip covers only fields MG itself writes. Shape: a
  `pixi run` task handing an emitted `.lhe` to `Pythia::init`, requiring it to
  read every event and reconstruct the hard process — the colour lines in
  particular, which no other gate exercises as *input*. (Note 23 §E4 outcome;
  `vibegraph-cli/tests/cli_generate.rs` module doc.)
- **Event-sample vs MG statistical comparison** — deferred out of E3 by decision.
  We do not share MG's RNG, so no per-event comparison exists; owed is a
  **distribution-level** comparison of our unweighted sample against MG's banked
  one: invariant masses, angles, and — the fields nothing else covers — the
  empirical `SPINUP` helicity and `ICOLUP` flow frequencies, which E1/E2 pin only
  as *rules* (`∝ |M_hel|²`, `∝ JAMP2`), never against MG's realised sample. Needs
  designing (binning, observables, per-process statistics). (Note 23 §E3 outcome;
  `validate_lhef.rs` module doc lists what E3 provably cannot detect.)
- **MG-plot distribution comparison** — L5 validated histograms against *analytic*
  BW/t-channel oracles with MG σ as coarse backstop; comparing against MG's own
  `.lhe`/plots needs the MG toolchain. Same machinery as the row above, and the
  vehicle for `low-mll-reconciliation` below. (Note 21 close-out.)

### Standing discrepancies to resolve (never a loosened tolerance)

- **`low-mll-reconciliation`** — `ee_to_mumu_tata_qcd0` samples stably (5 seeds
  within 0.45%, χ²/dof 0.97–1.21) but sits **+2.2% above** banked MG (pull
  +6.7…+8.3; was +3.0% before per-channel grids), entirely below `m_ll ≈ 20 GeV`
  (cutting there agrees to −0.1%). The *sign* rules out under-coverage on this
  side, so either MG under-counts (its `set_peaks` massless-pole grid floor
  truncates the same region) or this sampler over-weights it. **A scalar σ cannot
  decide it** — needs differential `dσ/dm_ll`. Row stays `Plan::Info`; 5
  `#[ignore]`d probes in place in `validate_sigma.rs`.
- **`uux_to_uux` residual bias** — hard σ GATE, but the five-seed mean is
  **~−0.30%** since per-channel grids (was ~−0.17% shared-grid) and does not
  shrink with budget. Sharper per-channel grids cover the spacelike collinear
  tail *less* — the region a single-rung t-channel spine under-resolves. Evidence
  for the multi-rung spine (feature backlog), not a new defect.
  (`validate_sigma.rs` `probe_qcd_seed_stability`.)

### Deferred coverage

- **V7 per-flavor diagram matching** — deferred from `validation-2`: Python
  extractor + Rust sorted-PDG matching + JSON regen, with a real-finding risk
  (whether vibegraph enumerates MG's exact concrete-subprocess union). Design
  preserved in note 19 §3 / §V7.
- **`IdentityAmp` process-level coverage** — the last `KNOWN_UNCOVERED` op; needs
  an `Identity` scalar bilinear the SM lacks, so it rides with `non-sm-ufo`
  (feature backlog).
- **Minor pinned discrepancies** (note 22 close-out): `ee_to_wpwm` topology mask
  unpinned between D4's derivation and D2's declaration (tie-break never reaches
  the scale); `run_card_dy.dat` fixture disagrees with the banked `dy13` cards on
  `fixed_ren_scale` (asserted, not aligned).

---

## 🧩 Feature backlog

- **`identical-particle-permutation`** — make the symmetry factor a property of
  the phase-space map. `dΦ_n` over-counts a final state with identical particles
  by `Π_s n_s!`; `dynamical-scales` added `final_state_symmetry_factor`
  (`hadronic.rs`) but as a per-integrand scalar — the wrong home. Two latent
  consequences, both smooth factor-of-`n!` σ errors: `FixedBeamIntegrand::new`
  derives the factor from `amps[0]` and applies it to every subprocess, but in
  `p p > j j` the factor differs between subprocesses whose mass lists agree
  (`gg→gg` needs 1/2, `qq̄→qq̄` needs 1, both `[0,0]`); `DrellYanIntegrand`
  carries an implicit 1, assumed rather than derived. The map knows its own
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
  refused; **hard prerequisite for gating any QCD process beyond 2→2** — the
  no-strong-coupling short-circuit stops covering it the moment the matrix
  element carries `G`. Note 22 §1.3 pins the degenerate closed-form cases;
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
     enforced per-event replays (`SCALUP`/`<rscale>`/`<pdfrwt>`), then gate a
     first beyond-2→2 QCD σ row (`pp_to_llj` is already banked).
  (`coupling/scales.rs`, `validate_scales.rs`, note 22 §1.3/§5.)
- **Proton-beam (`lpp = 1`) event generation** — ▶ **promoted to the active
  sprint plan (note 24, Track P `proton-events`)**: hadronic `ChannelIntegrand`
  (τ,y outer map + per-event-ŝ multichannel), generalized flavor classes,
  `integrate`/`generate` at `lpp = 1`, gated on a fixed-scale MG rebank of
  `p p > l+ l- j` (fixed scale sidesteps the `kt-clustering` prerequisite).
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
- **`user-distribution`** — ▶ **promoted to the active sprint plan (note 24,
  Track U)**: release binaries (CI), default-PDF interning (license check
  first), `~/.vibegraph` name-resolution cache, first-run fetch-prompt UX;
  acceptance is driving Track P's fixed-scale `p p > l+ l- j` from a clean
  environment, cards → `.lhe`. Session detail lives in the note.
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
