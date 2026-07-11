# vibegraph — Task Backlog

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 11 processes bit-match MadGraph (≤6e-13, incl. 2→6, VVV, massive externals); single color flow only |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | Lepage VEGAS on `AmplitudeEvaluator::eval_m2`; `validate_vegas.rs`: `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%) |
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

## ✅ Closed sprints (merged to `main`)

Detail lives in git history and the linked notes; kept here as one-line anchors plus
the perf-sprint timing table (still the reference baseline for the post-CSE tracks).

- **`cleanup-refactor` ✅ CLOSED 2026-07-10** — post-`mg-validation-coverage`
  structural cleanup, four tasks, all bit-for-bit vs the 11-process net.
  `intern-sm-model` (SM baked into the binary via `ufo::sm::sm_model`; default
  build/test never touches the submodule) + `global-config`; `feature-gate-mg-tests`
  (default `cargo test` needs no MG/HELAS reference data); `diagram-canonical-stream`
  (owned feyngraph-free `diagrams::Diagram`, single `from_view` boundary);
  `typed-repr-conventions` (conventions into types: `DiracAdjoint`/`Ket`/`Bra`,
  variance-parameterized `VectorWf`, typed propagator seam, one contravariant vector
  convention, `Op`↔s-expr bijection with `KNOWN_UNCOVERED` allowlist for `MetricNegI`
  /`IdentityAmp`). Design: `research/notes/13-typed-repr-conventions-design.md`.
  Deferrals → `validation-sprint` (op-coverage gaps) and the perf sprint (Stage-A
  fuse, `generate_*` lazy iterator Part B).
- **`performance-sprint` ✅ CLOSED 2026-07-11 — merged to `main`** — evaluator perf,
  gate = `validate_helas_mg` at `REL_TOL` 1e-12. Landed P1 (MG-vs-Rust timing rig),
  P2 (reusable `ScratchSpace` + hash-cons CSE), P4 (stack evaluator built + measured
  — forward scan wins uniformly, per-node 112-B slot traffic dominates), P6 (hot-path
  `#[inline(always)]` survey + Stage-A chiral-pair FFV fusion + true-arity kernels
  taking operands by reference), P5 (binary-arity lowering + `flatten_adds` — the egg
  arity groundwork; `BoundAmplitudeStack` deleted). P3 (ext-wavefunction pool)
  cancelled — `build_external_core` is 0.4% of eval. Net vs MG MATRIX1: **8.6×–110×**
  (see table). The dominant scaling term was duplicated-subtree width (CSE), then
  per-node combinator/slot overhead (fusion + true-arity kernels); post-CSE the 2→6s
  sit at ~15 nodes/diagram, so **further cuts need algebraic rewrites** structural
  hash-consing cannot see → the post-CSE optimization program below.

Timing table (dev machine, `--profile profiling`, `--test-threads=1`; rust ns/eval
per session vs MG MATRIX1 ns/eval). Final column = merged `main`:

| process | MG | P1 baseline | P2 | P6 +arity | P5 (final) |
|---|---:|---:|---:|---:|---:|
| ee_to_zh | 206 | 5,479 (27×) | 3,272 (16×) | 1,931 (9.4×) | 1,947 (9.5×) |
| ee_to_mumu | 328 | 14,093 (43×) | 8,198 (25×) | 3,882 (12×) | 3,980 (12×) |
| pp_to_ll_qcd0 | 298 | 14,027 (47×) | 8,335 (28×) | 4,447 (15×) | 4,551 (15×) |
| ee_to_ttx | 357 | 14,588 (41×) | 8,520 (24×) | 4,728 (13×) | 4,728 (13×) |
| ee_to_ee | 743 | 27,467 (37×) | 15,102 (20×) | 6,447 (8.7×) | 6,419 (8.6×) |
| ee_to_tatah | 859 | 50,894 (59×) | 21,996 (26×) | 11,196 (13×) | 11,327 (13×) |
| ee_to_wpwm | 776 | 64,059 (83×) | 30,145 (39×) | 20,710 (27×) | 20,709 (27×) |
| ee_to_mumua | 1,510 | 133,751 (89×) | 58,403 (39×) | 28,102 (19×) | 28,520 (19×) |
| ee_to_mumu_tata_qcd0 | 6,404 | 1,319,279 (206×) | 356,126 (56×) | 143,596 (22×) | 145,032 (23×) |
| uux_to_ccx_emmm_qcd0 | 100,230 | 216,900,033 (2,164×) | 28,505,729 (284×) | 10,897,954 (109×) | 10,981,139 (110×) |
| bbx_to_ccx_emmm_qcd0 | 141,430 | 236,382,600 (1,671×) | 29,593,298 (209×) | 11,774,924 (83×) | 11,821,262 (84×) |

---

## 🚀 Post-CSE optimization program — 3 tracks (plan: `research/notes/15-eval-optimization-plan.md`)

Planned 2026-07-11 from the research pass over rooting symmetry, MadGraph's
optimization stack (helicity recycling = CSE across the unrolled helicity loop,
arXiv:2102.00773), egglog 2.0 extraction semantics, and measured evaluator layouts
(`WaveformSlot<f64>` = 104 B, `Node<Const>` = 12 B). Key structural finding: egglog's
extractor is **tree-cost only**, so every rewrite whose payoff is *sharing* (re-rooting,
chiral decomposition, coupling factoring) is invisible to it — that blocker is Track 3,
and the sharing half of `egraph-rewrite` waits on it. Tracks 1 and 2 need no egraph.

### ⚡ Track 1: `eval-layout` — evaluator memory layout & recycling (branch TBD; merge to `main`)

Gate: 11-process `validate_helas_mg` REL_TOL 1e-12; bit-for-bit where order-preserving
(noted per session). Baseline: the P5 timing table above. `wavefn.rs` is untouched —
it remains the public hand-built-amplitude component and unit-test vocabulary; the
runtime grows its own internal storage. Sessions in dependency order (detail in note 15 §2):

- **A0** — instruction-size sensitivity check (pad `Node<Const>` to 16/24/32 B and
  measure; also the free 12→8 B pack). Informs A3 and the typed egglog constructors.
- **A1** — static node analysis pass: per-node output type (realizes the
  `ScalarConst`/`ScalarWf` taxonomy), constness, momentum id (signed external-momentum
  combination, interned), helicity-support mask. Pure analysis + runtime
  cross-assertions; everything downstream (and the egraph typed schema) consumes it.
- **A2** — constant-subgraph folding into bind-time pools (extends `fold.rs`; deletes
  the per-point re-evaluation of card-constant `g_L`/`g_R` subgraphs — the P6
  follow-up, formerly slated as the first egglog rule; needs no rules). Bit-for-bit.
- **A3** — SoA scratch + typed instruction stream: per-type result arenas replace
  `Vec<WaveformSlot>`; typed operand indices; `Node` repack. Element types keep the
  `wavefn.rs` structs at first (arithmetic untouched). Bit-for-bit.
- **A4** — momentum pool: per-point helicity-independent momentum table; SoA elements
  become bare `Bispinor`/`ComplexVector`/`C<F>`; `mul_apply` momentum routing leaves
  the hot path; `PMom`/`PMomOut` become table reads. Reassociates momentum sums →
  REL_TOL gate.
- **A5** — helicity-support recycling in `eval_m2`: odometer-ordered helicity loop,
  skip nodes whose support mask misses the changed legs. Bit-for-bit vs A4.
  (Integration-phase win; final accept/reject samples a *specific* helicity
  configuration, so see `mg-single-helicity-bench` below for the fair comparison.)
- **A6** — close-out: re-record the timing table vs MG and the P5 baseline; update
  TODO + note 15.

### 🌲 Track 2: `rooting-exploration` — throwaway rooting study (branch `explore/rooting`, not merged)

Root choice is currently `VtxIdx(0)` — an accident of feyngraph ordering that
cross-diagram CSE silently depends on. Over all rootings a diagram has only ~2·E
distinct directed currents, so the *floor* is computable. Measure post-CSE node count
(and slot-cost-weighted) across `MG_VALIDATED_PROCESSES` for: baseline; canonical
heuristics (lowest-leg anchor; most/fewest contributing external momenta — measure
both directions); greedy iterative rooting (each diagram tries all rootings against
the cumulatively-interned arena, min new nodes; both diagram orders). Every variant
runs the full validation net (rootings hit new kernel paths — may incidentally cover
`MetricNegI`). Results committed on the branch for posterity + tables appended to
note 15 on `main`. Decision output: if greedy wins big, promote a production
greedy-rooting pass into Track 1; headroom informs the Track 3 go/no-go.

### 🧮 Track 3: `dag-extraction` — DAG-cost extractor for egglog (investigation)

egglog 2.0 has no sharing-aware extraction (verified in `extract.rs`:
`TreeAdditiveCostModel`; the `CostModel` trait is tree-shaped). Milestones (note 15 §4):
**M1** enumerate e-classes/e-nodes from `egglog::EGraph` in Rust; **M2** greedy DAG
extractor (extraction-gym style) with slot-traffic costs — sanity gate: reproduces the
input DAG on the rule-free round-trip; ILP in reserve as a quality oracle; **M3** first
sharing-rule demo — chiral decomposition `FfvVout(a,b,gl,gr) → gl·J_L + gr·J_R` on
`e+ e- > mu+ mu-` (γ/Z share `J_L`/`J_R`), showing DAG cost picks the shared form and
tree cost doesn't; **M4** write-up + go/no-go for `egraph-rewrite` integration.

---

## 🧬 `egraph-rewrite` — algebraic rewrite stage (**blocked on Track 3**; scope revised 2026-07-11)

Cut per-process node count below what CSE alone reaches by factoring shared algebraic
structure across diagrams (post-CSE 2→6s are ~15 nodes/diagram). Every rule is a place
a sign bug can hide: each lands guarded by the 11-process `validate_helas_mg` net
(REL_TOL 1e-12), bit-for-bit where order-preserving. Design references:
`research/notes/14-egglog-notes.md` (language), note 15 (plan + schema decisions).

Slots into `lower::optimize` as **egg → flatten → CSE → fold** (P5 put the lowered AST
in the binary-arity form egg requires; `flatten_adds` inverts it back for evaluation).

**Landed groundwork (skeleton, on `main`):** `helas/eval/egraph.rs`
`roundtrip(&Ast<Sym>) -> Ast<Sym>` — encodes the binary-arity AST into an egglog
`datatype` (one constructor per `Op`, leaf payloads as leading `i64`/`f64` fields;
`PMomOut` variadic via `(Vec Node)`), built programmatically as `egglog::ast::Command`s:
one `let $root = <inlined tree>` then `extract $root` (node-at-a-time `let`s forced
O(nodes) rebuilds — ~90 s over the suite; one command drops it to ~1 s release). The
`TermDag` decodes structurally back to `Ast<Sym>`. No rules ⇒ structural identity;
`#[allow(dead_code)]`, not wired into `optimize`. The `let`↔`extract` gap is the seam
for the rule schedule (Track 3 M2's extractor replaces the `extract` side). Round-trip
tested byte-for-byte: all-op / all-leaf / DAG-sharing fixtures,
`rewrite_dev_processes_roundtrip` (2→2/2→3/2→4, 0.14 s debug — the fast rule-dev
harness) ungated, and `representative_processes_roundtrip` (all 11
`MG_VALIDATED_PROCESSES`, incl. both 2→6 EW) behind `extended-validation`.

**Scope changes (2026-07-11, note 15 §5):**
- Constant folding **moved out** → Track 1 session A2 (a `fold.rs` constness pass;
  needs no rules).
- The remaining rule families are all *sharing* rules — coupling regrouping, chiral
  decomposition + propagator linearity (γ/Z structure sharing), re-rooting
  (propagator-commute + per-vertex rotation rules) — and are **blocked on Track 3**
  (tree-cost extraction cannot see their payoff) and informed by Track 2's headroom.
- **Schema decisions adopted:** per-kind leaf sorts (`CouplingId`/`ParticleId`/`Real`/
  `ExtLegInfo` as separate datatypes), `ScalarConst` vs `ScalarWf` sort split (required
  for soundness — `mul_apply` routes momentum for scalar *wavefunctions*, so
  scalar-motion rules are restricted to momentum-free scalars by type), typed
  constructor slots (`(Propagate Node Mass Width)`, `Mul` split by operand class).
  Apply when Track 3 touches the schema; `schema_covers_every_op` + round-trip suite
  remain the guard.
- **Perf posture** (unchanged): full-suite round-trip ~1.2 s release / 142 s debug —
  develop and measure rewrites in release/profiling; rule-dev tight loop on the
  ungated 2→2/2→3/2→4 harness. Open question once real rules run: saturation +
  extraction scaling on the 2→6 QCD ASTs — bound the schedule, consider one `EGraph`
  reused across subprocesses.

---

## 🧪 `validation-sprint` — stub, to be planned

Collects the validation follow-ups deferred from `cleanup-refactor`:

- **Primitive-op coverage gaps** (found at cleanup close-out, 2026-07-10):
  process-level MG coverage for the two `KNOWN_UNCOVERED` ops in
  `mg_validated_suite_exercises_every_op`:
  - `MetricNegI`: needs a process whose diagrams amplitude-root a pure-metric vertex —
    e.g. fermion-free externals (`w+ w- > z h`-like) so the `VtxIdx(0)` rooting cannot
    pick a fermion line, or a rooting-choice override for tests.
  - `IdentityAmp`: needs a non-SM UFO model with an `Identity` scalar bilinear (SM has
    none); could ride on a small dedicated test model.
- **Branch-level coverage**: op counts don't see rooting branches — verify the
  scalar-rooted pure-metric −1 branch (root_lorentz) and, generally, consider
  rooted-tree pattern assertions per MG-pinned convention (each "pinned by X" comment
  should have a test that fails if the pinning process stops exercising the branch —
  the stale `MetricNegI` comment was exactly this failure mode).
- **`madgraph-diagram-cmp-per-flavor`** (deferred from cleanup task 2; design in its
  own section below): per-flavor subprocess matching in diagram validation.
- Optional CI job: `gen_sm_blob` + `git diff --exit-code` to catch a stale interned SM
  blob vs the pinned submodule (cleanup task 1 follow-up).

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

### `global-config` — Implement `vibegraph_lib::config::GlobalConfig` ✅ Done

_Folded into cleanup task 1 `intern-sm-model` (same model-loading wiring)._ Landed as
`config::GlobalConfig::load_ufo(&Option<ModelImport>) -> Arc<UFOModel>`: interned SM
for `import model sm[-variant]`, else a UFO dir under `ufo_search_path`. CLI wiring of
a full proc card is still pending.

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

### `feyngraph-perf` — Fix feyngraph allocation hot spot

**Hot spot identified** (samply profile, pp→qq̃4l run): `workspace.rs:L122` in
`AssignWorkspace::assign()` calls `.counts()` (itertools) on every candidate vertex for every
topology for every subprocess. Each `.counts()` call allocates a fresh `HashMap<particle_index,
count>`. For pp→qq̃4l: ~1,664 subprocesses × 34,300 topologies × O(vertices) = ~340M HashMap
allocations. **Fix**: pre-compute per-vertex particle counts in `AssignWorkspace::new()` and
reuse them in the inner loop. This is a change to the `feyngraph` submodule; deferred to a
dedicated feyngraph session.

Vibegraph-side mitigations already applied:
- Topology caching: `generate_topologies()` called once per `(n_ext, n_loops)`; all subprocesses
  share the same `Vec<Topology>` via `DiagramGenerator::assign_topologies()` (pp→qq̃4l: 4.86s once
  vs ~15h naive).
- Charge conservation pre-filter: eliminates ~86% of alias-expanded candidates before topology
  assignment (11,520 → ~1,664 for pp→qq̃4l).

Also a `performance-sprint` backlog item: `generate-stream` Part B (lazy `generate_*`
iterator, deferred from cleanup task 3) and `C<F>`-vs-`F` multiply peepholes.

### `madgraph-diagram-cmp-per-flavor` — Match subprocesses by flavor in diagram validation

_Was to fold into cleanup task 2 `feature-gate-mg-tests`; deferred as its own session —
it is an independent, verification-heavy refactor (Python extractor + Rust matching +
JSON regen), not part of the feature-gating itself._

The `validate_madgraph_diagrams` reference count now uses the representative subprocess's
true Feynman-diagram count (`NGRAPHS` from `matrix1_orig.f`), not `MAPCONFIG(0)` from
`configs.inc` (which counts the phase-space integration-channel *union* across all flavor
variants in a P-class — e.g. 2672 vs the actual 2316 for `u u~ > u u~ l+ l- l+ l-`).

**Remaining gap**: the comparison (`count_mg_style_topologies` in
`vibegraph-lib/tests/validate_madgraph_diagrams.rs`) still collapses vibegraph subprocesses
into coarse particle-type classes (`quark`/`lepton`/…) and compares one representative per
class against the summed `total_diagrams`. Fragile: it assumes vibegraph's first-enumerated
subprocess in each class matches MadGraph's `matrix1` representative.

**Design for the refinement** (per-flavor matching, validates *all* variants incl. the 40
of the qq4l class):
- **Robust flavor source — the matrix-file header, not `IDUP`.** Each
  `SubProcesses/P*/matrix<N>_orig.f` carries `C     Process: u u~ > u u~ e+ e- e+ e- QCD=0 @1`
  comment lines — one per concrete flavor process sharing that variant's `NGRAPHS` (u/c and
  e/mu are grouped). Parse these directly: it avoids reverse-engineering MG's fragile
  `matrix<N> ↔ IDUP(I,J,K)` 3-index mapping in `leshouche.inc`. `extract_diagrams.py` grows
  a per-concrete-process `{in:[pdg…], out:[pdg…], ngraphs}` list (name→PDG via a bounded SM
  dict: the full token set is `a b b~ c c~ d d~ e± g h mu± s s~ t t~ ta± u u~ w± z`).
- **Rust side**: key each MG entry and each vibegraph subprocess by
  `(sorted initial PDGs, sorted final PDGs)`; look up and compare per-subprocess
  (`set.diagrams.len()` vs `ngraphs`).
- **Known risk to resolve first**: this exposes whether vibegraph enumerates the *same set*
  of concrete subprocesses as MG's `C Process:` union — i.e. whether the multiparticle `p`/`l`
  definitions and flavor-symmetry pruning align. Validate on a small process (`pp_to_ll`)
  before the qq4l class; a set mismatch here is a real finding, not a test bug, and needs
  physics judgment (note-12 territory: MG-convention reconciliation is a bug magnet).

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

### `mg-single-helicity-bench` — MG comparison at a fixed helicity configuration (low priority)

The timing table compares against MG MATRIX1, which sums helicities — so MG's
helicity recycling (CSE across its unrolled helicity loop) is baked into its side of
the ratio, while vibegraph re-runs the full arena per combination. A parallel
benchmark evaluating **one fixed helicity configuration** on both sides isolates
kernel-level performance gaps from missed-CSE / helicity-loop effects. Also the fair
comparison for the event-generation regime: once the importance-sampling reference
distribution is established, final accept/reject evaluates a specific helicity
configuration, where helicity recycling buys nothing (the recycling win belongs to
the integration-grid phase and its cumulative per-helicity ledger). Natural landing
spot: alongside `eval-layout` A6 while the timing rig is warm.

---

## Dependency graph

```
feyngraph-ufo-replace (✅) ──→ lorentz-runtime-eval (✅) ──→ helas-generalize (✅) ──→ event-output-lhef
lorentz-parse (✅) ──────────────────────────────────────┘              │
diagram-enum (✅) ──────────────────────────────────────────────────────┤
color-flow ──→ mg-validation-coverage #8, hadronic pp→ll               │
lips-nbody ─────────────────────────────────────────────────────────────┴──→ event-output-lhef

cleanup-refactor (✅ closed 2026-07-10) ──→ validation-sprint, performance-sprint
performance-sprint (✅ closed 2026-07-11, merged) ──┬──→ eval-layout (A0→A1→A2→A3→A4→A5→A6)
                                                    ├──→ rooting-exploration ──┐
                                                    └──→ dag-extraction ←──────┘ (headroom informs go/no-go)
dag-extraction ──→ egraph-rewrite (sharing rules)
eval-layout A6 ──→ mg-single-helicity-bench (optional rider)
```
