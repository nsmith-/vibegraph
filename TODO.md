# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 11 processes bit-match MadGraph (≤6e-13, incl. 2→6, VVV, massive externals); single color flow only |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | σ ≈ 2025 pb at √s = 91.2 GeV vs MadGraph ref |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

`helas-generalize` is **done** for single-color-flow processes: `AmplitudeEvaluator`
drives the VEGAS integrand, and `validate_helas_mg` enforces bit-for-bit agreement
with MadGraph across **11 processes** (all ≤6.3e-13): `ee_to_mumu`, `pp_to_ll_qcd0`
(×CF=3), `ee_to_mumu_tata_qcd0`, `uux_to_ccx_emmm_qcd0` (×CF=9), plus the 7
`mg-validation-coverage` additions below (`ee_to_ee`, `ee_to_mumua`, `ee_to_ttx`,
`ee_to_wpwm`, `ee_to_zh`, `ee_to_tatah`, `bbx_to_ccx_emmm_qcd0` ×CF=9). The
three-week continuum bug hunt that got there is written up in
`research/notes/12-helas-continuum-bugfix-journey.md`.

---

## 🔴 High — broaden the MadGraph amplitude validation surface

### `mg-validation-coverage` — New processes for `validate_helas_mg` ✅ Done

All 7 single-flow processes are enforced bit-for-bit in `validate_helas_mg`
(≤6.3e-13); `u u~ > u u~` (#8) remains blocked on color flow. Each added exactly one
convention axis; the fixes each landed with a per-diagram AMP-dump cross-check:

1. **`ee_to_ee` (Bhabha)** — s⊕t interference with identical flavors. Needed the
   crossed-line `−1` (s-channel has one crossed line, t-channel none) and ZERO width
   on the t-channel Z (MadGraph passes ZERO for spacelike propagators). 2.7e-14.
2. **`ee_to_mumua`** — first external vector wavefunction (`vxxxxx`) vs MG. Required
   the massless-vector 3→2 helicity fix (`[-1,0,1]` → `[-1,1]`). 3.9e-13.
3. **`ee_to_ttx`** — massive external fermions. 4.8e-15.
4. **`ee_to_wpwm`** — VVV triple-gauge + massive charged vector externals + t-channel
   ν. Needed `LowerVout` (VVV's P-carrying structure lowers its output index without
   the vertex −i, vs VVS's `MetricVout`). 4.4e-14.
5. **`ee_to_zh`** — external scalar + on-shell VVS + massive s-channel Z propagator.
   9.5e-14.
6. **`ee_to_tatah`** — external FFS Yukawa emission. Needed goldstone/ghost exclusion
   in unitary gauge (`is_goldstone` + `ghost_number` filter in `topo.rs`). 3.9e-13.
7. **`bbx_to_ccx_emmm_qcd0`** — 2-propagator spine with massive internal fermions +
   massive-vector propagators fed by index-flipped (VVS `MetricVout`) currents. Needed
   the `PropagateLowered` op: the massive-vector longitudinal term reads its `g^{μν}`
   term off the raised current and undoes the `MetricVout` storage sign. 6.3e-14.

Infra delivered with this work (the reusable-scripts request):
- `wrappers/generic.f` — one Fortran wrapper (`MG_EVAL_M2` / `..._BATCH`) for all
  momentum-CSV processes; `TS` sized `3**NEXTERNAL` for massive-vector helicities.
  `ee`/`pp_to_ll` migrated off their bespoke wrappers.
- `gen_amplitude.py` — registry-driven (`Process` dataclass + `PROCESSES` list),
  massive RAMBO (Newton ξ-rescale), momentum-based CSV schema shared by all.
- `build_amplitude.sh` — registry-driven (`GENERIC_PROCESSES`, `AMP_PROBE_PROCESSES`);
  `subprocess_dir()` glob helper.
- `wrappers/amp_probe.f.in` + `compare_amps.py` — one process-parameterized
  per-diagram AMP-dump probe + matcher, replacing the two bespoke note-12 probes.
  Driven by the `probe_process_diagrams` Rust test (`VG_PROBE_NAME`/`VG_PROBE_CF`).

### `color-flow` — Multi-flow color algebra

For NCOLOR=1 processes the scalar color factor `CF(1,1)` suffices (implemented in
`validate_helas_mg::color_factor`). True multi-flow color (same-flavor `u u~ > u u~`,
gluon exchange) needs per-flow amplitudes and the color matrix. Prerequisite for
hadronic cross sections and any QCD≠0 validation.

_Unblocks: `mg-validation-coverage` #8, PDF-weighted pp→ll σ_

### Hadronic pp→ll cross section (after color flow)

σ = Σ_q ∫ dx₁ dx₂ f_q(x₁) f_q̄(x₂) σ̂(q q̄ → l⁺ l⁻). Blocked on: color flow, a PDF
interface (e.g. LHAPDF), and n-body LIPS for the partonic √ŝ scan. Flavors group by
charge type since MG treats light quarks as massless.

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

_Depends on: `helas-generalize` (✅)_
_Unblocks: Full CLI with process cards_

---

## 🟢 Later — polish and extensibility

### `lorentz-eval-node-2level` — Two-level LorentzEvalNode + variance-aware slots

Reorganize the Lorentz eval nodes (`helas/eval/root_lorentz.rs` + `run.rs`) into two
levels:
- **outer = output type**, carrying variance/flow: `ScalarOut` / `VectorOut<V>` /
  `SpinorOut<Flow>` (+ tensor later);
- **inner = the UFO primitive** (Gamma, Metric, P, Proj, …), with one node per
  (structure × distinct output-leg type) — e.g. `Metric`→scalar vs `MetricVout`
  →vector are the same structure with different outputs.

Make `VectorWf` (and `WaveformSlot`) variance-parameterized so each vertex returns its
natural variance, the propagator's raise/lower is type-checked, and contractions can't
silently double-apply or drop the metric. This removes the manual component/index
hand-coding in `MetricVout`/`LowerVout` and the `PropagateLowered` branch (`run.rs`),
which bypass the typed repr because `VectorWf.eps` is pinned to `Contravariant` — the
exact variance-bug class the typing was meant to prevent. Note 12's lesson 10 is the
motivation: every convention bug in the hunt lived at a duality boundary (flow,
crossing, variance) that was hand-coded — and the `bbx` lowered-propagator fix added
another `lowered_storage` flag + hand-raised `g^{μν}` term to that pile.

**Now unblocked**: the continuum bug is fixed, VVS is regression-pinned
(`test_metric_vout_vs_aloha_vvs1p1n1`), and VVV + lowered-vector propagation are now
MG-validated (`ee_to_wpwm`, `bbx_to_ccx_emmm_qcd0`). This refactor is the natural next
structural cleanup: it would collapse `Metric`/`MetricNegI`/`MetricVout`/`LowerVout`
and `Propagate`/`PropagateLowered` into variance-typed nodes.

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
(The MG validation side already generates n-body points via RAMBO in
`gen_amplitude.py`; the MG-computed partonic σ̂ = 6.556e-7 pb for the uux 2→6 at
√s=500 is banked as a future `validate-vegas` reference.)

_Depends on: `xsec-ee-mumu` (✅)_

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.).

_Depends on: `helas-generalize` (✅)_

### `typed-units` — Typed physical units

Research `uom`/`dimensioned`/`units` crates for typed four-momenta and cross sections.

---

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (✅) ──→ helas-generalize (✅) ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┤
color-flow ──→ mg-validation-coverage #8, hadronic pp→ll               │
lips-nbody ─────────────────────────────────────────────────────────────┴──→ event-output-lhef
global-config ──→ CLI
```
