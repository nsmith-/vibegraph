# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate. Current position: `color-flow` (feature, ✅ merged 2026-07-12) →
**`validation-sprint`** (now) → post-CSE optimization program (performance) → next
feature (hadronic pp→ll / event output).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 13 processes agree with MadGraph (11 bit-identical ≤6.3e-13, incl. 2→6/VVV/massive externals; `uux_to_uux` 5.61e-14 and `gg_to_ttx` 1.89e-15 via the multi-flow CF-weighted eval); `gg_to_gg` informational only, blocked on a pre-existing VVVV Lorentz phase bug (`validation-sprint`) |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | Lepage VEGAS on `AmplitudeEvaluator::eval_m2`; `validate_vegas.rs`: `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%) |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

Closed-sprint history (`helas-generalize`, `mg-validation-coverage`,
`cleanup-refactor`, `performance-sprint`, `color-flow`) lives in git history and
`research/notes/` (12: continuum bug hunt, 13: typed conventions, 15: eval
optimization plan, 16: color-flow design + debrief).

---

## 🧪 `validation-sprint` — NOW: harden the gate before the optimization program

The post-CSE program rewrites the evaluator against `validate_helas_mg` + the
op-coverage suite as its gate, so the gate gaps opened by `color-flow` get closed
first. Two items; everything else from the original stub is deferred to the
validation backlog under **Later**.

- **Op-coverage bookkeeping — promote multi-flow into the suite lists.**
  `Op::Flows`/`Op::CoeffRat` are still in `KNOWN_UNCOVERED`
  (`helas/eval/compile.rs`) even though `uux_to_uux` and `gg_to_ttx` now
  bit-validate them, because `MG_VALIDATED_PROCESSES` is still the original 11
  NCOLOR=1 processes. Add the two enforced multi-flow processes to
  `MG_VALIDATED_PROCESSES` and drop `Flows`/`CoeffRat` from `KNOWN_UNCOVERED`.
  Every list that iterates `MG_VALIDATED_PROCESSES` picks the coverage up for
  free — the op-coverage tests, the egglog `representative_processes_roundtrip`,
  and Track 2's rooting study — which is why this lands before the tracks start.

- **VVVV `GC_12` Lorentz phase bug** (blocks `gg_to_gg` enforcement): the 4-gluon
  contact diagram's imaginary coupling `GC_12 = i·g²` carries a spurious +90°
  Lorentz phase relative to the 3 exchange diagrams (pre-existing, not a color
  bug — the CF matrix and per-flow color coefficients are proven correct against
  MG, and the exchange diagrams are bit-for-bit). MG reference data + the JAMP
  probe (`gg_to_gg` registry entry) are already in place; expected to enforce
  straight to ≤1e-12 once fixed, adding the only NCOLOR=6 pure-gluon process to
  the gate.

---

## 🚀 Post-CSE optimization program — 3 tracks (plan: `research/notes/15-eval-optimization-plan.md`)

Planned 2026-07-11 from the research pass over rooting symmetry, MadGraph's
optimization stack (helicity recycling = CSE across the unrolled helicity loop,
arXiv:2102.00773), egglog 2.0 extraction semantics, and measured evaluator layouts
(`WaveformSlot<f64>` = 104 B, `Node<Const>` = 12 B). Key structural finding: egglog's
extractor is **tree-cost only**, so every rewrite whose payoff is *sharing* (re-rooting,
chiral decomposition, coupling factoring) is invisible to it — that blocker is Track 3,
and the sharing half of `egraph-rewrite` waits on it. Tracks 1 and 2 need no egraph.

**Revised 2026-07-12 after `color-flow`** — deltas relative to the note-15 plan:
- The gate is now the **13-process** `validate_helas_mg` net (11 NCOLOR=1
  bit-identical + `uux_to_uux` ≤5.7e-14 + `gg_to_ttx` ≤2e-15), plus `gg_to_gg` if
  the `validation-sprint` VVVV fix lands first.
- `eval_m2` is now the CF-weighted multi-flow loop (per-flow JAMPs, MG's ZTEMP
  accumulation order, NCOLOR=1 op-order rule for bit-for-bit) — affects A5.
- Rooting is per (diagram, color-chain), still anchored at `VtxIdx(0)`; cross-flow
  CSE (NCOLOR=6 costs ~2× NCOLOR=2, ≪ naive NCOLOR×) depends on chains of the same
  diagram rooting consistently — affects Track 2.
- `fold.rs` already carries exact-rational pools for `CoeffRat` (C4) — A2 extends
  the same file; the egglog schema + round-trip already cover `Flows`/`CoeffRat`.

Baseline timing table (dev machine, `--profile profiling`, `--test-threads=1`;
ns/eval, Rust `main` vs MG MATRIX1 — the `performance-sprint` P5 close-out plus the
`color-flow` close-out rows; A6 re-records against this):

| process | MG | main | ratio |
|---|---:|---:|---:|
| ee_to_zh | 206 | 1,947 | 9.5× |
| ee_to_mumu | 328 | 3,980 | 12× |
| pp_to_ll_qcd0 | 298 | 4,551 | 15× |
| ee_to_ttx | 357 | 4,728 | 13× |
| uux_to_uux | 278 | 5,121 | 18.5× |
| ee_to_ee | 743 | 6,419 | 8.6× |
| gg_to_ttx | 659 | 9,148 | 13.9× |
| ee_to_tatah | 859 | 11,327 | 13× |
| ee_to_wpwm | 776 | 20,709 | 27× |
| gg_to_gg (informational) | 949 | 24,110 | 25.4× |
| ee_to_mumua | 1,510 | 28,520 | 19× |
| ee_to_mumu_tata_qcd0 | 6,404 | 145,032 | 23× |
| uux_to_ccx_emmm_qcd0 | 100,230 | 10,981,139 | 110× |
| bbx_to_ccx_emmm_qcd0 | 141,430 | 11,821,262 | 84× |

Post-CSE the 2→6s sit at ~15 nodes/diagram, so beyond Track 1's layout wins,
**further cuts need algebraic rewrites** structural hash-consing cannot see.

### ⚡ Track 1: `eval-layout` — evaluator memory layout & recycling (branch TBD; merge to `main`)

Gate: 13-process `validate_helas_mg` REL_TOL 1e-12; bit-for-bit where order-preserving
(noted per session). Baseline: the timing table above. `wavefn.rs` is untouched —
it remains the public hand-built-amplitude component and unit-test vocabulary; the
runtime grows its own internal storage. Sessions in dependency order (detail in note 15 §2):

- **A0** — instruction-size sensitivity check (pad `Node<Const>` to 16/24/32 B and
  measure; also the free 12→8 B pack). Informs A3 and the typed egglog constructors.
- **A1** — static node analysis pass: per-node output type (realizes the
  `ScalarConst`/`ScalarWf` taxonomy), constness, momentum id (signed external-momentum
  combination, interned), helicity-support mask. Pure analysis + runtime
  cross-assertions; everything downstream (and the egraph typed schema) consumes it.
  `color-flow` C4 landed first (`Op::Flows`, `Op::CoeffRat`), so this pass must also
  classify them: `CoeffRat` is a scalar const (folds like any other rational leaf);
  `Flows` is a sink (variadic root, never an operand — no output type to assign).
- **A2** — constant-subgraph folding into bind-time pools (extends `fold.rs`, which
  already carries the C4 `CoeffRat` rational pools; deletes the per-point
  re-evaluation of card-constant `g_L`/`g_R` subgraphs — the P6 follow-up, formerly
  slated as the first egglog rule; needs no rules). Bit-for-bit.
- **A3** — SoA scratch + typed instruction stream: per-type result arenas replace
  `Vec<WaveformSlot>`; typed operand indices; `Node` repack. Element types keep the
  `wavefn.rs` structs at first (arithmetic untouched). Bit-for-bit.
- **A4** — momentum pool: per-point helicity-independent momentum table; SoA elements
  become bare `Bispinor`/`ComplexVector`/`C<F>`; `mul_apply` momentum routing leaves
  the hot path; `PMom`/`PMomOut` become table reads. Reassociates momentum sums →
  REL_TOL gate.
- **A5** — helicity-support recycling in `eval_m2`: odometer-ordered helicity loop,
  skip nodes whose support mask misses the changed legs. `eval_m2` is now the
  CF-weighted multi-flow loop — recycling applies across flows for free (flows share
  the arena), but the bit-for-bit-vs-A4 claim must preserve the per-flow JAMP
  accumulation order. (Integration-phase win; final accept/reject samples a
  *specific* helicity configuration, so see `mg-single-helicity-bench` below for the
  fair comparison.)
- **A6** — close-out: re-record the timing table vs MG and the baseline above; update
  TODO + note 15.

### 🌲 Track 2: `rooting-exploration` — throwaway rooting study (branch `explore/rooting`, not merged)

Root choice is currently `VtxIdx(0)` per (diagram, color-chain) — an accident of
feyngraph ordering that cross-diagram CSE silently depends on; post-`color-flow`,
cross-flow CSE additionally depends on chains of the same diagram rooting
consistently, so any candidate heuristic must be applied uniformly across a
diagram's chains and measured on the multi-flow processes too. Over all rootings a
diagram has only ~2·E distinct directed currents, so the *floor* is computable.
Measure post-CSE node count (and slot-cost-weighted) across `MG_VALIDATED_PROCESSES`
(13 once the `validation-sprint` bookkeeping lands) for: baseline; canonical
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
a sign bug can hide: each lands guarded by the 13-process `validate_helas_mg` net
(REL_TOL 1e-12), bit-for-bit where order-preserving. Design references:
`research/notes/14-egglog-notes.md` (language), note 15 (plan + schema decisions).

Slots into `lower::optimize` as **egg → flatten → CSE → fold** (P5 put the lowered AST
in the binary-arity form egg requires; `flatten_adds` inverts it back for evaluation).

**Landed groundwork (skeleton, on `main`):** `helas/eval/egraph.rs`
`roundtrip(&Ast<Sym>) -> Ast<Sym>` — encodes the binary-arity AST into an egglog
`datatype` (one constructor per `Op`, incl. the C4 `Flows`/`CoeffRat` additions; leaf
payloads as leading `i64`/`f64` fields; `PMomOut` variadic via `(Vec Node)`), built
programmatically as `egglog::ast::Command`s: one `let $root = <inlined tree>` then
`extract $root` (node-at-a-time `let`s forced O(nodes) rebuilds — ~90 s over the
suite; one command drops it to ~1 s release). The `TermDag` decodes structurally back
to `Ast<Sym>`. No rules ⇒ structural identity; `#[allow(dead_code)]`, not wired into
`optimize`. The `let`↔`extract` gap is the seam for the rule schedule (Track 3 M2's
extractor replaces the `extract` side). Round-trip tested byte-for-byte: all-op /
all-leaf / DAG-sharing fixtures, `rewrite_dev_processes_roundtrip` (2→2/2→3/2→4,
0.14 s debug — the fast rule-dev harness) ungated, and
`representative_processes_roundtrip` (all of `MG_VALIDATED_PROCESSES`, incl. both
2→6 EW) behind `extended-validation`.

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

## 🔴 High — next feature: hadronic pp→ll cross section

σ = Σ_q ∫ dx₁ dx₂ f_q(x₁) f_q̄(x₂) σ̂(q q̄ → l⁺ l⁻). `color-flow` unblocked the
partonic side; remaining blockers: a PDF interface (e.g. LHAPDF — not yet a task) and
`lips-nbody` for the partonic √ŝ scan. Flavors group by charge type since MG treats
light quarks as massless. Sequenced after `validation-sprint` + the optimization
program per the feature→validation→performance loop.

---

## 🟡 Medium — CLI integration

### `cli-proc-card` — wire a full process card through the CLI

`config::GlobalConfig::load_ufo(&Option<ModelImport>) -> Arc<UFOModel>` (landed with
`intern-sm-model`) already provides the `ParsedProcCard` → `UFOModel` seam: interned
SM for `import model sm[-variant]`, else a UFO dir under `ufo_search_path`. Remaining
work is the CLI wiring of a full proc card end-to-end.

---

## 🟢 Later — polish and extensibility

### Validation backlog (deferred from `validation-sprint`)

Deferred to the next validation pass of the loop — none of these guard the surface
the optimization program touches:

- **`MetricNegI` process-level coverage**: needs a process whose diagrams
  amplitude-root a pure-metric vertex — e.g. fermion-free externals (`w+ w- > z h`-like)
  so the `VtxIdx(0)` rooting cannot pick a fermion line, or a rooting-choice override
  for tests. `color-flow`'s `gg_to_ttx`/`gg_to_gg` did not cover it (note 16 §3 had
  flagged them as a maybe). Track 2's rooting study may incidentally cover it — check
  its results before building a dedicated process.
- **`IdentityAmp` process-level coverage**: needs a non-SM UFO model with an
  `Identity` scalar bilinear (SM has none); could ride on a small dedicated test model.
- **Rationalize `Coeff(f64)` onto `CoeffRat`** (note 16 §5): now that `Op::CoeffRat`
  exists for color coefficients, the remaining `Coeff(f64)` leaves (Lorentz-structure
  and symmetry/fermi-sign coefficients) could migrate onto it too — optional cleanup,
  not required by anything currently blocked.
- **Branch-level coverage**: op counts don't see rooting branches — verify the
  scalar-rooted pure-metric −1 branch (root_lorentz) and, generally, consider
  rooted-tree pattern assertions per MG-pinned convention (each "pinned by X" comment
  should have a test that fails if the pinning process stops exercising the branch —
  the stale `MetricNegI` comment was exactly this failure mode).
- **Optional CI job**: `gen_sm_blob` + `git diff --exit-code` to catch a stale
  interned SM blob vs the pinned submodule.
- **`madgraph-diagram-cmp-per-flavor`** — per-flavor subprocess matching in diagram
  validation (design below).

#### `madgraph-diagram-cmp-per-flavor` — Match subprocesses by flavor in diagram validation

An independent, verification-heavy refactor (Python extractor + Rust matching + JSON
regen). The `validate_madgraph_diagrams` reference count now uses the representative
subprocess's true Feynman-diagram count (`NGRAPHS` from `matrix1_orig.f`), not
`MAPCONFIG(0)` from `configs.inc` (which counts the phase-space integration-channel
*union* across all flavor variants in a P-class — e.g. 2672 vs the actual 2316 for
`u u~ > u u~ l+ l- l+ l-`).

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

Also deferred perf backlog (from `cleanup-refactor`/`performance-sprint`):
`generate-stream` Part B (lazy `generate_*` iterator) and `C<F>`-vs-`F` multiply
peepholes.

### `lips-nbody` — n-body LIPS phase-space generator

Generalize phase-space sampling to 3+ final-state particles using recursive 2-body
decomposition (RAMBO-style). Research Rust options before committing to an approach.
(The MG validation side already generates n-body points via RAMBO in
`gen_amplitude.py`; the MG-computed partonic σ̂ = 6.556e-7 pb for the uux 2→6 at
√s=500 is banked as a future `validate-vegas` reference.)

**Design inputs for the sprint plan** (fold into the design note):

- **Abstraction is the point**: structure the phase-space module so sampler,
  channel mapping, and integrator are separately swappable and composable —
  flat RAMBO vs. recursive 2-body propagator-pole channels, single- vs.
  multi-channel weighting, classic VEGAS vs. VEGAS+ stratification should be
  mix-and-match choices, not rewrites. The known endgame is MG-style
  per-diagram multi-channel (one channel per diagram parametrised by its
  propagator poles, combined with the variance-minimising weight `1/Σᵢ(1/Jᵢ)` —
  note 01 phase-space-optimisation section), and possibly Sherpa-style
  sampling over color/helicity instead of summing.
- **Reference implementations** (submodules; key paths in
  `research/refs/README.md`): Sherpa `PHASIC++/Main/` (multi-channel adaptive
  integrator with separate `Color_Integrator`/`Helicity_Integrator`; note 03
  §1.5), POWHEG `integrator.f` (MINT), MG `madgraph/various/rambo.py` (carries
  the line-218 overflow-warning sign bug documented in note 07).
- **Hazard catalog**: note 07 "Numerical Precision / Stability" and
  "Phase-Space / Integration" test lists. MG's sampler bugs (BW mapping,
  T-channel ordering, threshold kinematics, conflicting-BW configurations)
  stayed latent 5–10 years because sampler errors shift σ smoothly rather than
  tripping a bit-exact gate — plan the validation regime alongside the feature.
- **Validation regime**: bit-for-bit gating exists only with a pinned RNG seed
  and unchanged sampling order; otherwise gate statistically — σ within quoted
  MC uncertainty (the `validate_vegas.rs` targets plus the banked σ̂ above) and
  distribution comparisons, since σ-agreement alone is a weak oracle, blind to
  mis-sampled regions of small measure. For optimization work the figure of
  merit is variance × CPU-time at fixed target precision, not ns/point.

_Unblocks: hadronic pp→ll, `event-output-lhef`._

### `event-output-lhef` — Unweighted events in LHEF format

Accept/reject sampling with `w(p) = |M(p)|²/w_max`; serialize to Les Houches Event File
format for downstream tools (Pythia, Herwig, etc.).

LHEF color tags need MG's *leading-Nc* flow decomposition (`color_flow_decomposition`
/ `get_color_flow_string` in `color_amp.py`) to assign a `(color, anticolor)` integer
pair per external leg — a separate small feature on top of the trace/δ basis
`color-flow` built (note 16 §5); not needed for the multi-flow `|M|²` machinery itself.

_Depends on: `lips-nbody` (n-body final states)._

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
