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

## Sprint outcome — ✅ CLOSED 2026-07-21 (all four sessions merged to `main`, HEAD `6ab25f1`)

All four sessions landed. **Cumulative** `eval_strategies/forward` speedup, measured
clean end-to-end (pre-sprint `86b3e0a` → post-S4 `6ab25f1`, release, Apple M3 Max,
criterion median; times are per 16-eval batch):

| process | before | after | speedup | Δ |
|---|--:|--:|--:|--:|
| ee_to_mumu (2→2) | 5.51 µs | 4.66 µs | **1.18×** | −16% |
| ee_to_ee (2→2) | 15.56 µs | 8.50 µs | **1.83×** | −45% |
| uux_to_uux (2→2, NCOLOR=2) | 11.24 µs | 6.40 µs | **1.76×** | −43% |
| gg_to_gg (2→2, NCOLOR=6) | 54.78 µs | 25.07 µs | **2.19×** | −54% |
| ee_to_mumua (2→3) | 38.72 µs | 24.26 µs | **1.60×** | −37% |
| ee_to_mumu_tata (2→4) | 175.9 µs | 114.2 µs | **1.54×** | −35% |
| uux_to_ccx_emmm (2→6) | 3901 µs | 2194 µs | **1.78×** | −44% |

Every benchmarked process improved (1.18×–2.19×), on top of the earlier eval program's
8.6×–110× → 1.2×–3.5× vs MG — the fresh vs-MG measurement (§ below, 2026-07-28) puts
the residual gap at **0.72×–1.69×, geomean 1.24×**. Session contributions:

- **S1 `mul-split`** (`95fca7f`): the broad base win — helped every process (−13…−44%),
  biggest on the Mul-heavy `gg_to_gg`. Bit-exact.
- **S2 `dag-validate-once`** (`b53faf6`): no release change (validation is `cfg`'d out);
  retired the "extended-validation timings run ~4–5× hot" caveat (3.1×–5.4× faster there).
- **S3 `zeroamp-skip`** (`c9f826d`): the colored-2→2 win (ee_to_ee, uux, gg_to_gg) —
  beat its "likely small" prior. Bit-exact.
- **S4 `rooting-cse`** (`6ab25f1`): the multi-leg win (2→3/2→4/2→6, −18…−26%), neutral on
  2→2 (their vertices tie on external-leg count, so no re-rooting). Reassociating (values
  shift) but **`validate_helas_mg` stayed at its tight 1e-12** — the plan's feared
  "1e-14 → 1e-10 agreement cost" did **not** materialize (shorter/more-shared current
  chains actually reduced FP drift; several processes improved to ~1e-14). Only the
  `rooting_soundness.rs` all-rootings gate uses 1e-10.

Not pursued: greedy rooting (marginal ~1% over the canonical `fewest-ext-legs` rule).
Deferred backlog below unchanged.

### Fresh vs-MG measurement (2026-07-28, `scripts/mg_perf_compare.sh`)

The direct joint rerun the close-out deferred. Fingerprint: Darwin arm64
(Apple M3 Max), rustc 1.94.1, RUSTFLAGS unset (default codegen, both sides),
`mg_timings.json` of 2026-07-21 (same host, MG side unchanged since), vibegraph
at `405d18b` (+doc-only edits). All **14 gated processes**, `eval_m2/forward`
criterion medians:

| process | MG ns/eval | vg ns/eval | vg/MG |
|---|--:|--:|--:|
| ee_to_zh | 206 | 242 | 1.17× |
| uux_to_uux | 278 | 399 | 1.44× |
| ee_to_mumu | 287 | 289 | 1.01× |
| pp_to_ll_qcd0 | 292 | 288 | 0.99× |
| ee_to_ttx | 343 | 480 | 1.40× |
| gg_to_ttx | 655 | 928 | 1.42× |
| ee_to_ee | 724 | 519 | **0.72×** |
| ee_to_wpwm | 756 | 1,199 | 1.59× |
| ee_to_tatah | 837 | 946 | 1.13× |
| gg_to_gg | 941 | 1,550 | 1.65× |
| ee_to_mumua | 1,443 | 1,514 | 1.05× |
| ee_to_mumu_tata_qcd0 | 6,260 | 6,982 | 1.12× |
| uux_to_ccx_emmm_qcd0 | 97,107 | 135,962 | 1.40× |
| bbx_to_ccx_emmm_qcd0 | 135,073 | 227,831 | **1.69×** |

**Geomean 1.24×, range 0.72×–1.69×.** Composing this note's cumulative speedups
with note 15 §2.3's per-process ratios predicts the measurement closely (ee_to_ee
0.73 vs 0.72, gg_to_gg 1.65 vs 1.65, uux_to_uux 1.44 vs 1.44) — the two passes'
benches stayed mutually consistent. `ee_to_ee` now beats MG outright; the widest
gaps are no longer a colored-2→2 story alone (`ee_to_wpwm` 1.59×, massive-b 2→6
1.69×). Per-platform caveat from note 15 §2.4 still applies.

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

**Outcome (`c9f826d`, DONE — beat the prior).** The "expected small" guess was wrong:
`forward` improved **−17%…−32%** on exactly the colored 2→2s that were the widest
gaps vs MG (ee_to_ee −32%, uux_to_uux −27%, gg_to_gg −17%), neutral elsewhere (no
regressions). Node reductions: ee_to_ee 165→109, uux 138→98, gg_to_gg 775→625,
gg_to_ttx 447→277 (−38%), ee_to_wpwm −10%, bbx 2→6 −3%. Implemented as
`Folded::prune_zero_scalar_operands` (fold.rs) run by `prune_zero_helicities` after
the combination filter, detecting exact zeros at the **scalar `Add`-operand** level
(sufficient — dropping a zero scalar diagram-amplitude operand DCEs its entire private
vector/spinor subtree without repr-internal access) over the shared
`generic_probe_points`. Only *exact* zeros qualify (MHV residues are non-zero at this
level, so the sub-ulp case never arises); each modified `Add` is re-folded over its
survivors at every probe point and **reverted unless byte-identical**, closing the
`−0.0`+`+0.0` sign hazard. Bit-exact 14/14 (independently re-run). Getter
`AmplitudeEvaluator::zeroamp_node_reduction()` exposes before/after counts.

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

**Outcome (`6ab25f1`, KEEP).** Production `choose_root` now returns the vertex with
the **fewest directly-attached external legs** (ties → lowest vertex index):
`canonical_root` in `root_diagram.rs`, wired into both production and test `choose_root`.
Soundness confirmed (`all_rootings_preserve_amplitude` 0/133 at 1e-10, both before and
after). `forward` −19% (2→3) / −18% (2→4) / −26% (2→6); 2→2 neutral (vertices tie →
`canonical_root == VtxIdx(0)`, no re-rooting). The convention signs still read off the
canonical `VtxIdx(0)` tree (V5), which is what keeps re-rooting sound. **Key close-out
finding: `validate_helas_mg` stayed at 1e-12** — values shift (genuine reassociation)
but agreement is unchanged-or-better (ee_to_mumua 3.92e-13 → 1.62e-14), so the plan's
anticipated agreement-loosening tradeoff was a non-issue; `REL_TOL` was **not** relaxed
in the production gate. S3's ZEROAMP pass re-verified intact under the new rooting; the
partonic-CM ±z frame contract untouched. The `egraph-rewrite` backlog's "manual rooting
win a future DAG-cost extractor must beat" bar is now −19…−26% ns/eval on the post-S1/S3
program.

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
