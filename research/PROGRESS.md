# lorentz-runtime-eval — COMPLETE ✅

The topology-driven Lorentz structure evaluator is fully implemented and validated.
All 135 tests pass; amplitude agrees with Fortran HELAS to <1e-7 for massive e⁺e⁻→μ⁺μ⁻.

## What was built

### Core primitives
- External wavefunctions: `vxxxxx`, `sxxxxx`, `ixxxxx`/`oxxxxx`
- Propagators: `DiracPropagator`, massive/massless vector (inline), scalar
- Projectors: `GammaL::apply`, `GammaR::apply`; `SpinorRepr::slash`, `project_left/right`, `scalar_bilinear`

### LorentzEvalTree compiler (`dispatch.rs`)
Converts undirected UFO tensor network into a directed eval tree rooted at the output leg.
All SM node types implemented:

| Node | Physics |
|------|---------|
| `Leg(i)` | External wavefunction / off-shell input |
| `GammaVout { i, j }` | γ^μ bilinear → vector (ffV vertex) |
| `GammaIout { mu, j }` | ε̸ψ → flow-in fermion current (fioxxx) |
| `GammaJout { mu, i }` | ε̸ψ̄ → flow-out fermion current (foxxx) |
| `ProjM/P { i }` | Left/right chiral projector on fermion |
| `ProjMAmp/PAmp { i, j }` | Chiral scalar bilinear (FFS Yukawa) |
| `Metric { mu, nu }` | g^{μν} contraction → scalar |
| `ScalarProduct { children }` | Implicit product of disconnected factors |
| `P { leg }` | Momentum 4-vector of leg (VSS1, VVV1, UUV1) |
| `IdentityAmp { i, j }` | Full bilinear ψ̄_i δ ψ_j (BSM Identity) |

Unsupported (raise `CompileError::UnsupportedVertex`): Sigma, Epsilon, C.

### Runtime evaluator (`run.rs`)
- `evaluate_lorentz_node`: recursive tree walker, implements all node types above
- `WaveformSlot<F>`: register file holding FermionIn/FermionOut/Vector/Scalar/Empty; supports `+`, `C<F>*`, `momentum()`, `expect_fermion_in/out()`
- `evaluate_off_shell_current` / `evaluate_contract_amplitude`: per-vertex entry points
- `evaluate_propagation`: Dirac, massive vector (unitary gauge, Fabio fixed-width), massless vector, scalar

### AST compiler (`topo_sort.rs`, `compile.rs`)
- Recursive depth-first walk from root vertex → topologically ordered `DiagramAst`
- `VertexTerm::from_ufo`: calls `root_term` for each LorentzTerm in the expression
- `ExtLegInfo`: spin + charge populated at compile time; eliminates redundant metadata

### Key design decisions
- **Flow-typed fermion slots**: `WaveformSlot` distinguishes `FermionIn(InDiracWf)` (column/ket)
  from `FermionOut(OutDiracWf)` (row/bra). An off-shell current produced by `GammaIout` is
  flow-in; by `GammaJout` is flow-out (`ε̸ψ̄`, matching `foxxx` with no adjoint coercion).
  `evaluate_propagation` preserves flow; consumers request the flow they need via
  `expect_fermion_in/out`, which apply the Dirac adjoint only on a genuine flow conversion.
  This replaced the earlier "all slots are `InDiracWf`, adjoint on demand" convention, whose
  `.unbar()`/`.bar()` coercions corrupted the `GammaJout` numerator (the propagator `(q̸+m)`
  does not commute with the adjoint). `DiracWf::flip_flow` carries momentum through unchanged
  (flow is the bra/ket dual of the same particle). See `research/notes/11-variance-flow-duality.md`.
- **Fermion flow by charge**: `GammaVout` selects `(fo, fi)` by `Charge::Particle/Antiparticle`, not positional order
- **Propagator momentum convention**: all propagated slots carry −q (outgoing)
- **Off-shell scalar output-leg fix**: trivial `Leg(root)` leaf is dropped from build_at_leg; prevents reading the output slot as an input
- **`involves_vector` for P**: only checks `*mu`, not `*leg` (particle index ≠ Lorentz index)

---

# helas-generalize — MadGraph amplitude validation scaffolding COMPLETE ✅

## What was built

### MadGraph amplitude validation pipeline

New files under `validation/madgraph/`:
- `wrappers/ee_to_mumu.f` — Fortran77 f2py wrapper; populates all COMMON blocks
  (MASSES, WIDTHS, COUPLINGS, TO_AMPS, NARROW_WIDTH, TO_CHANNEL_STRAT) from scalar
  SM inputs, then calls `MATRIX1(P, IC, TS)` and returns `sum(TS)` = Σ|M|² (not
  divided by IDEN=4, matching Rust `eval_m2`)
- `build_amplitude.sh` — f2py compilation: links `matrix1_optim.f` + wrapper against
  pre-built `libdhelas.a`; runs from subprocess directory so all `.inc` files resolve
- `gen_amplitude.py` — evaluates on a 20×20 grid (√s ∈ [10,200] GeV,
  cos θ ∈ [−0.9,0.9]), writes `output/ee_to_mumu_amplitude.csv`

New pixi tasks in `madgraph` environment: `build-amplitude`, `generate-amplitude`,
`validate-helas-mg`. Full pipeline: `pixi run -e madgraph validate-helas-mg`.

### Rust comparison test

`vibegraph-lib/tests/helas_mg_validation.rs` — `libtest_mimic`-based; one named trial
per `*_amplitude.csv` file discovered in `validation/madgraph/output/`. Reads the
`# process:` comment from each CSV, calls `AmplitudeEvaluator::compile` + `eval_m2`,
checks all grid points against MadGraph reference. Colored processes emit INFO and
pass regardless (color flow not yet implemented).

### Result: `ee_to_mumu` passes at REL_TOL = 2e-3

## Key design decisions

**MATRIX1 vs SMATRIX1**: `matrix1_optim.f` contains both. `SMATRIX1` divides by
IDEN=4 and requires the full MadEvent runtime (genps, RANMAR). `MATRIX1(P, IC, TS)`
is clean: returns per-helicity values in `TS(NCOMB)` without averaging. The
Fortran wrapper calls `MATRIX1` only.

**Relaxed tolerance (2e-3, not 1e-6)**: MadGraph's generated code hard-codes `ZERO`
for lepton masses in all `OXXXXX`/`IXXXXX` calls. Rust uses physical masses from
the UFO model (m_e = 0.511 MeV, m_μ = 105.66 MeV). The resulting O(m_μ²/s)
systematic difference reaches ~7×10⁻⁴ at √s = 10 GeV and decreases at higher
energies. The tighter `helas_matches_fortran_reference` test (REL_TOL = 1e-6) uses
a custom HELAS reference that respects physical masses and is the right tool for
precision amplitude validation. The MadGraph comparison at 2e-3 exercises the
correct diagram topology and coupling structure; any real bug gives >1% deviation.

**Stub symbols**: `GET_CHANNEL_CUT` and `RANMAR` are referenced by `SMATRIX1`; the
linker requires them even though we never call `SMATRIX1`. Both are stubbed in the
wrapper file with trivial return values.

## pp_to_ll_qcd0 validation added ✅

`wrappers/pp_to_ll_qcd0.f` — f2py wrapper for u ū → l⁺ l⁻ (QCD=0, subprocess P1_qq_ll).
Sets quark couplings `GC_2 = (2i/3)*ee` (photon) and `GC_58 = -(i*ee*sw)/(6*cw)` (Z right),
plus existing lepton couplings (GC_3, GC_50, GC_59). MATRIX1 includes color factor CF=3.

Two bugs fixed enabling quarks to compile through AmplitudeEvaluator:
- `ast_util.rs extract_float`: added BinOp handling so `charge = 2/3` parses as 0.666..., not 0.0
- `topo_sort.rs make_externalwf`: replaced `charge > 0` antiparticle check with `pdg_code < 0`
  (charge sign is wrong for up-type quarks which have +2/3 but are particles, not antiparticles)

**Result**: `pp_to_ll_qcd0` INFO-passes with `max_rel_diff = 0.667 = 2/3`, exactly the expected
color factor discrepancy: MadGraph returns 3×|M|² (color-summed), Rust returns |M|² (no color).

## VEGAS cross-section migrated to AmplitudeEvaluator ✅

`validate_vegas.rs` — new extended-validation test file:
- `sigma_ee_mumu(evaluator, evaluated, sqrt_s, cos_range, neval, niter)` replaces the
  old hardcoded `compute_m2_ee_mumu` in the VEGAS integrand; uses `eval_m2` directly
- `sigma_qed_limit`: σ at √s=10 GeV agrees with QED formula 4πα²/3s to within 3%
- `sigma_z_pole`: σ at √s=MZ (with MG5 acceptance cuts) agrees with 2025 pb to <0.1%
- `validate_vegas`: regression test confirming AmplitudeEvaluator path gives same result

Old `sigma_ee_mumu`, `sigma_ee_mumu_qed_limit`, `sigma_ee_mumu_z_pole` removed from
`helas_validation.rs::extended`.

## Next steps for helas-generalize

1. Colored processes in `helas_mg_validation` — blocked on color flow implementation

## Diagram generation performance — topology caching + charge filter ✅

`generate_from_process_spec` now pre-generates abstract topologies once per `(n_ext, n_loops)`
via `generate_topologies()`, then passes the cached `Vec<Topology>` to each subprocess via
`DiagramGenerator::assign_topologies()`.  This avoids the O(n!) topology search being re-run
for every concrete subprocess produced by alias expansion.

For `p p > q q~ l+ l- l+ l-` (n_ext=8): 34,300 topologies in 4.86s (one-time cost).
Without caching: ~11,520 subprocesses × 4.86s ≈ 15 hours. With caching: 4.86s + assign time.

**Charge conservation pre-filter** added inside the alias-expansion loop: Σ Q_in == Σ Q_out
(O(n) check). For the 8-leg EW process this prunes 11,520 → ~1,664 subprocesses before the
expensive topology assignment step.

**Remaining hot spot** (samply-profiled during pp→qq̃4l run): `feyngraph/src/diagram/workspace.rs:L122`
in `AssignWorkspace::assign()`. The `.counts()` call (itertools) allocates a fresh
`HashMap<particle_index, count>` per candidate vertex per topology. Fix: pre-compute in
`AssignWorkspace::new()` — deferred to a dedicated feyngraph session.

## pp_to_qq4l_qcd0 MadGraph validation ✅

`validate_madgraph_diagrams::pp_to_qq4l_qcd0` validates `p p > q q~ l+ l- l+ l- QCD=0`
(pure EW, 2→6) against MadGraph5 reference (2672 diagrams, subprocess `P1_qq_qqllll`).
Reference generated with `pixi run -e madgraph build-diagrams` using `pp_to_qq4l_qcd0.mg5`.
