# vibegraph — Task Backlog

**Working rhythm**: sprints cycle **feature → validation → performance**. A feature
lands behind the MG validation net, a validation pass then hardens the net around
what the feature exposed, and a performance pass optimizes against the hardened
gate. Current position: `color-flow` (feature, ✅ merged 2026-07-12) →
`validation-sprint` (validation, ✅ closed 2026-07-13) →
**post-CSE optimization program** (performance, ✅ closed 2026-07-14) →
**helicity-expansion session** (performance follow-on, ✅ merged 2026-07-16, note 15
§2.2) → **next: hadronic pp→ll / event output** (feature).

## Pipeline Status

| Step | Component | Status | Notes |
|------|-----------|--------|-------|
| 1 | UFO model loading (particles, parameters, couplings, vertices) | ✅ Done | Python AST parser; restrict cards baked into params |
| 2 | Feynman diagram enumeration | ✅ Done | feyngraph + process grammar; validated vs MadGraph |
| 3 | HELAS helicity amplitudes (topology-driven, arbitrary process) | ✅ Done | 14 processes agree with MadGraph (11 bit-identical ≤6.3e-13, incl. 2→6/VVV/massive externals, all NCOLOR=1; `uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14 via the multi-flow CF-weighted eval, NCOLOR=2/2/6) |
| 4 | Phase-space sampling (LIPS + VEGAS) | ✅ Done | Lepage VEGAS + 2-body LIPS |
| 5 | Cross-section integration (e⁺e⁻→μ⁺μ⁻) | ✅ Done | Lepage VEGAS on `AmplitudeEvaluator::eval_m2`; `validate_vegas.rs`: `sigma_z_pole` σ≈2025 pb at √s=91.2 (<0.1% vs MG), `sigma_qed_limit` (√s=10 vs 4πα²/3s, 3%) |
| 6 | Unweighted event output (LHEF) | 🔲 Pending | Accept/reject sampling + Les Houches format |

Closed-sprint history (`helas-generalize`, `mg-validation-coverage`,
`cleanup-refactor`, `performance-sprint`, `color-flow`, `validation-sprint`) lives in
git history and `research/notes/` (12: continuum bug hunt, 13: typed conventions, 15:
eval optimization plan, 16: color-flow design + debrief, incl. the VVVV phase-bug root
cause and fix).

---

## 🚀 Post-CSE optimization program — 3 tracks ✅ CLOSED 2026-07-14 (plan: `research/notes/15-eval-optimization-plan.md`; close-out: note 15 §2.1)

**Outcome:** Track 1 (`eval-layout`) shipped A0–A5 to `main` (A3c cancelled — eval stays
100% safe Rust); Tracks 2 (`rooting-exploration`) and 3 (`dag-extraction`) both closed
**NO-GO**. Cumulative honest evaluator speedup over the P5 baseline `7a1a66d` is
**1.4×–2.1×** (typically ~1.8×) across 2→2…2→6, narrowing the vs-MG gap to **4.9×–68×**
(from ~9×–124× at P5). Re-recorded honest table below (§2.1 has the full ledger).
Forward perf work now needs algebraic rewrites CSE cannot see (`egraph-rewrite`, blocked
on a global/ILP extractor + compute-aware cost model + ≥3-consumer demo) or the
`rooting-soundness` correctness fix — neither is in the feature→validation→performance
critical path; next up is the hadronic pp→ll / event-output feature.

Planned 2026-07-11 from the research pass over rooting symmetry, MadGraph's
optimization stack (helicity recycling = CSE across the unrolled helicity loop,
arXiv:2102.00773), egglog 2.0 extraction semantics, and measured evaluator layouts
(`WaveformSlot<f64>` = 104 B, `Node<Const>` = 12 B). Key structural finding: egglog's
extractor is **tree-cost only**, so every rewrite whose payoff is *sharing* (re-rooting,
chiral decomposition, coupling factoring) is invisible to it — that blocker is Track 3,
and the sharing half of `egraph-rewrite` waits on it. Tracks 1 and 2 need no egraph.

**Revised 2026-07-13 after `color-flow` + `validation-sprint`** — deltas relative to
the note-15 plan:
- The gate is now the **14-process** `validate_helas_mg` net (11 NCOLOR=1
  bit-identical + `uux_to_uux` ≤5.7e-14 + `gg_to_ttx` ≤2e-15 + `gg_to_gg` ≤8.3e-14,
  NCOLOR=6, `validation-sprint`'s VVVV phase fix).
- `eval_m2` is now the CF-weighted multi-flow loop (per-flow JAMPs, MG's ZTEMP
  accumulation order, NCOLOR=1 op-order rule for bit-for-bit) — affects A5.
- Rooting is per (diagram, color-chain), still anchored at `VtxIdx(0)`; cross-flow
  CSE (NCOLOR=6 costs ~2× NCOLOR=2, ≪ naive NCOLOR×) depends on chains of the same
  diagram rooting consistently — affects Track 2.
- `fold.rs` already carries exact-rational pools for `CoeffRat` (C4) — A2 extends
  the same file; the egglog schema + round-trip already cover `Flows`/`CoeffRat`.

Pre-program baseline table (dev machine, `--profile profiling`, `--test-threads=1`;
ns/eval, Rust `main` vs MG MATRIX1 — the `performance-sprint` P5 close-out plus the
`color-flow` close-out rows). ⚠️ These `main` figures were taken through the
`validate_helas_mg` timing report, which A3 later made **non-representative** — its
`extended-validation` per-node cross-check (`cross_check_node`) compiles into the
`eval_m2` loop and roughly doubles ns/eval. The A6 re-record below uses the honest
release `eval_strategies` bench instead (cross-checks compiled out); numbers are not
directly comparable across the two tables.

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
| gg_to_gg | 949 | 24,110 | 25.4× |
| ee_to_mumua | 1,510 | 28,520 | 19× |
| ee_to_mumu_tata_qcd0 | 6,404 | 145,032 | 23× |
| uux_to_ccx_emmm_qcd0 | 100,230 | 10,981,139 | 110× |
| bbx_to_ccx_emmm_qcd0 | 141,430 | 11,821,262 | 84× |

**A6 re-record (2026-07-14, honest release `eval_strategies` bench).** ns per `eval_m2`
(criterion median), P5 baseline `7a1a66d` vs post-A5 `main`, MG from `mg_timings.json`.
Covers the 7 all-massless-external `MG_VALIDATED_PROCESSES` (massive-external ones need
mass-aware kinematics the massless-RAMBO bench lacks — not re-measured):

| process | mult | MG | P5 `7a1a66d` | post-A5 `main` | P5→A5 | A5 vs MG |
|---|:--:|--:|--:|--:|--:|--:|
| ee_to_mumu | 2→2 | 283 | 4,000 | 1,930 | 2.07× | 6.8× |
| ee_to_ee | 2→2 | 731 | 6,619 | 3,609 | 1.83× | 4.9× |
| uux_to_uux | 2→2 | 278 | 4,158 | 2,936 | 1.42× | 10.6× |
| gg_to_gg | 2→2 | 949 | 22,646 | 11,365 | 1.99× | 12.0× |
| ee_to_mumua | 2→3 | 1,438 | 28,826 | 15,188 | 1.90× | 10.6× |
| ee_to_mumu_tata_qcd0 | 2→4 | 6,337 | 149,569 | 82,550 | 1.81× | 13.0× |
| uux_to_ccx_emmm_qcd0 | 2→6 | 97,172 | 12,075,000 | 6,649,375 | 1.82× | 68.4× |

Post-CSE the 2→6s sit at ~15 nodes/diagram, so beyond Track 1's layout wins,
**further cuts need algebraic rewrites** structural hash-consing cannot see. (One
exception surfaced after close-out: the helicity dimension — see the follow-on
session below.)

## ⚡ Helicity-expansion session ✅ MERGED TO MAIN 2026-07-16 (note 15 §2.2)

A5's recycling replaced wholesale: `Folded::expand_helicities` bakes every helicity
combination into one hash-consed arena under an `Op::Hels` root (`External` leaves
specialized per `(leg, helicity)`; `PMom`/`PMomOut` shared outright), so each
distinct current is computed exactly once per point and `eval_m2` is a single linear
pass — no support masks, no skip scan, no shadow-recompute assert (all deleted).
Result slots are liveness-allocated (2→6: 543k nodes, ~27k peak live slots ≈ 1.7 MB);
expansion is lazy (`OnceLock`, ~150 ms one-time for the 2→6). **Bit-for-bit** vs the
per-helicity sum through the unexpanded program (pinned by
`expanded_eval_m2_matches_per_helicity_sum`); gate 14/14 with `max_rel_diff`
unchanged. Same-day companions: bare kernels by reference (the by-value ABI copies
were real, ~9% on big amplitudes), `LowerVout`→`NegVout` rename + stale-doc fix,
node-pad scaffolding removed, packed-index asserts hardened, `egraph.rs` marked
parked.

Honest bench (release `eval_strategies`, ns/eval; cumulative table in note 15 §2.2):
2→2s 765–6,765 (2.4–2.6× over post-A5; gg_to_gg 1.68×), 2→4 27,908 (2.96×), 2→6
2,429,125 (2.74×). **Gap to MG now 1.9×–25×** (was 4.9×–68× post-A5, ~9×–124× at P5).
⚠️ `validate_helas_mg`'s printed timings now run ~4–5× the honest bench (per-node
cross-checks over the expanded arena); never quote them.

### ⚡ Track 1: `eval-layout` — evaluator memory layout & recycling ✅ CLOSED 2026-07-14 (merged to `main`)

All sessions landed on `main`: A0 ✅, A1 ✅, A2 ✅, A3 ✅, A3b ✅ (memo, note 17),
**A3c ❌ cancelled** (no safe bounds-check-elimination mechanism), A4 ✅, A5 ✅, A6 ✅
(close-out, note 15 §2.1 + the re-record table above). Cumulative honest eval speedup
1.4×–2.1× over P5 `7a1a66d`. Gate held 14/14 throughout: 14-process `validate_helas_mg`
REL_TOL 1e-12, bit-for-bit where order-preserving (A0/A2/A3/A5 bit-for-bit; A4 REL_TOL
for reassociated momentum sums). `wavefn.rs` untouched (still the public
hand-built-amplitude + unit-test vocabulary; the runtime grew its own internal storage).
Original per-session plan (design record, detail in note 15 §2):

- **A0** — instruction-size sensitivity check (pad `Node<Const>` to 16/24/32 B and
  measure; also the free 12→8 B pack). Informs A3 and the typed egglog constructors.
  ✅ Done 2026-07-13 (`eval-layout/a0`, merges after A1): instruction-stream width is
  **not** a bottleneck (flat 8→32 B within ±2–3% noise); the 8 B pack is a free
  bit-for-bit ~0–3% win + a sound `ConstKind` pool-kind API. A3 may widen the
  instruction node for typed operands without a width penalty — spend effort on the
  SoA result buffers instead. Details in note 15 §1.5/A0.
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
- **A3b** — bounds-check-branch investigation (added 2026-07-13): A3 removes the
  enum-unwrap branches from the hot loop, leaving (believed) only slice
  bounds-check branches on arena indexing. Investigate **safe** mechanisms to
  eliminate them — e.g. pre-resolving operand/result locations at bind time into
  lifetime-guaranteed references (`&'a Cell<T>`-style arenas, split borrows), or
  restructuring so LLVM provably elides the checks (bind-time index validation +
  hoisted asserts). `unsafe`/`get_unchecked` is out of scope. Deliverable:
  feasibility memo + microbenchmark, go/no-go.
- **A3c** — implement the A3b mechanism (only if A3b says go). Bit-for-bit;
  serializes with A4 (whichever lands second rebases).
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

### 🌲 Track 2: `rooting-exploration` — throwaway rooting study ✅ DONE 2026-07-13 (branch `explore/rooting` @ `9bb8e14`, not merged; results `research/notes/rooting-study-results.md` + note 15 §3.1)

**Outcome:** headroom is real (greedy −21% nodes / −34% slot traffic; cheap "fewest ext
legs" heuristic captures −20% of it, so greedy buys only ~1% over a one-liner; `VtxIdx(0)`
already == the free `lowest-leg` optimum) **but currently unrealizable** — every
node-reducing rooting *silently corrupts the amplitude* (max_rel up to 1.7e+3) on
multi-boson + ≥6-point processes, a latent orientation-dependence in the rooting primitives
(`mul_apply` momentum routing / Lorentz-output rooting / fermion-spine sign validated only
for feyngraph's `VtxIdx(0)` orientation). **Decisions:** (a) do NOT promote a production
rooting pass now — blocked on a `rooting-soundness` fix (see below), and the realizable win
over the free status quo is small next to A3/A4's slot-traffic targets; (b) Track 3
re-rooting rule family = conditional GO, **correctness-first** (soundness spike precedes the
extractor). The M3 chiral-decomposition family is unaffected (it doesn't re-root).

<details><summary>original Track 2 plan (for reference)</summary>

Root choice is currently `VtxIdx(0)` per (diagram, color-chain) — an accident of
feyngraph ordering that cross-diagram CSE silently depends on; post-`color-flow`,
cross-flow CSE additionally depends on chains of the same diagram rooting
consistently, so any candidate heuristic must be applied uniformly across a
diagram's chains and measured on the multi-flow processes too. Over all rootings a
diagram has only ~2·E distinct directed currents, so the *floor* is computable.
Measure post-CSE node count (and slot-cost-weighted) across `MG_VALIDATED_PROCESSES`
(14) for: baseline; canonical heuristics (lowest-leg anchor; most/fewest contributing
external momenta — measure both directions); greedy iterative rooting (each diagram
tries all rootings against the cumulatively-interned arena, min new nodes; both
diagram orders). Every variant runs the full validation net (rootings hit new kernel
paths). Results committed on the branch for posterity + tables appended to note 15 on
`main`. Decision output: if greedy wins big, promote a production greedy-rooting pass
into Track 1; headroom informs the Track 3 go/no-go.

</details>

#### `rooting-soundness` — make re-rooting orientation-independent (prerequisite, surfaced by Track 2)

Blocks any production rooting change AND the Track 3 re-rooting rule family. Today the
amplitude is correct only for feyngraph's `VtxIdx(0)` edge orientation; reversing an
internal edge silently changes the value (Track 2: max_rel up to 1.7e+3 on multi-boson /
≥6-point). Fix the momentum-routing (`mul_apply` bra-add/ket-subtract), Lorentz-output
rooting, and fermion-spine sign to be invariant under root choice. **First test** (the
`set_root_override` hook from `explore/rooting` is ready for it): assert *all V rootings* of
every diagram in `MG_VALIDATED_PROCESSES` pass the `validate_helas_mg` gate. This is
`gg_to_gg`-VVVV-class territory (an unexercised branch drifting out of sync) — a bug magnet;
sequence it as its own spike before the re-rooting extractor, not folded into a perf pass.

### 🧮 Track 3: `dag-extraction` — DAG-cost extractor for egglog ✅ DONE 2026-07-13 → **NO-GO** (decision record: note 15 §4)

egglog 2.0 has no sharing-aware extraction (verified in `extract.rs`:
`TreeAdditiveCostModel`; the `CostModel` trait is tree-shaped). Outcome by milestone:
- **M1** ✅ `enumerate(&Ast) -> DagEGraph` via `EGraph::serialize` (`egraph.rs`).
- **M2** ✅ greedy DAG extractor (`faster-greedy-dag`-style `extract` + `CostModel`/
  `CostKind{Dag,Tree}`/`SlotTrafficCost`/`UnitCost`/`decode_extraction`); sanity gate
  met — reproduces the input DAG byte-for-byte on the rule-free round-trip over the
  dev processes + `MG_VALIDATED_PROCESSES` (`extended-validation`).
- **M3** ✅→ no-go, nothing committed. Chiral-decomposition demo on `e+ e- > mu+ mu-`
  fails for two structural reasons: (1) greedy decides each e-class independently and
  never takes the locally-worse split the *global* co-commit needs (0/4 classes split,
  DAG cost unchanged) → **any sharing-payoff rewrite needs a global/ILP extractor, not
  greedy**; (2) under `SlotTrafficCost` the split is a net loss even at the global
  optimum (forced-split 2816 > fused 2048–2144) — a half-current is charged full output
  slot bytes; the ≥2×-share premise only holds under a compute-aware `WorkCost` (split
  935 < fused 1080, ~13%). `e+ e- > mu+ mu-` is marginal (2 consumers).
- **M4** ✅ write-up + go/no-go (this + note 15 §4).

**Go/no-go: NO-GO under current scope.** Path to yes needs all three: **(a)** a
global/ILP extractor (the reserved ILP oracle promoted to a prerequisite); **(b)** a
compute-aware `WorkCost` model (intersects the §1.6 static output-type analysis / A1);
**(c)** a **≥3-consumer** demo process for a non-marginal payoff. M1/M2 stand as the
reusable extraction substrate. Known issue (note 15 §4.2): the run-to-run DAG-cost
variance is **upstream of `egraph.rs`** (a `HashSet` iteration in `root_diagram`/`lower`
emits a ±1-CSE-node AST per hash seed; correctness-neutral, gate passes), **not** in the
extractor — a cost oracle must compile the AST once and reuse it; the fix belongs to the
lowering owners.

---

## 🧬 `egraph-rewrite` — algebraic rewrite stage (**blocked on Track 3**; scope revised 2026-07-11)

Cut per-process node count below what CSE alone reaches by factoring shared algebraic
structure across diagrams (post-CSE 2→6s are ~15 nodes/diagram). Every rule is a place
a sign bug can hide: each lands guarded by the 14-process `validate_helas_mg` net
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
  (propagator-commute + per-vertex rotation rules). Track 3 resolved to **NO-GO** (note
  15 §4.1): its DAG-cost extractor (M1/M2) is correct but greedy + `SlotTrafficCost`
  provably cannot realize a sharing payoff. Reviving these rules needs all of a
  global/ILP extractor, a compute-aware `WorkCost`, and a ≥3-consumer demo process;
  re-rooting additionally needs the `rooting-soundness` fix. Informed by Track 2's
  headroom.
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
light quarks as massless. Sequenced after the post-CSE optimization program per the
feature→validation→performance loop.

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

- **`IdentityAmp` process-level coverage**: moved to the non-SM UFO boundary
  list below — it needs a non-SM model, so it rides with that work.
- **Rationalize `Coeff(f64)` onto `CoeffRat`** (note 16 §5): now that `Op::CoeffRat`
  exists for color coefficients, the remaining `Coeff(f64)` leaves (Lorentz-structure
  and symmetry/fermi-sign coefficients) could migrate onto it too — optional cleanup,
  not required by anything currently blocked.
- **Branch-level coverage**: op counts don't see rooting branches. The pure-metric
  −1 vertex branch (`root_lorentz`) used to fork on amplitude-root vs scalar-root
  (the latter pinned, the former the unexercised `MetricNegI` op); `validation-sprint`
  found the fork itself was the bug — `gg_to_gg` amplitude-roots a pure-metric VVVV
  contact term, and its separate −i lowering was a spurious phase — and collapsed both
  paths onto one real-−1 branch, now exercised by both `gg_to_gg` (amplitude-rooted)
  and the 2→6 H-current processes (scalar-rooted). More generally, consider rooted-tree
  pattern assertions per MG-pinned convention: each "pinned by X" comment should have a
  test that fails if the pinning process stops exercising the branch — an unexercised
  branch silently drifting out of sync with its exercised sibling is exactly the
  failure mode that produced the `gg_to_gg` bug.
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

### `non-sm-ufo` — collected boundaries a non-SM UFO model will hit

The UFO surface is deliberately model-generic, but "generic" currently ends at the
SM's feature set. None of these block anything (the interned SM avoids them all);
they are collected here so a future BSM-model task scopes against a checklist
instead of rediscovering each wall one hard error at a time. A small dedicated
test model (or a public BSM UFO) would be the natural vehicle for several at once.

- **Color sextets and baryonic epsilons**: the color engine handles
  Singlet/Triplet/AntiTriplet/Octet only (`helas/repr/color.rs`); the sextet
  tensors `K6`/`K6Bar`/`T6` (diquark models) and the baryon-number-violating
  `Epsilon`/`EpsilonBar` (e.g. RPV SUSY) are deliberate hard errors
  (`ufo/color.rs::SextetUnsupported`, `helas/color/tensor.rs`). Note the two
  distinct "6"s: NCOLOR=6 (flow-basis dimension, e.g. `gg_to_gg`) is fully
  supported; the 6-dimensional sextet *representation* is not. MG's reference
  algebra for the missing tensors lives in `color_algebra.py` (K6/T6/ε
  Clebsches); support means new `ColorTensor` atoms + trace-basis reduction
  rules + CF products, validated the color-flow way (CF oracle vs MG's DATA CF,
  then the JAMP-weighted |M|² gate).
- **Spin codes beyond {1, 2, 3}**: `helicity_states_for_spin` (`eval/compile.rs`)
  future-proofs the spin-2 helicity list (code 5), but nothing downstream builds
  tensor external wavefunctions or propagators; spin-3/2 (code 4, gravitinos) is
  an `UnsupportedSpin` error. Ghost codes (negative) stay irrelevant at LO.
- **Majorana fermions** (MSSM neutralinos, gluinos): fermion-flow handling
  assumes Dirac-continuous lines end to end — there is no flow-flip /
  charge-conjugation machinery. This is HELAS's classically subtle sign
  territory; the `color-flow` fermion-flow slot-swap bug shows how delicate the
  flow conventions are even in the pure-Dirac case.
- **`IdentityAmp` process-level coverage** (deferred from `validation-sprint`,
  the last `KNOWN_UNCOVERED` op): needs an `Identity` scalar bilinear in the
  Lorentz sector, which the SM lacks — a natural rider on whichever small test
  model lands first.
- **Loop-level UFOs** (`loop_sm`, NLO models): out of the LO charter. Note 04
  records the parser history — the Python-AST parser replaced the FeynGraph/PEG
  split that choked on `loop_sm`'s attribute assignments, but counterterm
  content (`CT_vertices.py` etc.) has no consumer regardless.

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

The timing table compares against MG MATRIX1, which sums helicities. Since the
helicity-expansion session (2026-07-16) both sides now share currents across the
helicity loop — MG via its restructured-call recycling, vibegraph via the baked
`Op::Hels` expansion — so the helicity-sum ratio is a fair like-for-like; a parallel
benchmark evaluating **one fixed helicity configuration** on both sides still
isolates kernel-level gaps from expansion/sharing effects. It is also the relevant
comparison for the event-generation regime: final accept/reject evaluates a specific
helicity configuration through the *unexpanded* program, where the expansion buys
nothing (its win belongs to the integration-grid phase and its helicity-summed
`eval_m2`).

**A6 go/no-go (2026-07-14): DEFER — not pulled in.** The vibegraph half
(`eval_amplitude` at one fixed helicity) is a cheap bench addition, but the *fair*
comparison needs an MG single-helicity timing, and MG's MATRIX1 driver hardcodes the
helicity-sum loop — a single-config timing means editing the generated Fortran driver +
the `gen_amplitude.py` timing harness and regenerating reference data (a
reference-data/Fortran task, not a warm-rig freebie), and a vibegraph-only number is
half an oracle. No live consumer until `event-output-lhef` accept/reject makes
single-helicity the actual hot path. Recommendation: land it **alongside
`event-output-lhef`**, when the comparison has a consumer and the MG-harness change is
on the critical path anyway.
