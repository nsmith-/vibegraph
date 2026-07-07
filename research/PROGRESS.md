# lorentz-runtime-eval — COMPLETE ✅

The topology-driven Lorentz structure evaluator is fully implemented and validated.
All 135 tests pass; amplitude agrees with Fortran HELAS to <1e-7 for massive e⁺e⁻→μ⁺μ⁻.

> **Update 2026-07-06 — ee→μμττ continuum chirality bug FIXED (per-diagram exact vs MG).**
> Root cause: feyngraph presents outgoing legs in the all-incoming (crossed) convention,
> so every final-state fermion binds at its antiparticle's UFO slot with the conjugate
> wavefunction type (outgoing μ⁺ = a *bra* at the `mu-` slot). By the reversal identity
> `ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁` a crossed pair is exact for vector structures but needs
> `P_χ → P_χ̄` (no sign) on gamma-chained chiral projectors and an explicit −1 on scalar
> bilinears (`CΓᵀC⁻¹ = Γ` for `1`/`P_χ`). Since crossing inverts slot identity and flow
> together, flow-vs-slot inspection cannot detect it — a per-leg `crossed` bit
> (`LegFlow`) is now threaded through the bake (externals: `!incoming`; off-shell
> currents inherit it). Together with the flow-driven rooting corrections for uncrossed
> reversals (initial-state annihilation pair, e-spine absorptions; NO explicit Denner
> sign — the runtime reversed-bilinear sign supplies it) and a scalar-propagator −i
> (S-chain phase relative to the F-chain, pinning the Higgs-diagram interference), the
> full-helicity oracle `validation/madgraph/compare_full_hel.py` gives −1.000 in every
> 25-diagram × 16-helicity cell (≤1.5e-13) and per-helicity |M|² matches MG to 4e-14.
> `validate_helas_mg`: **ee→μμττ max_rel_diff 1.8e-14 over 50 points** (the historical
> 0.2–0.57% residual was this bug). ee→μμ / pp→ll unchanged (1e-14); uux 2→6 (2.66e1)
> is a separate open bug. 166/166 lib tests, Ward suite, and the Fortran HELAS
> reference all pass.

> **Update 2026-06-19 — eval/ unified-AST refactor (Steps 3+4) complete.** The two
> nested eval trees (`DiagramEvalTree` + `LorentzEvalTree`) are now flattened into ONE
> egglog-ready arena `Ast<T>` over the whole amplitude, evaluated by a single forward
> pass / one `apply` match. New modules `op.rs` (dataless `Op` + `Sym`/`Const` leaves),
> `ast.rs` (CSR arena, `Tree`, s-expr `Display`/`FromStr`), `lower.rs` (DiagramEval →
> `Ast<Sym>`), `fold.rs` (intern couplings/masses/widths/coeffs into deduped C<F>/F
> pools → `Ast<Const>`); the old pass-1+2 descriptors moved to `diagram_eval.rs`. The
> sections below naming `dispatch.rs`/`topo_sort.rs`/`evaluate_lorentz_node` describe the
> pre-refactor layout (now `root_lorentz.rs`/`root_diagram.rs` + the unified `apply`).
> Bit-for-bit |M|²-preserving: ee 4.17e-14, pp 2.14e-14, tata 5.73e-3, uux 2.66e1 vs MG.
> See memory `helas-eval-tree-refactor` for the full module layout.

> **Update 2026-06-21 — continuum parity-ratio: e-spine chiral fermion current RULED OUT.**
> The leading suspect for the per-helicity parity-conjugate `gL/gR` reweighting
> (`c_h·c_{63−h}=1`) was the chiral off-shell fermion current on the incoming e-pair
> spine, which the bake roots at the `ProjM`/column fermion leg → composition
> `propagate(ProjM(ε̸·ψ) [+ 2·ProjP(ε̸·ψ)])` (projector AFTER the gamma, vs the leg-2
> path's `ε̸·ProjM(ψ)`). Since `γ^μ P_L = P_R γ^μ` these orderings carry opposite
> chirality, so this is a distinct code path ee→μμ and the leg-2 tests never exercise.
> A textbook Dirac-matrix reconstruction (explicit Weyl-basis γ, new test
> `test_chiral_off_shell_fermion_espine_vs_textbook`) proves the eval's e-spine current
> equals `−S(q)·(P_L+2P_R)·ε̸·ψ` EXACTLY (clean −1 propagator-normalisation factor) for
> every helicity × charge × mass, with a *generic* ε carrying longitudinal content. So
> the e-spine chiral fermion current — handedness AND relative `gL/gR` weight — is
> physically correct; the parity reweighting is NOT here. Remaining suspects: the
> longitudinal-Z Ward cancellation across the full massless-spine diagram sum, and the
> end-to-end coupling application (GC_50→FFV2, GC_59→FFV4) for the fermion-output rooting
> (same per-term mechanism as the bit-matching vector output, so likely fine). See memory
> `helas-2to6-offshell-bug`.

> **Update 2026-06-21 (cont.) — longitudinal-Z production ALSO ruled out (Ward test).**
> `test_longitudinal_z_current_transverse_for_massless_fermions` builds the off-shell Z
> current from a massless e⁺e⁻ pair via the production eval with the real `ℓ̄ℓZ` vertex
> (FFV2·GC_50 ⊕ FFV4·GC_59, so `gL ≠ gR` — a genuine *axial* current the vector
> photon-Ward tests can't probe) and verifies `q_μ J^μ = 0` at √s=1 and √s=m_Z (where the
> `q^μq^ν/m²` numerator is largest). It passes: the current is exactly transverse, so the
> longitudinal mode decouples on the production side; with the absorption side already
> cleared (any ε handled correctly), longitudinal-Z is clean end-to-end for massless
> fermions. Both chiral suspects for the parity reweighting are now ruled out — the
> massless-τ matcher residual is either a per-diagram basis/convention artifact vs MG
> (coherent total only 0.2% off → likely |M|²-harmless) or a coherent-sum relative
> sign/coupling that no per-primitive test can isolate.

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

## uux 2→6 continuum fixed — all validate_helas_mg processes enforced ✅ (2026-07-06)

`u u~ > c c~ e+ e- mu+ mu-` QCD=0: max_rel_diff **2.66e1 → 2.14e-13** over 50 points;
`uux_to_ccx_emmm_qcd0` and `ee_to_mumu_tata_qcd0` promoted to enforced in
`validate_helas_mg` (all four processes now bit-match MadGraph, color factor 9/1).

Diagnosis (new per-diagram MG oracle for uux): every one of the 579 diagrams already
matched MadGraph's per-helicity magnitudes to machine precision; the residual was purely
diagram-class **global phases** (528 diagrams at +i, the 48 u-spine diagrams at −i, the
3 ZZH Higgs diagrams at −1) whose mixing broke the strongly-cancelling coherent sum
(coh/incoh ≈ 3e-3 → 26.6× |M|² error). Two production fixes in `helas/eval/`:

1. **Chain-phase normalization** (`run.rs propagate_core`): fermion propagator
   `−(q̸+m)/D → −i(q̸+m)/D`, scalar propagator `−i/D → 1/D`. All three chain types
   (V/F/S) now carry the same phase relative to MadGraph; previously each F- or S-chain
   deviated by a factor i, invisible while every diagram in a process had identical
   chain content and fatal when classes mix (uux continuum: 2 F-chains; Higgs class:
   1 S-chain, 0 F-chains — the class the "#vertices − #fermion-lines" count missed
   because ZZH vertices are not on fermion lines).
2. **Initial-spine sign per propagator** (`root_diagram.rs spine_sign_from_flow`):
   the crossing sign for a fermion line joining two incoming legs is −1 per internal
   fermion propagator (flip iff odd count), not once per line. Equivalent for 1-prop
   spines (the 2→4 e-spine), wrong for the uux u-spine (2 props, 48 diagrams).

New committed oracle tooling: `validation/madgraph/probe_uux_amp.py` +
`compare_uux_amps.py` + `wrappers/uux_amp_probe.f` (per-diagram AMP dump via a
build-time-patched `matrix1_orig.f`, `build_amplitude.sh build_uux_amp_probe`);
VG side `run::tests::probe_uux_diagram_classes` (class decomposition + full
[diagram×helicity] dump). Test re-pins: production fermion chain = `i·fvixxx`
= `−ffv2_2(bare)`; off-shell e-line now equals MG exactly. Post-fix: uux per-diagram
ratios uniform −i; compare_full_hel.py 25×16 cells magnitude 1.000, uniform phase;
Ward suite, Fortran HELAS reference, 166 lib tests, clippy clean.
