# 15 — Post-CSE Evaluator Optimization: Research Findings & 3-Track Plan

**Status:** Plan of record (2026-07-11). Successor planning to the closed
`performance-sprint` (see `TODO.md` timing table — the reference baseline). Companion
to `research/notes/14-egglog-notes.md` (egglog language reference).

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
  worth pulling in while the timing rig is warm.

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
the crate doesn't provide it (§1.2). Milestones:

- **M1 — e-graph enumeration.** Find the supported path to enumerate e-classes and
  e-nodes from `egglog::EGraph` in Rust (function-table iteration / serialization);
  the extraction seam in `egraph.rs::roundtrip` is where it plugs in.
- **M2 — greedy DAG extractor.** Implement extraction-gym-style greedy DAG extraction
  (à la `faster-greedy-dag`) with per-op costs ≈ slot traffic. Sanity gate: on the
  rule-free round-trip it must reproduce the input DAG (current CSE node counts) over
  the validated suite. Keep ILP (e.g. `good-lp`) in reserve for small graphs / as an
  oracle for greedy quality.
- **M3 — first sharing-rule demo.** Chiral decomposition (§1.4) on `e+ e- > mu+ mu-`:
  show DAG-cost extraction picks the shared `J_L`/`J_R` form and tree-cost extraction
  doesn't. Requires the pure-chiral kernel ops (or unit-coupling `Ffv*` forms) and the
  §1.6 schema updates.
- **M4 — write-up + go/no-go** for wiring into `egraph-rewrite` proper: extractor
  performance on the 2→6 ASTs, greedy-vs-ILP quality, and the integration design.

## 5. Consequences for `egraph-rewrite`

- Constant folding **moves out** (Track 1, A2 — no rules needed).
- All sharing rules — coupling regrouping, chiral decomposition, propagator
  linearity, re-rooting — are **blocked on Track 3** (and informed by Track 2's
  headroom numbers).
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
