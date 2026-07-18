# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate. Current position: `color-flow` (feature, ✅ merged 2026-07-12) →
`validation-sprint` (validation, ✅ closed 2026-07-13) →
**post-CSE optimization program** (performance, ✅ closed 2026-07-14) →
**helicity-expansion session** (performance follow-on, ✅ merged 2026-07-16, note 15
§2.2) → **next: hadronic pp→ll / event output** (feature).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 14 processes agree with MadGraph (11 bit-identical ≤6.3e-13, incl. 2→6/VVV/massive externals, all NCOLOR=1; `uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | Lepage VEGAS on `AmplitudeEvaluator::eval_m2`; `validate_vegas.rs`: `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%) |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

Closed-sprint history (`helas-generalize`, `mg-validation-coverage`,
`cleanup-refactor`, `performance-sprint`, `color-flow`, `validation-sprint`) lives in
git history and `research/notes/` (12: continuum bug hunt, 13: typed conventions, 15:
eval optimization plan, 16: color-flow design + debrief, incl. the VVVV phase-bug root
cause and fix).

---

## 🚀 Post-CSE optimization program ✅ CLOSED 2026-07-14

The completed evaluator-layout work delivered a cumulative honest `eval_m2` speedup of
**1.4×–2.1×** over P5 across the benchmark suite while retaining the 14-process MadGraph
gate; the full plan, measurements, and close-out ledger are in
`research/notes/15-eval-optimization-plan.md` §2.1. The rooting study and DAG-cost
extractor were both no-go: useful re-rooting remains blocked by `rooting-soundness`, and
sharing-oriented algebraic rewrites need a global/ILP extractor, compute-aware cost model,
and a ≥3-consumer demonstration before `egraph-rewrite` can resume. These are deferred
follow-ups, outside the feature→validation→performance critical path.

### `rooting-soundness` — make diagram rooting orientation-independent

The `explore/rooting` study found a real greedy-rooting headroom (−21% nodes, −34% slot
traffic), but every node-reducing alternate root silently corrupts amplitudes: the current
momentum routing, Lorentz-output rooting, and fermion-spine signs are only sound for
feyngraph's default `VtxIdx(0)` orientation. This is not a production bug while that
orientation remains fixed, but it is a hard correctness prerequisite for any canonical,
greedy, or e-graph re-rooting.

Use the existing test-only `root_diagram::set_root_override` hook as the initial fuzz
harness: assert that **every vertex rooting of every diagram** in all 14
`MG_VALIDATED_PROCESSES` passes `validate_helas_mg` (REL_TOL). Diagnose and repair the
orientation-dependent primitives before promoting any rooting strategy or adding
propagator-commute/per-vertex rotation rewrites. The gate must distinguish gross
value-level corruption, not just $|M|^2$-blind global phases. See note 15 §3.1 and
`research/notes/rooting-study-results.md`.

## ⚡ Helicity-expansion session ✅ MERGED TO MAIN 2026-07-16

Helicity expansion bakes the surviving nonzero configurations into one hash-consed
`Op::Hels` arena with liveness-allocated result slots, making `eval_m2` one linear pass
and preserving bit-for-bit agreement with the unexpanded per-helicity sum. The
`prune_zero_helicities` probe matches MadGraph's filter, removing the structural
combination-count handicap and narrowing the honest gap to **1.2×–3.5×** (2→6:
**25× → 2.5×**); see note 15 §§2.2–2.3 for the implementation and benchmark tables.
Follow-up performance work is limited to the deferred `mg-single-helicity-bench` alongside
`event-output-lhef`, since unweighted event generation evaluates one helicity configuration
through the unexpanded program.

---

## 🧬 `egraph-rewrite` — algebraic rewrite stage (**blocked**)

The parked `helas/eval/egraph.rs` round-trip skeleton preserves the future rewrite seam,
but every remaining rule targets cross-diagram sharing beyond CSE; constant folding is
already complete. Resuming requires a global/ILP extractor, compute-aware `WorkCost`, and
a ≥3-consumer demonstration; re-rooting also requires `rooting-soundness`. Apply the
adopted typed schema (per-kind leaf sorts, `ScalarConst`/`ScalarWf`, typed constructor
slots), bound saturation on 2→6 QCD ASTs, and guard each rule with the 14-process
`validate_helas_mg` net plus byte-for-byte round trips where order-preserving. See
`research/notes/14-egglog-notes.md` and note 15 §§4–5 for the extractor decision,
schema, and implementation detail.

---

## 🔴 High — next feature: hadronic pp→ll cross section (**planned — note 18**)

σ = Σ_q ∫ dx₁ dx₂ f_q(x₁) f_q̄(x₂) σ̂(q q̄ → l⁺ l⁻). Sprint design + session plan in
`research/notes/18-hadronic-xsec-design.md` (branch `hadronic-xsec`): H1/H2 pure-Rust
LHAPDF6 PDF module (parton-oracle-gated, scirs2-interpolate trial with in-house
bicubic fallback), H3 massive RAMBO generic over `F: Real` + splittable ChaCha8
substreams (first stage of `lips-nbody`), H4 SIMD lane-batched `eval_m2` via
`numeric-array` (**done — negative result: 1.4–2.7× *slower* on NEON, the
indexed-arena interpreter does not auto-vectorize; infra landed + parked, `Real`
relaxed to method-based `Zero`/`One`, bit-identity gate green; H7 stays scalar —
see note 18 §5**), H5 serde-split two-phase VEGAS (adapt/save grid, frozen sample;
deterministic rayon), H6 MG run-card parser on `GlobalConfig` + compiled cuts
filter (MG's default lepton cuts are active out of the box; unimplemented-but-
active cut = hard error), H7 the convolution + MG σ(pp→e⁺e⁻) gate under a shared
run card, H8 minimal `integrate` CLI, H9 close-out.
Waves: {H1,H3,H4,H5,H6} → H2 → H7 → H8 → H9.

- **H5 — `vegas-serde-split` ✅ done (branch `hx/h5-vegas-serde-split`).**
  `VegasGrid` (serde + validating deserialize) with `adapt`/`sample_frozen`
  split, batched-integrand variants (bit-identical to unbatched for any batch
  size), and deterministic-parallel `adapt_parallel`/`sample_frozen_parallel`
  (ChaCha8 substream per chunk, 1-vs-N-thread bit-identity); `Vegas::integrate`
  kept as a compat shim, pinned-seed bit goldens guard the refactor. serde_json
  `float_roundtrip` footgun recorded in note 18 §5.

- **H3 — `rambo-real-generic` ✅ done (branch `hx/h3-rambo-real-generic`).**
  `phasespace/` module tree — `rng.rs` ChaCha8 substreams + bits→`F` uniform
  rule, `rambo.rs` massive `rambo::<F>` with KSE weight; uniforms-replay oracle
  ≤1e-13, invariant fuzz, banked-σ̂ flat-MC check; `rambo_massless` kept
  bit-compatible.

- **H1 — `pdf-grid-io` ✅ done (branch `hx/h1-pdf-grid-io`).** `pdf::grid` parses
  LHAPDF6 `.info` + member `.dat` (`lhagrid1` format) into `SetInfo`/`SubGrid`;
  `pdf::PdfSet`/`PdfMember` skeleton + 0↔21 gluon-alias flavor indexing; no
  interpolation. Gated by an **LHAPDF C++** oracle (`validation/pdf/gen_oracle.cpp`,
  built + run in the `madgraph` env against MG's bundled LHAPDF 6.5.6): `pixi run
  -e madgraph fetch-pdf` → `build-pdf-oracle` → `generate-pdf-oracle` →
  `validate-pdf-grid`. On-knot x·f values match **bit-for-bit** (rel 0.0, gate at
  ≤1e-12), malformed input hits typed `GridError` variants. The oracle backend
  is LHAPDF (not parton) because MG evaluates PDFs through LHAPDF and its
  log-bicubic is the correct off-knot reference; the parton/`pdf-validation`
  python env is removed. **Findings for H2**: (1) the pinned NNPDF23_lo_as_0130_qed
  set ships as a *single* subgrid (nx=100, nq=50, 14 flavors, no internal Q²
  threshold split) — the oracle's `seam` category degenerates to the global
  QMin/QMax edge, so seam-walk coverage needs a genuinely multi-subgrid set or a
  synthetic fixture. (2) The H2 target is LHAPDF's **log-bicubic**, not a
  scipy-style B-spline: scirs2's `RectBivariateSpline` is a different algorithm
  and will likely miss the 1e-9 off-knot bar (LHAPDF-vs-scipy diverged ~120% at
  some interior points) — budget for the in-house log-bicubic fallback (note 18
  §5).

---

## 🟡 Medium — CLI integration

### `cli-proc-card` — wire a full process card through the CLI

`config::GlobalConfig::load_ufo(&Option<ModelImport>) -> Arc<UFOModel>` (landed with
`intern-sm-model`) already provides the `ParsedProcCard` → `UFOModel` seam: interned
SM for `import model sm[-variant]`, else a UFO dir under `ufo_search_path`. Remaining
work is the CLI wiring of a full proc card end-to-end.

---

## 🟢 Later — polish and extensibility

### Validation backlog (deferred from `validation-sprint`)

Deferred to the next validation pass of the loop — none of these guard the surface
the optimization program touches:

- **`IdentityAmp` process-level coverage**: moved to the non-SM UFO boundary
  list below — it needs a non-SM model, so it rides with that work.
- **Rationalize `Coeff(f64)` onto `CoeffRat`** (note 16 §5): now that `Op::CoeffRat`
  exists for color coefficients, the remaining `Coeff(f64)` leaves (Lorentz-structure
  and symmetry/fermi-sign coefficients) could migrate onto it too — optional cleanup,
  not required by anything currently blocked.
- **Branch-level coverage**: op counts don't see rooting branches. The pure-metric
  −1 vertex branch (`root_lorentz`) used to fork on amplitude-root vs scalar-root
  (the latter pinned, the former the unexercised `MetricNegI` op); `validation-sprint`
  found the fork itself was the bug — `gg_to_gg` amplitude-roots a pure-metric VVVV
  contact term, and its separate −i lowering was a spurious phase — and collapsed both
  paths onto one real-−1 branch, now exercised by both `gg_to_gg` (amplitude-rooted)
  and the 2→6 H-current processes (scalar-rooted). More generally, consider rooted-tree
  pattern assertions per MG-pinned convention: each "pinned by X" comment should have a
  test that fails if the pinning process stops exercising the branch — an unexercised
  branch silently drifting out of sync with its exercised sibling is exactly the
  failure mode that produced the `gg_to_gg` bug.
- **Optional CI job**: `gen_sm_blob` + `git diff --exit-code` to catch a stale
  interned SM blob vs the pinned submodule.
- **`madgraph-diagram-cmp-per-flavor`** — per-flavor subprocess matching in diagram
  validation (design below).

#### `madgraph-diagram-cmp-per-flavor` — Match subprocesses by flavor in diagram validation

An independent, verification-heavy refactor (Python extractor + Rust matching + JSON
regen). The `validate_madgraph_diagrams` reference count now uses the representative
subprocess's true Feynman-diagram count (`NGRAPHS` from `matrix1_orig.f`), not
`MAPCONFIG(0)` from `configs.inc` (which counts the phase-space integration-channel
*union* across all flavor variants in a P-class — e.g. 2672 vs the actual 2316 for
`u u~ > u u~ l+ l- l+ l-`).

**Remaining gap**: the comparison (`count_mg_style_topologies` in
`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) still collapses vibegraph subprocesses
into coarse particle-type classes (`quark`/`lepton`/…) and compares one representative per
class against the summed `total_diagrams`. Fragile: it assumes vibegraph's first-enumerated
subprocess in each class matches MadGraph's `matrix1` representative.

**Design for the refinement** (per-flavor matching, validates *all* variants incl. the 40
of the qq4l class):
- **Robust flavor source — the matrix-file header, not `IDUP`.** Each
  `SubProcesses/P*/matrix<N>_orig.f` carries `C     Process: u u~ > u u~ e+ e- e+ e- QCD=0 @1`
  comment lines — one per concrete flavor process sharing that variant's `NGRAPHS` (u/c and
  e/mu are grouped). Parse these directly: it avoids reverse-engineering MG's fragile
  `matrix<N> ↔ IDUP(I,J,K)` 3-index mapping in `leshouche.inc`. `extract_diagrams.py` grows
  a per-concrete-process `{in:[pdg…], out:[pdg…], ngraphs}` list (name→PDG via a bounded SM
  dict: the full token set is `a b b~ c c~ d d~ e± g h mu± s s~ t t~ ta± u u~ w± z`).
- **Rust side**: key each MG entry and each vibegraph subprocess by
  `(sorted initial PDGs, sorted final PDGs)`; look up and compare per-subprocess
  (`set.diagrams.len()` vs `ngraphs`).
- **Known risk to resolve first**: this exposes whether vibegraph enumerates the *same set*
  of concrete subprocesses as MG's `C Process:` union — i.e. whether the multiparticle `p`/`l`
  definitions and flavor-symmetry pruning align. Validate on a small process (`pp_to_ll`)
  before the qq4l class; a set mismatch here is a real finding, not a test bug, and needs
  physics judgment (note-12 territory: MG-convention reconciliation is a bug magnet).

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

The UFO surface is deliberately model-generic, but "generic" currently ends at the
SM's feature set. None of these block anything (the interned SM avoids them all);
they are collected here so a future BSM-model task scopes against a checklist
instead of rediscovering each wall one hard error at a time. A small dedicated
test model (or a public BSM UFO) would be the natural vehicle for several at once.

- **Color sextets and baryonic epsilons**: the color engine handles
  Singlet/Triplet/AntiTriplet/Octet only (`helas/repr/color.rs`); the sextet
  tensors `K6`/`K6Bar`/`T6` (diquark models) and the baryon-number-violating
  `Epsilon`/`EpsilonBar` (e.g. RPV SUSY) are deliberate hard errors
  (`ufo/color.rs::SextetUnsupported`, `helas/color/tensor.rs`). Note the two
  distinct "6"s: NCOLOR=6 (flow-basis dimension, e.g. `gg_to_gg`) is fully
  supported; the 6-dimensional sextet *representation* is not. MG's reference
  algebra for the missing tensors lives in `color_algebra.py` (K6/T6/ε
  Clebsches); support means new `ColorTensor` atoms + trace-basis reduction
  rules + CF products, validated the color-flow way (CF oracle vs MG's DATA CF,
  then the JAMP-weighted |M|² gate).
- **Spin codes beyond {1, 2, 3}**: `helicity_states_for_spin` (`eval/compile.rs`)
  future-proofs the spin-2 helicity list (code 5), but nothing downstream builds
  tensor external wavefunctions or propagators; spin-3/2 (code 4, gravitinos) is
  an `UnsupportedSpin` error. Ghost codes (negative) stay irrelevant at LO.
- **Majorana fermions** (MSSM neutralinos, gluinos): fermion-flow handling
  assumes Dirac-continuous lines end to end — there is no flow-flip /
  charge-conjugation machinery. This is HELAS's classically subtle sign
  territory; the `color-flow` fermion-flow slot-swap bug shows how delicate the
  flow conventions are even in the pure-Dirac case.
- **`IdentityAmp` process-level coverage** (deferred from `validation-sprint`,
  the last `KNOWN_UNCOVERED` op): needs an `Identity` scalar bilinear in the
  Lorentz sector, which the SM lacks — a natural rider on whichever small test
  model lands first.
- **Loop-level UFOs** (`loop_sm`, NLO models): out of the LO charter. Note 04
  records the parser history — the Python-AST parser replaced the FeynGraph/PEG
  split that choked on `loop_sm`'s attribute assignments, but counterterm
  content (`CT_vertices.py` etc.) has no consumer regardless.

### `feyngraph-perf` — Fix feyngraph allocation hot spot

**Hot spot identified** (samply profile, pp→qq̃4l run): `workspace.rs:L122` in
`AssignWorkspace::assign()` calls `.counts()` (itertools) on every candidate vertex for every
topology for every subprocess. Each `.counts()` call allocates a fresh `HashMap<particle_index,
count>`. For pp→qq̃4l: ~1,664 subprocesses × 34,300 topologies × O(vertices) = ~340M HashMap
allocations. **Fix**: pre-compute per-vertex particle counts in `AssignWorkspace::new()` and
reuse them in the inner loop. This is a change to the `feyngraph` submodule; deferred to a
dedicated feyngraph session.

Vibegraph-side mitigations already applied:
- Topology caching: `generate_topologies()` called once per `(n_ext, n_loops)`; all subprocesses
  share the same `Vec<Topology>` via `DiagramGenerator::assign_topologies()` (pp→qq̃4l: 4.86s once
  vs ~15h naive).
- Charge conservation pre-filter: eliminates ~86% of alias-expanded candidates before topology
  assignment (11,520 → ~1,664 for pp→qq̃4l).

Also deferred perf backlog (from `cleanup-refactor`/`performance-sprint`):
`generate-stream` Part B (lazy `generate_*` iterator) and `C<F>`-vs-`F` multiply
peepholes.

### `lips-nbody` — n-body LIPS phase-space generator

Generalize phase-space sampling to 3+ final-state particles using recursive 2-body
decomposition (RAMBO-style). Research Rust options before committing to an approach.
**First stage pulled into the `hadronic-xsec` sprint** (note 18, session H3): massive
RAMBO generic over `F: Real` with the KSE weight, splittable-substream RNG, and the
banked σ̂ flat-MC check below. Remaining scope here = channel mappings + multi-channel
weights on top of those seams.
(The MG validation side already generates n-body points via RAMBO in
`gen_amplitude.py`; the MG-computed partonic σ̂ = 6.556e-7 pb for the uux 2→6 at
√s=500 is **now consumed** by H3's flat-MC weight-normalization check
`rambo_oracle::flat_mc_partonic_sigma`.)

**Design inputs for the sprint plan** (fold into the design note):

- **Abstraction is the point**: structure the phase-space module so sampler,
  channel mapping, and integrator are separately swappable and composable —
  flat RAMBO vs. recursive 2-body propagator-pole channels, single- vs.
  multi-channel weighting, classic VEGAS vs. VEGAS+ stratification should be
  mix-and-match choices, not rewrites. The known endgame is MG-style
  per-diagram multi-channel (one channel per diagram parametrised by its
  propagator poles, combined with the variance-minimising weight `1/Σᵢ(1/Jᵢ)` —
  note 01 phase-space-optimisation section), and possibly Sherpa-style
  sampling over color/helicity instead of summing.
- **Reference implementations** (submodules; key paths in
  `research/refs/README.md`): Sherpa `PHASIC++/Main/` (multi-channel adaptive
  integrator with separate `Color_Integrator`/`Helicity_Integrator`; note 03
  §1.5), POWHEG `integrator.f` (MINT), MG `madgraph/various/rambo.py` (carries
  the line-218 overflow-warning sign bug documented in note 07).
- **Hazard catalog**: note 07 "Numerical Precision / Stability" and
  "Phase-Space / Integration" test lists. MG's sampler bugs (BW mapping,
  T-channel ordering, threshold kinematics, conflicting-BW configurations)
  stayed latent 5–10 years because sampler errors shift σ smoothly rather than
  tripping a bit-exact gate — plan the validation regime alongside the feature.
- **Validation regime**: bit-for-bit gating exists only with a pinned RNG seed
  and unchanged sampling order; otherwise gate statistically — σ within quoted
  MC uncertainty (the `validate_vegas.rs` targets plus the banked σ̂ above) and
  distribution comparisons, since σ-agreement alone is a weak oracle, blind to
  mis-sampled regions of small measure. For optimization work the figure of
  merit is variance × CPU-time at fixed target precision, not ns/point.

_Unblocks: hadronic pp→ll, `event-output-lhef`._

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.).

LHEF color tags need MG's *leading-Nc* flow decomposition (`color_flow_decomposition`
/ `get_color_flow_string` in `color_amp.py`) to assign a `(color, anticolor)` integer
pair per external leg — a separate small feature on top of the trace/δ basis
`color-flow` built (note 16 §5); not needed for the multi-flow `|M|²` machinery itself.

_Depends on: `lips-nbody` (n-body final states)._

### `typed-units` — Typed physical units

Research `uom`/`dimensioned`/`units` crates for typed four-momenta and cross sections.

### `mg-single-helicity-bench` — MG comparison at a fixed helicity configuration (low priority)

The timing table compares against MG MATRIX1, which sums helicities. Since the
helicity-expansion session (2026-07-16) both sides now share currents across the
helicity loop — MG via its restructured-call recycling, vibegraph via the baked
`Op::Hels` expansion — so the helicity-sum ratio is a fair like-for-like; a parallel
benchmark evaluating **one fixed helicity configuration** on both sides still
isolates kernel-level gaps from expansion/sharing effects. It is also the relevant
comparison for the event-generation regime: final accept/reject evaluates a specific
helicity configuration through the *unexpanded* program, where the expansion buys
nothing (its win belongs to the integration-grid phase and its helicity-summed
`eval_m2`).

**A6 go/no-go (2026-07-14): DEFER — not pulled in.** The vibegraph half
(`eval_amplitude` at one fixed helicity) is a cheap bench addition, but the *fair*
comparison needs an MG single-helicity timing, and MG's MATRIX1 driver hardcodes the
helicity-sum loop — a single-config timing means editing the generated Fortran driver +
the `gen_amplitude.py` timing harness and regenerating reference data (a
reference-data/Fortran task, not a warm-rig freebie), and a vibegraph-only number is
half an oracle. No live consumer until `event-output-lhef` accept/reject makes
single-helicity the actual hot path. Recommendation: land it **alongside
`event-output-lhef`**, when the comparison has a consumer and the MG-harness change is
on the critical path anyway.
