# `eval-perf-2` — second evaluator performance sprint (plan)

Follows the eval performance program (note 15, closed 2026-07-17, gap to MG
1.2×–3.5×) and the two validation passes that hardened the gate. Runs against the
same figure of merit: release `eval_strategies` median ns/eval per process, all
sessions behind the 14-process `validate_helas_mg` bit-exact net.

Two of the four sessions are unblocked *because* of `validation-2` V5
(`rooting-soundness`, note 19 §V5): re-rooting is now amplitude-invariant
(`all_rootings_preserve_amplitude`, 0/133 re-rootings) and `REL_TOL` relaxed to
1e-10, which together turn the rooting-CSE headroom (S4) from "silently corrupts
the amplitude" into a realizable win.

## Measured motivation for S1 (histogrammed 2026-07-21)

Instruction census over all 14 pruned, helicity-expanded programs (the real hot
path): **`Mul` is 57.6% of all instructions** (74,379 / 129,045), and **every one
is binary** — constant-folding collapses `Mul(Coupling,Coeff)` into single
constants and `flatten_adds` leaves Muls alone, so no eval-time Mul is n-ary or
carries two currents. Operand-class breakdown of the 74,379:

| pattern | share | typed variant |
|---|--:|---|
| `real × scalar` → scalar | 62.1% | `MulScalarR { s, r }` |
| `scalar × scalar` → scalar | 23.8% | `MulScalarC { a, b }` |
| `scalar × vector` | 4.4% | `ScaleVecC { v, s }` |
| `real × vector` | 4.3% | `ScaleVecR { v, r }` |
| `× fout` (scalar/real) | 3.5% | `ScaleFoutC/R` |
| `× fin` (scalar/real) | 1.8% | `ScaleFinC/R` |

Today's generic `exec_mul` pays, per binary node: an operand loop, a 5-arm class
match per operand, a `MulCurrent` enum build, and **two redundant identity
multiplies** (`C(1,0)·scalar`, then `cplx_acc·real_acc`). The `real ×` variants
(66% of Muls) also promote a real to `C(r,0)` and do a full complex multiply
(4 mul + 2 add) where a real-scale (2 mul) suffices.

---

## Sessions

### S1 — `mul-split` (release-perf, dispatch first)

**Change.** In `Program::build` (`layout.rs`), peephole every binary `Mul` into one
of eight typed `Instr` variants by inspecting its two operands' storage classes:

- scalar producers: `MulScalarC { a, b }`, `MulScalarR { s, r }`
- current scalers: `ScaleVecC/R`, `ScaleFinC/R`, `ScaleFoutC/R` (operand fields
  `{ current, scale }`, scale read from the scalar or real arena)

Each runtime arm is a single field read + one arithmetic op — no loop, no
`MulCurrent` enum, no identity multiplies; the `…R` arms use a real-scale of the
complex value. Dispatch stays a jump table on the `Instr` discriminant, so the
extra variants cost nothing (this is why we split the op rather than bake a
real/complex bool leaf).

**No fallback — total enumeration.** Binary-Mul is a production invariant, so the 8
variants are exhaustive and `Instr::Mul { start, len }` + `exec_mul` + `MulCurrent`
are **deleted**, not kept as a cold path. `Program::build` asserts the invariant
(arity == 2, ≤ 1 non-scalar operand, no `real × real`) and emits exactly one variant;
an invariant violation is a hard panic, not a silent slow path.

Why the invariant holds (not just the histogram's 0% non-binary): every Mul-emitting
site goes through `reduce_balanced(_, Op::Mul, _)` (balanced binary trees; returns the
lone node for a 1-element list, so never arity-1) or an explicit binary `b.add`;
`flatten_adds` deliberately skips Muls; `fold` can only collapse a binary Mul to a
leaf or leave it binary; CSE and `expand_helicities` preserve arity. `real × real`
cannot reach `build` because every real-class node is a card constant, so any all-real
product is folded away. The only theoretical n-ary source is a future egraph
extraction — and the existing "egg wants binary" lowering contract already requires
re-balancing to binary before `build`, which this assertion now documents and
enforces. The test-only generic evaluator (`run_forward_slot`/`apply`/`mul_apply`,
`#[cfg(test)]`) stays variadic and untouched — keeping the reference oracle
structurally distinct from the production stream preserves its independence.

**Coverage.** All 8 variants — the current-scaling arms are 14% of Muls but each drops
a full complex-mul, so they are worth the variants; and the enumeration must be total
to drop the fallback.

**Gate + measure.** `validate_helas_mg` bit-for-bit (arithmetic is byte-identical —
same operations, fewer of them); `eval_strategies` before/after table with host
fingerprint. This is the long-standing "`C<F>`-vs-`F` multiply peepholes" backlog
item, now sized.

### S2 — `dag-validate-once` (dev-ergonomics / honest timing; depends on S1)

`cross_check_typed`/`cross_check_node` run per-node-per-point under
`debug_assertions`/`extended-validation` (compiled out of release, so **no release
cost**). After S1's total typed enumeration, the output-type and constness assertions are
*provably* tautological: each typed `Instr` writes a fixed arena by construction and
Mul has no generic arm left that could produce an unexpected class. The only
surviving real check —
momentum-routing / pool-index consistency — is a property of the compiled DAG,
invariant across phase-space points.

**Change.** Delete the tautological type/const assertions. Lift the momentum-routing
check out of the forward scan into a **one-shot** DAG validation: one full pass with
cross-checks over a single probe point, run once per bound amplitude (first `eval_m2`
call, or an explicit `debug_validate` at bind — implementer's call), gated by the
same `cfg`. Removes the per-node hook from the hot loop.

**Payoff.** Faster debug tests, and it retires the note-15 "`validate_helas_mg`
printed timings run ~4–5× hot because per-node cross-checks compile into the loop"
caveat — those timings become honest. Not a release win.

**Gate.** Full unit suite + both MG gate suites under `extended-validation` (the
one-shot pass must still catch a deliberately corrupted mom-table index — add a
negative test).

### S3 — `zeroamp-skip` (release-perf; depends on S1)

MG's second helicity-filter layer (note 15 §2.3): inside a *surviving* helicity
combination, individual diagram amplitudes can still be identically zero for that
combination. `prune_zero_helicities` removed whole zero combinations; this reclaims
zero *diagrams* within kept combinations.

**Change.** A probed-zero **node-elimination** pass over the helicity-expanded arena
(`Folded::expand_helicities` output): reuse the existing numeric-zero probe (same 10
deterministic generic partonic-CM points, Schwartz–Zippel argument, note 15 §2.3) to
mark nodes that are exactly/sub-ulp zero at every probe point, then dead-code
eliminate nodes private to a zero diagram and fold them out of their consumer
`Add`s. This is a rewrite pass on the expanded arena, **not** a filtered
re-expansion.

**Bit-for-bit.** Only exact zeros and sub-half-ulp residues are dropped (same
threshold discipline as the combination filter), so the pruned sum stays byte-equal.

**Headroom.** Unmeasured, expected small — the combination filter already removed
most zeros, and elimination only reclaims nodes reachable solely through a zero
diagram. **First deliverable is the measurement** (node-count reduction per process
+ `eval_strategies` delta); if the win is within `eval_strategies` noise (±2–3%),
land the measurement and stop rather than carrying complexity for nothing.

### S4 — `rooting-cse` (release-perf; depends on S1; unblocked by V5)

The rooting study (`rooting-study-results.md`) measured **−21% post-CSE nodes /
−34% slot-weighted traffic** from re-rooting diagrams away from feyngraph's
`VtxIdx(0)`, but every node-reducing rooting **failed the gate** — re-rooting
silently corrupted multi-boson and ≥6-point amplitudes. `validation-2` V5 fixed
exactly that root cause (all rooting-convention signs lifted to `fermi_sign`,
`all_rootings_preserve_amplitude` green, `REL_TOL` 1e-10 absorbs the benign
reassociation residues the study saw at ~1e-11). **The blocker is gone.**

**Spike, then decide.** This is an exploration, not a committed productionization:

1. **Re-run the study's gate under the current tree** (V5 + 1e-10 tolerance) to
   confirm the −20%/−21% rootings now pass 14/14 — both the benign-reassociation
   variants (`most ext legs` on 6/8-point) and the previously-gross-wrong ones
   (`fewest ext legs`, greedy). The study harness (`rooting_study.rs`, throwaway on
   `explore/rooting`) + the `set_root_override` hook are the starting point.
2. If sound, **productionize a rooting-choice pass**. `fewest external legs` is the
   simplest canonical rule and captured nearly all the headroom (−20% nodes);
   greedy (−21%) needs iterative cumulative-arena machinery for a marginal extra 1%
   — prefer the canonical rule unless the bench says otherwise.
3. **Measure what matters: ns/eval, not node count.** The study counted nodes in the
   *pre-expansion* per-diagram arena; the payoff must be confirmed on the
   helicity-expanded, pruned program through `eval_strategies`. Node/traffic
   reduction is a proxy, not the deliverable.

**Caveats.** Rootings reassociate momentum sums, so this path is **not** bit-for-bit
against the current baseline — gate at `REL_TOL` 1e-10 (the V5 floor) via
`validate_helas_mg` + `all_rootings_preserve_amplitude`, not byte-equality. Sequence
S4 last: it is the highest-risk/highest-variance session and benefits from S1's
instruction census already in place.

---

## Sequencing

`S1` → then `S2`, `S3`, `S4` in any order (all depend only on S1's typed stream and
are mutually independent). S1 and S2 are low-risk; S3 is measurement-gated; S4 is the
exploratory spike. Dispatch S1 first, measure, then decide the rest from its result.

## Deferred perf backlog (recorded, not this sprint)

- **`feyngraph-perf`** — `workspace.rs:L122` ~340M `HashMap` allocations
  (`AssignWorkspace::assign` `.counts()`); a `feyngraph` submodule change, own
  session. (TODO.md `feyngraph-perf`.)
- **CF-factoring across combinations** — shelved 2026-07-17 (note 15 §2.2): rebalances
  arithmetic rather than shrinking it and reordering the |M|² sum breaks bit-for-bit.
- **`egraph-rewrite`** — blocked (notes 14 + 15 §4–5): remaining rule families are
  sharing rewrites invisible to tree-cost extraction; needs a global/ILP extractor +
  compute-aware `WorkCost` + a ≥3-consumer demo process. S4's re-rooting rule family
  is one of these — S4 tests the *manual* rooting win that a future extractor would
  have to beat.
- **`mg-single-helicity-bench`** — rides with `event-output-lhef` (unexpanded
  single-helicity becomes the hot path there; the fair comparison needs an MG
  single-config timing, a reference-data change).
- **`generate-stream` Part B** (lazy `generate_*` iterator).
