# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 4 processes bit-match MadGraph (≤2e-13, incl. 2→6); single color flow only |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | σ ≈ 2025 pb at √s = 91.2 GeV vs MadGraph ref |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

`helas-generalize` is **done** for single-color-flow processes: `AmplitudeEvaluator`
drives the VEGAS integrand, and `validate_helas_mg` enforces bit-for-bit agreement
with MadGraph (`ee_to_mumu` 4.2e-14, `pp_to_ll_qcd0` 2.1e-14 ×CF=3,
`ee_to_mumu_tata_qcd0` 1.8e-14, `uux_to_ccx_emmm_qcd0` 2.1e-13 ×CF=9). The
three-week continuum bug hunt that got there is written up in
`research/notes/12-helas-continuum-bugfix-journey.md`.

---

## 🔴 High — broaden the MadGraph amplitude validation surface

### `mg-validation-coverage` — New processes for `validate_helas_mg`

The four enforced processes cover external fermions, γ/Z/H propagators, and FFV/FFS/VVS
vertices. Each proposed process below adds exactly one untested axis, in rough
priority order (all use the existing generic wrapper + RAMBO CSV pipeline:
`wrappers/generic.f`, `gen_amplitude.py`, `build_amplitude.sh`):

1. **`e+ e- > e+ e-` (Bhabha)** — s⊕t-channel interference with *identical*
   initial/final flavors: the sharpest regression for the crossed-line conjugation
   (note 12, fix 5) and the relative sign between crossed diagram classes. No new
   evaluator features needed.
2. **`e+ e- > mu+ mu- a`** — first *external vector* wavefunction (`vxxxxx`) vs MG.
   Already Ward-tested internally, never compared to MadGraph values.
3. **`e+ e- > t t~`** — massive external fermions (top mass/width in wavefunctions
   and phase-space); tests the massive-fermion spinor conventions the massless
   continuum never touched.
4. **`e+ e- > w+ w-`** — triple-gauge vertex (VVV) + massive *charged* vector
   externals + t-channel ν exchange. First process where the V-chain phase ledger
   meets VVV.
5. **`e+ e- > z h`** — external scalar + on-shell VVS; complements the internal-H
   ZZH class from uux.
6. **`e+ e- > ta+ ta- h`** (massive-τ card) — external FFS Yukawa emission.
7. **`b b~ > c c~ e+ e- mu+ mu-` QCD=0 (massive-b card)** — the S20 loose end: a
   2-propagator initial spine with *massive* internal fermions. The spine-parity
   sign derivation assumes the massless `S(−q) = −S(q)` identity; this is the
   process that would catch it if that's wrong.
8. **`u u~ > u u~` QCD=0** — NCOLOR=2, same-flavor interference. **Blocked on color
   flow** (below); everything else in this list is single-flow.

Infra follow-ups:
- Generalize the per-diagram AMP-dump oracle (currently two bespoke probes:
  `probe_eemumutata_diagrams`, `probe_uux_diagram_classes` + matching Python) into
  one process-parameterized probe + matcher, so any new failing process gets the
  note-12 treatment immediately.
- Migrate `ee`/`pp_to_ll` off their bespoke Fortran wrappers to `generic.f`.

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
hand-coding in `MetricVout` (`run.rs`), which bypasses the typed repr because
`VectorWf.eps` is pinned to `Contravariant` — the exact variance-bug class the typing
was meant to prevent. Note 12's lesson 10 is the motivation: every convention bug in
the hunt lived at a duality boundary (flow, crossing, variance) that was hand-coded.

**Now unblocked**: the continuum bug is fixed and VVS is regression-pinned
(`test_metric_vout_vs_aloha_vvs1p1n1`). Prefer to also land a VVV MG validation
(`mg-validation-coverage` #4) first, since this refactor changes the convention
surface where off-shell-current sign/metric bugs live.

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
