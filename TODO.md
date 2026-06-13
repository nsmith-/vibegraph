# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (e⁺e⁻→μ⁺μ⁻, hardcoded) | ✅ Done | Validated against MadGraph to <0.1% |
| 3′ | HELAS generalized (topology-driven, arbitrary process) | ✅ Done | Agrees with Fortran HELAS to <1e-7 |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | σ ≈ 2025 pb at √s = 91.2 GeV vs MadGraph ref |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

---

## 🔴 High — wire up the generalized evaluator

### `helas-generalize` — Topology-driven HELAS evaluator

Replace the hardcoded `compute_m2_ee_mumu` with the generalized `AmplitudeEvaluator` and
validate a second process (e.g. uū→dd̄) vs MadGraph.

**Scaffolding done** ✅:
- MadGraph amplitude validation infrastructure: `validation/madgraph/wrappers/`,
  `build_amplitude.sh`, `gen_amplitude.py`, pixi tasks `build-amplitude` /
  `generate-amplitude` / `validate-helas-mg`
- `vibegraph-lib/tests/helas_mg_validation.rs` — libtest_mimic test, one trial per process
- `ee_to_mumu` passes at REL_TOL=2e-3; the looser tolerance is intentional: MadGraph's
  generated `matrix1_optim.f` treats all leptons as massless (hard-coded `ZERO` in HELAS
  calls), while Rust uses physical masses. The systematic O(m_μ²/s) difference reaches
  ~7×10⁻⁴ at √s=10 GeV; any real amplitude bug gives >1% error.

**Amplitude validation for pp→ll (QCD=0) done** ✅:
- `wrappers/pp_to_ll_qcd0.f` wraps P1_qq_ll (u ū → l⁺ l⁻) with correct quark couplings
- `max_rel_diff = 2/3` in `helas_mg_validation` — exactly the color factor CF=3 missing in Rust
- Two parser/assertion bugs fixed: fractional charge `2/3` now parses correctly; antiparticle
  check uses `pdg_code < 0` (not `charge > 0`, which is wrong for up-type quarks)

**Remaining tasks**:
1. ✅ Replace calls to `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2` in the VEGAS integrand (`validate_vegas.rs`)
2. ✅ Break out VEGAS cross-section tests into `validate_vegas.rs` (`sigma_qed_limit`, `sigma_z_pole`, `validate_vegas`)
3. 🟡 **Off-shell fermion-line momentum routing FIXED; residual |M|² value error remains** (2→6).
   - ✅ **Momentum routing fixed** (`helas/eval/run.rs`): three conflated bugs in how
     off-shell currents chain — (a) `evaluate_propagation` flipped `-wf.momentum` on every
     propagator (reference HELAS already emits conserved momentum → flip mis-routed every
     >1-vertex line, wrong q², spurious poles); (b) flow-in `GammaIout` must subtract the
     vector `q = fi.p − v.p` (Fortran `fvixxx`), opposite to flow-out `GammaJout`/`fvoxxx`
     (`fo+vc`) — both used `f+v`; (c) earlier leg-index crash already fixed. After: every
     internal q² is physical (u-ū s-channel boson recovers q²=s exactly); `ee_to_mumu`/
     `pp_to_ll` still pass; uux rel diff **2.26e10 → 6.95e6**.
   - ✅ **Flow-typed fermion slots (recommended fix implemented).** `WaveformSlot::Fermion`
     was split into `FermionIn(InDiracWf)` / `FermionOut(OutDiracWf)` so an off-shell fermion
     line carries its flow in the slot. `GammaIout` → `FermionIn`, `GammaJout` → `FermionOut`
     (now mirrors `foxxx`'s `ε̸ψ̄` directly, no adjoint hack); `evaluate_propagation` is
     flow-preserving for both; consumers pull the flow they need via
     `WaveformSlot::expect_fermion_in/out` (Dirac adjoint applied only on genuine flow
     conversion). This removed the wrong `.unbar()`/`.bar()` coercions: `GammaJout ≅ foxxx`
     now matches exactly. Also fixed `DiracWf::flip_flow`, which was negating the momentum
     (flow is the bra/ket dual of the *same* particle — momentum carries through unchanged).
   - 🟡 **Residual uux value error — re-measure.** With flow-typed slots the row/column
     propagator-numerator ambiguity that was the leading suspect is resolved at the unit
     level. The uux 2→6 |M|² discrepancy needs re-measuring against MadGraph (it was ~6.95e6
     before this change); also re-check fermion permutation signs. (Verified NOT caused by the
     `charge()`-vs-structural row/col choice in `GammaVout` — identical result either way.)
     `uux` stays informational in `validate_helas_mg`; trace tool: `run::tests::debug_uux_trace`
     (ignored).
4. ✅ **Single-color-flow validation via scalar color factor.** For NCOLOR=1 processes,
   `MG = CF(1,1)·eval_m2_rust` (e.g. Nc=3 for `pp_to_ll`, Nc²=9 for `uux_to_ccx`).
   `validate_helas_mg::color_factor` applies it; `pp_to_ll_qcd0` now *enforced* (was
   informational), matching at cf=3. True multi-flow color (e.g. same-flavor
   `u u~ > u u~`, NCOLOR=2) still needs a color-flow implementation.
5. ✅ **Generic MadGraph wrapper + n-body validation infra.**
   - `wrappers/generic.f`: one f2py wrapper for any process — calls `setpara` (couplings
     from `param_card.dat`, no hand-coded `GC_*`) and links the launch-built `libmodel.a`.
     Validated bit-for-bit against the old `ee`/`pp_to_ll` wrappers.
   - `scripts/uux_to_ccx_emmm_qcd0.mg5`: dedicated single-flow 2→6 (`u u~ > c c~ e+ e- mu+ mu-`,
     QCD=0); `launch` with `lpp=0`, √s=500 → optimized matrix element + partonic σ̂ =
     6.556e-7 pb (future `validate-vegas` reference).
   - `gen_amplitude.py`: RAMBO n-body momenta + momenta-based CSV schema (`# n_ext:` header).
   - `build_amplitude.sh`: `compile_process_generic`.
   - Follow-up: migrate `ee`/`pp_to_ll` off their bespoke wrappers to `generic.f`.

**Future: hadronic cross section for pp→ll requires PDF sampling**:
- Amplitude validation only tests u ū → l⁺ l⁻ (one parton flavor).  A hadronic σ
  requires integrating over parton flavors weighted by the PDF: σ = Σ_{q} ∫ dx₁ dx₂
  f_q(x₁) f_{q̄}(x₂) × σ̂(q q̄ → l⁺ l⁻).  Since MadGraph's subprocess treats all quarks
  as massless, the matrix element structure is the same for u and c (charge 2/3) and
  separately for d and s (charge −1/3, different coupling constant); flavors can be
  grouped by charge type.  Blocked on: color flow, PDF interface (e.g. LHAPDF), and
  n-body phase space generalization for the partonic √ŝ scan.

_Depends on: `lorentz-runtime-eval` (✅)_
_Unblocks: Process generalization beyond e⁺e⁻→μ⁺μ⁻_

---

## 🟡 Medium — CLI integration

### `global-config` — Implement `vibegraph_lib::config::GlobalConfig`

A thin coordinator that wires `ParsedProcCard` → `UFOModel` loading for the CLI.

```rust
pub struct GlobalConfig {
    pub ufo_search_path: PathBuf,
    pub restrict_path_override: Option<PathBuf>,
}
impl GlobalConfig {
    pub fn load_ufo(&self, spec: &Option<ModelImport>) -> Result<UFOModel, UfoError> { ... }
}
```

_Depends on: `helas-generalize`_
_Unblocks: Full CLI with process cards_

---

## 🟢 Later — polish and extensibility

### `feyngraph-perf` — Fix feyngraph allocation hot spot

**Hot spot identified** (samply profile, pp→qq̃4l run): `workspace.rs:L122` in
`AssignWorkspace::assign()` calls `.counts()` (itertools) on every candidate vertex for every
topology for every subprocess. Each `.counts()` call allocates a fresh `HashMap<particle_index,
count>`. For pp→qq̃4l: ~1,664 subprocesses × 34,300 topologies × O(vertices) = ~340M HashMap
allocations. **Fix**: pre-compute per-vertex particle counts in `AssignWorkspace::new()` and
reuse them in the inner loop. This is a change to the `feyngraph` submodule; deferred to a
dedicated feyngraph session.

Vibegraph-side mitigations already applied:
- Topology caching: `generate_topologies()` called once per `n_ext`; all subprocesses share the
  same `Vec<Topology>` via `DiagramGenerator::assign_topologies()`.
- Charge conservation pre-filter: eliminates ~86% of alias-expanded candidates before topology
  assignment (11,520 → ~1,664 for pp→qq̃4l).

### `madgraph-diagram-cmp-per-flavor` — Match subprocesses by flavor in diagram validation

The `validate_madgraph_diagrams` reference count now uses the representative subprocess's
true Feynman-diagram count (`NGRAPHS` from `matrix1_orig.f`), not `MAPCONFIG(0)` from
`configs.inc` (which counts the phase-space integration-channel *union* across all flavor
variants in a P-class — e.g. 2672 vs the actual 2316 for `u u~ > u u~ l+ l- l+ l-`).

**Remaining gap**: the comparison still assumes vibegraph's first-enumerated subprocess in
each particle-type group matches MadGraph's `matrix1` representative. That holds for the
current process set but is fragile. Refinement: have `count_mg_style_topologies` (in
`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) match each vibegraph subprocess to the
MadGraph variant with the same flavors (via `leshouche.inc` `IDUP`) rather than picking one
representative per coarse particle-type class, and compare per-subprocess `NGRAPHS`. This
would also let the test validate *all* 40 variants of the qq4l class instead of just one.

### `lips-nbody` — n-body LIPS phase-space generator

Generalize phase-space sampling to 3+ final-state particles using recursive 2-body
decomposition (RAMBO-style). Research Rust options before committing to an approach.

_Depends on: `xsec-ee-mumu` (✅)_

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.).

_Depends on: `helas-generalize`_

### `typed-units` — Typed physical units

Research `uom`/`dimensioned`/`units` crates for typed four-momenta and cross sections.

---

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (✅) ──→ helas-generalize ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┘
lips-nbody ─────────────────────────────────────────────────────────────────────────────┘
global-config ───────────────────────────────────────────────────────────────────────────┘
```
