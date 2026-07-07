# Research & Implementation Status (2026-07-07)

# lorentz-runtime-eval — COMPLETE ✅

The topology-driven Lorentz structure evaluator is fully implemented and validated:
amplitude agrees with Fortran HELAS to <1e-7 (massive e⁺e⁻→μ⁺μ⁻) and with MadGraph
bit-for-bit on eleven processes (see helas-generalize below).

## What was built

### Core primitives
- External wavefunctions: `vxxxxx`, `sxxxxx`, `ixxxxx`/`oxxxxx`
- Propagators: Dirac, massive/massless vector, scalar — phase-normalized per chain
  type against MadGraph (V: −i/D, F: −i(q̸+m)/D, S: 1/D)
- Projectors: `GammaL::apply`, `GammaR::apply`; `SpinorRepr::slash`, `project_left/right`, `scalar_bilinear`

### Unified eval AST (`helas/eval/`)
One egglog-ready arena `Ast<T>` over the whole amplitude, evaluated by a single
forward pass (`run.rs apply`). Modules: `op.rs` (dataless `Op` + `Sym`/`Const`
leaves), `ast.rs` (CSR arena, s-expr `Display`/`FromStr`), `lower.rs`
(DiagramEval → `Ast<Sym>`), `fold.rs` (intern couplings/masses/widths into deduped
pools → `Ast<Const>`), `diagram_eval.rs` (pass-1/2 descriptors),
`root_diagram.rs`/`root_lorentz.rs` (rooting the undirected UFO tensor network at
the output leg, flow/crossing-aware). All SM node types implemented; unsupported
Lorentz primitives (Sigma, Epsilon, C) raise `CompileError::UnsupportedVertex`.

### Key design decisions
- **Flow-typed fermion slots**: `WaveformSlot` distinguishes `FermionIn(InDiracWf)`
  (column/ket) from `FermionOut(OutDiracWf)` (row/bra); propagation preserves flow;
  consumers request a flow via `expect_fermion_in/out` (strict — panics on mismatch),
  with the Dirac adjoint applied only on genuine flow conversion. External slots are
  flow-typed by *physical* flow (`FermionIn` iff `is_incoming == is_particle`);
  UFO i/j only selects input vs output leg. See `research/notes/11-variance-flow-duality.md`.
- **Per-leg crossing bit**: feyngraph binds outgoing legs in the all-incoming
  (crossed) convention, so final-state fermions sit at their antiparticle's UFO slot
  with the conjugate wavefunction type. A `LegFlow { flow, crossed }` is threaded
  through the bake; crossed gamma-chained chiral projectors conjugate (`P_χ→P_χ̄`,
  no sign), crossed scalar bilinears take −1. No explicit Denner sign anywhere —
  the runtime reversed-bilinear sign and the conjugation's negative gR/gL carry it.
- **Initial-spine parity sign**: a fermion line joining two incoming legs flips sign
  once per internal fermion propagator (odd-count flip), derived from baked flow
  (`root_diagram.rs spine_sign_from_flow`).
- **Fermion flow by charge**: `GammaVout` selects `(fo, fi)` by
  `Charge::Particle/Antiparticle`, not positional order.
- **Propagator momentum convention**: propagated slots carry the routed momentum
  unchanged (flow-in currents subtract the boson momentum, `fi.p − v.p`; flow-out
  add it — the fvixxx/fvoxxx conventions).

The full history of how these conventions were pinned down (six root causes,
the per-diagram MadGraph oracle, false leads) is in
`research/notes/12-helas-continuum-bugfix-journey.md`.

---

# helas-generalize — COMPLETE for single color flow ✅

## MadGraph amplitude validation pipeline

Files under `validation/madgraph/`, all registry-driven so a new process is one
entry in each registry (no new bespoke scripts):
- `wrappers/generic.f` — one f2py wrapper for every process: calls `setpara`
  (couplings from `param_card.dat`, no hand-coded `GC_*`) and links the
  launch-built `libmodel.a`. `TS` is sized `3**NEXTERNAL` so massive external
  vectors (3 helicities) fit. `ee_to_mumu` / `pp_to_ll_qcd0` migrated onto it; no
  bespoke wrappers remain.
- `build_amplitude.sh` — registry-driven (`GENERIC_PROCESSES`,
  `AMP_PROBE_PROCESSES`); `subprocess_dir()` globs the unique `P1_*` dir. f2py
  compilation against pre-built `libdhelas.a`; also builds the generic per-diagram
  AMP-dump probes from `wrappers/amp_probe.f.in` (`@NGRAPHS@` substituted, `sed`
  strips `SMATRIX1` to drop MadEvent deps).
- `gen_amplitude.py` — registry-driven (`Process` dataclass + `PROCESSES` list),
  RAMBO n-body momenta with massive rescaling (Newton solve for ξ), one
  momenta-based CSV schema (`# process:` / `# n_ext:` headers) for all processes.
- `compare_amps.py` — one process-parameterized per-diagram matcher (reads the
  `#hel` dump written by the `probe_process_diagrams` Rust test, evaluates the MG
  `mg_amp_probe_<name>` module at the same helicity combos, auto-matches diagrams,
  factors a global phase). Replaces the two bespoke note-12 probes/matchers.

Pixi tasks (`madgraph` env): `build-amplitude`, `generate-amplitude`,
`validate-helas-mg`.

## Rust comparison test — all eleven processes ENFORCED

`vibegraph-lib/tests/validate_helas_mg.rs` (libtest_mimic; one trial per
`*_amplitude.csv`), evaluated with each process's actual MG `param_card.dat`
(restrict cards baked into UFO params) for bit-for-bit comparison at REL_TOL 1e-10:

| process | max_rel_diff | color factor | new axis |
|---------|-------------|--------------|----------|
| `ee_to_mumu` | 1.1e-14 | 1 | baseline 2→2 |
| `pp_to_ll_qcd0` (u ū → l⁺ l⁻) | 1.7e-14 | 3 | quark initial state |
| `ee_to_mumu_tata_qcd0` (2→4) | 5.4e-13 | 1 | internal-H, spine |
| `uux_to_ccx_emmm_qcd0` (2→6, 579 diagrams) | 2.1e-13 | 9 | massless 2→6 |
| `ee_to_ee` (Bhabha) | 2.7e-14 | 1 | s⊕t, crossed-line sign, ZERO-width t-channel |
| `ee_to_mumua` | 3.9e-13 | 1 | external vector `vxxxxx` |
| `ee_to_ttx` | 4.8e-15 | 3 | massive external fermions |
| `ee_to_wpwm` | 4.4e-14 | 1 | VVV, massive charged externals, `LowerVout` |
| `ee_to_zh` | 9.5e-14 | 1 | external scalar, on-shell VVS, massive V propagator |
| `ee_to_tatah` | 3.9e-13 | 1 | external FFS Yukawa, goldstone/ghost exclusion |
| `bbx_to_ccx_emmm_qcd0` (2→6, 615 diagrams) | 6.3e-14 | 9 | massive internal fermions, `PropagateLowered` |

For NCOLOR=1 processes `MG = CF(1,1)·eval_m2_rust`
(`validate_helas_mg::color_factor`). True multi-flow color (e.g. same-flavor
`u u~ > u u~`, NCOLOR=2) awaits a color-flow implementation.

### Evaluator features added for the new axes
- **Massless-vector helicities**: an external vector with `mass_param == "ZERO"`
  has 2 helicity states (`[-1,1]`), massive 3 (`[-1,0,1]`)
  (`compile::helicity_states_for_spin`).
- **`LowerVout`** (VVV): a P-carrying Lorentz structure lowers its vector-output
  index (`g·J`) without the vertex `−i`, vs VVS's `MetricVout` (`−g·J`, carries the
  `−i` via `MetricNegI` when rooted as an amplitude).
- **t-channel ZERO width**: a spacelike propagator (one incoming external in its
  subtree) gets zero width, matching MadGraph (`PropInfo::t_channel`, dropped in
  `lower.rs`).
- **Crossed-line sign**: a fermion line joining two final-state legs (all-incoming
  crossing) carries an extra `−1` (`spine_sign_from_flow`).
- **Goldstone/ghost exclusion**: unitary-gauge like MadGraph — particles with
  `goldstoneboson`/`ghost_number` are dropped from the feyngraph model
  (`ufo/topo.rs`, `Particle::is_goldstone`).
- **`PropagateLowered`**: the massive-vector propagator fed an index-flipped
  (`MetricVout`) current forms its `g^{μν}` term off the raised current and its
  longitudinal `q^μ q^ν/m²` off the natural pairing, then undoes the `MetricVout`
  storage sign (`PropInfo::lowered_storage`, `propagate_core`).

MG's partonic σ̂ for the uux 2→6 at √s=500 (lpp=0) = 6.556e-7 pb is banked as a
future `validate-vegas` reference.

## VEGAS cross-section on the generalized evaluator

`validate_vegas.rs` (extended-validation): the VEGAS integrand uses
`AmplitudeEvaluator::eval_m2` (hardcoded `compute_m2_ee_mumu` removed).
- `sigma_qed_limit`: σ(√s=10) vs 4πα²/3s within 3%
- `sigma_z_pole`: σ(√s=MZ, MG5 cuts) vs 2025 pb to <0.1%
- `validate_vegas`: regression on the evaluator path

## Diagram generation performance

- Topology caching: `generate_topologies()` once per `(n_ext, n_loops)`, shared via
  `DiagramGenerator::assign_topologies()` (pp→qq̃4l: 4.86s once vs ~15h naive).
- Charge-conservation pre-filter in alias expansion: 11,520 → ~1,664 subprocesses.
- Remaining hot spot in the feyngraph submodule (`AssignWorkspace::assign`
  `.counts()` allocation) — see TODO `feyngraph-perf`.

`validate_madgraph_diagrams::pp_to_qq4l_qcd0` validates `p p > q q~ l+ l- l+ l-
QCD=0` (2→6 EW) diagram counts against MadGraph (`NGRAPHS` from `matrix1_orig.f`).

## Next steps

1. Color flow (NCOLOR≥2) — unblocks same-flavor QCD=0 validation (`u u~ > u u~`,
   the one remaining `mg-validation-coverage` process) and hadronic σ.
2. `lorentz-eval-node-2level` refactor — now fully unblocked (VVV + lowered-vector
   propagation are MG-validated). Would collapse the `Metric`/`MetricNegI`/
   `MetricVout`/`LowerVout` and `Propagate`/`PropagateLowered` op pairs into
   variance-typed nodes, removing the hand-coded index juggling in `run.rs`.
3. Unweighted event output (LHEF).
