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

### `lorentz-eval-node-2level` — Two-level LorentzEvalNode + variance-aware slots

Reorganize `LorentzEvalNode` (`helas/eval/dispatch.rs`) into two levels:
- **outer = output type**, carrying variance/flow: `ScalarOut` / `VectorOut<V>` /
  `SpinorOut<Flow>` (+ tensor later);
- **inner = the UFO primitive** (Gamma, Metric, P, Proj, …), with one node per
  (structure × distinct output-leg type) — e.g. `Metric`→scalar vs `MetricVout`→vector
  are the same structure with different outputs.

Make `VectorWf` (and `WaveformSlot`) variance-parameterized so each vertex returns its
natural variance, the propagator's raise/lower is type-checked, and contractions can't
silently double-apply or drop the metric. This removes the manual component/index
hand-coding currently in `MetricVout` (`run.rs`), which had to bypass the typed repr
because `VectorWf.eps` is pinned to `Contravariant` — the exact variance-bug class the
typing was meant to prevent. (FFV currents are contravariant `J^μ`; the metric vertex
emits covariant `g^{μν}V_ν`, so a single `Covector` slot variant is insufficient — the
variance must be tracked per output.)

**Sequence after** the continuum cancellation bug below, and only once VVS (ideally also
VVV) regression tests exist — this refactor changes the convention surface where the
off-shell-current sign/metric bugs live.

### `helas-2to6-continuum` — Fix the pure-EW continuum |M|² (uux 2→6)

**2026-07-06 ee→μμττ (2→4 continuum) FIXED — per-diagram, per-helicity exact vs MG.**
Root cause: feyngraph binds outgoing legs in the all-incoming (crossed) convention, so
final-state fermion pairs sit in each other's UFO slots with conjugate wavefunction types;
`ū₁Γv₂ = −ū₂(CΓᵀC⁻¹)v₁` then requires conjugating gamma-chained chiral projectors
(`P_χ→P_χ̄`, no sign) and negating scalar bilinears on crossed lines. A per-leg `crossed`
bit (`LegFlow`, `root_lorentz.rs`) is threaded through the bake; uncrossed reversals
(annihilation pair, e-spine absorptions) are handled by the flow-driven re-rooting /
projector flip with no explicit sign; the scalar propagator gains a −i (S-chain phase
relative to the F-chain, pinned by the Higgs-diagram interference). Oracle
`validation/madgraph/compare_full_hel.py`: all 25×16 cells −1.000 (≤1.5e-13); per-hel
|M|² vs MG 4e-14; `validate_helas_mg` ee→μμττ max_rel_diff **1.8e-14** (the historical
0.2–0.57% residual was this bug). uux 2→6 unchanged at 2.66e1 — still open, below.

**2026-06-14 fermion-FLOW fix: max_rel_diff 7.26e3 → 3.96e1 (~180×).** The bulk of the
continuum error was that off-shell fermion currents and bilinears took their bra/ket flow
from the UFO `Gamma` i/j leg position (structural), while `build_external_slot` built EVERY
external as `FermionIn` and relied on `expect_fermion_in/out` to silently `dualize` at
consumption. That coercion is wrong for any leg whose physical flow disagrees with the
structural slot (e.g. an incoming particle = ket landing in the barred slot, ISR), corrupting
a mid-line spinor. Fixes (`helas/eval/`):
- `build_external_slot` flow-types externals: `FermionIn` iff `is_incoming == is_particle`.
- flow-driven dispatch: `resolve_bra_ket` (GammaVout/ProjMAmp/ProjPAmp/IdentityAmp) picks
  bra/ket by ACTUAL flow; `off_shell_fermion_current` makes the current follow the input
  fermion's flow (fvixxx for ket / fvoxxx for bra). Structural i/j only selects which leg is
  input vs output, never the flow.
- `expect_fermion_in/out` now STRICT (panic on flow mismatch) — enforced invariant.
Validated reference-free by machine-precision (~1e-13) full-amplitude U(1) Ward tests
(`run::tests::test_ward_identity_full_amplitude_*`): 2→3 `eemumua`, 2→4 `eemumuaa` (chained
currents), quark `uumumua`. ee→μμ & pp→ll unchanged at 6.69e-4; `test_eval_jioxxx` (massive-Z
off-shell vector current incl. longitudinal term) passes → massive-Z confirmed correct.

**2026-06-14 FFS Higgs current momentum fix.** The 2→5 Ward test
`test_ward_identity_full_amplitude_eemumutata_a` (`e+ e- > mu+ mu- ta+ ta- a`, 3 fermion lines)
was failing at ~1e-4. Cause: the off-shell SCALAR bilinear nodes (`ProjMAmp`/`ProjPAmp`/
`IdentityAmp`) computed momentum `fo.p + fi.p`, but the analogous off-shell VECTOR current
`GammaVout` uses `fo.p − fi.p` (HELAS jioxxx convention). Harmless at the amplitude sink
(momentum unused) but non-conserving when the scalar is an off-shell Higgs current feeding a VVS
vertex (only with ≥3 fermion lines). Found via `probe_2to5_momentum` (6/174 diagrams, all Higgs,
non-conserving). Fix: `+`→`−`. Now all 174 conserve, 2→5 Ward passes ~1e-13 (un-ignored as a
guard); `probe_uux_momentum` shows all 579 uux diagrams conserve.

**2026-06-14 mass/param consistency: ee→μμ & pp→ll now BIT-MATCH MadGraph (~1e-14, was 6.69e-4).**
Root cause (UFO loading bug): `UFOModel::load` found `restrict_default.dat` but used it only for
vertex pruning — its parameter VALUES were discarded, so `model.evaluate()` used parameters.py
defaults (physical MM=0.10566) while MadGraph bakes the restriction in (massless light fermions,
rounded SM inputs). Fixes: `ParameterSet::apply_restrict` bakes the restrict card's externals into
the param defaults (called from `load`); slha parses `DECAY <pdg> <width>` (was skipped) so width
params resolve; `validate_helas_mg` evaluates with each process's actual MG `param_card.dat` for a
bit-for-bit comparison (REL_TOL 2e-3→1e-10). uux stayed at 3.96e1 under this exact comparison ⇒ the
uux residual is genuinely mass/param-independent, and the MG validation is now a clean oracle.

**REMAINING (~24% at pt0, 40× worst-case): uux continuum γ/Z relative-phase — a SEPARATE bug.**
The FFS fix did NOT move uux: uux's Higgs diagrams are VVS (HZZ), and with massless c the
c-Yukawa is 0 so there are no FFS-Higgs diagrams; the continuum is pure γ/Z. The residual is
momentum-CONSERVING, mass-independent (zeroing c/e/mu leaves ratio 0.76), ~1% per-diagram,
amplified by the strong cancellation (coh/incoh≈3e-3). It is NOT constrained by the external-
photon Ward (which only fixes the photon gauge structure, not the internal γ/Z magnitude), so
the passing 2→3/2→4/2→5/quark Ward tests can't see it — and those tests never checked |M|²
against a reference. NEXT: generate a MadGraph |M|² reference for a small **massless pure-γ/Z
continuum** process (e.g. `e+ e- > mu+ mu- ta+ ta-`, 2→4, 3 lines) and compare |M|² directly —
any disagreement >1e-6 is the bug (no mass systematic). If it agrees, escalate to 4 lines (a
line absorbing 3 internal bosons, as the uux u-line does). Per-diagram `AMP()` comparison vs
`matrix1_orig.f` would pinpoint the offending class. pixi/MadGraph is available
(`pixi run -e madgraph ...`); add the process to `validation/madgraph/scripts/` + `gen_amplitude.py`.

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
