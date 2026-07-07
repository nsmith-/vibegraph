# Research & Implementation Status (2026-07-06)

# lorentz-runtime-eval — COMPLETE ✅

The topology-driven Lorentz structure evaluator is fully implemented and validated:
amplitude agrees with Fortran HELAS to <1e-7 (massive e⁺e⁻→μ⁺μ⁻) and with MadGraph
bit-for-bit on four processes (see helas-generalize below).

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

Files under `validation/madgraph/`:
- `wrappers/generic.f` — one f2py wrapper for any process: calls `setpara`
  (couplings from `param_card.dat`, no hand-coded `GC_*`) and links the
  launch-built `libmodel.a`. (Legacy bespoke wrappers `ee_to_mumu.f` /
  `pp_to_ll_qcd0.f` still exist; migration to `generic.f` is a follow-up.)
- `build_amplitude.sh` — f2py compilation against pre-built `libdhelas.a`; also
  builds the per-diagram AMP-dump probes (`build_uux_amp_probe` awk-patches
  `matrix1_orig.f`).
- `gen_amplitude.py` — RAMBO n-body momenta, momenta-based CSV schema
  (`# n_ext:` header); 20×20 (√s, cosθ) grid for the 2→2 processes.
- Per-diagram oracle tooling: `probe_amp.py` / `probe_uux_amp.py` (MG side),
  `compare_full_hel.py` / `compare_uux_amps.py` (matchers), plus `#[ignore]`
  probes `probe_eemumutata_diagrams` / `probe_uux_diagram_classes` in
  `helas/eval/run.rs` dumping [diagram × helicity] complex amplitudes.

Pixi tasks (`madgraph` env): `build-amplitude`, `generate-amplitude`,
`validate-helas-mg`.

## Rust comparison test — all four processes ENFORCED

`vibegraph-lib/tests/helas_mg_validation.rs` (libtest_mimic; one trial per
`*_amplitude.csv`), evaluated with each process's actual MG `param_card.dat`
(restrict cards baked into UFO params) for bit-for-bit comparison at REL_TOL 1e-10:

| process | max_rel_diff | color factor |
|---------|-------------|--------------|
| `ee_to_mumu` | 4.2e-14 | 1 |
| `pp_to_ll_qcd0` (u ū → l⁺ l⁻) | 2.1e-14 | 3 |
| `ee_to_mumu_tata_qcd0` (2→4, 25 diagrams) | 1.8e-14 | 1 |
| `uux_to_ccx_emmm_qcd0` (2→6, 579 diagrams) | 2.1e-13 | 9 |

For NCOLOR=1 processes `MG = CF(1,1)·eval_m2_rust`
(`validate_helas_mg::color_factor`). True multi-flow color (e.g. same-flavor
`u u~ > u u~`, NCOLOR=2) awaits a color-flow implementation.

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

1. Broaden MG validation coverage (Bhabha, external γ/W/Z/h, massive fermions,
   massive-b 2→6 spine) — prioritized list in TODO `mg-validation-coverage`.
2. Color flow (NCOLOR≥2) — unblocks same-flavor QCD=0 validation and hadronic σ.
3. `lorentz-eval-node-2level` refactor — now unblocked; prefer after a VVV MG
   validation exists (it changes the convention surface).
