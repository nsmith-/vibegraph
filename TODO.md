# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate. Current position: `color-flow` (feature, ✅ merged 2026-07-12) →
`validation-sprint` (validation, ✅ closed 2026-07-13) → **eval performance
program** (performance, ✅ closed 2026-07-17: post-CSE 3-track program + helicity
expansion + helicity filtering; vs-MG gap 8.6×–110× → **1.2×–3.5×**; summary below,
full record note 15) → **next: hadronic pp→ll / event output** (feature).

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
`cleanup-refactor`, `performance-sprint`, `color-flow`, `validation-sprint`, the
**eval performance program**) lives in git history and `research/notes/` (12:
continuum bug hunt, 13: typed conventions, 15: eval optimization program incl. its
§2 close-outs, 16: color-flow design + debrief incl. the VVVV phase-bug root cause
and fix, 17: bounds-check-elimination memo, `rooting-study-results.md`: rooting
headroom study).

---

## ⚡ Eval performance program ✅ CLOSED 2026-07-17 (full record: `research/notes/15-eval-optimization-plan.md`)

Three phases, all behind the 14-process `validate_helas_mg` gate, all merged to
`main` (phase 3 via `eval-hel/helicity-filter`, 2026-07-18):

1. **Post-CSE 3-track program** (2026-07-11 → 07-14, note 15 §2.1): Track 1
   `eval-layout` shipped A0–A5 — instruction pack, static node analysis,
   constant-subgraph folding, SoA typed arenas, momentum pool, helicity recycling
   (A3c bounds-check elimination cancelled: eval stays 100% safe Rust, note 17).
   Tracks 2 (`rooting-exploration`, branch `explore/rooting` +
   `rooting-study-results.md`) and 3 (`dag-extraction`, egglog DAG-cost extractor)
   both closed **NO-GO**. Cumulative 1.4×–2.1× over the P5 baseline.
2. **Helicity expansion** (2026-07-16, note 15 §2.2): `Folded::expand_helicities`
   bakes every helicity combination into one hash-consed arena (liveness-allocated
   slots, lazy `OnceLock`), replacing A5's recycling — each distinct current
   computed exactly once per point. Bit-for-bit; a further 2.4×–3×.
3. **Helicity filtering** (2026-07-17, note 15 §2.3): `prune_zero_helicities`
   reproduces MadGraph's `GOODHEL`/`LIMHEL` filter as a compile-level numeric probe
   + re-expansion over the surviving combinations, survivor counts pinned against
   MG's generated `NHEL` tables. Bit-for-bit (dropped terms < ½ ulp of every
   partial sum).

**End state**: honest gap to MG MATRIX1 (release `eval_strategies` bench) went
8.6×–110× → **1.2×–3.5×**; the 2→6 sits at 240,925 ns/eval (2.5× vs MG), and the
widest remaining gaps are the colored 2→2s (uux_to_uux 2.5×, gg_to_gg 3.5× —
NCOLOR=6 makes the per-combination CF contraction relatively expensive). **Contract
change**: pruned evaluators require partonic-CM beams-along-±z momenta (frame-bound
J_z zeros — MG's own contract). ⚠️ Timing claims come from `eval_strategies` only;
the `validate_helas_mg` printed timings compile per-node cross-checks into the loop
and run ~4–5× hot.

All vs-MG ratios are single-host (Apple M3 Max) measurements; whether they hold on other
microarchitectures (esp. AVX-512 x86, where MG's straight-line Fortran auto-vectorizes and
our interpreter doesn't) is untested. Rerun kit for other boxes:
`scripts/mg_perf_compare.sh` + recipe in note 15 §2.4 (regenerate MG reference natively →
14/14 gate → bench-vs-MATRIX1 ratio table with host fingerprint, banked in `target/mg-perf/`).

### Deferred performance work

- **Per-(hel,diagram) `ZEROAMP` skipping** (MG's second filter layer, note 15 §2.3):
  inside surviving combinations, individual diagram amplitudes can still be
  identically zero for that helicity; skipping them needs probed-zero *node*
  elimination in the expanded arena (a rewrite pass, not a filtered re-expansion).
  Unmeasured headroom, likely small — the combination filter already removed most
  zeros, and elimination only reclaims nodes private to a zero diagram.
- **CF-factoring across combinations** — analyzed and shelved 2026-07-17 (note 15
  §2.2): accumulating `M_ij = Σ_hel JAMP_i·JAMP_j*` and contracting CF once
  rebalances the arithmetic rather than shrinking it, and reordering the |M|² sum
  breaks bit-for-bit. Its diagonal is MG's `JAMP2` (the leading-color flow-sampling
  input), but that dual use is served by a cheap diagonal-only accumulator in the
  existing `eval_m2` loop — rides with `event-output-lhef`, no restructure needed.
- **`egraph-rewrite`** (blocked; notes 14 + 15 §4–5): the remaining rule families
  (coupling regrouping, chiral decomposition, re-rooting) are all *sharing*
  rewrites, invisible to tree-cost extraction. Path to yes needs a global/ILP
  extractor **and** a compute-aware `WorkCost` model **and** a ≥3-consumer demo
  process. Substrate on `main`: the egglog round-trip skeleton (`egraph.rs`,
  parked) and Track 3's M1/M2 DAG-cost extractor; adopted schema decisions in note
  15 §5. Known issue for any cost oracle: lowering emits a ±1-CSE-node AST per hash
  seed (`HashSet` iteration in `root_diagram`/`lower`) — compile once and reuse;
  the fix belongs to the lowering owners.
- **`rooting-soundness`** (prerequisite surfaced by Track 2, note 15 §3 +
  `rooting-study-results.md`): the amplitude is correct only for feyngraph's
  `VtxIdx(0)` edge orientation — every node-reducing rooting silently corrupts
  multi-boson/≥6-point amplitudes (max_rel up to 1.7e+3). Fix momentum routing,
  Lorentz-output rooting, and fermion-spine sign to be root-invariant; first test
  asserts all V rootings of every `MG_VALIDATED_PROCESSES` diagram pass the gate
  (`set_root_override` hook ready on `explore/rooting`). The measured prize is −21%
  nodes / −34% slot traffic, so this is a correctness spike with a modest perf
  payoff — sequence it as its own spike, blocking any production rooting change and
  the Track 3 re-rooting rule family.
- **`mg-single-helicity-bench`** — still deferred to `event-output-lhef` (entry in
  the Later section), where single-helicity evaluation through the *unexpanded*
  program becomes the actual hot path.
- Long-tail perf backlog (Later section): `feyngraph-perf` allocation hot spot,
  `generate-stream` Part B, `C<F>`-vs-`F` multiply peepholes.

### New validation follow-ups (fold into the next validation pass)

- **Pruned-frame contract guard**: nothing enforces the partonic-CM ±z-beam
  requirement of a pruned evaluator — a boosted input silently revives
  J_z-forbidden combinations (up to 3e-3 of the sum, note 15 §2.3). Before
  `lips-nbody`/hadronic pp→ll route boosted momenta anywhere near `eval_m2`, add a
  debug-build frame assertion (or an explicit boost-to-CM seam) plus a test that a
  boosted point on a pruned evaluator is caught.
- **`NHEL`-table pinning coverage**:
  `prune_zero_helicities_matches_madgraph_filter_bitwise` pins survivor sets for 7
  processes; extend toward the full `MG_VALIDATED_PROCESSES` list (the 2→6 counts —
  16/256 uux, 32/256 bbx — were checked against MG's reports and deserve the same
  in-test pinning as reference tables are generated).
- **Flow→LHEF color-string dictionary** (rides with `event-output-lhef`):
  leading-color assignment = sample flow `i` ∝ `JAMP2(i)`, then map the flow index
  to a color string. Pin the mapping against MG's
  `SELECT_COLOR`/`color_flow_decomposition` conventions — the gg_to_gg NCOLOR=6
  flow-basis ordering caveat applies, and a transposed dictionary is invisible to
  any |M|²-level gate.
- **Retire (or label) the `validate_helas_mg` timing print**: it is
  non-representative by construction and has now misled twice (note 15 §2.1/§2.2
  warnings); the honest bench is `eval_strategies`.

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
see note 18 §5, incl. the AVX-512 rerun kit w/ `scripts/dump_lane_asm.sh`**),
H5 serde-split two-phase VEGAS (adapt/save grid, frozen sample;
deterministic rayon), H6 MG run-card parser on `GlobalConfig` + compiled cuts
filter (MG's default lepton cuts are active out of the box; unimplemented-but-
active cut = hard error), H7 the convolution + MG σ(pp→e⁺e⁻) gate under a shared
run card, H8 minimal `integrate` CLI, H9 close-out.
Waves: {H1,H3,H4,H5,H6} → H2 → H7 → H8 → H9.

- **H7 — `hadronic-sigma` ✅ done (branch `hx/h7-hadronic-sigma`).**
  `hadronic.rs` assembles σ(pp→e⁺e⁻): up/down flavor classes resolved from the
  `p p > e+ e-` enumeration and asserted against it (u/c, d/s; no b, no gluon),
  the "one σ̂ per class" premise pinned by a u ū == c c̄ |M|² test. |M|² evaluated
  in the partonic CM (pruned-eval frame contract); cuts applied to lab-frame
  momenta boosted by the parton-system rapidity. **Switched the VEGAS variables
  from (x₁,x₂) to (τ=ŝ/s, y): the mass window becomes a 1-D bound on τ, killing a
  ~6% VEGAS-convergence bias on the mmll run** (note 18 §5). MG reference
  generation (`pixi run -e madgraph generate-hadronic-sigma` → banked
  `hadronic_sigma_reference.json`) drives `p p > e+ e-` at 13 TeV from the shared
  committed run cards `dy13_*_run_card.dat`; **PDF pinned to lhaid 247000 =
  NNPDF23_lo_as_0130_qed** (the note's 244600 is the NLO_qed set, run_card_dy.dat's
  230000 is NNPDF23_nlo_as_0119 — both wrong). Both runs *enforced* in
  `validate_hadronic` within combined MC error: default 934.4±0.9 vs MG
  933.11±0.45 pb (0.14%), mmll[60,120] 644.9±0.6 vs MG 644.42±0.32 pb (0.07%).
  Pointwise integrand oracle (`generate-dy-oracle`: LHAPDF xfxQ2 × MG standalone
  MATRIX1/MATRIX2) pins PDF×flux×|M|² + cut indicator at 10 points (incl. two
  straddling pT_ℓ=10) to worst rel **1.15e-14**. Informational dσ/dm_ℓℓ table
  `dy_dsigma_dmll.md` (γ* continuum + Z peak) committed. MG-LHAPDF link bug
  (conda LDFLAGS suppresses MG's `-lc++`) worked around in the gen script.

- **H2 — `pdf-interpolate` ✅ done (branch `hx/h2-pdf-interpolate`).**
  `pdf::interp` — local **log-bicubic** in `(ln x, ln Q²)` replicating LHAPDF6's
  `LogBicubicInterpolator` (precomputed x-cubic Hermite coeffs `[a,b,c,d]` per
  `(x-interval, Q²-knot, flavor)` à la `KnotArray::coeff` + `GridPDF::_ddx`,
  Q²-direction Hermite assembled at eval with one-sided/central FD slopes),
  behind a minimal `Bicubic2D` trait; `PdfMember::xfx_q2::<F: Real>`/`try_xfx_q2`
  with subgrid walk (first in-range band) + 0↔21 gluon alias, out-of-grid →
  typed `OutOfRange` (extrapolation a recorded non-goal). Coeff tables in `f64`,
  arithmetic cast to `F` per point. **scirs2 rejected** (note 18 §5): trialled
  `scirs2-interpolate::RectBivariateSpline` (kx=ky=3, s=0) against the LHAPDF
  oracle — worst rel **9.86e-1** on interior off_knot (charm), a global B-spline
  is the wrong algorithm, and it drags ~40 crates (nalgebra/ndarray/oxiblas
  BLAS-LAPACK/simba/statrs). In-house log-bicubic vs LHAPDF oracle: off_knot
  **1.3e-15**, seam 1.3e-16, x→1 tail 2.0e-11, corner ≤1e-9, on-knot |Δ| 2.7e-20
  — all far under the 1e-9 bar (`validate_pdf_grid`, extended-validation). Walk +
  flavor-alias + analytic bilinear-in-log oracle unit-tested in the default
  suite.

- **H6 — `run-card-cuts` ✅ done (branch `hx/h6-run-card-cuts`).**
  `runcard.rs` (typed parser + MG LO defaults table, banner.py JSON oracle via
  `pixi run -e madgraph dump-runcard-defaults`), `cuts.rs` (compiled letter-class
  filter: ŝ window, single-leg pT/E/η, pairwise ΔR + mass, `ptll`, `mmnl`;
  everything else parse-and-detect → `CutError::UnimplementedCutActive`),
  `GlobalConfig::load_run_card` seam. cuts.f convention pins + decision record in
  note 18 §5. Cuts consume `ExternalLeg { pdg, mass, is_final }` — H7 builds these
  from subprocess metadata.

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
The per-event flow is sampled ∝ `JAMP2(i) = Σ_hel |JAMP_i|²` (MG's `SELECT_COLOR`
input) — a cheap diagonal accumulator on the `eval_m2` combination loop, note 15
§2.2; the flow→string dictionary must be pinned against MG's conventions (see the
performance-program validation follow-ups).

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
