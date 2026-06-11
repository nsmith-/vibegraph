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

**Tasks**:
1. Replace calls to `compute_m2_ee_mumu` with `AmplitudeEvaluator::eval_m2`
2. Update phase-space loop to pass `&DiagramSet` and `&EvaluatedModel`
3. Validate σ(e⁺e⁻→μ⁺μ⁻) unchanged vs hardcoded reference
4. Validate a second process vs MadGraph

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
