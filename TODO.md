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
3. Colored processes in `helas_mg_validation` — blocked on color flow implementation

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

### `feyngraph-perf` — Investigate feyngraph allocation overhead

Profiling (`pixi run profile-diagrams`) shows the hot loop is feyngraph's recursive topology
workspace. Two directions: try `mimalloc`/`tikv-jemallocator` (two-line change), or upstream
a cargo feature to gate rayon for sequential callers.

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
