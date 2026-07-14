# Rooting-exploration study — results (branch `explore/rooting`, throwaway)

Measurement study for Track 2 of the post-CSE optimization program (note 15 §1.3, §3):
how much does the post-CSE amplitude node count depend on which vertex each diagram is
rooted at, and is a production greedy/canonical rooting pass worth building?

Harness: `vibegraph-lib/src/helas/eval/rooting_study.rs` (test-only), driven by the
`root_diagram::set_root_override` hook. Reproduce:

```
RUST_MIN_STACK=134217728 cargo test -p vibegraph-lib --profile profiling \
  --features extended-validation rooting_study::rooting_headroom_study \
  -- --ignored --nocapture --test-threads=1
```

## Definitions

- **nodes** — reachable nodes of `optimize(lower_flows(basis, evals))` (the real
  color-aware post-CSE arena), under the given rooting. Cross-flow CSE is captured for
  the multi-flow processes because the root is chosen **per diagram** (all color-chains
  of a diagram share one root by construction).
- **weighted (B)** — Σ over reachable nodes of the output-slot byte weight (note 15 §1.5,
  coarse static map: scalars 16 B, fermion/vector currents 96 B; `Add`/`Mul` inherit
  their non-scalar operand).
- **#prop** — number of `Propagate` nodes = distinct realized off-shell currents after CSE.
- **floor(cur)** — `dedup_currents`: distinct directed-current signatures over all
  diagrams and both directions of every internal edge (note 15's "(edge, direction)
  currents deduped across diagrams"). `sum_edges` = Σ internal edges (realized currents
  with zero sharing); `n_ext` = external wavefunctions.
- **gate** — full `validate_helas_mg` net, all 14 processes, REL_TOL 1e-12. Rootings
  reassociate momentum sums and are never bit-for-bit. `PASS` / `FAIL n/m` (points over
  tolerance). max_rel is the worst relative diff vs MadGraph over the CSV point set.

## Variants

0. **baseline VtxIdx(0)** — production rooting (feyngraph's first vertex).
1. canonical heuristics (pure function of the diagram):
   - **lowest-leg anchor** — root at the vertex holding the lowest-index external leg.
   - **most ext legs** — root at the vertex with the most directly-attached external legs.
   - **fewest ext legs** — root at the vertex with the fewest.
2. greedy iterative — lower diagrams one at a time, each choosing the root that adds the
   fewest new nodes to the cumulative arena; **as-generated** and **largest-first** orders.

## Per-process tables

## `e+ e- > mu+ mu-`  (n_ext=4, n_diagrams=2, n_flows=1)
floor: dedup_currents=4, sum_edges=2, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |
| canon: lowest-leg anchor | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |
| canon: most ext legs | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |
| canon: fewest ext legs | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |
| greedy: as-generated | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |
| greedy: largest-first | 37 | 1552 | 2 | 4 | PASS | 1.1e-14 |

## `u u~ > mu+ mu-`  (n_ext=4, n_diagrams=2, n_flows=1)
floor: dedup_currents=4, sum_edges=2, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |
| canon: lowest-leg anchor | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |
| canon: most ext legs | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |
| canon: fewest ext legs | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |
| greedy: as-generated | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |
| greedy: largest-first | 45 | 1680 | 2 | 4 | PASS | 1.7e-14 |

## `e+ e- > e+ e-`  (n_ext=4, n_diagrams=4, n_flows=1)
floor: dedup_currents=8, sum_edges=4, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |
| canon: lowest-leg anchor | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |
| canon: most ext legs | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |
| canon: fewest ext legs | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |
| greedy: as-generated | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |
| greedy: largest-first | 52 | 2592 | 4 | 8 | PASS | 2.7e-14 |

## `e+ e- > mu+ mu- a`  (n_ext=5, n_diagrams=8, n_flows=1)
floor: dedup_currents=24, sum_edges=16, n_ext=5

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 95 | 5280 | 11 | 24 | PASS | 3.9e-13 |
| canon: lowest-leg anchor | 95 | 5280 | 11 | 24 | PASS | 3.9e-13 |
| canon: most ext legs | 99 | 5504 | 12 | 24 | PASS | 3.9e-13 |
| canon: fewest ext legs | 87 | 4672 | 8 | 24 | PASS | 1.6e-14 |
| greedy: as-generated | 98 | 5488 | 12 | 24 | PASS | 3.9e-13 |
| greedy: largest-first | 98 | 5488 | 12 | 24 | PASS | 3.9e-13 |

## `e+ e- > t t~`  (n_ext=4, n_diagrams=2, n_flows=1)
floor: dedup_currents=4, sum_edges=2, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |
| canon: lowest-leg anchor | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |
| canon: most ext legs | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |
| canon: fewest ext legs | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |
| greedy: as-generated | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |
| greedy: largest-first | 45 | 1680 | 2 | 4 | PASS | 5.0e-15 |

## `e+ e- > W+ W-`  (n_ext=4, n_diagrams=3, n_flows=1)
floor: dedup_currents=6, sum_edges=3, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 77 | 4272 | 3 | 6 | PASS | 4.2e-14 |
| canon: lowest-leg anchor | 77 | 4272 | 3 | 6 | PASS | 4.2e-14 |
| canon: most ext legs | 77 | 4272 | 3 | 6 | PASS | 4.2e-14 |
| canon: fewest ext legs | 77 | 4272 | 3 | 6 | PASS | 4.2e-14 |
| greedy: as-generated | 94 | 3184 | 3 | 6 | **FAIL 50/50** | 1.7e3 |
| greedy: largest-first | 94 | 3184 | 3 | 6 | **FAIL 50/50** | 1.7e3 |

## `e+ e- > Z H`  (n_ext=4, n_diagrams=1, n_flows=1)
floor: dedup_currents=2, sum_edges=1, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |
| canon: lowest-leg anchor | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |
| canon: most ext legs | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |
| canon: fewest ext legs | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |
| greedy: as-generated | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |
| greedy: largest-first | 26 | 1216 | 1 | 2 | PASS | 9.6e-14 |

## `e+ e- > ta+ ta- H`  (n_ext=5, n_diagrams=5, n_flows=1)
floor: dedup_currents=15, sum_edges=10, n_ext=5

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 82 | 4832 | 8 | 15 | PASS | 3.9e-13 |
| canon: lowest-leg anchor | 82 | 4832 | 8 | 15 | PASS | 3.9e-13 |
| canon: most ext legs | 82 | 4832 | 8 | 15 | PASS | 3.9e-13 |
| canon: fewest ext legs | 76 | 4416 | 5 | 15 | **FAIL 50/50** | 3.3e-2 |
| greedy: as-generated | 82 | 5232 | 6 | 15 | **FAIL 50/50** | 2.9e-2 |
| greedy: largest-first | 82 | 5232 | 6 | 15 | **FAIL 50/50** | 2.9e-2 |

## `e+ e- > mu+ mu- ta+ ta- QCD=0`  (n_ext=6, n_diagrams=25, n_flows=1)
floor: dedup_currents=82, sum_edges=75, n_ext=6

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 228 | 13088 | 34 | 82 | PASS | 5.4e-13 |
| canon: lowest-leg anchor | 228 | 13088 | 34 | 82 | PASS | 5.4e-13 |
| canon: most ext legs | 244 | 14624 | 42 | 82 | **FAIL 5/50** | 4.1e-11 |
| canon: fewest ext legs | 198 | 10368 | 19 | 82 | **FAIL 50/50** | 1.2e-2 |
| greedy: as-generated | 204 | 11104 | 23 | 82 | **FAIL 50/50** | 1.2e-2 |
| greedy: largest-first | 204 | 11104 | 23 | 82 | **FAIL 50/50** | 1.2e-2 |

## `u u~ > c c~ e+ e- mu+ mu- QCD=0`  (n_ext=8, n_diagrams=579, n_flows=1)
floor: dedup_currents=2090, sum_edges=2895, n_ext=8

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 3876 | 229376 | 642 | 2090 | PASS | 2.3e-13 |
| canon: lowest-leg anchor | 3876 | 229376 | 642 | 2090 | PASS | 2.3e-13 |
| canon: most ext legs | 4584 | 297344 | 930 | 2090 | **FAIL 1/50** | 2.1e-11 |
| canon: fewest ext legs | 3040 | 149440 | 251 | 2090 | **FAIL 50/50** | 3.7e-1 |
| greedy: as-generated | 2963 | 142128 | 275 | 2090 | **FAIL 50/50** | 3.7e-1 |
| greedy: largest-first | 2963 | 142128 | 275 | 2090 | **FAIL 50/50** | 3.7e-1 |

## `b b~ > c c~ e+ e- mu+ mu- QCD=0`  (n_ext=8, n_diagrams=615, n_flows=1)
floor: dedup_currents=2232, sum_edges=3075, n_ext=8

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 4248 | 256448 | 669 | 2232 | PASS | 6.1e-14 |
| canon: lowest-leg anchor | 4248 | 256448 | 669 | 2232 | PASS | 6.1e-14 |
| canon: most ext legs | 4973 | 325568 | 972 | 2232 | **FAIL 1/50** | 1.8e-12 |
| canon: fewest ext legs | 3304 | 166224 | 264 | 2232 | **FAIL 50/50** | 2.1e-1 |
| greedy: as-generated | 3254 | 165264 | 290 | 2232 | **FAIL 50/50** | 2.1e-1 |
| greedy: largest-first | 3254 | 165264 | 290 | 2232 | **FAIL 50/50** | 2.1e-1 |

## `u u~ > u u~`  (n_ext=4, n_diagrams=2, n_flows=2)
floor: dedup_currents=4, sum_edges=2, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |
| canon: lowest-leg anchor | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |
| canon: most ext legs | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |
| canon: fewest ext legs | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |
| greedy: as-generated | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |
| greedy: largest-first | 41 | 1936 | 2 | 4 | PASS | 5.6e-14 |

## `g g > t t~`  (n_ext=4, n_diagrams=3, n_flows=2)
floor: dedup_currents=6, sum_edges=3, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |
| canon: lowest-leg anchor | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |
| canon: most ext legs | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |
| canon: fewest ext legs | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |
| greedy: as-generated | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |
| greedy: largest-first | 70 | 2800 | 3 | 6 | PASS | 1.9e-15 |

## `g g > g g`  (n_ext=4, n_diagrams=4, n_flows=6)
floor: dedup_currents=6, sum_edges=3, n_ext=4

| variant | nodes | weighted (B) | #prop | floor(cur) | gate | max_rel |
|---|--:|--:|--:|--:|---|--:|
| baseline VtxIdx(0) | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |
| canon: lowest-leg anchor | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |
| canon: most ext legs | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |
| canon: fewest ext legs | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |
| greedy: as-generated | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |
| greedy: largest-first | 189 | 8224 | 3 | 6 | PASS | 8.2e-14 |

## Cross-process totals (Σ over 14 processes)

| variant | Σ nodes | Σ weighted (B) | vs baseline nodes | gate |
|---|--:|--:|--:|---|
| baseline VtxIdx(0) | 9111 | 534976 | — | PASS 14/14 |
| canon: lowest-leg anchor | 9111 | 534976 | 0% | PASS 14/14 |
| canon: most ext legs | 10564 | 673824 | +15.9% | FAIL 4/14 |
| canon: fewest ext legs | 7287 | 361072 | −20.0% | FAIL 5/14 |
| greedy: as-generated | 7200 | 354080 | −21.0% | FAIL 5/14 |
| greedy: largest-first | 7200 | 354080 | −21.0% | FAIL 5/14 |

## Findings

1. **The headroom is real.** Greedy cuts total post-CSE nodes 9111 → 7200 (−21%) and
   slot-weighted traffic 535 kB → 354 kB (−34%). "Fewest external legs" captures nearly
   all of it (−20% nodes). "Most external legs" moves the *wrong* way (+16%): rooting at
   a central, high-degree vertex duplicates currents. Diagram order (as-generated vs
   largest-first) makes no difference on this suite.

2. **But the win is currently unrealizable: re-rooting away from `VtxIdx(0)` breaks the
   amplitude.** Every node-reducing rooting fails the gate, in two regimes:
   - *Benign reassociation* — `canon: most ext legs` on the 6/8-point processes fails
     only 1–5/50 points at max_rel ~1e-11..1e-12 (physically correct, just over the
     tight 1e-12 tolerance). Amplitude is right.
   - *Gross wrong amplitude* — `canon: fewest` and both greedy variants fail 50/50 with
     max_rel 1e-2 … 1.7e+3 on `e+e-→W+W-`, `e+e-→τ+τ-H`, and every ≥6-point QCD=0
     process. These are not tolerance issues; the re-rooted amplitude is simply wrong.
   Failures compile and evaluate (no `COMPILE-ERR`/`PANIC`), so this is a **silent
   orientation-dependence in the production rooting machinery**, not a missing kernel op.
   The likely locus is momentum routing / Lorentz-output rooting / fermion-spine sign,
   which are validated only for the `VtxIdx(0)` orientation feyngraph hands us.

3. **The floor is far below baseline; baseline is far above the floor.** Baseline
   realizes 642/669 currents (`#prop`) on the 8-point processes against a floor of
   2090/2232 *distinct* directed-current signatures — i.e. the dedup pool is ~3× the
   currents any single rooting realizes, so most of that "floor" is unreachable reverse
   directions, not achievable sharing. The genuinely-informative gap is `#prop` vs the
   no-share bound `sum_edges`: baseline already shares heavily (642 vs 2895), and greedy
   pushes further (275 vs 2895) — but only by realizing currents whose amplitudes the
   machinery computes incorrectly.

4. **`canon: lowest-leg anchor` ≡ baseline on every process.** Feyngraph's `VtxIdx(0)`
   is already the vertex adjacent to the lowest-index external leg, so this "canonical"
   choice is a no-op — production is already lowest-leg-anchored.

## Recommendation

- **(a) Do NOT promote a production greedy/canonical rooting pass into Track 1 yet.** The
  ~21% node headroom is real, but no correctness-preserving rooting other than the
  status quo was found: re-rooting silently corrupts the amplitude on multi-boson and
  ≥6-point processes. A production pass is blocked on first making the rooting genuinely
  orientation-independent (momentum-routing + Lorentz-output + fermion-spine signs
  provably invariant under edge reversal), guarded by exactly this per-rooting gate.
- **(b) Track 3 (e-graph re-rooting) go/no-go:** the headroom that would justify DAG-cost
  extraction *exists* (−21% nodes / −34% traffic), so the payoff is not illusory. But the
  prerequisite is the same orientation-invariance fix, not the extractor: propagator-
  commute + per-vertex rotation rules (note 15 §1.3) must be **correctness-preserving
  rewrites**, and this study shows the current primitives are not invariant under naive
  re-rooting. Recommendation: keep Track 3 gated behind a correctness-first "make
  re-rooting sound" spike; the DAG-cost extractor is worthwhile only after that, and the
  greedy oracle here (−21%) is the target it must beat.
