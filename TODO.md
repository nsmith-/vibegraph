# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate. Current position: `color-flow` (feature, ✅ merged 2026-07-12) →
`validation-sprint` (validation, ✅ closed 2026-07-13) → **eval performance
program** (performance, ✅ closed 2026-07-17: post-CSE 3-track program + helicity
expansion + helicity filtering; vs-MG gap 8.6×–110× → **1.2×–3.5×**; summary below,
full record note 15) → **hadronic-xsec** (feature, ✅ closed 2026-07-19: PDF
convolution + run-card cuts + two-phase VEGAS give σ(pp→e⁺e⁻) vs MG within
0.14%/0.07%; summary below, full record note 18) → **next: a validation pass**
over what this sprint exposed (pruned-frame contract guard, multi-subgrid PDF
seam coverage, and the pre-existing NHEL-table/flow-dictionary/timing-print
items — collected in the validation backlog below).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 14 processes agree with MadGraph (11 bit-identical ≤6.3e-13, incl. 2→6/VVV/massive externals, all NCOLOR=1; `uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS, now a two-phase serde object (`adapt`/`sample_frozen` split, deterministic rayon chunking) + 2-body LIPS + massive RAMBO generic over `F: Real` with splittable `ChaCha8` substreams; channel mappings + multi-channel weights remain (`lips-nbody`) |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻, pp→e⁺e⁻ Drell–Yan) | ✅ Done | Leptonic: `validate_vegas.rs` `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%). Hadronic: PDF-convolved σ(pp→e⁺e⁻) via a pure-Rust LHAPDF6 grid parser + log-bicubic interpolation (`pdf/`) and compiled MG run-card cuts (`runcard.rs`/`cuts.rs`), integrated over (τ,y); vs MG within 0.14% (default cuts, 934.42±0.87 vs 933.11±0.447 pb) / 0.07% (m_ℓℓ∈[60,120], 644.86±0.57 vs 644.42±0.315 pb); `vibegraph integrate` CLI drives proc-card + run-card → σ + persisted VEGAS grid artifact |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format; substrate now in place (frozen VEGAS grid, RAMBO, cuts accept-gate — see `event-output-lhef` below); still depends on `lips-nbody` channel mappings for genuine n-body final states |

Closed-sprint history (`helas-generalize`, `mg-validation-coverage`,
`cleanup-refactor`, `performance-sprint`, `color-flow`, `validation-sprint`, the
**eval performance program**, **`hadronic-xsec`**) lives in git history and
`research/notes/` (12: continuum bug hunt, 13: typed conventions, 15: eval
optimization program incl. its §2 close-outs, 16: color-flow design + debrief
incl. the VVVV phase-bug root cause and fix, 17: bounds-check-elimination memo,
18: hadronic cross-section design + outcome, `rooting-study-results.md`: rooting
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

## 🧬 Hadronic pp→ℓ⁺ℓ⁻ cross section ✅ CLOSED 2026-07-19 (full record: `research/notes/18-hadronic-xsec-design.md`)

σ = Σ_q ∫ dx₁ dx₂ f_q(x₁) f_q̄(x₂) σ̂(q q̄ → l⁺ l⁻), the first hadron-collider
observable. Eight sessions on branch `hadronic-xsec`, all behind the 14-process
`validate_helas_mg` net (only H4 touched evaluator-adjacent code, gated by its
own bit-exact test), merged to `main` in waves {H1,H3,H4,H5,H6}+H2 on
2026-07-18 and H7+H8 on 2026-07-19:

1. **H1 `pdf-grid-io`**: pure-Rust LHAPDF6 `.info`/`.dat` parser (`pdf::grid`) →
   `SetInfo`/`SubGrid`/`PdfSet`/`PdfMember` skeleton, 0↔21 gluon alias. Gated by
   an **LHAPDF C++** oracle (`validation/pdf/gen_oracle.cpp`, built + run
   against MG's own bundled LHAPDF 6.5.6 — swapped in from an initial `parton`/
   scipy trial once it was clear MG evaluates PDFs through LHAPDF's log-bicubic,
   not scipy's B-spline); on-knot x·f values match bit-for-bit.
2. **H2 `pdf-interpolate`**: `pdf::interp` — in-house log-bicubic exactly
   replicating LHAPDF6's `LogBicubicInterpolator`, off-knot rel **1.3e-15**
   vs the oracle. `scirs2-interpolate::RectBivariateSpline` trialled and
   **rejected** (worst rel 9.86e-1 — a global B-spline is the wrong algorithm
   class off-knot — plus ~40 extra transitive crates for a BLAS/LAPACK stack).
3. **H3 `rambo-real-generic`**: `phasespace/` module tree; massive
   `rambo::<F: Real>` with the KSE weight, splittable `ChaCha8Rng` substreams
   (`(stream, position)` addressing, a documented bits→`F` uniform rule); the
   first stage of `lips-nbody`. Uniforms-replay oracle ≤1.3e-15, QED flat-MC
   normalization check 6e-4.
4. **H4 `eval-simd-lanes` — negative result.** `numeric-array` lane-batched
   `eval_m2` measured 1.4–2.7× *slower* than scalar on NEON at every width
   tried (N=2/4/8): the indexed-arena interpreter doesn't auto-vectorize, so
   widening only amortizes call overhead rather than winning SIMD throughput.
   Infrastructure lands parked and bit-identical to scalar; `Real`'s
   `ConstZero`/`ConstOne` relaxed to method-based `Zero`/`One` (a real,
   permanent simplification — both are structurally impossible for a
   runtime-length SIMD array). H7 ships scalar-only; an AVX-512 rerun kit
   (`scripts/dump_lane_asm.sh`) is recorded in case wider hardware changes the
   verdict.
5. **H5 `vegas-serde-split`**: `VegasGrid` (serde + validating deserialize)
   split into `adapt`/`sample_frozen` phases, batched-integrand variants, and
   deterministic `adapt_parallel`/`sample_frozen_parallel` (ChaCha8 substream
   per rayon chunk, 1-vs-N-thread bit-identity).
6. **H6 `run-card-cuts`**: `runcard.rs` (MG `run_card.dat` parser, 209-entry
   defaults table pinned against a `banner.py` JSON dump) + `cuts.rs` (compiled
   filter: ŝ window, single-leg pT/E/η, pairwise ΔR + mass, `ptll`, `mmnl`;
   everything else parse-and-detect, hard-erroring if active and
   unimplemented). Convention pins vs `cuts.f`: rapidity (not pseudorapidity)
   for η and ΔR, and the `dr` threshold is squared once at first use.
7. **H7 `hadronic-sigma`**: `hadronic.rs` assembles σ(pp→e⁺e⁻) — PDF luminosity
   × up/down coupling classes (asserted against the `p p > e+ e-` enumeration)
   × compiled cuts × VEGAS. |M|² evaluated in the partonic CM (pruned-eval
   frame contract); cuts applied to lab-frame momenta boosted by the
   parton-system rapidity. **Switching the VEGAS variables from (x₁,x₂) to
   (τ=ŝ/s, y) fixed a ~6% convergence bias** on the mass-windowed run (the
   direct x-map makes the mass window a thin diagonal band VEGAS can't
   resolve; τ makes it a 1-D bound). Corrected the pinned PDF set's `lhaid` to
   **247000** (NNPDF23_lo_as_0130_qed) — 244600 and 230000 are different, wrong
   sets. Found and worked around a conda-`LDFLAGS` bug that silently dropped
   MG's `-lc++` link flag when generating the MG reference.
8. **H8 `cli-integrate`**: `vibegraph integrate <proc_card> [--run-card …]
   [--out <dir>] [--force] [--pdf-set/--pdf-dir] [--neval/--niter/--seed]` —
   assembles the H7 integrand, adapts the grid, prints σ±err, persists
   `artifact::IntegrateArtifact` (bincode+zstd: trained grid + run metadata).
   Cold-start reproduces the H7 σ bit-for-bit.

**Headline** (σ(pp→e⁺e⁻) at 13 TeV, NNPDF23_lo_as_0130_qed, μF=μR=m_Z):
default cuts **934.42±0.87** vs MG **933.11±0.447 pb** (0.14%); m_ℓℓ∈[60,120]
**644.86±0.57** vs MG **644.42±0.315 pb** (0.07%); pointwise PDF×flux×|M|²
integrand oracle **1.15e-14**.

### Deferred engineering work

- **DY integrand parallelism**: `hadronic.rs`'s per-flavor-class integrand
  scratch is `RefCell`-based `FnMut`, so H5's `adapt_parallel`/
  `sample_frozen_parallel` (which need `Fn + Sync`) can't be used yet —
  reworking the scratch to per-thread (`thread_local`/per-chunk) unlocks both
  multi-threaded local runs and the distributed-sharding design (§2.4, note
  18); not pulled in since the whole default-cut run is only ~2s single-threaded.
- `cli-proc-card`, `event-output-lhef`, and `lips-nbody`'s remaining scope
  (channel mappings + multi-channel weights) — updated in place below.

### New validation follow-ups (fold into the next validation pass)

- **Pruned-frame contract guard**: nothing enforces the partonic-CM ±z-beam
  requirement of a pruned evaluator (surfaced by the eval performance program,
  note 15 §2.3); `hadronic.rs` satisfies it by construction (it builds CM
  momenta directly) but that's convention, not enforcement — add a debug-build
  frame assertion (or an explicit boost-to-CM seam) before any future caller
  routes boosted momenta near `eval_m2`.
- **Multi-subgrid PDF seam behavior**: the LHAPDF oracle only covers the
  pinned NNPDF23_lo_as_0130_qed set, which ships as a *single* subgrid — the
  subgrid-walk's "first in-range band" logic and the two-Q²-knot bilinear
  fallback are pinned only by synthetic fixtures (`grid.rs::parses_multiple_subgrids`),
  not a real multi-subgrid reference, and `gen_oracle.cpp` hard-errors on
  repeated Q² knots rather than describing the seam-derivative-flattening
  behavior LHAPDF actually has there. Needs a genuine multi-subgrid set or a
  richer synthetic fixture.
- **NHEL-table pinning coverage** (pre-existing, eval performance program):
  still only 7 of 14 `MG_VALIDATED_PROCESSES` have pinned survivor sets.
- **Flow→LHEF color-string dictionary** (pre-existing, rides with
  `event-output-lhef`): still unimplemented.
- **Retire or label the `validate_helas_mg` timing print** (pre-existing,
  non-representative by construction): still open.

---

## 🟡 Medium — CLI integration

### `cli-proc-card` — wire a full process card through the CLI

`config::GlobalConfig::load_ufo(&Option<ModelImport>) -> Arc<UFOModel>` (landed with
`intern-sm-model`) already provides the `ParsedProcCard` → `UFOModel` seam: interned
SM for `import model sm[-variant]`, else a UFO dir under `ufo_search_path`. Remaining
work is the CLI wiring of a full proc card end-to-end. `vibegraph integrate`
(`hadronic-xsec` H8) hard-codes the `p p > e+ e-` process — a real consumer that
this item's full proc-card coverage would generalize.

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
**First stage shipped in the `hadronic-xsec` sprint** (✅ closed 2026-07-19, note 18
session H3): massive RAMBO generic over `F: Real` with the KSE weight,
splittable-substream RNG, and the banked σ̂ flat-MC check below. `hadronic-xsec`'s own
σ(pp→e⁺e⁻) integrand is 2→2 and used a direct 2-body LIPS map, not RAMBO, so it
didn't need the channel-mapping generalization — **remaining scope here** = channel
mappings + multi-channel weights on top of the RAMBO/RNG seams.
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

_Unblocks: `event-output-lhef` (n-body final states)._

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.). `hadronic-xsec` (✅ closed
2026-07-19) built most of the non-n-body substrate this needs: H5's frozen VEGAS
grid (`sample_frozen`/`sample_frozen_parallel`, no further adaptation — the
accept/reject primitive), H3's RAMBO + splittable RNG, and H6's compiled `Cuts`
(already an accept-gate shape, `cuts.pass(&momenta) -> bool`). H8's
`IntegrateArtifact` (bincode+zstd: trained grid + run metadata) is the natural
handoff format from the `integrate` phase into a future `generate` phase, which
would deserialize it and refuse a mismatched run rather than take raw CLI flags
again. Still missing: the `generate` CLI phase itself, and genuine n-body final
states (depends on `lips-nbody`'s channel-mapping scope, since accept/reject
against a single flat map is a poor sampler once propagator peaks appear).

LHEF color tags need MG's *leading-Nc* flow decomposition (`color_flow_decomposition`
/ `get_color_flow_string` in `color_amp.py`) to assign a `(color, anticolor)` integer
pair per external leg — a separate small feature on top of the trace/δ basis
`color-flow` built (note 16 §5); not needed for the multi-flow `|M|²` machinery itself.
The per-event flow is sampled ∝ `JAMP2(i) = Σ_hel |JAMP_i|²` (MG's `SELECT_COLOR`
input) — a cheap diagonal accumulator on the `eval_m2` combination loop, note 15
§2.2; the flow→string dictionary must be pinned against MG's conventions (see the
validation backlog).

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
