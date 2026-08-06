# 15 — Post-CSE Evaluator Optimization: Research Findings & 3-Track Plan

**Status:** CLOSED 2026-07-17 — the program is complete and this note is its design
and measurement record. Planned 2026-07-11 as successor to the closed
`performance-sprint`; §2.1 (layout tracks close-out), §2.2 (helicity expansion), and
§2.3 (helicity filtering) record the outcomes, and the pre-program baseline table
lives at the end of §2.1. Deferred items are summarized in `TODO.md`. Companion to
`research/notes/14-egglog-notes.md` (egglog language reference).

The perf sprint left the evaluator 8.6×–110× slower than MG MATRIX1 with post-CSE
2→6s at ~15 nodes/diagram. This note records the research that scoped what comes
next, the design decisions taken, and the plan: three tracks that can proceed in
parallel — **(1) `eval-layout`**, non-egraph evaluator work merged to `main`;
**(2) `rooting-exploration`**, a throwaway study on a branch; **(3) `dag-extraction`**,
an investigation that gates the sharing half of `egraph-rewrite`.

---

## 1. Research findings

### 1.1 What MadGraph does before emitting Fortran

Two generations of optimization (refs at bottom):

- **Wavefunction reuse across diagrams** (MG5, arXiv:1106.0522): within one helicity
  configuration, identical wavefunction calls are emitted once and shared. This is the
  analog of our hash-cons CSE pass.
- **Helicity recycling** (arXiv:2102.00773, EPJC 81:435, the ~2× "speeding up" paper):
  CSE **across the unrolled helicity loop**. MG generates the matrix element normally,
  samples a few events to find contributing helicity configurations (filtering the
  rest permanently), then rewrites the code: the helicity loop is unrolled, a DAG of
  wavefunction/amplitude dependencies is built, and calls whose inputs coincide
  between configurations are deduplicated. Three levels: external spinors (spinor
  `1+` is identical in `(1⁻2⁺3⁻4⁺)` and `(1⁻2⁺3⁺4⁻)`), internal currents (a current
  depending only on legs 3,4 is shared by every configuration of legs 1,2), and
  partial amplitude factors (for ψ̄₁γ_μψ₂·ε^μ, store ψ̄₁γ_μψ₂ and contract with each
  polarization; the factorization to recycle is chosen by reuse count). Plus numeric
  pruning of calls feeding only vanishing amplitudes, and ~20% more from CSE on color
  factors. Net 2.27× for gg→ttgg, ~1.3× for fermion-heavy processes. ALOHA grew new
  output-form routines to support it.

**vibegraph angle:** `eval_m2` re-runs the whole arena per helicity combination (a
2→6 has 256), while most internal currents depend on ≤3 legs' helicities. Each node's
helicity support is static (the `External` legs in its subtree), so skipping unchanged
nodes across the helicity loop is a large, purely structural win — **for the
helicity-summed path**. Design caveat (decided 2026-07-11): the helicity sum drives
integration-grid generation, where a cumulative per-helicity-combination ledger reuses
partial results very effectively; but final unweighted-event accept/reject evaluates a
*specific* helicity configuration, where recycling buys nothing. Hence the
`mg-single-helicity-bench` backlog item: a parallel MG benchmark at one fixed helicity,
to separate kernel-level gaps from CSE/helicity-loop gaps.

### 1.2 egglog 2.0 extraction is tree-cost — the central blocker

Verified in the vendored crate (`egglog-2.0.0/src/extract.rs`): the default
`TreeAdditiveCostModel` sums children **per occurrence**, and the pluggable
`CostModel` trait (`fold(head, children_cost, head_cost)`) is inherently tree-shaped.
There is no DAG-cost (sharing-aware) extraction. Consequence: **every rewrite whose
payoff is enabling sharing is invisible to extraction** — the shared form has equal
or worse tree cost, so tree-cost extraction never picks it. This blocks, identically:

- **Re-rooting rules** (§1.3): all rootings of a diagram have *equal* tree cost;
  extraction tie-breaks arbitrarily and won't align rootings across diagrams.
- **Chiral decomposition / coupling factoring** (§1.4): the split form costs more as
  a tree, and only wins when the pure current is shared ≥2×.

Constant folding is the exception (strictly removes nodes, tree-cost-visible) — and
it turns out not to need egglog at all (§2, session A2).

### 1.3 Rooting symmetry

Today the root is literally `VtxIdx(0)` (`root_diagram::root_tree`) — feyngraph's
vertex ordering. Cross-diagram CSE therefore depends on an accident of orientation.

Structure of the problem: a rooting orients every internal edge; over all V rootings
of a tree diagram the set of distinct off-shell currents is one per (edge, direction)
— **~2·E per diagram, linear**. (This is the Berends–Giele current set restricted to
the topology, and a computable *floor* on node count achievable by rooting freedom.)

- **e-graph formulation is bounded and local.** Re-rooting decomposes into
  (a) propagator-commute — symmetric boson numerator:
  `Contract(X, Propagate(Y,m,w)) = Contract(Propagate(X,m,w), Y)`; (b) per-vertex
  rotation rules relating the amplitude form to each rooted-at-leg-i form (e.g.
  `Metric(GammaVout(f1,f2), v)` ↔ the `GammaIout`-rooted contraction). O(vertex
  shapes × legs) rules; saturation adds only the ~2E currents per diagram; congruence
  finds cross-diagram matches for free. Fermion-line rules need care (bra/ket flips,
  transposition signs) — start bosonic. **But worthless without DAG-cost extraction
  (§1.2).**
- **Cheap non-egraph alternatives:** (a) *canonical rooting* — a deterministic anchor
  (e.g. vertex adjacent to the lowest-index external leg) makes isomorphic subgraphs
  orient consistently, nearly free; (b) *greedy trial rooting* — lower diagrams
  sequentially, try all V rootings per diagram against the cumulative hash-consed
  arena, keep the one minimizing new nodes. O(V × nodes) per diagram at compile
  time. A good greedy approximation to the (NP-hard, overkill) max-common-subgraph
  objective, and the natural **go/no-go oracle** for the DAG-extraction investment.

Validation caveat: new rootings exercise kernel paths the `VtxIdx(0)` rooting never
hits — including the `KNOWN_UNCOVERED` amplitude forms `MetricNegI`/`IdentityAmp` —
so every rooting variant must run under the full `validate_helas_mg` net (REL_TOL;
rootings are not order-preserving).

### 1.4 Sharing vertices across different propagating particles

`Propagate(current, Mass, Width)` separation means γ and Z exchanged between the same
fermion pair *could* share structure — but the fused `FfvVout(f1, f2, gL, gR)` carries
couplings as operands (γ: gL=gR=e; Z: gL′≠gR′), so the nodes are distinct and
structural CSE can never merge them. Two rewrite families expose it:

- **Chiral decomposition:** `FfvVout(a,b,gl,gr) → gl·J_L(a,b) + gr·J_R(a,b)` (pure
  chiral kernels as extraction targets). γ and Z then share `J_L`/`J_R` and differ
  only in scalar recombination. This is the *inverse* of the P6 fusion — fusion wins
  used-once, splitting wins shared ≥2× — exactly what cost-based extraction should
  arbitrate. Blocked on DAG cost (§1.2). Minimal demo process: `e+ e- > mu+ mu-`
  (γ and Z s-channel over the same spinor pairs).
- **Propagator linearity:** `Propagate(Mul(s, x), m, w) ↔ Mul(s, Propagate(x, m, w))`
  to float scalar couplings out so propagated structural currents unify.

⚠️ **Momentum-routing soundness constraint:** `mul_apply` *routes momentum* when a
scalar multiplies a current (ket subtracts, bra adds — the FFS convention pinned by
e⁺e⁻→τ⁺τ⁻H). Scalar-motion rewrites are only sound for **momentum-free** scalars
(Coupling/Coeff subgraphs), never scalar *wavefunctions*. The rewrite domain must
distinguish them → the `ScalarConst` vs `ScalarWf` sort split (§1.6).

### 1.5 Measured sizes (scratch crate, 2026-07-11)

`WaveformSlot<f64>` = **104 B** (96 B payload + tag; the sprint's "112 B slot
traffic" figure included adjacent accesses), `Node<Const>` = **12 B**, `Const` = 8 B,
`Op` = 1 B. Observations:

- The `Const` discriminant is fully redundant — `Op` determines the pool kind
  (Coupling→Complex; Mass/Width/Coeff→Real; External→Ext; else None). `{op: u8,
  payload: u32}` = 8 B is free. **Correction (A0, 2026-07-13):** op *alone* is not
  quite sufficient — `CoeffRat` folds to a real **or** complex pool entry, so pool
  kind is not a pure function of the op. The 8 B pack instead tags `Const` as a
  4-byte `u32` (2-bit `ConstKind` in the top bits, 30-bit index below), which stays
  8 B for `Node<Const>` and is fully general. `ConstKind` + `Const::{kind,index,…}`
  is now the sound source of truth for pool kind (A1's constness analysis and A3's
  typed encoder should use it rather than re-deriving from the op).
- **Boxing large slot variants is a dead end:** fermion/vector currents (96 B) are the
  hot *majority*; boxing adds a pointer chase per operand read plus per-node
  allocation churn, to shrink the enum the cold variants don't dominate anyway.
- The real wins are structural: **(a) static output types → SoA result buffers** —
  every node's slot variant is statically known (the evaluator already panics on
  mismatch), so per-type arenas remove the tag, padding, and runtime variant dispatch,
  and scalars drop from 104 B slots to 16 B; **(b) momentum pooling** — every slot
  drags a 32 B momentum and `mul_apply` does momentum arithmetic at runtime, yet every
  current's momentum is a compile-time-known signed combination of external momenta,
  *independent of helicity*. A per-point momentum table (one entry per distinct
  line) shrinks payloads (fermion/vector 96→64 B) and stops redoing identical
  momentum arithmetic once per helicity combination. (b) is also groundwork for
  helicity recycling (§1.1).

### 1.6 egglog schema decisions (adopted 2026-07-11)

A separate `Leaf` datatype vs inline `i64` fields is semantically equivalent in
egglog (hash-consing/congruence identical; one extra table join per leaf pattern —
negligible). Decisions:

- **Split every leaf kind into its own sort** (`CouplingId`, `ParticleId`, `Real`,
  `ExtLegInfo` as distinct datatypes, not one shared `Leaf` sort — a shared sort lets
  `(Mass (CouplingId 5))` typecheck). Runtime re-checks types anyway; this just
  catches rule bugs sooner, and most rules bind a leaf as one opaque variable.
- **`ScalarConst` vs `ScalarWf` sort split** in the expression grammar, so
  scalar-motion rules (§1.4) are well-typed by construction rather than guarded.
- **Typed constructor slots** where the op fixes them: `(Propagate Node Mass Width)`,
  and `Mul` split by operand class (`MulConst` over constants vs current-scaling
  `Mul`). This widens the runtime instruction node — acceptable if slot traffic
  dominates; **session A0 verifies that assumption before committing** (artificially
  inflate `Node<Const>` and measure).

---

## 2. Track 1 — `eval-layout` sprint (merge to `main`)

Non-egraph evaluator work. Gate: 11-process `validate_helas_mg` (REL_TOL 1e-12);
bit-for-bit wherever the change is order-preserving (noted per session). Baseline:
the P5 timing table in `TODO.md`. `wavefn.rs` is **not** modified — it stays the
public hand-built-amplitude component and the unit-test cross-check vocabulary; the
runtime grows its own internal storage types.

Sessions, in dependency order:

- **A0 — instruction-size sensitivity check** (throwaway measurement). Pad
  `Node<Const>` to 16/24/32 B, re-run the timing rig; also measure the 12→8 B pack.
  Establishes how much instruction-stream width matters before A3's typed (wider)
  instruction stream and §1.6's typed constructors commit to it. Results → this note.
  **Done 2026-07-13 (branch `eval-layout/a0`, commits `e76717f`/`e7c1075`; merges
  after A1).** Finding: **instruction-stream width is not a bottleneck** — min
  ns/eval is flat across 8→32 B within the ±2–3% run-to-run noise floor on every
  process, including the 2→6 giants (the padding sweep itself establishes that noise
  floor: a functionally-identical rebuild wandered ee_to_mumu 3937–4152). The hot
  loop is dominated by 104 B `WaveformSlot` result traffic + kernel arithmetic, as
  §1.5 predicted. The 8 B pack is a **free, bit-for-bit ~0–3% win** (also drops the
  enum-discriminant unwrap in `apply`) — kept as an A3 input, not the lever.
  **Consequences for A3:** adopt typed constructors + typed operand indices freely
  (no measurable width cost out to 32 B), but prefer the narrowest encoding that
  expresses them (keep the pack; pack operand indices tightly); spend A3 effort on
  the SoA result buffers (the slot-traffic side), not on defending instruction width.
  The `node-pad-16/24/32` Cargo features + `tests/instruction_size_bench.rs` stay on
  the branch (default-off, zero-cost) so A3 can re-measure any candidate layout.
- **A1 — static node analysis pass.** One compile-time forward scan annotating every
  node: (i) output type (real const / scalar const / scalar wf / vector / fermion-in /
  fermion-out — mirrors `apply`'s dispatch, and realizes the `ScalarConst`/`ScalarWf`
  taxonomy of §1.6); (ii) constness (all descendants Coupling/Coeff/Mass/Width);
  (iii) momentum id (signed external-momentum combination, interned into a table);
  (iv) helicity-support mask (external legs in subtree). Validated by debug
  assertions cross-checking predictions against actual runtime slots over the full
  suite. Pure analysis — no behavior change. Everything downstream consumes it; (i)
  is also the egraph-rewrite schema groundwork.
- **A2 — constant-subgraph folding.** Nodes marked const by A1 collapse into
  bind-time pool entries evaluated once per parameter card (extends `fold.rs`).
  Deletes the P6 follow-up: `g_L`/`g_R` scalar subgraphs are card-time constants
  currently re-evaluated per point per helicity. Evaluation order of the folded
  subgraph preserved → **bit-for-bit**. (This was previously slated as the first
  egglog rule; it needs no rules.)
- **A3 — SoA scratch + typed instruction stream.** Replace `Vec<WaveformSlot>` with
  per-type result arenas; instructions become typed ops with typed operand indices
  (from A1); `Node` repack (§1.5) folds in, informed by A0. Element types initially
  stay the `wavefn.rs` structs (momentum still embedded) so arithmetic is untouched →
  **bit-for-bit**. Removes enum tag/padding/variant dispatch (`expect_fermion_*`).
- **A3b — bounds-check-branch investigation** (added 2026-07-13). Post-A3 the hot
  loop's remaining branches should be slice bounds checks on arena indexing (verify
  this claim first — inspect the generated code). Investigate *safe* ways to remove
  them: bind-time pre-resolution of operand/result locations into lifetime-bound
  references (aliasing rules block plain `&'a mut T` for shared operands — evaluate
  `&'a [Cell<T>]` / per-instruction split borrows), `OnceCell<T>`-element arenas
  (candidate caveats: the get-path is-initialized check may just replace the
  bounds-check branch, and A5's cross-helicity slot rewriting needs reset
  semantics OnceCell lacks), or making the checks provably
  elidable (validate all indices once at bind time, hoist asserts so LLVM drops the
  per-access checks). `unsafe` is out of scope. Deliverable: a short feasibility
  memo with a microbenchmark of the winning candidate, and a go/no-go.
- **A3c — bounds-check elimination** (contingent on A3b go). Production
  implementation of the chosen mechanism; **bit-for-bit** (no arithmetic change).
  Serializes with A4 — whichever lands second rebases onto the other.
- **A4 — momentum pool.** Per-point table of external + internal-line momenta
  computed once (helicity-independent), nodes reference momentum by id; SoA elements
  become bare `Bispinor`/`ComplexVector`/`C<F>`; `mul_apply` momentum routing and
  momentum-equality asserts leave the hot path; `PMom`/`PMomOut` become table reads.
  Externals still constructed via `wavefn.rs` then stored. Momentum sums are
  reassociated (≤ n_ext terms) → **REL_TOL gate**, not bit-for-bit.
- **A5 — helicity-support recycling.** In `eval_m2`, iterate helicity combinations in
  odometer order (last leg fastest); per combination compute the changed-legs mask and
  skip every node whose A1 support mask doesn't intersect it (results persist in the
  A3/A4 scratch across combinations). Same arithmetic, same order → **bit-for-bit**
  vs A4. Single-helicity `eval_amplitude` path unchanged. (Integration-phase win;
  see §1.1 caveat re: unweighting.)
- **A6 — close-out.** Re-record the timing table vs MG and vs the P5 baseline; update
  `TODO.md` and this note; decide whether `mg-single-helicity-bench` (backlog) is
  worth pulling in while the timing rig is warm. **Done 2026-07-14 (§2.1).**

### 2.1 Close-out results (A6, 2026-07-14)

Track 1 delivered A0–A5; A3c was cancelled. The final measurement uses the honest
release hot path — the `eval_strategies` criterion bench (release profile, per-node
cross-checks compiled out) — **not** the `validate_helas_mg` timing report, whose
`required-features = ["extended-validation"]` forces the shadow-workspace cross-check
(`cross_check_node`) into the `eval_m2` loop and roughly doubles the ns/eval (the A4/A5
sessions flagged this; the honest bench reproduces the recorded P5 table here, confirming
the harness is the non-representative one). Each process's AST is compiled once per bench
run — the lowering path emits a ±1-node AST per hash seed (§4.2) — so the numbers carry
≈±2–3% run-to-run noise, well below the measured wins.

**Cumulative evaluator speedup, P5 baseline `15c4d7c` → post-A5 `main`.** ns per
`eval_m2` (helicity-summed |M|², one phase-space point), criterion median; MG = MATRIX1
`ns_per_eval` from `validation/madgraph/output/mg_timings.json`:

| process | mult | NCOLOR | MG | P5 `15c4d7c` | post-A5 `main` | P5→A5 | A5 vs MG |
|---|:--:|:--:|--:|--:|--:|--:|--:|
| ee_to_mumu | 2→2 | 1 | 283 | 4,000 | 1,930 | 2.07× | 6.8× |
| ee_to_ee | 2→2 | 1 | 731 | 6,619 | 3,609 | 1.83× | 4.9× |
| uux_to_uux | 2→2 | 2 | 278 | 4,158 | 2,936 | 1.42× | 10.6× |
| gg_to_gg | 2→2 | 6 | 949 | 22,646 | 11,365 | 1.99× | 12.0× |
| ee_to_mumua | 2→3 | 1 | 1,438 | 28,826 | 15,188 | 1.90× | 10.6× |
| ee_to_mumu_tata_qcd0 | 2→4 | 1 | 6,337 | 149,569 | 82,550 | 1.81× | 13.0× |
| uux_to_ccx_emmm_qcd0 | 2→6 | 1 | 97,172 | 12,075,000 | 6,649,375 | 1.82× | 68.4× |

Cumulative eval speedup **1.4×–2.1×** (typically ~1.8×) across 2→2…2→6; the honest
post-A5 gap to MG is **4.9×–68×** (vs ~9×–124× at P5 on the same honest bench).
`uux_to_uux` gains least (1.42×) — a small NCOLOR=2 QCD process has few constant
subgraphs for A2 to fold and little slot traffic for A3/A4 to compress. The heavy
processes (gg_to_gg 1.99×, the 2→6 1.82×) gain most, as the layout program targeted.

The bench covers 7 of the 14 `MG_VALIDATED_PROCESSES` — every all-massless-external one
(so plain massless RAMBO supplies the kinematics), including both colored 2→2s
(NCOLOR=2/6, the CF-weighted multi-flow `eval_m2` A5 recycles across). The 7
massive-external processes (ee_to_zh/ttx/wpwm/tatah, gg_to_ttx, the bbx 2→6,
pp_to_ll_qcd0) are absent from the honest bench (they need mass-aware kinematics), so
their pre-program figures in the baseline table below were not honestly re-measured;
the layout wins are process-structural, so the same ~1.8× is expected but unverified here.

**Pre-program baseline** (recorded 2026-07-13; dev machine, `--profile release-debug`,
`--test-threads=1`, ns/eval — the `performance-sprint` P5 close-out plus the
`color-flow` close-out rows). ⚠️ Taken through the `validate_helas_mg` timing report,
which compiles the per-node cross-check into the loop — **not comparable** to the
honest `eval_strategies` tables in this note. Kept as the program's starting record
and the only figures covering the 7 massive-external processes:

| process | MG | pre-program `main` | ratio |
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

**Per-session ledger** (gate = 14-process `validate_helas_mg`):
- **A0** — 8 B tagged `Const` pack + instruction-width sensitivity harness. Bit-for-bit.
  Finding: instruction-stream width is not a bottleneck (flat 8→32 B); the pack is a free
  ~0–3% win and a sound `ConstKind` pool-kind API.
- **A1** — static per-node analysis (output type, constness, momentum id, helicity-support
  mask). Pure analysis, no behavior change.
- **A2** — constant-composite folding into bind-time pools. Bit-for-bit. Node counts
  −29% (ee→tt) / −15% (ee→μμ); gg→gg ~flat.
- **A3** — SoA per-type result arenas + typed instruction stream (folds in A0's pack).
  Bit-for-bit. −8% gg→gg, −5% on the 2→6 at the session measurement.
- **A4** — per-point momentum pool + bare SoA elements; momentum resolved once before the
  helicity loop. REL_TOL (momentum sums reassociate 1–3 ULP; 14/14 unchanged to 3 sig figs).
- **A5** — helicity-support recycling in `eval_m2` (skip nodes whose support mask misses
  the changed legs; FillMode Full/Recycle). Bit-for-bit vs A4. Integration-phase eval_m2
  speedup 1.29× (2→2) → 1.08× (2→6); the recompute fraction rises with multiplicity, so
  fine node granularity caps the win below MG's wavefunction-level ~2×.
- **A3b/A3c** — bounds-check-elimination feasibility (note 17): a +7–11% ceiling is real
  but no *safe* mechanism captures it, so **A3c was cancelled** (eval stays 100% safe Rust,
  user decision). Probe/equality-twin harness parked on `eval-layout/a3b`.

**Tracks 2 and 3 both closed NO-GO** (details unchanged in §3.1 / §4.1): Track 2 (rooting)
found −21% node headroom that is currently unrealizable — the rooting primitives are
orientation-dependent, so a `rooting-soundness` correctness fix must precede any rooting
pass; Track 3 (dag-extraction) shipped a correct DAG-cost extractor (M1/M2) but proved
greedy + `SlotTrafficCost` cannot realize a sharing payoff. The sharing half of
`egraph-rewrite` stays blocked on a global/ILP extractor + a compute-aware cost model +
a ≥3-consumer demo process.

**`mg-single-helicity-bench`: recommend defer (not pulled in).** The vibegraph half
(`eval_amplitude` at one fixed helicity) is cheap to bench, but the *fair* comparison
needs an MG single-helicity timing, and MG's MATRIX1 driver hardcodes the helicity-sum
loop — extracting a single-config timing means editing the generated Fortran driver and
the `gen_amplitude.py` timing harness and regenerating reference data. That is a
reference-data/Fortran task, not a warm-rig freebie, and a vibegraph-only number is half
an oracle. It also has no live consumer until unweighted-event accept/reject
(`event-output-lhef`) makes single-helicity the actual hot path. Kept as a scoped backlog
entry; land it alongside `event-output-lhef`, when the comparison has a consumer and the
MG-harness change is on the critical path anyway.

### 2.2 Helicity-expansion session (2026-07-16, merged to `main`)

A5's recycling underdelivered (1.29× → 1.08× with multiplicity) for two structural
reasons: (i) the Recycle scan still walked *every* instruction per combination (support
load + branch even when skipping, O(n_nodes × n_combos) regardless of skip fraction),
and (ii) odometer recycling reuses only the *previous* combination's slots — a node
whose support contains the fastest-varying leg recomputes on every combination even
though it takes only `2^|support|` distinct values. The fix is the trick §1.1 credits
MadGraph with (arXiv 2102.00773), done through our own hash-consing instead of call
restructuring: **bake every helicity combination into one arena at compile time**
(`Folded::expand_helicities`, `Op::Hels` root, `External` leaves specialized to
`(leg, hel)` entries) and intern nodes across combinations, so each distinct current
is computed **exactly once** per phase-space point — the FLOP minimum — in one linear
pass with no skip predicate. `PMom`/`PMomOut` read only routed (helicity-independent)
momenta, so they are memoized per pre-expansion node and shared outright.

Two companion changes made it viable:
- **Liveness slot allocation** in `Program::build`: a slot is recycled once its last
  arena read has executed (roots pinned live-to-end; an instruction never writes over
  its own operands). Peak live width, not node count, sizes the arenas — the 2→6
  holds 543k nodes but peaks at ~27k live slots (~1.7 MB total, cache-resident).
  Arenas are no longer cleared per pass (every slot is written before read).
- The expansion is **lazy** (`OnceLock` on `AmplitudeEvaluator`): only `eval_m2`
  forces it (one-time cost: ~150 ms for the 2→6, µs–ms elsewhere);
  `eval_amplitude`/Ward/probes keep the unexpanded program.

Sharing measured (expanded vs combinations × base nodes): 2.8× (ee_to_mumu, 16
combos), 2.0× (gg_to_gg), 2.3× (2→4, 64 combos), 1.8× (2→6, 256 combos: 543k vs
990k). The A5 support-mask machinery (FillMode, shadow-recompute assert,
`NodeAnalysis::support`) is deleted; the multi-flow CF contraction now scales JAMPs
by the real CF entry (matching MADGRAPH's real×complex product) and reads them
straight from the arena (no per-combination `Vec`).

**Exactness**: the expansion copies each node's arithmetic verbatim, so `eval_m2` is
**bit-for-bit** against the per-helicity sum through the unexpanded program — pinned
by `expanded_eval_m2_matches_per_helicity_sum` (exact `assert_eq!` across colorless,
massive-external, NCOLOR=2 and NCOLOR=6 processes). Gate 14/14 with `max_rel_diff`
identical to the §2.1 record (uux_to_uux 5.61e-14, gg_to_ttx 1.89e-15, gg_to_gg
8.25e-14).

**Honest bench** (release `eval_strategies`, criterion median ns/eval; includes the
same-day by-reference bare-kernel + cleanup merges):

| process | mult | MG | post-A5 | now | A5→now | now vs MG |
|---|:--:|--:|--:|--:|--:|--:|
| ee_to_mumu | 2→2 | 283 | 1,930 | 765 | 2.52× | 2.7× |
| ee_to_ee | 2→2 | 731 | 3,609 | 1,390 | 2.60× | 1.9× |
| uux_to_uux | 2→2 | 278 | 2,936 | 1,224 | 2.40× | 4.4× |
| gg_to_gg | 2→2 | 949 | 11,365 | 6,765 | 1.68× | 7.1× |
| ee_to_mumua | 2→3 | 1,438 | 15,188 | 5,824 | 2.61× | 4.1× |
| ee_to_mumu_tata_qcd0 | 2→4 | 6,337 | 82,550 | 27,908 | 2.96× | 4.4× |
| uux_to_ccx_emmm_qcd0 | 2→6 | 97,172 | 6,649,375 | 2,429,125 | 2.74× | 25.0× |

Cumulative vs the P5 baseline this is ~3.7×–5.4×; the vs-MG gap narrows from
4.9×–68× (post-A5) to **1.9×–25×**. gg_to_gg gains least (1.68×) — 16 combos over an
NCOLOR=6 flow basis leaves less cross-combination sharing (2.0×) than the leptonic
processes.

⚠️ The `validate_helas_mg` timing report is now even *less* representative than §2.1
warned: its `extended-validation` build compiles `cross_check_typed` (per-node slot
reconstruction + momentum-table resolve + type/momentum asserts) into the expanded
pass — ~543k cross-checks per 2→6 `eval_m2` — putting its printed ns/eval at ~4–5×
the honest bench (e.g. 2→6: 9.69 ms there vs 2.43 ms honest). Release/bench builds
compile the checks out entirely (verified: no `cross_check` symbols in the bench
binary). Use `eval_strategies` for timing claims, always.

**Deferred**: expansion-aware `mg-single-helicity-bench` framing still lands with
`event-output-lhef` (accept/reject samples one helicity, where the *unexpanded*
program is the hot path); a smarter-than-hash-consing expansion (e.g. factoring the
CF contraction across combinations) has no identified headroom yet.

**CF-factoring analysis (2026-07-17, program wrap-up)** — the "no identified
headroom" verdict, worked out. The candidate restructure accumulates the Hermitian
flow matrix `M_ij = Σ_hel JAMP_i·JAMP_j*` across combinations and contracts
`Σ_ij CF_ij·M_ij` once at the end, instead of `eval_m2`'s per-combination quadratic
form. Counting the arithmetic, it rebalances rather than shrinks: the CF multiply
leaves the helicity loop but an NCOLOR² complex outer-product accumulation enters
it — the same O(N_hel·NCOLOR²) multiply-adds with similar constants (Hermitian
halving is available to both forms). And it reorders the |M|² floating-point sum,
so the bit-for-bit gate would drop to REL_TOL on every multi-flow process. Shelved
without a measured win to justify that; if headroom exists anywhere it is the
colored 2→2s (gg_to_gg 3.5× vs MG — NCOLOR=6 makes the per-combination contraction
relatively expensive).

The **diagonal** of that matrix has an independent consumer: `M_ii = Σ_hel
|JAMP_i|²` is exactly MadGraph's `JAMP2` array. madevent's `MATRIX1` accumulates it
per helicity call (`JAMP2(I) = JAMP2(I) + DABS(DBLE(JAMP(I,M)*DCONJG(JAMP(I,N))))`,
summed over good helicities at the phase-space point) and `SELECT_COLOR` draws the
event's color flow `i` with probability ∝ `JAMP2(i)` — the leading-color flow
assignment that LHEF color tags need. The off-diagonals play no role there by
construction: flow interference is 1/N²-suppressed and not sign-definite, so it
cannot be a flow probability. Crucially this dual use does **not** strengthen the
factoring case — the diagonal alone is an O(N_hel·NCOLOR) accumulator bolted onto
the existing per-combination loop, leaving the |M|² operation order (and the
bit-for-bit gate) untouched. That bolt-on rides with `event-output-lhef`, together
with pinning the flow-index → color-string dictionary against MG's
`SELECT_COLOR`/`color_flow_decomposition` conventions (the gg_to_gg NCOLOR=6
flow-basis ordering caveat applies; a transposed dictionary is invisible to any
|M|²-level gate). In the accept/reject regime one helicity is sampled, so `JAMP2`
degenerates to that combination's `|JAMP_i|²` — equally valid at leading color.

### 2.3 Helicity filtering (`prune_zero_helicities`, 2026-07-17)

The §2.2 expansion still evaluated *every* helicity combination; MadGraph does not.
Research findings on how MadGraph decides which to skip:

- **Runtime (standalone `SMATRIX`)**: first 20 calls evaluate all `NCOMB`
  combinations; `GOODHEL(IHEL)` is latched by the exact test `T .NE. 0D0`;
  afterwards only good ones run. Filter disabled for `NEXTERNAL ≤ 3` (spin-2/frame
  caveat).
- **Runtime (madevent)**: same loop but with a relative threshold
  `DABS(TS(I)) .GT. ANS*LIMHEL/NCOMB`, `LIMHEL = 1e-8` (run-card default).
- **Codegen (helicity recycling, MG ≥3.x)**: `gen_ximprove.get_helicity` runs a
  `madevent_forhel` init-mode survey (~1000 points, the LIMHEL criterion) and the
  generated `matrix1_optim.f` bakes in only the surviving `NHEL` rows (plus
  per-(hel,diagram) `ZEROAMP` skipping). **Our timed MG column is this code** — the
  gap we'd been chasing included a structural handicap of evaluating 4–16× more
  combinations than MG (e.g. 16/64 for the 2→4, 16/256 for the 2→6).

An easy *symbolic* zero test was considered and rejected: the arena ops are
slot-level HELAS kernels, so zero-structure propagation would need a hand-written
zero-mask transfer function per op (~20 new convention hypotheses, each needing its
own pinning test). The numeric probe is equivalent in detection power
(Schwartz–Zippel: a rational function of the momenta that vanishes at generic random
points vanishes on the whole manifold, a.s.) and matches MG's semantics exactly.

**Implementation** (`AmplitudeEvaluator::prune_zero_helicities(&mut self,
&EvaluatedModel)`): probe the full expansion at 10 deterministic generic CM points
(massive RAMBO — new `phasespace::rambo_massive` — at two √s scales, seeded RNG),
mark combinations by the MG criterion with `HEL_PRUNE_REL = 1e-24`, retain the
survivors in order, and re-expand (`expand_helicities` already took the combo list;
the pruned program *is* the expansion of the surviving subset). `eval_m2` unchanged.

Two measured facts anchor the threshold and the frame contract
(`helicity_contribution_spectrum`, ignored diagnostic):

- The per-combination spectrum is **bimodal**: chirality-forbidden combinations are
  exact `0.0` (massless-spinor structural zeros propagate through the kernels);
  MHV-type zeros (all-plus gluons) cancel *across* diagrams leaving O(ε²) residues
  ≲ 2.6e-31 of the sum; the smallest genuine contribution observed is ~1e-5, with
  a conservative floor ≳1e-12 for doubly mass-suppressed combinations. `1e-24` sits
  mid-gap, and dropped terms are below half an ulp of every partial sum — so the
  pruned sum is **bit-for-bit** the unpruned one (MG's own `LIMHEL=1e-8` would not
  guarantee that).
- Some zeros are **frame-bound, not identities**: `g g > t t~` same-helicity gluons
  with opposite-helicity tops vanish by J_z conservation about the beam axis in the
  partonic CM only — massive-particle helicity is not boost invariant (a z-boost
  raises those combinations from 1e-32 to 3e-3 of the sum). MG prunes them, so its
  (and now our) contract is: **pruned matrix elements take partonic-CM momenta with
  beams along ±z**. The probe set is therefore pure-CM; transverse or longitudinal
  boosted inputs are out of contract on a pruned evaluator.

**Validation**: survivor counts pinned against MG's generated `NHEL` tables for 7
processes (`prune_zero_helicities_matches_madgraph_filter_bitwise`: ee_to_mumu
16→4, ee_to_zh 12→6, uux_to_uux 16→6, gg_to_gg 16→6, gg_to_ttx 16→12, ee_to_wpwm
36→16, 2→4 64→16) plus bitwise pruned-vs-unpruned equality there and — enforced per
reference point — in the 14/14 `validate_helas_mg` gate, which now also times the
pruned evaluator and reports `hels kept/total`. All MG-reported counts agree,
including 2→6 16/256 (uux) and 32/256 (bbx: massive b keeps helicity-flip combos).

**Honest bench** (release `eval_strategies`, criterion median ns/eval; bench now
prunes after compile, matching MG's filtered MATRIX1 like-for-like):

| process | mult | MG | §2.2 | now | gain | now vs MG | hels |
|---|:--:|--:|--:|--:|--:|--:|:--:|
| ee_to_mumu | 2→2 | 283 | 765 | 342 | 2.24× | 1.2× | 4/16 |
| ee_to_ee | 2→2 | 731 | 1,390 | 952 | 1.46× | 1.3× | 6/16 |
| uux_to_uux | 2→2 | 278 | 1,224 | 696 | 1.76× | 2.5× | 6/16 |
| gg_to_gg | 2→2 | 949 | 6,765 | 3,341 | 2.02× | 3.5× | 6/16 |
| ee_to_mumua | 2→3 | 1,438 | 5,824 | 2,393 | 2.43× | 1.7× | 8/32 |
| ee_to_mumu_tata_qcd0 | 2→4 | 6,337 | 27,908 | 10,965 | 2.55× | 1.7× | 16/64 |
| uux_to_ccx_emmm_qcd0 | 2→6 | 97,172 | 2,429,125 | 240,925 | 10.1× | 2.5× | 16/256 |

The vs-MG gap narrows from **1.9×–25×** to **1.2×–3.5×**; the 2→6 collapse (25× →
2.5×) is the expansion finally being compared against MG on equal combination
counts. The colored 2→2s (uux 2.5×, gg 3.5×) are now the widest gaps.

**Deferred**: per-(hel,diagram) `ZEROAMP` skipping inside surviving combinations
(MG's second filter layer) — would need probed-zero node elimination in the
expanded arena; unmeasured headroom, likely small next to the combination filter.

### 2.4 Cross-platform rerun kit (vs-MG comparison)

Every vs-MG ratio in this note was measured on one Apple-Silicon host (M3 Max,
NEON). The two sides need not scale together across microarchitectures: MG's
MATRIX1 is straight-line gfortran `-O3` Fortran that auto-vectorizers and wide
x86 FMA units can feed directly, while our indexed-arena interpreter does not
auto-vectorize (note 18 §5), so on AVX-512 silicon the gap could plausibly widen
— the recorded 1.2×–3.5× is a per-platform measurement, not a constant. To rerun
the comparison on another box:

1. **Rebuild the MG side natively**: `pixi run -e madgraph generate-amplitude`.
   The checked-in `mg_*.so` f2py modules and `output/mg_timings.json` are
   host-specific artifacts of the recording machine; the ratios are meaningless
   until both are regenerated on the target host (slow — chains the MG5
   `build-diagrams` launch on first run).
2. **Correctness gate before any timing claim**:
   `pixi run --skip-deps -e madgraph validate-helas-mg` must pass 14/14. A
   REL_TOL failure on new silicon is a finding, not noise — record it.
3. **`scripts/mg_perf_compare.sh`** runs the honest bench (the
   `eval_m2/forward/*` criterion rows only) and joins the medians against
   `mg_timings.json`, printing per-process ns/eval, the vg/MG ratio column, the
   geomean, and a host fingerprint (CPU, rustc, RUSTFLAGS, git HEAD, MG-timing
   mtime); the same lands as markdown + TSV in `target/mg-perf/` for banking
   here. `--skip-bench` re-joins existing criterion results (each row prints
   its measurement date, so stale joins are visible).
4. **Codegen fairness**: the recorded tables use default codegen on both sides
   (rustc default `target-cpu`, f2py's default gfortran `-O3`, no
   `-march=native` anywhere). Keep it that way, or raise both sides together —
   `RUSTFLAGS="-C target-cpu=native"` *and* `-march=native` in
   `build_amplitude.sh`'s `--f77flags` — never one side only. The SIMD
   lane-width question (`lanes{N}` rows) is the separate AVX-512 kit
   (note 18 §5, `scripts/dump_lane_asm.sh`).
5. **Reading the result**: compare the *ratio* column against the recorded
   tables, never absolute ns (clocks differ). The Rust side carries the ±2–3%
   criterion/AST-seed noise floor; the MG side is a single warm-up + one-shot
   batch, not a rigorous benchmark — treat ratio shifts under ~10% as noise.
   The bench's pruning state must match the table you compare against
   (unpruned §2.2 vs pruned §2.3 — the fingerprint's git HEAD pins which).
   Bank the emitted table + fingerprint in this section.

## 3. Track 2 — `rooting-exploration` (throwaway, branch `explore/rooting`)

Goal: quantify how much post-CSE node count depends on rooting choice, cheaply, before
investing in the e-graph formulation. Code quality is exploration-grade; the branch is
committed and kept for posterity, not merged. Results (tables) get appended to this
note on `main` in a docs commit.

- **Metrics harness:** per process over `MG_VALIDATED_PROCESSES`: post-CSE node count
  and a slot-cost-weighted variant (per-op weights ≈ output-slot bytes). Also compute
  the **rooting floor**: distinct (edge, direction) currents deduped across diagrams —
  the best any rooting strategy can reach.
- **Variants:** (0) baseline `VtxIdx(0)`; (1) canonical heuristics — anchor at the
  vertex adjacent to the lowest-index external leg; root at the vertex seeing the
  *most* contributing external momenta by momentum flow; and the *fewest* (both
  directions of that intuition are plausible — measure, don't argue); (2) greedy
  iterative — per diagram try all rootings against the cumulatively-interned arena,
  keep the min-new-nodes one; try both diagram orders (as-generated, largest-first).
- **Correctness:** every variant runs the full `validate_helas_mg` net (REL_TOL).
  Side benefit: alternate rootings may incidentally cover `MetricNegI` (the
  `validation-sprint` gap).
- **Decision output:** headroom number → (a) if greedy wins big, promote a production
  greedy-rooting pass (small, non-egraph — slots into `compile_diagram_ast`/`lower`)
  into Track 1's scope; (b) informs the go/no-go on Track 3 (the e-graph re-rooting
  formulation only makes sense if the headroom is real *and* greedy leaves a gap).

### 3.1 Results (branch `explore/rooting` @ `9bb8e14`; full tables in `rooting-study-results.md`)

Done 2026-07-13. Per-diagram root override (`root_diagram::set_root_override`, test-only;
production stays `const VtxIdx(0)`, byte-identical), 6 variants × 14 processes, each under
the full `validate_helas_mg` REL_TOL gate. Σ over the 14 processes:

| variant | Σ nodes | vs base | Σ weighted B | vs base | gate |
|---|--:|--:|--:|--:|---|
| baseline `VtxIdx(0)` | 9111 | — | 534976 | — | PASS 14/14 |
| canon: lowest-leg anchor | 9111 | 0% | 534976 | 0% | PASS 14/14 |
| canon: most ext legs | 10564 | +15.9% | 673824 | +25.9% | FAIL 4/14 |
| canon: fewest ext legs | 7287 | −20.0% | 361072 | −32.5% | FAIL 5/14 |
| greedy: as-generated | 7200 | −21.0% | 354080 | −33.8% | FAIL 5/14 |
| greedy: largest-first | 7200 | −21.0% | 354080 | −33.8% | FAIL 5/14 |

**Two findings, both load-bearing for Track 3:**

1. **The headroom is real but small-lever:** greedy cuts −21% nodes / −34% slot traffic
   vs baseline; the cheap "fewest ext legs" canonical heuristic captures nearly all of it
   (−20%), so greedy's per-diagram trial machinery buys only ~1% over a one-line heuristic.
   "Most ext legs" moves the wrong way (+16% — central high-degree roots duplicate
   currents). Diagram order (as-generated vs largest-first) makes no difference. The
   deduped `(edge,direction)` "floor" overcounts (both directions of every edge, ~3× the
   currents any single rooting realizes) — not a reachable target; the honest share metric
   is `#Propagate` vs the no-share `sum_edges` bound, where baseline already shares heavily
   (642/2895) and greedy pushes to 275/2895. `lowest-leg anchor` reproduces baseline
   exactly — feyngraph's `VtxIdx(0)` *is* the lowest-leg-anchored vertex, i.e. the status
   quo is already the best zero-cost canonical choice.

2. **⚠️ The reductions are currently UNREALIZABLE — silent orientation-dependence in the
   rooting primitives.** Every node-reducing rooting *silently corrupts the amplitude*
   (max_rel up to **1.7e+3**, 50/50 points wrong — gross wrong values, not benign
   phase/reassociation) on `e+e-→W+W-`, `e+e-→τ+τ-H`, `e+e-→μμττ`, and both 2→6 QCD=0
   processes. These compile and evaluate with no panic and no missing-op error: momentum
   routing (`mul_apply` bra-add/ket-subtract), Lorentz-output rooting, and the fermion-spine
   sign are only *validated* for the `VtxIdx(0)` orientation feyngraph happens to emit, and
   are **not invariant under edge reversal**. This is the same class of bug as the
   `gg_to_gg` VVVV phase (an unexercised branch drifting out of sync) — see the
   `validation-sprint` "branch-level coverage" backlog item. Not a live bug (production is
   `VtxIdx(0)`, gate 14/14), but a hard prerequisite for any rooting change.

**Decisions:**
- **(a) Do NOT promote a production greedy/canonical rooting pass into Track 1 now.** The
  headroom is real but blocked on first making rooting **orientation-independent** — a
  correctness fix, not a perf pass. And the realizable win over the free `lowest-leg`
  status quo is only ~21%, small next to the slot-traffic wins A3/A4 target.
- **(b) Track 3 e-graph re-rooting rule family: conditional GO, correctness-first.** The
  payoff exists (−21%/−34% vs the greedy oracle), so it *could* justify DAG-cost
  extraction — but the propagator-commute + per-vertex-rotation rewrites (§1.3) **cannot be
  assumed correctness-preserving** with today's primitives. The soundness fix — a
  `rooting-soundness` spike, first test: assert *all V rootings* of every diagram pass the
  gate (the `set_root_override` hook is ready for exactly this fuzz) — is the real
  prerequisite, ahead of building the re-rooting extractor. The *chiral-decomposition* rule
  family (M3) is unaffected — it does not re-root.
- No new op coverage: the failures are wrong values, not `MetricNegI`/`IdentityAmp` hits,
  so the `KNOWN_UNCOVERED` gap is untouched.

## 4. Track 3 — `dag-extraction` investigation

What it takes to extract with a **DAG (sharing-aware) cost** from egglog 2.0, since
the crate doesn't provide it (§1.2). The investigation is complete; the decision
record is §4.1. All extractor code lives in `helas/eval/egraph.rs`.

- **M1 — e-graph enumeration (done).** `enumerate(&Ast<Sym>) -> DagEGraph` goes through
  egglog's supported `EGraph::serialize` export (the same backend-canonicalized path
  its GraphViz/JSON tooling uses) and translates the serialized view into owned
  structures (`DagEGraph`/`EClass`/`ENode`/`Payload`): e-classes with their e-nodes,
  each child node-id edge resolved to the e-class it belongs to, primitive leaf
  payloads recovered. The function-table (`function_to_dag`) alternative routes through
  the tree-cost extractor per row and cannot recover raw child edges, so `serialize` is
  the seam.
- **M2 — greedy DAG extractor (done).** `trait CostModel` (per-op `node_cost`),
  `enum CostKind{Dag,Tree}`, and `extract(&DagEGraph, &dyn CostModel, CostKind)` — an
  extraction-gym `faster-greedy-dag`-style worklist fixpoint: each e-class takes its
  min-cost node whose children are all costed, a candidate's DAG cost is its op cost
  plus the cost of the *union* of its children's chosen descendant sets (shared classes
  counted once), and a class's parents are re-examined when its cost improves.
  `decode_extraction` walks the chosen e-nodes back into an `Ast<Sym>`, memoized on
  e-class id so shared classes become shared arena nodes. Cost models: `SlotTrafficCost`
  (per-op ≈ output-slot bytes) and `UnitCost`. Sanity gate met: on the rule-free
  round-trip the DAG extractor reproduces the input DAG byte-for-byte with the CSE node
  count intact, over the dev processes (ungated) and all of `MG_VALIDATED_PROCESSES`
  (behind `extended-validation`). ILP (`good-lp`) was held in reserve as a quality
  oracle; §4.1 promotes it to a prerequisite.
- **M3 — chiral-decomposition sharing demo (no-go, nothing committed).** The demo
  target was `FfvVout(a,b,gl,gr) → gl·J_L(a,b) + gr·J_R(a,b)` on `e+ e- > mu+ mu-`,
  expecting DAG-cost extraction to pick the shared `J_L`/`J_R` form where tree-cost
  picks the fused current. It does not, for two independent structural reasons (§4.1);
  the rule was built as throwaway instrumentation, measured, and reverted.
- **M4 — write-up + go/no-go (this section).**

### 4.1 Go/no-go for `egraph-rewrite` sharing-rule integration — **NO-GO under current scope**

Sharing-payoff rewrites (chiral decomposition, coupling factoring, re-rooting) cannot
be demonstrated, let alone shipped, with the M2 greedy extractor + `SlotTrafficCost`.
Two findings, both structural, not tuning:

1. **Greedy cannot realize cross-diagram sharing, under any cost model.** M2's greedy
   decides each e-class independently, taking the locally-min-cost node. Chiral sharing
   is only cheaper *globally* — both diagrams must co-commit so `J_L`/`J_R` are the same
   e-classes; at any single current class in isolation the split form is strictly more
   expensive than the fused one. Greedy therefore never takes the locally-worse move
   that the global optimum requires. Measured on `e+ e- > mu+ mu-`: greedy picked the
   fused form at **0 of 4** decomposable current classes, DAG cost unchanged. This is
   the direct answer to the greedy-vs-ILP question the milestone posed: **greedy DAG
   extraction is insufficient for any sharing-*payoff* rewrite; a global/ILP extractor
   is required.**
2. **`SlotTrafficCost` makes the rewrite a net loss even at the global optimum.**
   Forcing the split at every decomposable class (the best a global extractor could do)
   gave root_cost **2816 vs 2048–2144 fused** — worse. Slot-traffic charges a
   pure-chiral half-current the same output-slot bytes (~96 B) as a full current, so
   splitting only adds recombination scaffolding with no offsetting saving. The "split
   wins when the pure current is shared ≥2×" premise (§1.4) holds only under a
   **compute-aware / work cost model** where a chiral half-current is materially cheaper
   to produce than a full one. Under a prototype `WorkCost` the optimum is real: split
   **935 vs 1080 fused** (~13%) — but finding #1 still blocks greedy from reaching it.

3. **`e+ e- > mu+ mu-` is the marginal case.** Exactly two consumers (γ, Z) share each
   spinor pair, so even under `WorkCost` the win is only ~13%. A process with **≥3
   consumers** of the same pure current would separate the shared and fused forms
   decisively and is the right demo target for a future attempt.

**Path to yes (all three required before re-attempting the demo):**
- **(a) A global / ILP extractor.** The ILP oracle note 15 held "in reserve" (M2) must
  be promoted to a *prerequisite* — greedy provably cannot express the co-commit.
- **(b) A compute-aware cost model.** `SlotTrafficCost` cannot see the payoff; a
  `WorkCost` that charges a pure-chiral half-current less than a full current is needed.
  This intersects the §1.6 "static output-type analysis" work (A1 output types feed the
  per-op work estimate), so it is not free-standing.
- **(c) A ≥3-consumer demo process** so the payoff is non-marginal.

Until (a)+(b)+(c) exist, the sharing half of `egraph-rewrite` stays blocked; the
re-rooting rule family additionally waits on the `rooting-soundness` fix (§3.1). M1/M2
stand as the reusable DAG-cost extraction substrate for when the prerequisites land —
they are correct and gated, just not sufficient on their own.

### 4.2 Known issue — run-to-run AST cost variance is **upstream of `egraph.rs`**

`enumerate()` → `extract()` is **deterministic relative to its input**: on the rule-free
graph every e-class holds exactly one e-node (no extraction ties), the greedy choice is
forced, and the identity gate (`extract_dev_processes_identity`) reproduces the input
DAG byte-for-byte without flaking. What varies run-to-run is the **lowered AST itself**:
`compile_diagram_ast` + `lower` emit a 37- or 38-subterm AST for `e+ e- > mu+ mu-`
depending on the process's hash seed (constant within one process, differing across
process invocations), which the extractor then faithfully reproduces — so the DAG cost
tracks it exactly (37↔2048, 38↔2144 under `SlotTrafficCost`). The origin is a
`HashSet`/`HashMap` iteration in the diagram-lowering path (`root_diagram`/`lower`)
selecting between structurally-equivalent node emissions, which changes what CSE can
merge by ±1 node. It is **correctness-neutral** — both ASTs evaluate to the same |M|²
and pass `validate_helas_mg` — a missed-CSE reproducibility wart, not a wrong value, and
**not in the extractor**. Consequences: (i) M3's attribution of the variance to the
"serialization/greedy path" is incorrect — the extractor is not the source; (ii) any
DAG-cost comparison used as an extraction oracle must compile the AST **once** and reuse
it (the extract tests already do), or the upstream lowering order must be pinned; (iii)
the fix belongs to the `eval-layout` / lowering owners (a `HashSet`→`BTreeSet` or
sorted-iteration change in `root_diagram`/`lower`), not to `egraph.rs`, and is out of
Track 3's scope.

## 5. Consequences for `egraph-rewrite`

- Constant folding **moves out** (Track 1, A2 — no rules needed).
- All sharing rules — coupling regrouping, chiral decomposition, propagator
  linearity, re-rooting — are **NO-GO under current scope** (§4.1): Track 3 shipped a
  correct DAG-cost extraction substrate (M1/M2) but proved greedy + `SlotTrafficCost`
  cannot realize a sharing payoff. Reviving them requires all three of a global/ILP
  extractor, a compute-aware `WorkCost` model, and a ≥3-consumer demo process
  (re-rooting additionally waits on the `rooting-soundness` fix, §3.1). Informed by
  Track 2's headroom numbers.
- Schema updates decided (§1.6): per-kind leaf sorts, `ExtLegInfo` wrapper,
  `ScalarConst`/`ScalarWf` split, typed constructor slots. Apply when Track 3 touches
  the schema; keep `schema_covers_every_op` and the round-trip suite as the guard.
- The A1 type/constness analysis doubles as the typed-schema encoder's source of
  truth.

## References

- R. Frederix et al., "Speeding up MadGraph5_aMC@NLO", EPJC 81:435 (2021),
  arXiv:2102.00773 — helicity recycling.
- J. Alwall et al., "MadGraph 5: Going Beyond", arXiv:1106.0522 — diagram-level
  wavefunction reuse.
- egglog 2.0 crate source, `src/extract.rs` — `TreeAdditiveCostModel` / `CostModel`
  (tree-shaped fold).
- egg community "extraction gym" (github.com/egraphs-good/extraction-gym) — greedy
  DAG / ILP extractor implementations to crib from.
- `research/notes/14-egglog-notes.md` — egglog language summary.
