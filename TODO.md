# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate.

**Current position**: **between sprints.** The `banked-open-ends` validation
sprint closed 2026-09-07; note 36 §7 is its close-out record and PR #6 carries it.
**Next**: the **`process-grammar`** feature sprint (note 38). It takes the slot
the rhythm would give to performance, because performance work is already running
in parallel on evaluator PRs (user, 2026-09-25). Sprint sessions stay out of
`helas/eval` where they can.

**Census**, counted from `validation/manifest.toml`: **178 measured cells — 171 ✅,
7 ⚠️ — plus 4 ⏳ at the long tier and 22 uncovered.** The cells that stay
informational, one clause each: `ee_to_wpwm_cw` (a single |M|² point at 2.08e-12);
`ee_to_zh_smeft` (MadGraph's Python-to-Fortran writer rounds the UFO's `11/24`
literal in `GC_303` to seven digits — a defect on its side, note 35 §3 E1);
`wpwm_to_wpwmz_cw` (the five-vector residual, |M|² 2.20e3); the two `NGRAPHS`
diagram counts (a counting convention, not an amplitude); `ee_to_mumua` (a fixed
+1.04% ours-high residual in the radiative-return windows, reference-adjudicated
but unattributed); and `gg_to_gg_cg`'s σ (a converged −0.22% offset). Each has a
validation-backlog entry below saying what would flip it.

**Scope decision (user, 2026-08-02)**: the release goal is restricted to
**arbitrary fixed-order Standard Model processes** over unpolarized
proton–proton or fixed-energy partonic beams, without decay-chain syntax.
Every extension beyond that — BSM UFO support, other beam configurations,
polarization, decay chains — is in the feature backlog under "Descoped from v1",
and every descoped surface a card can still reach must be a **hard error**,
never a silent acceptance.

**Scope decision (user, 2026-09-25)**: the release goal widens to **MadGraph
leading-order process parity, without MLM matching and without NLO**. That adds
decay-chain syntax, 1→n decay processes, the s-channel restrictions (`>`, `$`,
`$$`), or-multiparticles and `add process` over processes with the same final-state
multiplicity. MLM, then NLO, follow in later sprints, and the data structures
passed along the pipeline leave room for both (note 38 §3.2). The hard-error rule
is enforced in one place: the proc card is parsed in full into MadGraph's
`ProcessDefinition` shape, and a single check refuses every unsupported feature
before anything downstream reads the card (note 38 §3.1). Its `Unsupported` enum
is the feature backlog against MadGraph. Audited 2026-09-25 (note 38 §2): today
`>`, `$`, `[…]`, `set` lines, a second `generate`, duplicates across process
lines, PDG-code legs and `@N` downstream are all **silently** mishandled, so the
sprint's first session (G1) closes those before any feature lands.

**Open, and the user's call** (nothing here is blocked on code):
- the repository is now public (confirmed 2026-09-25: the `refdata-7` release
  asset downloads unauthenticated), so `acceptance.yml`'s release-asset 404 should
  be gone; see the gate-hygiene entry below;
- reading the first green `acceptance.yml` run, whenever that is, since the
  workflow has still never passed;
- switching GitHub Pages to the "GitHub Actions" source in the repository
  settings, which the `docs.yml` deploy job needs before it can succeed;
- whether to run `scripts/rewrite-ai-trailers.py` over the existing history —
  it rewrites every commit hash, so it must precede the first public clone.

**Standing measurement facts.** The performance layer's numbers live in notes
30–32 rather than here: the note-30 baseline and note 31 §6 / note 32 §5's
close-outs carry the per-category timings, the vs-MadGraph per-point ratios and
the parallel-scaling figures, all on one M3 Max host. Two properties of that
host matter to anyone reading a new measurement against them: the layer's own
run-to-run spread is **0.8% median / 3.4% worst** on rows above 1 s, so a
sub-1% claim is not measurable there, and a noisy host costs ~12.7%, which is
why CPU time is the more reliable instrument than wall. One caveat outlives the
notes' numbers: a partonic σ quoted from `refdata-2` is **not comparable** to one
from `refdata-3` and later (MadGraph 3.5.7 applied the PDF set's `αs(M_Z) = 0.130`
to `lpp = 0` runs, 3.7.1 keeps the model's `0.118`), nor are the four re-carded
runs' σ across the `refdata-4`→`refdata-5` boundary (different densities;
`p p > b b~` moves −9.8%).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params; model identity (label + SHA-256 over the parsed model) banked into artifacts |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 19 rows agree with MadGraph at ≤5.9e-13 on the fixed grid (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) and at ≤6e-14 on MadGraph's own banked events — except the two `ee_to_mumu_tata_qcd0` events near the Higgs pole, where the point's own one-ulp conditioning exceeds the deviation. Beneath \|M\|²: per-diagram `c_i·AMP(i)` on every single-flow row with ≤64 diagrams, per-flow `JAMP()` on all 19, one fitted constant `G = ±i` serving both |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS (two-phase `adapt`/`sample_frozen` serde object, deterministic rayon chunking, one grid **per channel**) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams + MadGraph-style multichannel (per-diagram propagator-pole channel trees, BW/t-channel/massless-log maps, variance-minimising weight, α-adaptation), rebuilt per event ŝ at proton beams with the t-channel draw floored by `Cuts::spacelike_floor()`. The multi-rung t-channel spine and the per-subprocess identical-particle factor are in production (`kt-spine` Track S, note 28). Map choices are `--map-*` flags with measured `auto` rules, banked in the artifact (schema 9): the 2-body angle from the parent's flight direction with a `1/(z(1−z))` shape inside the cuts' energy window (the default on gluon/photon emissions, `soft-all` opt-in), and `--map-tau inverse-square` opt-in — note 37 |
| 5 | Cross-section integration + running couplings | ✅ Done | Leptonic `sigma_z_pole`/`sigma_qed_limit`; hadronic σ(pp→e⁺e⁻) via pure-Rust LHAPDF6 parser + log-bicubic interp and compiled MG run-card cuts, vs MG 0.14%/0.07%; MG's `αs` RGE + per-event `μR`/per-beam `μF` (`coupling/`); `vibegraph integrate` persists per-channel VEGAS grids in `IntegrateArtifact` (fv5: model identity + a per-channel subsampler summary). `lpp = 1` over an **arbitrary** process via `ProtonIntegrand` — measured flavour groups (pointwise \|M\|² + masses + `Cuts` + colour basis), both beam orderings by outgoing-leg reflection, `αs` off the PDF grid. σ gates: 17 partonic GATE rows incl. the 3 QCD 2→2s, `pp_to_bb_fixed` and all 4 llj subprocesses at the kT-clustered per-event scale, σ(pp→e⁺e⁻) on both dy13 cards through the *general* path (**933.905 ± 0.567** vs MG 933.230 ± 0.480; **644.203 ± 0.384** vs 644.330 ± 0.283), and σ(pp→ℓ⁺ℓ⁻j) fixed-scale **424.428 ± 0.432 pb** over three seeds vs MG 423.840 ± 1.518 (pull +0.37). At a *dynamical* scale each point's cluster scale is taken in the integration configuration drawn from the point's own squared amplitudes (`∝ AMP2_c/Σ AMP2`, MadEvent's enhancement-weight conditional, note 29 chain B): `gu_to_epemu` **+0.029%** (pull +0.13) / `gux_to_epemux` **−0.165%** (pull −0.70) and σ(pp→ℓ⁺ℓ⁻j) **+0.25%** (pull +0.73), all GATE at `rel_tol` 0.005 set by the references' own errors. The four `refdata-5` re-carded rows gate on the same path at their own reference-precision-sized budgets (addendum S6, note 32): `pp_to_bb` +0.05% at 75k, `pp_to_bb_qcd2` +0.01% at 75k, `pp_to_llj` +0.18% at 150k (its ladder still climbs monotonically across 75k–600k, 0.04%→0.21%, which is why it did not cut to 75k with the other three), `pp_to_ll_scalefact2` +0.02% at 75k. The `p p > j j` capstone runs the same path on the canonical QCD process and is **GATE**: **6.813339e8 ± 5.496e5 pb** over three seeds vs MG 6.788500e8 ± 1.473e6, rel **+0.37%** at pull **+1.58**, at `rel_tol` 0.005 (75k, addendum S6) — the reference's own 0.22% with headroom, pull asserted, since its channel-partition ambiguity is only `1.0e-3` (its own Monte-Carlo error, because a 2 → 2 gives the clustering no merge to choose). It sums over MadGraph's own 65 concrete assignments, pinned entry for entry against the run's `leshouche.inc` (all figures in this row measured fresh 2026-08-05 by the addendum close-out session, note 32 S8; run `pixi run --skip-deps validate-sigma` / `validate-hadronic` to reproduce) |
| 6 | Unweighted event output (LHEF) | ✅ Done | Accept/reject over the frozen per-channel grids (channel `∝ w_maxⱼ`, overweights kept at weight `>1` and counted), per-event helicity (`∝ \|M_hel\|²`) selection, colour selection via MadEvent's `SELECT_COLOR` rule (configuration `∝ AMP2_d`, flow `∝ JAMP2` inside its `ICOLAMP` row) with **per-member colour-flow tables** — each flavour member's tags derived under the structurally-determined flow permutation, refuse-on-ambiguity — checked against MG's `leshouche.inc` (73/73 concrete subprocesses over 47 files; note 29 chain A), `SCALUP`/`AQCDUP` from `coupling::scales`, four-layer `lhef/` writer/reader that re-serialises all 37 banked MG runs byte-for-byte (744 759 events, both of MadGraph's serialisation dialects, source-text pass-through by construction). `vibegraph generate` refuses mismatched cards/models, swappable weight strategy (`Buffer` `IDWTUP=-4` / `StochasticRounding` `+3`). `lpp = 1` gated: `validate-generate-proton` takes the llj cards to a `.lhe` (flavour draw ∝ per-group luminosity × σ̂, sample σ within `SIGMA_MAX_REL = 0.015` of the banked run). `p p > e+ e-` reaches an event file too, on the same general path. Pythia 8.312 reads both emitted samples back end to end (2000/2000 each, colour-mutation negative control rejected). Event samples are compared against MadGraph's banked ones column by column (`samples` category: weighted-ECDF KS on the kinematics, chi-squared on `SPINUP`/`ICOLUP`/flavour) |

## Closed-sprint history

At most three lines each; the note is the full record. Earlier sprints
(`helas-generalize`, `mg-validation-coverage`, `cleanup-refactor`,
`performance-sprint`) live in git history and notes 12/13.

- **`color-flow`** (feature, 2026-07-12; note 16) — multi-flow JAMPs and an
  exact CF |M|², with the VVVV phase bug and the fermion-flow slot swap
  debriefed at the note.
- **`validation-sprint`** (validation, 2026-07-13; notes 12/16) — `gg_to_gg`
  enforced at NCOLOR=6 and the VVVV `−i` fixed.
- **Eval performance program** (performance, 2026-07-17; note 15) — the vs-MG
  evaluator gap closed from 8.6×–110× to 1.2×–3.5×. Standing contract: pruned
  evaluators require partonic-CM, beams-along-±z momenta.
- **`hadronic-xsec`** (feature, 2026-07-19; note 18) — PDF convolution,
  run-card cuts and a (τ,y) VEGAS; σ(pp→e⁺e⁻) within 0.14%/0.07% of MadGraph.
- **`validation-2`** (validation, 2026-07-21; note 19) — NHEL pinning, a
  σ-level gate, the PDF seam and rooting soundness (all signs lifted to
  `fermi_sign`); V7 deferred to the coverage backlog.
- **`eval-perf-2`** (performance, 2026-07-21; note 20) — mul-split, one-shot
  DAG validation, ZEROAMP skipping: `forward` 1.18×–2.19× on every process.
- **`resonance-sampling`** (feature, 2026-07-26; note 21) — MadGraph-style
  multichannel in production, two resonant σ rows SKIP → GATE. Lesson: a
  fixed-seed pull cannot validate a sampler, so seed sweeps are part of a gate.
- **`dynamical-scales`** (feature, 2026-07-27; note 22) — MadGraph's `αs` RGE
  and per-event `μR` / per-beam `μF` through the constant pools; found MG's
  `AQCDUP` π-truncation and `SCALUP` ≠ μR defects (note 07).
- **`event-output-lhef`** (feature, 2026-07-28; note 23) — JAMP2 flow
  selection against `leshouche.inc`, accept/reject unweighting, a byte-pinned
  LHEF writer/reader and `vibegraph generate`.
- **`user-distribution` + `proton-events`** (feature, 2026-07-31; note 24) —
  `ProtonIntegrand` with σ(pp→ℓ⁺ℓ⁻j) and `generate` gated at `lpp = 1`, plus the
  release/CI/acceptance workflows. Lesson: a seed sweep is necessary and not
  sufficient — budget convergence is a second axis.
- **`validation-3`** (validation, 2026-07-31; note 25) — three declared
  dependency layers with `validation/manifest.toml` as the single source of
  truth, plus the `samples` category. Lesson: a report is only evidence if
  every green cell is a recorded measurement.
- **`v3-backlog`** (validation follow-up, 2026-08-01; note 27) — every
  register finding resolved rather than tolerated (the h→ττ pole was
  MadGraph's own `get_channel_cut` defect), references re-banked on 3.7.1.
- **note-29 validation sprint** (validation, 2026-08-03; note 29) — per-member
  colour-flow tables, MadEvent's per-point `AMP2_c` scale-channel draw, and a
  hard error on every descoped card surface. Lesson: pre-register the may-move
  set, so a large diff landing exactly on it is auditable at a glance.
- **`kt-spine`** (feature, 2026-08-02; note 28) — MadGraph's general kT
  clustering reproduced merge for merge, the multi-rung t-channel spine in
  production, and the `p p > j j` capstone gated. Lesson: a per-event field is
  a finer oracle than a cross section, and it exists more often than it looks.
- **`perf-sprint-3`** (performance, 2026-08-05; note 31 §6) — eleven sessions
  across integration, PDF and evaluator: `validate` 691 s → 391 s, census
  unchanged. Lesson: four sessions refuted their own brief's mechanism and
  still delivered, each pre-committed to a killing measurement.
- **`perf-3-addendum`** (performance/validation cleanup, 2026-08-05; note 32
  §5) — `w_max` read off MadGraph's own truncation-ladder rule, five σ budgets
  sized to reference precision, the 2→6 rows turned on as `info`; S9 killed
  clean. Lesson: every session corrected something in its own brief.
- **`draw-followup`** (four sessions, 2026-08-07; note 34 §2) — α-survey cap
  confirmed, the llj σ-ladder climb shown to be a misread, an acceptance-scaled
  accepted-point floor landed. Lesson: the at-threshold gate statistic is a
  class, not an incident — match the statistic to its calibration, never widen.
- **`draw-performance`** (two sessions, 2026-08-06; note 34 §1) — the mixture
  density priced only after the cut (2→6 per-point 13.7×/10.1×) and cut-implied
  timelike floors on every drawn invariant. Lesson: the falsifier/payoff split
  earned its pre-registration — the falsifier passed while a payoff failed.
- **`convergence-and-2to3-abort`** (two sessions, 2026-08-06) — the 2→3 QCD
  abort fixed by a scale-relative momentum guard, and `--target-rel` stopped
  consuming the wide-split χ²/dof overflow. Lesson: the session falsified both
  its brief's hypothesis and the measurement note's own caveat.
- **`logging-tui`** (feature/UX, 2026-08-06; note 33) — `tracing` through the
  library with the stdout result contract frozen byte-for-byte at every
  verbosity, an inline ratatui viewport, and a two-stage `q`/`^C` stop.
- **UX mini-sprint** (2026-08-07) — the VEGAS bar tracks distance to the
  convergence target, the gauge carries elapsed and estimated-remaining, and `-`
  reads a proc card from stdin. Trap: `tracing`'s process-wide callsite interest
  cache poisons thread-scoped test subscribers under parallel tests.
- **`banked-open-ends`** (validation, eight sessions, 2026-09-07; note 36) —
  new incoming-leg, `SCALUP`/`AQCDUP`, coupling-level and channel-count oracles;
  census 176/166✅/10⚠️ → 178/171✅/7⚠️. Lesson: build the oracle before the fix
  and let it be wrong on purpose — each landed known-red and caught something
  its fix's brief had not named.
- **`splitting-kernel-sampling`** (feature/performance, 2026-09-07 → 09-23;
  note 37) — the user's observation that the 1→2 split is drawn isotropically
  while a splitting kernel goes like `1/(z(1−z))`, plus a survey of every
  MadEvent map. `--map-split-angle`/`--map-tau`/`--map-rung-order`, `auto`
  arms measured at ≥ 20 seeds: `soft-all` halves `p p > l+ l- j`'s
  evaluations (0.50 ± 0.02 — the lever is the lepton pair, not a gluon) and
  `1/τ²` cuts dijets to 0.77 ± 0.03; both opt-in until one gate cell each is
  decided (validation backlog). Lesson: three of the first session's
  three-seed readings flipped or vanished at twenty, and a σ pull that grew from 1.6 to
  2.2 with seeds fell to 0.4 at 160 — the growth-with-statistics test needs
  the statistics.
- **`ufo-lorentz`** (feature, fourteen sessions, 2026-09-07; note 35 §10) — the
  UFO surface past the Standard Model gated end to end (SMEFTsim plus two
  authored toy models; 29 rows → 51). Lesson: a toy model is a validation
  instrument, not a convenience — the rows nobody would generate for their
  physics are what separated conventions every SM row agreed on.

---

## 🔎 Validation backlog

### Standing findings to diagnose (from the note-29 sprint; never a loosened tolerance)

- **`pp_to_llj_dyn` scatter guard under `--map-split-angle soft-all`** (note
  37 §6.3; the user's decision). As the default, `soft-all` halves llj's
  evaluations but takes this cell's five-seed `χ²/dof` to 4.17 against 4.0.
  Forty seeds at the gate's configuration read 0.91 under both maps
  (`probe_llj_dyn_scatter_guard_calibration`), 1 of 8 quintets above 4.0
  under `soft-all` against 0 under isotropic, the one being the gate's own; a
  seed that reads low under every map drives it. Flipping the rule needs this
  guard matched to its calibration first — never a higher limit.
- **`pp_to_jj` samples flavour χ² under `--map-tau inverse-square`** (note
  37 §6.3; the user's decision). MadEvent's `τ` rule measures 0.77 ± 0.03 of
  the evaluations on dijets, but as the default it takes this cell's flavour
  χ² against MadGraph's banked events to p 2.8e-5 on one seed (floor 1e-4;
  three-seed sum 325 / 212, against 267 / 210 under the log map). Our own two
  maps agree at χ² 40.5 / 47 on 200 000 events each, so the map does not move
  the composition; the cell reads a fluctuation against a reference that
  already sits high. The `τ` rule flips once the cell's statistic is matched
  to its calibration — a larger generated sample, or a pooled-seed statistic —
  never a lower floor.

- **A nondeterministic heap-corruption abort under the proton-sample suite's
  concurrent load** (observed 2026-09-07, 1 run in 7; not caused by the sprint
  that saw it). A `vibegraph integrate` child on `pp_to_llj_dyn`'s card aborted
  in VEGAS warm-up with macOS libmalloc's "pointer being freed was not
  allocated", the crashed thread dropping a `Vec<f64>` inside `kt::Clustering`
  on a rayon worker while another worker was in `cluster::kt`/`cluster::graph`.
  Ruled out: `unsafe` in the workspace (none), a non-`System` allocator, and
  any local difference in `coupling/cluster/**`, `hadronic.rs`, `proton.rs` or
  `vegas` against `main`. Unreproduced in every targeted rerun since, including
  ~12 full `validate` runs at host loads up to 84. Crash report preserved at
  `~/Library/Logs/DiagnosticReports/vibegraph-2026-09-07-085712.ips`. Anyone
  reading a gate under this suite should rerun once on an abort and report both
  outcomes rather than treating one run as the measurement.

- **`ee_to_mumua` residual: a fixed +1.04% ours-high** (five seeds, χ²/dof
  0.50) in the radiative-return `pt(γ)` windows — the one thing chain D's D1
  verdict left unattributed, localised to MadGraph's `pt(γ)/η(γ)` coverage
  rather than to either matrix element or the Z propagator. The named next
  probe is the 2D `[39.4, 77) × [86, 96)` `(pt(γ), m(μμ))` cell. Both affected
  cells carry the pre-registered prescriptions and no threshold was moved: the
  `samples` cell is `info` with its measurement in the manifest note, and the σ
  pull is reported-not-asserted with `rel_tol 0.03` still enforced. Both re-arm
  as enforcement when this diagnosis lands.
- **`p p > j j`'s across-group scale spread is `4.999999e-7`, not zero**,
  while every within-group spread is exactly `0.0` (chain B-0's census).
  Nothing in production reads the group axis for scales, so it moves no cell —
  but it is far too large to be rounding on a 2→2 whose scale ought to be
  group-independent, and it is the size of the effect a future group-axis
  change would expose. Worth one look at where the group enters.
- **Bank `p p > j j j` and the 2→3 QCD partonic rows** — MG reference
  generation plus manifest rows, none of which exist; the processes became
  reachable when the momentum guard in `run.rs`'s `Op::Add` arm was made
  scale-relative (`abedb81`). Two facts for whoever takes it: all four 2→3
  reproducers hit the convergence cap with per-iteration χ²/dof 2.5–8.2 — the
  2→6 heavy-tail pathology one multiplicity lower and far cheaper to study, so
  budget a ≥5-seed sweep and never a single run; and the guard's measured
  residue distribution (max 1.4 ulps over 1088 sums) is in `abedb81`'s tests
  if its tolerance is ever revisited.
- **`wpwm_to_wpwmz_cw`'s diagram pairing, and the five-vector residual behind
  it** (note 35 §10.5). The row's `amplitudes` cell reports |M|² 2.20e3 rather
  than "no comparison", but the configuration partition itself is still
  uncompared, and the cheap route to it is falsified: the two partitions agree
  on the multiset of group sizes and on nothing finer, ours being the
  contiguous diagrams 62–221 against MadGraph's 2–186 with three gaps, so no
  index shift maps one onto the other. With 222 graphs no per-diagram table is
  banked, and a pairing written to satisfy the one check that reads it would be
  fitted to it. The honest route is MadGraph's own per-diagram cluster trees in
  `output/wpwm_to_wpwmz_cw.json`, and it belongs with whichever session takes
  the five-vector structures on — that is what the |M|² disagreement is about.

- **Open ends the `banked-open-ends` sprint left (2026-09-07, note 36 §7)**:
  - **`gg_to_gg_cg`'s σ is a converged −0.22% offset** (five seeds, χ²/dof 1.01,
    against a reference error of 8.5e-4): not the scale formula (the replay is
    at 0.999 of budget) and not the process (`gg_to_gg` under the card differing
    in that one field sits at +9.8e-6), but localised to the coupling this
    `SCALE_FALLBACK_ROWS` member runs at across the cut region rather than on
    MadGraph's kept events. Attribute before gating.
  - **Every σ calibration comment written before `e73b158` is stale** — the
    note-34 draw-performance commits moved the sampling streams and the older
    five-seed figures were never re-recorded (the thirteen written at `e73b158`
    reproduce to the digit; note 36a lists the rest). A mechanical re-recording
    session; falsifier is one run of `probe_gate_row_seed_headroom` at
    `f85718d^`.
  - **Two tolerance cells under 2× headroom by budget, not threshold** —
    `ddx_to_epemg` (1.6×, a converged +0.45% offset with a reference-bounded
    pull) and `gux_to_epemux` (1.9×, one-seed scatter that 4× budget takes to
    1.4e-3). Both buy margin with points: a budget decision, not a tolerance
    one. The two thinnest cells beside them are `ll_to_qqx_toy_tensor` σ (2.2×,
    χ²/dof 2.77, its channel set collapsed 3 → 1) and `ee_to_wpwm_cw` samples
    (KS p 2.4e-4 against the 1e-4 floor).
  - **The merged configuration's forest is not compared to `configs.inc` on a
    merging run** — the channel count is (against `coloramps.inc`) and the
    partition is (`amplitude_oracle` against `matrix1.f`), but
    `validate_kt_cluster`'s forest oracle covers only non-merging runs. Extend
    `derived_channel_forests_match_the_generated_ones` to the merging
    single-subprocess runs now that both sides have one channel count. The
    representative diagram is ours-first where MadGraph's is its-first; the two
    coincide wherever `MG_DIAGRAM_ORDER` is the identity and nothing measures
    the rest (a sampling-efficiency difference at most, never a wrong answer).

### Sharper oracles the sprint named but did not build

- **Expose the drawn scale configuration and close chain B's two accepted
  gaps**: assert the `∝ AMP2_c/Σ AMP2` frequency law end-to-end (today it is
  factored into four independently-gated pieces — `select.rs`'s binomial
  test, the order pin, the colour-draw χ², and σ), and promote the
  zero-spread census to a banked assertion on the cheapest inert rows so a
  future change that makes a scale configuration-dependent on a declared-inert
  row fails a standing gate instead of a one-time manual diff. Blocked on
  nothing but runtime cost for the latter.
- **`k`/`G` measured exactly ±1 (and the per-process route-sign patterns)** —
  chain F's settled leads: one `|Im(k/G)|` assertion converts free phases to
  pinned bits, with no reference data and no tolerance move; and
  `run_config_amps()[i]` disagrees in sign with the single-diagram compile on
  three processes (exactly ±1, spread 0, production evaluators, no production
  consumer since `eval_amp2` is sign-blind). Any future assertion must pin
  per-process sign patterns, not uniformity.

### Deferred coverage

- **`nhel = 1` run cards are refused**, so `wpwm_to_wpwmz_cw`'s `integrals` and
  `samples` cells are `uncovered`. MadGraph chose helicity importance sampling
  for that 222-diagram process itself. The refusal is correct — it changes the
  estimator and the per-event weight — but it means the reference's σ and events
  are unreachable from the run card that produced them. That row's amplitudes
  disagree grossly in any case (|M|² 2.20e3), so both cells would have been
  informational.
- **No `samples` cell for a 2 → 1 row, and none possible** — `bbx_to_h_identity`,
  `gg_to_h_cpeven` and `gg_to_h_cpodd` have no phase-space volume, so MadEvent
  banked neither a σ nor an event file. Both cells are `uncovered` with that
  reason (they previously read `covered-by = ["ee_to_ttx_smeft"]`, which was
  wrong: a `2 → 1` amplitude is not covered by a `2 → 2` cross section).

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
- **Flavour-group probe coverage** — `derive_flavor_groups` partitions on
  sampled `|M|²`: complete but unsound whatever the probe set, since two
  subprocesses differing only where the probe does not look are merged silently.
  The probe ladder is hardened (five rungs down to a fifth of the base energy
  and onto the `Z` mass; closest-pair separation measured at 0.74, asserted
  > 0.1) and the sound replacement is the s-expression criterion in the feature
  backlog. Accepted for v1 on the MG-helicity-filtering precedent.
  (`proton.rs`, note 24 §P2c.)
- **Pythia consumption gate — what it cannot see.** The gate reads both emitted
  samples n/n and its negative control proves it is not colour-blind, but four
  things stay outside it. (a) Only the `Buffer` strategy (`IDWTUP = -4`) is fed
  to Pythia; `StochasticRounding` (`+3`) writes a different `<init>` header and
  is unexercised. (b) The reconstruction check compares the multiset of outgoing
  PDG codes, so a permuted or corrupted momentum would be consumed silently —
  comparing Pythia's `process` four-momenta against the record's would close
  that. (c) The negative control mutates `ICOLUP(1)` only, on one event. (d)
  Nothing checks Pythia's interpretation of `SCALUP`, `AQCDUP` or the `<init>`
  cross section: the file is proven readable, not proven to mean what we meant.

### Gate + tooling hygiene

- **`acceptance.yml` has still never passed**. It needed the repo to be public,
  which it now is (2026-09-25); the next run is the first that can pass
  — `v0.1.0` published all four binaries, but acceptance 404s on
  `releases/download/...`: the script downloads unauthenticated *by design*, so
  it can reproduce on a clean VM with no checkout and no token, and a private
  repo serves 404 for that. Nothing is wrong with the release itself (the macOS
  asset was verified by hand against the published `SHA256SUMS`). Going public
  is the fix; an authenticated fallback was considered and declined to keep the
  script checkout-free. The release event alone will not start the workflow —
  see the `workflow_dispatch` step in `release.yml` — and until it runs the gate
  is unproven, so the first public run is the one to actually read.
- **Weekly `schedule` trigger on `acceptance.yml`** — left off because it can
  only fail until the job can pass at all (above). Turn it on after the first
  green run: it is also the second detector for the "CERN repackages the PDF
  archive" risk, whose only other detector is an `#[ignore]`d test nobody runs
  on a timer. (Note 24 §U2.)
- **Small hygiene left named by past sprints**: the `blocked` tier is a
  documented manifest schema slot used by nothing (keep or retire — a schema
  decision); the direct-vs-mirror ordering is a third partition axis chain B
  named with a falsifier but nothing measures; chain B's draw raises
  low-budget seed scatter (χ²/dof 6.38 at 75k, clean ≥150k), so a future
  budget reduction on `pp_to_llj_dyn` would bite; the `Opaque` run-card
  default payload fix (note 28 §C2.5).
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
- **`validate_hadronic.rs` carries pre-timelike-floor ladder figures in
  load-bearing doc comments** — the `LLJ_NEVAL` budget rationale,
  `LLJ_DYN_MAX_REL`, the `LLJ_MAX_CHI2_PER_DOF` calibration family and
  `measure_llj_dyn_sigma`'s note all quote pre-floor five-seed ladders. The
  constants and the gates are sound; the rationale numbers are stale.
  Mechanical re-record: re-run `probe_llj_fixed_budget_ladder` /
  `probe_llj_dyn_budget_ladder` on a quiet host (~357 s each on 16 cores) and
  rewrite the four comment sites — quoting the measured per-seed spread
  alongside any five-seed rung figure, since five-seed scatter understates this
  row's spread 2–5×.
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
- ~~Decay-chain process syntax and 1→n decay processes~~ — brought into scope
  2026-09-25; see `process-grammar` below.
- **Custom UFO propagators** (`propagators.py`, UFO 2.0) — parse the file and
  thread the propagator forms through the HELAS compiler.

### In-scope features

- **`process-grammar`** (feature, next; note 38) — MadGraph LO process parity
  without MLM and NLO. Sessions:
  - **G1** ✅ landed (`34d6d45`, `1f5f924`, `f67f787`): full parser, the one
    `check_supported` scan and the narrowed `SupportedCard`, gated against
    MadGraph's own parser. Closes every silent row in note 38 §2.
  - **S1** ✅ landed (`93fff0f`, `2d99872`; note 38 §4 S1 Landed): `Diagram` carries the
    full Fermi sign, `Diagram::anchor` replaces `VtxIdx(0)`, and `Diagram::canonical`
    gives container equality. Two findings remain open: the all-vector contact sign
    (wrong for `g g > g g g`), and the `u u~ > t t~ g NP<=1` four-quark mismatch.
  - **S2** ✅ landed (`c52e4f7`, the docs commit after it): `>`/`$$` filters inside the
    WEIGHTED search, `WEIGHTED<=n`, the five-flavour `p`/`j` rewrite, MadGraph
    census (43/43 generated cards) and two σ rows in agreement.
  - **D1**: 1→n decays, gated on partial widths.
  - **D2**: decay chains stitched from separate enumerations, gated on
    container equality against the filtered full final state.
  - **D3**: forced Breit–Wigner windows, decay-chain σ, and the sampler
    leg-count ladder that decides whether a MadSpin-style step is ever needed.
  - **S3**: `$` as a pointwise SDE-weighted integrand.
  - **P1** ✅ landed (`3b3f71e`, `3c023b2`; note 38 §4 P1 Landed): polarized external legs, NHEL/IDEN census against MadGraph (35 cards), six gated amplitude rows, `me_frame` consumed with a boosted-frame mutation pin, σ(`e+ e- > w+{0} w-`) +5e-4 over five seeds.
  - **E1**: status-2 resonance records, `@N` → `LPRUP`, and `add process`
    grouping.

  Polarized intermediate resonances (a change to the propagator in
  `helas/eval`) stay refused, with their own entry below.
- **Squared-order constraints** (`QCD^2==2`, `NP^2==1`; shelved, user
  2026-09-25). They need complex amplitudes grouped by coupling order, which is
  also what reweighting in a coupling would use. That is a sizable `helas/eval`
  refactor, deferred until the open evaluator performance PRs land. G1 parses
  them and `check_supported` refuses them.
- **Polarized intermediate resonances** (`p p > w+{0} w-, w+ > e+ ve`, and the
  propagator codes `{A}`/`{G}`/`{H}`/`{Q}`/`{W}`/`{S}`). A helicity-projected
  propagator numerator in `helas/eval`. Refused by `check_supported` until then
  (note 38 §4 P1).

- **`madgraph-style-enumeration`** (research, unscheduled) — feyngraph
  enumerates topology-first (QGRAF-style orderly generation, then particle
  assignment by backtracking), while MadGraph 5 recursively combines
  external-leg subsets through the vertex table, so it never visits a shape the
  model cannot fill and prunes coupling orders in-recursion (arXiv:1106.0522).
  **Question to answer before any code**: is enumeration ever on the critical
  path here? Measure feyngraph's share of `integrate` wall time on the widest
  cards against MadGraph's own generation time for the same cards. If it is,
  the design spike is a leg-combination enumerator over `ufo::topo`'s vertex
  table producing `diagrams::Diagram` unchanged (same slot-ordered rays,
  routing, Fermi sign, symmetry factor), a canonical diagram tag for duplicate
  elimination, the WEIGHTED bound applied in-recursion, and subprocess reuse
  across flavour relabellings. Gate: an identical diagram census against
  `validation/madgraph/diagrams.json` and byte-identical amplitude-oracle rows,
  since a diagram set differing only in ordering changes the rooting and
  therefore the arithmetic. `docs/src/guide/03-diagrams.md` records the
  algorithmic contrast.

- **`vibegraph enumerate`** (feature, user request on PR #4) — a command that
  takes a process card and reports every diagram that contributes: SVG
  drawings, a summary page (diagram counts per subprocess, coupling orders,
  the flavour groups), and a binary artifact `integrate` accepts in place of
  re-enumerating. MadGraph's `display diagrams` is the workflow: check that a
  card means the intended process and nothing more before spending an
  integration on it. The artifact half also closes the
  "bundle the compiled program" piece of the self-contained-artifact item
  below, since the enumerated diagrams are its input. feyngraph's `drawing/`
  module is a candidate for the drawings.
- **`--madgraph-compat` mode toggle** (feature, user request on PR #4) —
  several sites reproduce a MadGraph choice a clean-sheet design would not
  make; today they are unconditional, so the cleaner behaviour is unreachable
  and its cost unmeasured. One flag, default on (every banked gate depends on
  the compatible behaviour), recorded in the integrate artifact and the LHEF
  header so a file says which mode produced it. Sites, each carrying a
  "MadGraph compatibility" admonition in the docs: (a) `coupling/cluster/kt.rs`
  — the `1 + 1e-6` crossed beam–leg inflation, which does not cancel when every
  admissible candidate is crossed and so leaks a part in 1e6 into `SCALUP`, plus
  the first-pair-in-visit-order tie-break; off would use an inflation-free
  measure and a tie rule that cannot enter the value. (b) `coupling/alphas.rs` —
  every fixed "magic" coefficient of the evolution is a compatibility site: the
  threshold masses `CMASS = 1.42` / `BMASS = 4.7` / `ZMASS`, the `TOL = 5e-4`
  Newton stop (a specific iterate, not the root), and the β-function constants
  as transcribed; off would take the model's own quark masses, recompute those
  constants rather than carry them as literals, and run a proper ODE solver to
  convergence. (c) `lhef/mod.rs` + `lhef/write.rs` — the Python post-processor
  column layout and the two-dialect re-emission that keeps a Fortran-dialect
  file's seven significant digits on the scale and coupling columns; off would
  write one layout at full precision. Verified non-sites, to leave alone:
  `AQCDUP` is already written untruncated, the jet-count memo is already not
  carried across events, `SCALUP = max(μF)` is the accord's own definition, and
  the truncated `w_max` rule is an improvement rather than a concession. Gate:
  flag on, every banked byte and σ gate unchanged; flag off, a documented
  per-site delta table, and no validation gate runs in the off mode.
- **`reweight_card.dat`** (feature) — re-evaluate a stored event sample under
  alternative coupling values, MadGraph's reweighting workflow. The monomial
  exponent analysis in `helas::eval::rescale` is written for a generic model
  parameter `G` precisely so that moving the pools to a new value is one
  multiply per entry per event; a reweighting pass is that analysis over the
  card's requested parameters plus the per-event |M|² ratio written back as an
  extra weight (LHEF `<rwgt>` block). Parameters entering couplings other than
  as monomials fall back to the exact re-evaluation path automatically.
- **Direct-threaded interpreter dispatch** (performance, on hold) — the
  evaluator is a switch-dispatch interpreter: one `match` per instruction,
  whose indirect jump is what the op-blocked schedule (note 31 E1b) exists to
  make predictable. Direct threading — each handler tail-calling the next —
  gives the branch predictor one site per instruction kind and is the classic
  next step; in safe Rust it needs guaranteed tail calls, i.e. the nightly
  `become` feature, so it waits on that stabilising. Function-pointer threading
  was measured and rejected (+7.7%, note 31 E2), so the win, if any, is in
  the tail-call form specifically.
- **Alternating α / grid refinement** (research) — the Kleiss–Pittau
  α-adaptation runs on a survey before the per-channel grids train, and the α
  then stay fixed. An alternating scheme — train the grids with α fixed,
  re-derive α from the trained grids' variance shares, retrain, as in an
  expectation-maximisation loop — might converge to a lower-variance mixture
  than the one-shot survey. Measure offline first from recorded `g_j(x)`,
  `f(x)` on existing samples: the variance the alternation would reach against
  the points it costs, and whether it oscillates. Read the result against the
  estimator's *measured* seed spread (20+ seeds on both arrangements), since the
  α update is itself a survey estimate over a Pareto weight tail of index ≈ 2.
  Pre-registered failure criteria: α cycling rather than converging; a win
  inside the seed spread; any channel's reallocation falling below its coverage
  floor. Guardrail as everywhere in the multichannel: an α floor, never a
  coverage split.
- **VEGAS+ adaptive stratification** (research) — the integrator is classic
  Lepage importance sampling; VEGAS+ (arXiv:2009.05112) adds adaptive
  stratified sampling within the grid and reports 2–19× on integrands with
  multiple peaks or diagonal structure. The channel decomposition handles the
  diagonal structure and the budget is already stratified across channels,
  but whether within-channel stratification still buys convergence on these
  integrands has never been measured. Measure on the σ gates' rows at matched
  points (seed sweep, χ²/dof) before deciding; note that stratification
  changes the sampling order, so it cannot be bit-for-bit against banked
  artifacts.
- **|M|² by term rewriting** (research) — the helicity-summed |M|² is
  algebraically a sum over helicities of a current chain times its conjugate;
  completeness relations replace the external helicity sums by `p̸ + m` /
  `−g^{μν}` insertions and trace identities reduce the closed fermion lines to
  scalar products of momenta. An e-graph seeded with those identities (the
  `helas::eval::egraph` seam) could extract a specialised |M|² program with no
  helicity loop at all, kept beside the per-helicity program event generation
  needs. The same explicit-invariant form is the natural input to a phase-space
  map derived from the integrand's own structure rather than read off
  propagator poles. Both are gated on the extraction prerequisites note 15 §4.1
  lists.

- **s-expression program identity for flavour grouping** — a dedicated future
  sprint, user-scoped. `derive_flavor_groups` partitions subprocesses by sampled
  `|M|²` agreement: **complete but unsound**, since two programs differing only
  where the probe does not look are merged silently. **Accepted for v1** (user,
  2026-08-02) on the MadGraph precedent — MG's own helicity filtering drops
  vanishing configurations on the same sampled-probe basis — with the probe
  ladder hardened. The sound replacement is: two subprocesses share a group iff
  their compiled programs are identical as s-expressions. Three prerequisites,
  in order: (1) **universal constant ids**, comparing UFO-stable
  coupling/particle identities and never per-compilation pool slot indices,
  since flavour-dependent couplings can share a slot and slot equality would be
  unsound — the exact failure the new criterion exists to remove;
  (2) **canonicalization of the un-optimized s-expression**, because lowering
  carries a ±1-CSE-node nondeterminism and diagram order is unstable;
  (3) **colour folded into the s-expr language**, so the basis is part of the
  compared term. Being conservative it can only *split* genuinely-equal groups,
  costing compiled programs and never correctness. Keep the sampled criterion as
  an independent cross-check when it lands: a disagreement is a finding.
  (`proton.rs`, note 24 §P2c.)
- **Streaming `IDWTUP = -4`** by deterministic two-pass replay — the interface
  hook (`EventSource::restart`) is in place and contract-tested; not needed while
  100k-event runs buffer in ~42 MB. (Note 23 close-out.)
- **Massless-t-channel fiducial cut** (sprint plan: note 28 §S2/D3) — a
  massless beam pins `t_max = 0` (collinear edge) where the t-map falls back to
  flat; whether a fiducial cut is wanted instead is unresolved for a physical
  massless-initial-state t-channel. (Note 21 close-out.)
- **Re-examine the "no spine without a scale past two outgoing legs" policy** —
  it predates the peripheral-kinematics conditioning fixes (grouped Källén,
  `γ = E/√s`), which removed most of the unregulated-spine defect it guarded
  against: with the grouped form the massless transfer edge is the exact
  analytic zero whenever the emitted subsystem carries a fixed invariant, and
  only composite emitted sides still show the defect. The conservative fallback
  is kept; whether it is still the right default is an open measurement.
  (Note 28 §S3 deviations.)
- **Squared-order constraints (`NP^2==1`, interference-only |M|²)** — the
  grammar parses `^2` and `selector.rs` treats it as an amplitude order;
  MadGraph's per-order splitting of |M|² is a separate feature. Kept out of the
  `ufo-lorentz` sprint (note 35 §7 D4): every SMEFT row there compares the full
  |M|² at `NP<=1`, which MadGraph computes identically.
- **Spin-2 externals and propagators (UFO spin code 5), spin-3/2, Majorana
  fermions / `C`** — deferred from `ufo-lorentz` (note 35 §7 D2): the tensor
  type there is a Clifford-algebra element in the graded Dirac basis, so a
  spin-2 polarisation tensor is a separate type for a later sprint, and Majorana
  is fermion-flow machinery of its own (MadGraph itself refuses Majorana
  fermions in four-fermion vertices).
- **`typed-units`** — research `uom`/`dimensioned`/`units` crates for typed
  four-momenta and cross sections.
- **Self-contained `generate` artifact** (user, 2026-08-02; post-v0.1) — one
  file a clean worker machine can sample from. Today a proton-beam worker needs
  the binary, the artifact, both cards and the PDF set (unweighting reads
  densities and grid-αs per trial point), and a non-SM run needs its UFO
  directory too. Three pieces taken as one feature: (1) **bundle the compiled
  program** — design in note 23, keyed `(model digest, process, compiler schema
  version)` off fields already banked, with the recorded obstacles being no
  serde in `helas::eval`, `folded_hel`'s lazy `OnceLock` over a large expanded
  arena, and `prune_zero_helicities`' kinematic contract needing a recheck on
  load; (2) **bundle the PDF data the run reads**, either the member's grid file
  verbatim or a subgrid slice pinned to the run's (x, Q²) support — which, is
  part of the design — keeping the artifact's refuse-on-mismatch property;
  (3) **investigate compactifying the VEGAS grids**, which dominate artifact
  size on multichannel processes (quantization, sparser binning, shared axes
  all unexplored).
- **Quality sprint: tighten the `pub` API surface** (user, 2026-08-02) —
  before any backwards-compatibility promise (i.e. before 1.0): audit what
  `vibegraph-lib` exports, demote what only the CLI and the validation crates
  consume, and decide what the supported library surface actually is. Until
  then releases stay on the 0.x line (first tag `v0.1.0`).

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

**Rewritten from measurement 2026-09-07 (note 35 §10).** Two non-SM models are
loaded end to end and gated against MadGraph — the vendored SMEFTsim
`topU3l_MwScheme` and two authored toy UFOs — so "model-generic" is no longer
exercised on Standard-Model evidence alone. What that retired from this list is
recorded at note 35 §10.1, entry by entry with the row that gates it: the
coupling-order bundling and restriction semantics, the parser surface,
tree-shaped Lorentz primitives, four-fermion vertices, the literal `Sigma` in
every position it can occupy, colour sextets and baryonic epsilons, the
symmetric structure constant `d(a,b,c)`, and `IdentityAmp`/`Gamma5Amp` process
coverage. What is left below is what still refuses, and why.

**Still refused, each deliberately and each with a reason**:

- **Spin codes beyond {1, 2, 3}**. `helicity_states_for_spin` accepts the spin-2
  code (5) but nothing downstream builds a tensor external wavefunction or
  propagator; spin-3/2 (code 4) is an `UnsupportedSpin` error. Descoped from
  `ufo-lorentz` by decision (note 35 §7 D2): a symmetric Lorentz tensor is a
  different object from the antisymmetric grade-2 slice the sprint built.
  Ghost codes stay irrelevant at LO.
- **Majorana fermions and charge conjugation**. Fermion-flow handling assumes
  Dirac-continuous lines end to end — no flow-flip machinery, and the UFO `C`
  operator is unrooted. Descoped by the same decision; MadGraph itself refuses
  Majorana fermions in four-fermion vertices. Classically subtle sign territory,
  and the `color-flow` slot-swap bug shows how delicate flow conventions are
  even pure-Dirac. This is what keeps `vibegraph_toy_color_UFO` all-scalar: two
  same-representation fermions reach a diquark only through a
  fermion-number-violating vertex.
- **A `T6` carrying adjoint indices**. The sextet generator's expansion draws
  fresh summed indices from a module-global MadGraph counter; the algebra is
  unit-tested (`δ6(i,i) = 6`) but no banked row carries a sextet `Identity`, so
  the crossing rule for `T6` is unpinned and the case is refused rather than
  guessed.
- **An external sextet, or any basis key in which a baryonic or sextet tensor
  survives** — three colour indices tied at a point, or two colour lines on one
  leg, which no Les Houches record can write. Both gated rows keep their diquark
  internal, so their flow tags are ordinary triplet lines. `order_summation` is
  not ported either; it is a no-op while `K6`/`K6Bar` reduce away and would be
  needed for an external sextet.
- **Squared-order constraints** (`NP^2==1`) — a hard error, descoped by decision
  (note 35 §7 D4); every SMEFT row compares the full |M|² at `NP<=1`.
- **Loop-level UFOs** (`loop_sm`, NLO models) — out of the LO charter (parser
  history in note 04).

**Open questions the sprint left explicit, none of them a wall**:

- **A vertex mixing a Dirac-matrix bilinear with a matrix-free one** trips
  `carries_dirac_matrix`'s uniformity assertion. No model in the tree does it,
  and which vertex on such a line would own the reversal factor is a question no
  oracle in the suite resolves.
- **A same-flavour four-fermion process** (`e+ e- > e+ e- NP<=1`) enumerates one
  diagram per pairing where MadGraph draws one — the `gg_to_gg_cg` 21/27
  counting-convention class. An `ee_to_ee_4f` row would gate it.
- **MadGraph fixes a restrict-card parameter set to exactly `1`** alongside the
  zeros and this loader does not. Latent: no card in the repository uses `1.0`.

---

## ⚡ Performance backlog

- **Absolute grid coordinates** (note 37 §5.2, user-requested option). Bin
  each invariant's VEGAS coordinate on the absolute `s/s_tot` / `−t/s_tot`
  scale with the draw restricted to the point's window, as MadEvent's
  `sample_get_x` does, so a cut edge is a fixed grid location. Needs the
  VEGAS↔channel contract inverted (the channel drives the grid a coordinate at
  a time, eager path kept bit-identical) and every analytic map made a fixed
  transform with the window inverted through it, so the mixture density stays
  grid-free. Then `--map-grid-coords` and its `auto`. The payoff to look for
  is on the cut-edge rows (`pp_to_llj`, `pp_to_jj`).
- **Map follow-ups** (note 37 §4, §6). A one-sided `1/E_g` shape for
  `q* → q g`, where the symmetric map shapes the quark end `P_qq` lacks
  (`g g > g u u~` reads 1.02 ± 0.09 under `soft-all`). llj's own soft-gluon
  structure is a `1/(ŝ − ŝ_rest)` on the spine's remainder invariant, which
  competes with the `Z/γ*` pole on the same variable — a second channel per
  spine if anything, preceded by the weight-tail decomposition binned in
  `ŝ − ŝ_rest`. MadEvent's `tstrategy` ping-pong for ≥ 3-rung ladders
  (untested: no gated row has one; two-rung reversal reads 1.01 ± 0.02).
- **S5 — the phase-space map's lower edge lands on the cut edge** (note 34 §2;
  partly answered by note 37 §6: confining the lepton pair's decay angle to the
  cuts' energy window, and shaping it, halved llj's evaluations — remeasure
  llj's time-to-accuracy against MadGraph with the new rules before sizing
  what is left).
  With a small timelike floor the map's lower edge coincides with the fiducial
  boundary and concentrates the residual `ΔR`/`pT` weight there (`pp_to_llj`
  `m_ll [0,5)` var/σ 24.5 → 55.7). This is the named lever for what the
  time-to-accuracy remeasure localised (note 34 §3): `pp_to_llj` converges at
  0.99× parity with MadGraph, and the whole residual is 9.0× the points for the
  same accuracy plus a stable χ²/dof ≈ 1.4 priced into the stop — both
  signatures of the cut-boundary map edge, neither of the evaluator. The bias
  oracle for any future floor is
  `no_accepted_configuration_sits_below_a_subsystem_floor`; the `mmll = 50`
  bound is attained within 1.0002, so it cannot tighten.
- **α on wide splits is iteration-limited, not point-limited** (2026-08-06,
  note 34 S1). Independent surveys at *any* budget land a tenth to a third of
  the mixture mass apart (within-rung α L1 across survey seeds 0.15–0.74),
  and the last survey step is still O(0.1–0.4) at every rung — six
  `ADAPT_ITERS` is the binding limit, while σ is insensitive to which draw it
  runs under (which is why the cap is free). The lever, if α quality is ever
  worth buying: vary the iteration count with damping, not the points. No
  urgency — nothing measured is limited by it today.
- **`stop_scale` reads a near-degenerate iteration as a thousandfold
  disagreement, leaving `--target-rel` inert on 2→6 rows** (note 34 S1 Part C,
  mechanism corrected by S3). The factor is not calibratable as a constant:
  `scaled_rel/achieved_rel` spanned ×4.8–×30 270, structured by the α draw. The
  driver is **not** all-points-cut zero-variance iterations — S3 counted exactly
  zero of those on every wide-row seed — but iterations with one to three
  accepted points, whose sample variance is tiny and strictly positive, so a fix
  filtering on `variance == 0` would miss the entire effect. The accepted-point
  floor moved the spread four orders (to ×3.4–×758) and a 1% target now fires on
  some seeds, but a 0.2% target still cannot. The remaining fix is what
  `stop_scale` does with few-accepted-point iterations — a minimum-accepted-count
  qualification or equivalent — never a retuned factor and never the reported
  statistic. The plain quoted error at the 40k survey cap is well calibrated
  (achieved_rel 0.0021–0.0024 against realized sd/σ 0.0021; 2× optimistic below
  the cap).
- **`MIN_ADAPT_SURVEY` / `MAX_ADAPT_SURVEY` bind hardest where channel counts
  are hundreds** (`vibegraph-cli/src/integrate.rs`). The two bounds clamp
  `--neval` to set the α-survey's points per iteration, and above 40k `--neval`
  stops buying a better split at all. The cap itself is confirmed for the rows
  measured (note 34 S1), but the regime it was tuned against carries ~24
  channels: own-map exploration is ∝ `αⱼ`, so a channel at `αⱼ = 10⁻³` draws
  ~40 points from its own map per iteration at the cap, and `Wⱼ` is what would
  raise its `αⱼ`. Per-channel *estimator* starvation is not the worry — each
  drawn point updates every `Wⱼ`. The experiment, if this is ever taken up:
  fixed seed, `n_survey ∈ {10k, 40k, 160k, 640k}`, recording the α trajectory,
  the converged α vector, and σ with its ≥5-seed spread from the run those αs
  drive, on `bbx_to_ccx_emmm_qcd0` / `uux_to_ccx_emmm_qcd0` (615/579 channels,
  banked σ) against `pp_to_llj` (24) as control. Sequencing note: the survey's
  per-point cost *is* the `Σⱼ αⱼgⱼ` loop, so measuring it before a density-loop
  change prices a loop that is about to move. Reporting is not silent about
  either bound — a clamp warns, as does an iteration whose spend is set by the
  `MIN_CHANNEL_NEVAL` floor rather than by `--neval`.
- **2→6 residue** (the rows are `info`, note 32) — promotion to an enforced
  gate is blocked on the heavy multichannel tail: single-seed pulls ±3.5–4.8% at
  every budget while five-seed means hold inside 1.1% of a 0.30% reference. The
  cheap merge path is closed by measurement (channel-dedup census, verdict
  DIES): fingerprinting every channel by its full map determinant collapses
  579 → 411 and 615 → 447 classes with the largest class exactly two members, so
  "classes ≈ channels" and the pre-registered kill fired. The recorded falsifier
  (note 32 §5.4) is that a fix must make single-seed swings **shrink as budget
  grows**, which a constant-factor channel-count reduction could never satisfy.
  **The promotion measurement is now due**: the accepted-point floor moved
  over-seed χ²/dof 2.10 → 0.81 / 13.46 → 0.33 and worst single-seed rel
  +1.68% → +0.89% / +3.82% → +0.46%, but at one budget only, so the §5.4
  falsifier is explicitly not claimed. Next step is `probe_2to6_budget_ladder`
  re-run under the floor, read against AGENTS.md's rung-difference caveat (20+
  seeds on the rungs that matter); a wide-row rung now spends ~2×, which is the
  floor's purchase and not a regression. The two `samples` cells stay ⏳ at a
  recorded cost: efficiency is fine (117/45 trials per event) but the pair needs
  ~40 unparallelisable minutes of serial accept/reject. Separately, VEGAS's
  per-iteration χ²/dof overflows to ~1e254 on wide channel splits (`budget.rs`
  floors a channel's variance at `f64::MIN_POSITIVE`) — a reported statistic
  only, σ unaffected, root-cause fix touches the estimator. The 411/447 class
  counts were measured on floor-less channels; `probe_channel_dedup_census` is a
  cheap re-run for post-floor numbers (direction-safe — floors only split).
- **Note-30 timing leftovers**: the `refs` reference-generation stage (f2py
  modules, amplitude tables, α_s and PDF oracles) stays unmeasured because
  timing it means writing into the reference bank; whether MadEvent's
  `results.dat` point count includes the survey pass is unresolved; and a
  per-phase `duration_s` inside a report row is what would give our side a
  counterpart to MG's `output` + `compile` column. (Note 30 §8.)
- **Per-flow α tuning — offline gain measurement first** (user, 2026-08-01).
  Stratify the integrand by leading-colour share
  `s_i = |JAMP_i|²CF_ii / Σ_k |JAMP_k|²CF_kk` (positive, a partition of unity,
  interference apportioned pro rata) and tune a separate channel-mixture α per
  stratum. **Stage 1 is a measurement, not a sampler**: the Kleiss–Pittau
  optimal α and its variance are computable offline from recorded `g_j(x)`,
  `f(x)` and `s_i(x)` on existing samples, so report the achievable variance
  reduction against the ×(strata) evaluation overhead before building anything.
  No longer a no-op on `uux_to_uux`/`gg_to_gg` — their channel maps are no
  longer bit-identical and their α no longer uniform (note 28 §S4 B2) — but
  flows overlap heavily, so the gain is the inter-stratum covariance term and is
  expected modest. **Guardrail: split the tuning, never the coverage** — every
  stratum keeps every channel with an α floor, or the `sde_strategy`-class
  fragility (note 27 §B1) is rebuilt on our side.
- **Stratified-parallel integration axes** (user, 2026-08-01) — the iterative
  VEGAS+α loop needs an embarrassingly parallel axis for SIMD/multi-thread
  promotion, catalogued exact-first (no partition function, no fragility). Two
  are done: channel-block stratification (note 31 §I4, with the α-survey on the
  same deterministic chunking) and the batch-size-vs-iteration-count measurement
  (note 32 S3, adopted nowhere because it moves no gate). Left open: **helicity
  strata** — `Σ_hel |M_hel|²` is an exact orthogonal decomposition for
  unpolarized beams, so parity-folded helicity classes can carry their own
  budgets and grids, and this is the first real consumer for
  `mg-single-helicity-bench`; **flavour groups × beam orderings**, already
  independent integrals and no longer blocked now that both integrands are
  `Sync`; and **frozen-pass bulk**, where `sample_frozen` is already
  embarrassingly parallel, so the lever is keeping the sequential adapt phase
  short. Partition-based axes (per-diagram AMP2 shares à la MadEvent
  G-directories, per-diagram-class = per *distinct* map) are second tier: real
  cluster-scale precedent, but they carry the routing fragility and need the
  same coverage guardrail as the per-flow item above.
- **Scratch-reuse continuation into `setclscales.rs`** — the open remainder of
  E4 after the merge-table hoist. The scale path still costs 1 857–2 802
  ns/point against a 581–1 524 ns matrix element: `ScaleChoice::clustered`
  heap-allocates its beam–leg candidate list per event and `setclscales.rs`'s
  clustering allocates several `Vec`s per call, running 2–3× per event.
  Threading a scratch struct through `setclscales`/`cluster` is a real refactor
  worth its own session; bit-for-bit event bytes at fixed seed is the gate and
  `probe_scale_cost` the instrument. (Note 30 §7.2, note 31 §E4, note 32 S5.)
- **Tighter spacelike floor** — `Cuts::spacelike_floor() = pT_min²` is provable
  but 10–100× looser than the true fiducial floor: S2's D3 measurement found the
  cut-surviving region above `|t| ≈ 4 000–40 000 GeV²` where the floor sits at
  400. A tighter derived bound scales the bounded-`t_max` variance win (measured
  1.67–1.83×) with it. (Note 28 §S2.5.)
- **`feyngraph-perf`** — `AssignWorkspace::assign()` (`workspace.rs:L122`)
  calls itertools `.counts()` (a fresh `HashMap`) per candidate vertex per
  topology per subprocess: ~340M allocations for pp→qq̃4l. Fix: pre-compute
  per-vertex counts in `AssignWorkspace::new()`. A submodule change, so a
  dedicated session. Mitigations already applied on this side: topology caching
  per `(n_ext, n_loops)` and the charge-conservation pre-filter (~86% of
  candidates eliminated). Enumeration runs on one thread by default
  (`EnumerationPool::Serial`) with `--parallel-diagrams` opting into the `-j`
  pool, because feyngraph's internal fan-out is contended and its sign flips
  with process size (16 threads vs 1: `p p > j j j` 0.137 s vs 0.083 s, worse;
  `p p > e+ e- j j j` 3.36 s vs 8.90 s, 2.6× better). Fixing the allocation is
  what would let the small case parallelise too.
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
- **The lane-FMA commit's scalar toll, and `MulAdd` for `NumericArray`** —
  `be76771` shared one real-FMA complex path between the scalar and lane fields
  (lanes −22–35%) because `Complex<NumericArray>` lacks `num_traits::MulAdd`,
  and its own message recorded the price: scalar `forward` +3.5%, shipped as-is
  since `forward` is the least-used path — but lanes never entered production,
  so the toll lands on the production evaluator. The in-house workaround trait
  was killed clean by its pre-registered criterion (note 32 S9): the packed
  idiom is x86-specific and forcing it on this ARM host cost 8–9%, the opposite
  of a win. The clean long-term fix is an upstream `numeric_array` contribution
  implementing `num_traits::MulAdd` (the orphan rule forbids it in-tree); the
  in-house design stays at note 32 §2 S9 for whoever revisits this on x86.
- **Per-lane scales** — `eval_m2_lanes` can only batch points sharing one `αs`;
  a SIMD-batched dynamic-scale integrator would need the scaling fused into the
  constant loads. Nothing needs it today. (`helas/eval/rescale.rs`.)
- **Make the end-to-end computation generic in `F: Real`** — a prerequisite of
  the batched evaluation above. The matrix element is already generic, but
  `RunningAlphaS::eval`, `coupling/scales`, the code surrounding the cuts and
  the samplers, and the record path are all `f64`; a lane-batched evaluation
  needs the whole per-point chain — scale, coupling, cut, weight — in one
  scalar type, or every batch pays a scatter/gather at each `f64`-only
  boundary. First step is the audit: enumerate which functions on that chain
  are `f64`-only and which of those are `f64` by necessity (LHEF's printed
  fields, the PDF grid's own storage) rather than by default.
- **Scalar constants in generic code** — `let two = F::one() + F::one()` and
  `F::from(4).expect(..)` appear ~23 times in `vibegraph-lib/src`. After
  monomorphisation and inlining these fold to immediates in practice, but
  nothing guarantees it, and the `NumCast` route carries an `Option` branch in
  the source whatever the codegen does with it. Proposed: associated constants
  on `Real` (`ZERO`, `ONE`, `TWO`, `HALF`, `FOUR`, …) implemented for `f64`
  and `f32`, replacing every site. Verify by inspecting the emitted assembly
  of one hot kernel before and after — the repo has a precedent for that
  protocol in `research/notes/fill-arenas-asm-study-results.md`.
- **`generate-stream` Part B** — lazy `generate_*` iterator (long-tail, from
  `cleanup-refactor`).
- **`Coeff(f64)` → `CoeffRat`** — optional cleanup now that `Op::CoeffRat` exists
  for color; the remaining `f64` leaves (Lorentz-structure and fermi-sign
  coefficients) could migrate. No consumer blocked. (Note 16 §5.)
