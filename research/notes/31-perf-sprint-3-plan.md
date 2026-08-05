# 31 — Performance sprint 3: integration budget, PDF interpolation, evaluator schedule (plan)

**Status:** PLAN, drafted 2026-08-04 against the note-30 baseline (per-stage timings,
samply profiles, host block — all taken on `main` @ `d7b7e68`, M3 Max). Predecessor
performance programs: note 15 (eval layout/expansion/filtering, closed 2026-07-17)
and note 20 (`eval-perf-2`, closed 2026-07-21, vs-MG geomean 1.24×). The gate this
sprint optimizes against is the note-29-hardened validation layer
(29 rows × 4 categories, 96 ✅ / 2 ⚠️).

## 0. What the baseline says the sprint is allowed to believe

From note 30, all one host, one sitting:

- **97.9% of the layer's timed work is the integrals and samples categories**
  (842.6 s + 840.2 s of overlapping row time); 6 hadronic rows carry two thirds of it.
- Profiles agree across all four integrate/sample paths: **evaluator (`helas::*`)
  50–63% of self time, `fill_arenas` alone 28–34%**; **PDF interpolation 14–19%
  wherever there are protons** (`LogBicubic::xfx_q2` 10.4–13.8% + `PdfMember::xfx_q2`
  4.0–5.6%); allocator traffic 6–18% (peaks on accept/reject); nothing else over 13%.
- The chain-B configuration draw costs **≈1.0 µs/point (~21%) on partonic live-draw
  rows**, ≈0.2 µs/point (~3%) on `pp_to_llj_dyn` — the second-largest per-point item
  on those rows after the matrix element itself.
- Run-to-run spread on rows above 1 s: **median 0.8%, worst 3.4%**. That is the
  smallest layer-level effect this instrumentation can see; anything finer must be
  claimed through `eval_strategies` (criterion) or a dedicated probe, not row durations.
- vs MadGraph, same host, CPU-time denominators: geomean 6.8×, but **1.6–6× on the
  rows that carry the wall time** (`gg_to_gg`, `pp_to_jj`, `pp_to_llj*`).

Three levers follow, and they are the sprint's three tracks: **(I)** spend fewer
points (integration budget/bias), **(P)** make the PDF share small (interpolation
hot path), **(E)** make the evaluator loop faster (schedule + instruction-level
findings). They are mutually independent and can run as parallel sessions.

## 1. Track I — integration engine (the pre-committed pair)

The backlog names these as one holistic "how the budget splits across
adapt / scan / frozen phases" redesign; note 24 §P3/§P4 carry the measurements.

### I1 — VEGAS first-iteration convergence bias

`VegasGrid::adapt` feeds every iteration into `combine_iterations`' 1/σ² weighted
mean, including unadapted early ones that undersample the peak and return low
integral *and* low variance — weighted up. Measured on llj: −1.03% at 30k/iter →
+0.002% at 300k, the O(1/N) signature; the accept/reject pass (single pass over
frozen grids, no `combine_iterations`) independently converges to the true σ.

**Change**: discard the first `k` adaptation iterations from the combination (or
run an unweighted final pass over the trained grids); make `k`/the split a
`VegasGrid` parameter with the current behaviour recoverable.

**Payoff**: correctness headroom converts directly to budget — the llj gates could
run at ~¼ budget once the low-budget bias is gone. `pp_to_llj` integrals alone is
186 s at 600k points/iteration *because* its ladder was still climbing at 300k
(the note-29 hygiene item); this is the single biggest wall-time lever in the layer.

**Gate**: ≥5-seed sweeps with χ²/dof at two budgets per changed row (AGENTS.md
sampler discipline — a fixed-seed pull is not evidence; chain B's known low-budget
scatter on `pp_to_llj_dyn` — χ²/dof 6.38 at 75k, clean ≥150k — is the fragility a
budget reduction would bite). Re-pin the report budgets only after the sweep is
clean at the reduced budget.

**I1 — MERGED 2026-08-04 (`5b3952d`, merge `59887a3`): measured results.**

- **The plan's primary fix was wrong; its parenthetical was right.** A 4000-seed
  offline study (5-D Gaussian, known integral) showed the warm-up discard alone
  removes essentially none of the bias (−1.21% → −1.40% mean rel at 2k/iter with
  2 discarded): the estimate–weight correlation is in *every* iteration, so the
  lever is the combination rule. Landed: `VegasGrid::combination` (default
  **`Unweighted`**; `InverseVariance` recovers Lepage bit-exactly — how the
  pinned-seed goldens keep their bytes) plus `VegasGrid::warmup` (default 2 —
  needed for variance, not bias: RMS 0.53%→0.19% at warm-up 0→2). Both
  `#[serde(skip)]`: banked artifact bytes unchanged. Trained grid provably
  untouched (bin-edge equality test), so samples are unaffected.
- **Sweeps** (5 seeds × 75k/150k/300k/600k, before/after, same host; the before
  arm reproduced the repo's recorded ladders digit-for-digit): llj ladder spans
  collapse — `pp_to_llj` 2.09% → 0.46%, `llj_fixed` 0.80% → 0.12%, `llj_dyn`
  0.76% → 0.06%; chain B's χ²/dof 6.38 at 75k falls to 2.55 (shrank, did not
  migrate). Wide-channel rows (jj, bb) never had a climb and do not move.
- **Budgets re-pinned**: `LLJ_NEVAL` 300k → **150k**, `pp_to_llj` recarded row
  600k → **150k** (not 75k — largest inter-seed scatter rung). Gate pulls at
  the new budgets: +0.32 / +0.49 / +0.68. `pp_to_jj` is flat to 0.08% across
  the ladder and could take a 4× cut (~130 s) — left as a manager/backlog
  budget decision, since its budget was never bias-set.
- **Timing**: integrals category **842.6 s → 644.1 s (−24%)** vs note 30, each
  llj row tracking its budget factor; samples category moved only with host
  noise (smaller trained grids cost no measurable unweighting efficiency).
- **For I4**: the hard prerequisite is `IterationCombination::Unweighted`, not
  the discard — a convergence stop reading Lepage's error bar stops early on a
  confidently wrong number. The unweighted error is larger and better
  calibrated but still an underestimate at the starved end (1.70% quoted vs
  8.2% actual at 2k/iter), so the χ²/dof precondition does real work.
- **Same defect one level up, out of scope, session-worthy**: `combine_seeds`
  in `validate_hadronic.rs` combines seeds by inverse variance (second-order
  now that per-seed errors are well estimated).
- **Close-out documentation debt**: stale measured σ values remain in
  `BB_NEVAL`'s ladder comments, the DY row docs, and `validate_sigma.rs`'s
  per-process numbers — re-recording them needs the close-out re-measure.
- E3's probe rows now run 150k × 10; E3 measures against this commit.

### I2 — `w_max` scan budget decoupled from the integration budget

The frozen scan inherits the integration budget's undersampled small channels: on
llj the σ-share above the maxima is still falling at 600k (5.3e-3, vs 3.04e-4
fixed-beam). Unbiased (overweights kept at weight > 1) but costs unweighting
efficiency and sample lumpiness.

**Change**: an independent scan budget (points per channel, not a share of
`neval`), so the maxima estimate converges on its own schedule.
**Measure**: σ-share above maxima + overweight count trajectory + `samples`-row
durations; the largest `w/w_max` is an extremum estimate — do not read it as
convergence (note 24 §P4).

### I3 — parallel `integrate`: `Sync` hadronic integrand + `-j/--parallel` (user, 2026-08-04)

Note 30's single most important caveat — "MadGraph is a 16-way parallel job farm
and our integrator is one thread" — becomes a fix, not a footnote. The substrate
already exists and is better than the backlog remembered: `VegasGrid::adapt_parallel`
runs fixed-size chunks on `(iter, chunk_idx)`-keyed `ChaCha8` substreams with a
sequential chunk-order reduction, and the suite pins **thread-count bit-identity**
(`run_with_threads`, `vegas.rs` tests) — the same seed gives byte-identical results
at any thread count. What blocks the hadronic path is exactly the backlog's
DY-parallelism item: `hadronic.rs`'s per-flavor-class scratch is `RefCell`-based,
making the integrand `Fn` but `!Sync`, so `ProtonIntegrand` never reaches the
parallel entry points and note 30 §7's profiles show ~16 rayon workers parked in
`__psynch_cvwait`.

**Changes**:

1. **Per-thread scratch**: replace the `RefCell` scratch (`ScratchSpace`,
   `ScaleAwareAmplitude`, `scale_buf`/`amp2_buf`, the scale-draw substream holder)
   with per-thread instances (thread-local pool or per-chunk construction —
   implementer's call; the scale-draw stream is already `(seed, stream)`-keyed and
   must stay a pure function of the point, not of the thread). The integrand
   becomes `Fn + Sync` and the hadronic σ path takes the existing parallel VEGAS
   entry points.
2. **CLI `-j/--parallel <N>`** on `vibegraph integrate` (and `generate`, which
   shares the machinery): configure the global rayon pool at startup
   (`ThreadPoolBuilder::num_threads(N).build_global()`); default = all cores;
   `-j 1` reproduces today's serial behaviour. The CLI has no thread control today
   and rayon's default pool was being built and parked unused on the hadronic path.
3. **Bit-identity is the gate**: assert (test + one CLI-level check) that
   `integrate` at `-j 1` and `-j 16` produce byte-identical artifacts for a
   partonic and a hadronic card. The validation layer keeps running
   single-threaded rows (`--test-threads=1`, the note-30 comparability contract) —
   the bit-identity property is what makes that a measurement of the same numbers
   the parallel CLI produces.

**Measure**: `vibegraph integrate` wall time at `-j 1/4/8/16` on the `dy13_default`
and `pp_to_llj` cards, same host as note 30. Target: the 16-way speedup MadGraph
gets from its job farm — realistically ~10–12× on 12P+4E cores if the per-point
work parallelizes cleanly (chunks are embarrassingly parallel; the sequential
grid/α update between iterations is the Amdahl term, measured by the scan itself).
Close-out re-runs note 30 §5.2's wall-time comparison with a `-j 16` column so the
"16-way farm vs one thread" asymmetry is retired from the record.

**I3 — MERGED 2026-08-04 (`6497f7e`, merge `1b527cc`): measured results.**

- **Brief/plan corrections (three, all recorded):** (1) the substrate was NOT
  ready — `adapt_parallel`'s closure cannot know which point it is (the scale
  draw is a function of the point index via a stateful sequential `SubStream`)
  and its `(iter, chunk)` keying + per-chunk reduction does not reproduce
  `adapt`'s numbers; it stays unused and untouched. The session added
  **`adapt_parallel_seeded`**, bit-for-bit with sequential `adapt` by
  construction: each chunk seeks its generator to `p·ndim` draws and per-point
  values/bin indices are reduced in **global point order** (a per-chunk partial
  sum would reshape the grid). (2) `validation/validate.sh` never passed
  `--test-threads=1` — the §I3 "note-30 comparability contract" premise was
  wrong; bit-identity is what makes the layer's numbers the CLI's numbers.
  (3) The Amdahl term is the **α-adaptation survey** (serial,
  budget-independent, `neval.clamp(10k,40k)×6`), not the grid/α update:
  predicted 14.3%, measured 15.3–17.2%.
- **Implementation**: `SubprocessProto`/`BoundSubprocess` split +
  `ThreadLocal` scratch; both integrands `Sync` (asserted);
  `-j/--parallel` on `integrate` and `generate` (`-j 0` refused; `generate`'s
  accept/reject stays serial and says so). `Real`/`Channel`/`ScaledChannel`
  gained `Send + Sync`.
- **Bit-identity evidence**: artifact md5 identical across `-j 1/4/8/16` AND
  vs the pre-change serial binary at `af12e01`, on partonic, `dy13_default`
  and `pp_to_llj` cards, repeated at 4× budget. Full validate exit 0, census
  character-identical. Seven new tests, each negative-controlled; one vacuous
  check (fixed-beam trailing uniform is inert — 40/40 probes unmoved) was
  caught by its own control and replaced with a live-draw reference on the
  proton path; the blind spot is documented in the test.
- **Scaling** (min of 4 round-robin rounds, load 7–8 from a sibling session —
  speedups biased *down*): dy13 2.41→0.54 s (**4.46×**), llj 14.26→2.93 s
  (**4.87×**) at default budget; **7.30×/8.34×** at 4× budget. Fixed serial
  floor 0.39/0.55 s (the α-survey); the adapted phase alone parallelizes
  **~10.5–13.5×** — inside the target band. Peak RSS flat (~0.11 GB) at any
  thread count.
- **For I4**: the prerequisite is `adapt_parallel_seeded` — `(channel, chunk)`
  scheduling must keep the point-order reduction or every banked σ moves. The
  remaining tail is granularity: 24 channels × 12 iterations = 288 sequential
  parallel regions, floor channels yield 8 chunks vs 16 workers
  (`MIN_CHUNK = 64` binds; chunk size is a free, results-inert knob — untuned
  because the host was loaded). Parallelizing the α-survey
  (`survey_variance` is O(n_survey × n_channels)) is the other lever.
- **Layer timing semantics changed**: validation rows now use the machine
  internally — this validate ran 10m09s vs the banked ~42m52s wall, with σ,
  bytes and census untouched. **Close-out decision (manager)**: the close-out
  per-row `duration_s` table re-runs under `RAYON_NUM_THREADS=1` for
  comparability with note 30 §3.2, and separately records the default-parallel
  validate wall time and the `-j 16` CLI column as the new headline numbers.

### I4 — convergence-targeted integration with hard-split per-channel allocation (user, 2026-08-04)

Today every gate and the CLI spend a fixed `seeds × neval × niter`; MadGraph
instead refines until a requested accuracy is met (note 30 §5.1). Do the same, but
allocate better than MadGraph does: MG refines each channel/G-directory toward its
own target more or less independently, while the optimal spend for a *total*-σ
target puts points where they buy the most variance reduction — and our channels
converge at wildly different rates (the `w_max` scan measurements: small llj
channels are nowhere near converged at budgets where big ones long since were).

**Design shape** (the session refines it; constraints below are fixed):

- **Hard split**: deterministic per-channel point counts `N_j` (the backlog's
  channel-block stratification, promoted from stretch to substrate) instead of
  per-event multinomial channel draws. Exact — no partition function, no
  `sde_strategy`-class routing fragility — and it removes label noise while making
  each block one map = one code path.
- **Allocation rule**: per-channel empirical variance from the block estimator
  drives the next iteration's `N_j` (Neyman-style, `N_j ∝ α_j σ_j`), with a hard
  **per-channel floor** — split the spend, never the coverage (the note-27 §B1
  guardrail; a starved channel is how VEGAS becomes confidently wrong).
- **Stopping rule**: iterate until the combined estimate's relative uncertainty
  meets the target, gated by sanity preconditions: I1's warm-up discard must be
  active (a biased early combination produces a *confidently wrong* small error
  bar and a premature stop — I1 is a hard prerequisite, not a neighbour), a
  minimum iteration count, and an iteration-consistency χ²/dof check before the
  error bar is believed (MadGraph applies the same kind of guard to its own
  refine).
- **Rayon granularity**: the scheduling unit is `(channel, chunk)`, not the
  channel — a wide channel must not serialize an iteration behind it, and a
  narrow one must not pay a whole-thread quantum. Tune chunk size so an iteration
  yields roughly a small multiple of chunks per thread (a measured knob, not a
  guess): tight enough that the adapt-sample barrier has no long per-channel
  tails, loose enough that chunk dispatch overhead stays invisible. Depends on I3
  (the hadronic path must be `Sync` before any of this parallelizes).
- **Fixed-budget mode stays**: the banked validation rows are pinned to budgets
  and seeds for reproducibility; convergence mode is the CLI/user-facing default
  (`--target-rel <r>` or similar), and its own gate is a matched-accuracy
  comparison: reach MG's banked uncertainty on the llj/dy13 rows and compare
  spend (points and wall time) against note 30 §5.3's throughput denominators.

**Sequencing**: after I1 (prerequisite) and I3 (parallel substrate); supersedes
the old stretch-item I3 (channel-block stratification), which it absorbs.

## 2. Track P — PDF interpolation hot path

### 2.1 What the code inspection found (2026-08-04, this plan)

- **Subgrid structure** (user question answered): subgrids are Q² *bands* sharing
  one x axis — each band a full rectangular `nx × nq × nf` tensor. The production
  set `NNPDF23_lo_as_0130_qed` has **one** band (100 × 50 knots, **14 flavors**);
  `NNPDF31_lo_as_0130` has two (12 + 38 Q² knots, 11 flavors). Band selection is
  therefore a 1D lookup on Q² band edges — the "order-N scan" (`interp.rs:154`)
  walks 1–2 bands with 4 comparisons each. Real but small; the dominant per-call
  overheads are elsewhere:
- **Per-call repeated work**: every `xfx_q2` call redoes `normalize_flavor_pdg`,
  the finite/positivity guards, `has_flavor` (a linear scan over subgrids ×
  14 flavors), `in_grid_range`, the band walk, a linear `position()` over the
  flavor list, two `partition_point` binary searches (log₂100 + log₂50 steps),
  and two `ln` calls — before any interpolation arithmetic.
- **Call multiplicity**: the luminosity loops (`proton.rs:299–352`) make
  4 `xfx_q2` calls per member per beam ordering, summed over every member of every
  flavor group — yet all of them evaluate at exactly **2 distinct (x, Q²) points**
  per phase-space point ((x₁, μ²_F1), (x₂, μ²_F2); the mirror term swaps flavors,
  not points). Tens of calls per point collapse into two all-flavor evaluations
  plus dot products.
- **The coefficient layout is already batch-friendly**: `coeffs` is
  `(ix, iq, ifl, 4)` row-major — for a fixed knot cell the per-flavor `[a,b,c,d]`
  quadruples are contiguous.
- **Horner needs no preconversion**: the stored monomial coefficients `[a,b,c,d]`
  *are* the Horner coefficients — `((a·t + b)·t + c)·t + d` — so this is an
  evaluation-order change (`cubic_x`, `cubic_hermite` → `mul_add` chains), not a
  table change. The x86 study (x86-avx2-perf-study-results.md) already established
  the crate-wide pattern: route through the real `F::mul_add`; on this host (M3,
  native FMA) fusion is free.
- **The `F: Real` genericity has no consumer**: `ProtonIntegrand` and `hadronic.rs`
  are concretely `f64`; nothing in src/, tests/ or benches instantiates the PDF
  path at any other scalar. The generic threading (per-call `f::<F>` casts of every
  coefficient and knot) buys nothing today.

### 2.2 SIMT go/no-go (proposed): **go on the shape, no-go on a batched API this sprint**

The user's longer-term aim is a SIMT-friendly implementation (GPU-style lanes over
phase-space points), with divergence confined to grid selection. Assessment:

- The kernel itself SIMT-vectorizes well: band selection is 1–2 bands (a 1D binary
  search over band edges — uniform depth, benign divergence), knot location is
  fixed-depth binary search, and the only structurally divergent arithmetic is the
  Q²-edge slope cases (forward/backward/central) and the degenerate two-knot
  bilinear band — both removable branch-free (clamped ghost-knot indexing picks the
  slope stencil arithmetically; the degenerate case exists only for bands with
  nq = 2, decidable per band at build time). Extrapolation stays a scalar fallback
  (rare in production: μF ranges sit inside the grid).
- The *lift to actual SIMT* is not in the PDF kernel — it is consumer-side: the
  hadronic integrand is a single-point `FnMut` (the `RefCell`-scratch DY-parallelism
  item), and `eval_m2_lanes` cannot batch points with different αs ("per-lane
  scales" backlog). Until an integrator batches points, a lane-batched `xfx` has no
  caller.

**Verdict**: implement P1 as an **f64-only, SIMT-shaped scalar kernel** — index
computation separated from arithmetic, branch-free slope stencils, flavor-major
inner loops, no trait genericity — and record in the module docs that a
batch-of-points variant is a mechanical extension once an integrator can feed it.
Do not build the batched API now. This makes the fast-CPU form and the future-SIMT
form the same code shape, so nothing is thrown away either way.

### 2.3 P1 — the session

One session, `pdf/` + the `proton.rs` luminosity consumers:

1. **f64-in API**: drop `F: Real` from `interp.rs`/`extrap.rs`/`mod.rs`
   (`Bicubic2D`, `LogBicubic`, `PdfMember::{try_,}xfx_q2` take/return `f64`).
   Delete the `f()` cast scaffolding.
2. **Flavor index map**: per member (and per band), a 16-entry LUT
   `pdg ∈ {-6..6, 0, 21, 22} → Option<u8>` built once at load; kills the
   `position()` scan, the `has_flavor` scan, and repeated `normalize_flavor_pdg`.
3. **All-flavor evaluation**: `xfx_all(x, q2) → &FlavorRow` (or caller-provided
   buffer) computing the guards, band select, `ln`s, ix/iq searches, `tlogx`,
   `tlogq` and the four Hermite basis weights **once**, then evaluating all
   flavors' cubics off the contiguous per-cell coefficient block. Restructure
   `member_luminosity`/`luminosity`/`symmetry_weighted_luminosity` (and the
   flavour-draw path in `generate`) to take the two per-beam flavor rows per point.
   Single-flavor `xfx_q2` stays as a thin wrapper for tests/oracles.
4. **Horner + FMA** in `cubic_x`/`cubic_hermite` via `f64::mul_add`.
5. **Band selection by binary search** over the band-edge Q² array (with the
   first-in-range seam semantics pinned by the existing
   `subgrid_walk_selects_first_in_range_band` test preserved exactly).

**Tolerance position** (user-authorized relaxation, bounded): the LHAPDF oracle
gate (`validate_pdf_grid`, REL_TOL 1e-12) should *survive* Horner/FMA — the
reassociation error is O(ulp) (~1e-16 rel per operation, a few ops deep) — so the
expectation is **no relaxation needed**; if a point exceeds 1e-12, relax that gate
to 1e-11 with a one-line note, never further without a diagnosis (AGENTS.md: set
tolerance at the algorithm's own error scale; LHAPDF's own knot-derivative
finite-difference error is enormously larger than either). The σ/samples gates are
statistical and cannot see ulp-level PDF shifts. `probe_pdf`-based unit tests that
assert exact equality on interpolated values may need the same ulp-scale review.

**Expected size**: PDF is 14.5% (integrate llj_dyn) and 19.4% (generate proton) of
busiest-thread self time, and the restructure removes most interpolation work per
point (tens of calls → 2 all-flavor evals), plus per-call overheads. Target: PDF
group under ~5% on both paths, i.e. ~10% wall on hadronic integrate rows and ~15%
on proton generate — measurable against the 0.8%/3.4% noise floor via row
durations, plus a dedicated criterion micro-bench for `xfx_all` (add one; there is
none today).

### 2.4 P1 — MERGED 2026-08-04 (`71a7ef3`, merge `a66d58a`): measured results

All five items landed plus the `pdf_xfx` criterion bench; **no tolerance was
relaxed anywhere**. Gate: full banked layer green, cell-for-cell identical census
(96 ✅ / 2 ⚠️ / 4 ⏳), `validate_pdf_grid` 20/20; hadronic σ gates all green.

- **Brief correction (recorded)**: the "REL_TOL 1e-12 oracle gate" in §2.3 was
  wrong — `1e-12` in `validate_pdf_grid` only locates knots by coordinate. The
  real accept bars: 1e-9 (interpolation), 1e-11 (flat continuation), **1e-14**
  (conditioned residual — the tightest, and the one a reassociation would break).
- **Horner+FMA landed in `cubic_x` only.** In `cubic_hermite` it moved a single
  continuation probe at x = 1 (a ~1e-35 pure-cancellation residue that the
  oracle matches only because our operation order reproduces LHAPDF's own
  rounding) by 2.4e-5 relative; per the AGENTS.md reformulate-don't-relax rule,
  `cubic_hermite` keeps LHAPDF's operation order (documented in its doc comment
  as load-bearing). Worst conditioned residual 8.93e-16 → 1.08e-15 vs the 1e-14
  bar; all other categories unchanged or ulp-level.
- **Kernel**: `xfx_all` (one reading, 14 flavours) **112 ns** vs 504 ns for the
  14-call shape it replaces — 4.56×, essentially all from the all-flavour
  restructure (per-call overhead removal alone was only ~1.07×). M3 Max,
  `release`, criterion medians.
- **Layer**: hadronic row work (cells > 1 s) **2023.7 s → 1477.9 s (−27.0%)**,
  same host, back-to-back before/after in the session worktree. Largest:
  `pp_to_jj` integrals −39% / samples −51%. Partonic rows flat within the
  0.8%/3.4% noise floor.
- **Profiles**: PDF group **14.5% → 1.38%** (integrate llj_dyn) and
  **19.4% → 2.00%** (generate proton) of busiest-thread self time; the evaluator
  is now **62% / 79%** with `fill_arenas` alone at 33.1% / 42.6% — Track E's
  ceiling grew accordingly. Note 30 §7.3's "PDF 14–19% wherever there are
  protons" is retired; every Track-I duration claim must re-baseline against
  `a66d58a`, not note 30 §3.2's table.
- Ops note for future worktree sessions: `git worktree add` leaves the
  `research/refs/mg5amcnlo` submodule checkout empty, which aborts
  `cargo test --workspace` before most gates run; COW-copy the checkout in and
  point its `.git` file at the shared module.

**P1b follow-up — MERGED 2026-08-04 (`91c5a79`, merge `0fd6344`)**, user-directed:
the continuation oracle's relative-only comparison gains an absolute screen —
`|got − want| ≤ 1e-30 + 1e-11·|want|`, all five continuation categories, both
oracles. ABS_TOL 1e-30 is four orders below one ulp of the `ForcePositive`
1e-10 floor (the smallest magnitude LHAPDF itself treats as a density), and no
probe lies in the changeover gap (1e-30, 1e-19), so only pure-residue and
exact-zero probes see a different bar; the per-category sub-floor `worst |Δ|`
statistic reads 0.0 everywhere today — the screen is pure headroom. Relative
bars unchanged. **Horner+FMA in `cubic_hermite` stays rejected, now for a
stability reason independent of any oracle**: at t = 1 the Hermite basis
weights are exact in binary, so LHAPDF's operation order returns the knot value
`vh` bit-for-bit, while a Horner chain reaches it only through cancellation —
measured on-knot reproduction degrades 2.7e-20 → 2.7e-12 and exact corners pick
up 4.4e-16. That property survives any test-suite change; both reasons are in
`cubic_hermite`'s doc comment. Do not revisit. (Foregone payoff is bounded:
`cubic_hermite` runs once per flavour vs `cubic_x`'s four — ~1/5 of the cubic
evaluations in an `xfx_all` reading — and the session's host was too contended
to measure it anyway.)

## 3. Track E — evaluator

### E0 — `fill_arenas` instruction-level study ✅ (2026-08-04)

Done the day of this plan (subagent; full record with every command:
`fill-arenas-asm-study-results.md`, artifacts under `target/fill-arenas-study/`).
Per-address sample attribution over the linked `validate_sigma` binary (14 009
leaf samples = 33.6% of the σ-gate's busiest thread, reproducing note 30's 34.2%),
reconciled against `cargo asm --rust` (pre-LTO view agrees to 9 of ~2045
instructions). Headlines:

- **The dispatch is already a jump table** — 38-entry `u16` offset table
  (`ldrb`/`adr`/`ldrh`/`add`/`br`), one `br` in the whole function, verified
  entry-by-entry against the 38 `Instr` variants. No compare chain to fix.
- **Only ~1 instruction in 5 is arithmetic.** Sample budget: **30.5% loop
  control + dispatch** (10 instructions; 10.87% on the jump-table `ldrh` alone —
  the classic interpreter data-dependent indirect-branch stall), **22.9% loads**,
  **20.4% FP arithmetic**, **20.4% bounds checks** (302 instructions, 108
  `panic_bounds_check` sites), 3.5% stores.
- **The arena `Vec` headers are re-loaded on every instruction** — 143 loads off
  the `ScratchSpace` pointer re-fetch `ptr`/`len` because LLVM cannot prove arena
  stores don't alias the headers. Exhibit: `MulScalarR` is 17 instructions for
  one `fmul.2d`, and its two hottest instructions are the two `len` reloads.
- Hot regions: loop control 18.50%, `GammaVout` 18.17% (inlined, **scalar-lane**
  FP, not packed), `Metric` 14.09%, dispatch 11.99%, `MulScalarR` 7.31%.
- Four kernels stayed out-of-line (`ffv_vout_bare`, `propagate_{vector,fin,fout}_bare`);
  each call site pays an sret stack round-trip, a 64-byte stack→arena copy, and
  rematerializes the loop's invariant constants afterwards.
- Attribution caveats recorded in the study §5 (PC skid makes the within-block
  split of the 30.5% soft; sub-1% arm shares are ±0.5%).

### E1 — DAG linearization / execution-order study (user request, 2026-08-04)

`Program::build` (`layout.rs:325`) emits instructions **in AST-arena order** — the
order hash-consing/lowering/`expand_helicities` happened to intern nodes. No
scheduling pass exists; liveness slot recycling runs over that accidental order,
and the 2→6 peaks at ~27k live slots (~1.7 MB — L2-resident, not L1; note 15 §2.2).
Whether that order is good for producer→consumer distance, live width, dispatch
predictability, or pipelining has never been measured.

**Study first** (llj subprocesses + `uux_to_ccx_emmm_qcd0` as the stress case):
dump the compiled program stream and the pre-lowering DAG; compute per-program
metrics — producer→consumer distance histogram, live-set width profile over the
stream (peak and mean vs `arena_sizes`), same-`Instr`-discriminant run lengths
(dispatch/branch predictability), critical-path depth vs stream length (available
ILP). Then prototype 2–3 alternative topological orders behind a test-only hook:

- **depth-first chain-following** (minimize producer→consumer distance, keep
  operands hot in L1);
- **live-width-minimizing** (Sethi–Ullman-flavored; shrinks arenas — the 2→6's
  1.7 MB working set is the target);
- **op-type-blocked within dependency levels** (amortize dispatch; tension: longer
  lifetimes — measure, don't argue). E0 sharpened this one: the dispatch cost is
  a mispredicting data-dependent indirect branch, and discriminant run lengths
  directly set its predictability — measure jointly with E2 item 2.

**Key property**: any topological reorder leaves every node's arithmetic and the
root readout untouched — **bit-for-bit gateable** (`validate_helas_mg` byte
equality), unlike almost every other evaluator lever. Measure via `eval_strategies`
(±2–3% criterion noise floor; claims need to clear it). Deliverable: a go/no-go on
a production scheduling pass in `Program::build`, with the winning order's numbers.

**E1 — MERGED 2026-08-04 (`52327b2`, merge `7416c1d`): verdict GO** — study
instrumentation + `VIBEGRAPH_EVAL_SCHEDULE` hook (absent from release/validate
builds), default order unchanged and byte-identical to base (amplitude-oracle
digest equal; every prototype order `to_bits`-identical with anti-vacuity
guards). Winner: **op-blocked within ASAP dependency levels**
(`sort_by_key(|id| (level, instr_kind, id))`):

- **−17.9% geomean ns/eval, 18/18 rows improve** (−8.9%..−22.6%; 6-round
  round-robin, min over rounds — round-1 spreads reached 173% under load, the
  protocol was necessary). **MATRIX1 geomean 1.24× → 1.02×**; 6 of 14
  processes now beat MadGraph; the 2→6 rows go 1.17×→0.94× and 1.37×→1.06×.
- **Mechanism isolated by control**: `OpWindow{32,128,512}` variants share the
  winner's (worse) locality and live width, differing only in discriminant run
  length — the win is monotone in run length alone. Arena order's mean run is
  1.00–1.12: the mispredicting jump-table `ldrh` (E0's 10.87%) was being fed
  the worst possible input. E2's dead item 2 (threaded dispatch, +7.7%) is
  fully superseded — same stall attacked from the input side; do not revisit.
- **Non-levers, with numbers**: depth-first +1.2% (arena order already *is* a
  bottom-up DFS — the plan's "accidental order" premise was wrong); live-width
  minimization −0.4% (it shrinks peak bytes 2→6 236,760→206,160 and buys
  nothing — the working set was never the constraint); ILP is a non-question
  (depth 11–23 vs 36,523 instrs).
- **Brief correction**: note 15 §2.2's "~27k live slots / 1.7 MB" 2→6 working
  set does not exist at this HEAD — production (pruned) 2→6 is 305 KB
  allocated / 231 KB peak; the unpruned program peaks at 149k slots / 2.84 MB.
  Something between note 15 and now moved it (ZEROAMP/re-rooting candidates);
  not chased, out of scope.
- **Production pass (follow-up session E1b)**: ~20 lines (ASAP levels + one
  sort, variant computable from `(Op, storage class)` for one-pass); arenas
  grow under the winner (2→6 305→451 KB allocated) so add a cheap
  measured-bytes fallback threshold; the gate recipe is already built
  (`alternative_orders_are_bit_identical` + oracle digest — the banked layer
  must stay byte-identical including amplitude values). `opwin512` gives 99%
  of the win at mean-run ~103 if a bounded variant is ever wanted.
- **E3 caveat**: `eval_amp2` runs the unexpanded program, unmeasured here; a
  production pass hits it too — E3 re-baselines after E1b lands.
- Env-var caveat: under `cfg(test)` the hook exists in the lib unit-test
  binary; integration tests and the validate layer are hard-wired to arena
  order.

### E2 — `fill_arenas` overhead reduction (scoped by E0)

The study's budget says the ceiling plainly: dispatch + bounds checks + header
reloads together carry ~50–60% of the symbol, against 20% genuine FP. Sessions
stay inside the twice-affirmed 100%-safe-Rust charter (note 17 §9; the AVX2
study's `get_unchecked` NO-GO) — every item below is a safe-code shape change:

1. **Hoist the arenas into local slices** once per `fill_arenas` entry
   (split field borrows of `ScratchSpace`; same-arena read-then-write stays
   plain indexing on one `&mut [T]`). Keeps `ptr`/`len` in registers across all
   38 arms, kills most of the 143 header reloads, and makes the remaining bounds
   checks register-compare-cheap (some become hoistable). Targets the ~20%
   bounds-check + a large slice of the ~23% load budget. Bit-for-bit.
2. **Dispatch restructure**: replicate the dispatch tail into each arm (threaded
   dispatch — 38 independent branch histories instead of one mispredicting
   merge-point `br`), measured against/combined with E1's opcode-blocked program
   order (runs of identical discriminants make even one dispatch site predict
   well — the two levers overlap, so measure jointly). Targets the 30.5%
   loop+dispatch block. Bit-for-bit.
3. **Out-param the four non-inlined kernels** (write into
   `&mut scratch.<arena>[loc]` instead of sret-return) or force-inline them;
   removes the 64-byte stack round-trip and the constant rematerialization.
   Bit-for-bit.
4. (Measure-first) **`GammaVout` packed-complex form** — 18% of the symbol runs
   scalar-lane `d`-register FMA chains while cheap arms use packed `.2d`;
   whether packing beats dense scalar FMA on this core is an open measurement,
   and it is the only item here that touches arithmetic shape (REL_TOL gate if
   it reassociates).

Gate: `validate_helas_mg` byte-equality for items 1–3, `eval_strategies` medians
with host fingerprint for all; the honest ceiling for 1–3 combined is bounded by
the ~50% non-arithmetic share, discounted by whatever the loads/stalls overlap.

**E2 — MERGED 2026-08-04 (`54d666f`, merge `d2d7520`): measured results.**
Item 1 landed; items 2–4 implemented, measured, and deliberately not landed.

- **Gate-name correction**: `validate_helas_mg` no longer exists; the successor
  is `tests/amplitude_oracle.rs`. The session gated on it (20/20) plus a
  stronger oracle: byte comparison of all 100 banked category row files
  (durations stripped) — identical md5 before/after, so the change is
  bit-for-bit at the layer's own resolution. Full validate green, census
  unchanged.
- **Item 1 (arena hoisting)**: header reloads 143 → 20 (the whole mechanism;
  insns 2041 → 2064, still one `br`). `eval_m2/forward` criterion geomean
  **−4.23%** (14/14 rows improve; two independent measurement designs agree to
  0.01 pp under heavy sibling-session contention). `mg_perf_compare.sh`
  MATRIX1 geomean **1.29× → 1.23×**, no row worse.
- **Item 2 (dispatch replication): dead — +7.7% geomean.** True threaded
  dispatch is not expressible in safe Rust (no computed goto; LLVM won't
  tail-duplicate the jump-table block); the 2-way-unroll approximation grows
  the function to 4183 insns, 404 header reloads, and evicts six kernels out
  of line. E1's opcode-blocked order now stands alone against the single-`br`
  dispatch, whose mispredicting `ldrh` (10.87%) is untouched.
- **Item 3 (force-inline sret kernels): +0.18% alone, −2.5 pp worse on top of
  item 1** — the 64-byte round-trips were store-to-load-forwarded; +25% code
  growth costs more than they did. Dropped.
- **E0-claim correction: the ~20% bounds-check budget is intact** — 108
  `panic_bounds_check` sites before and after; hoisting did not make them
  hoistable because the indices come from the instruction stream.
- **Item 4 reframed**: "packed-complex GammaVout" is not a source-level lever
  in safe Rust (packing is a codegen outcome). Found instead: in
  `left_current`/`right_current` (`helas/repr/lorentz.rs:771,805`) the
  `cmul_add` fusion blocks CSE against the sibling `cmul`, so each of four
  spinor products is computed twice. Naming them once measured **−1.0 pp
  further geomean** (up to −2.8 pp on fermion-rich rows, exactly 0 on
  `gg_to_gg` — physics-consistent), `amplitude_oracle` 20/20, worst deviation
  unchanged at 1.776e-11. **Reassociating** and touches the public
  `SpinorRepr` vocabulary → held for its own REL_TOL-gated session (patch
  stashed in the session record).
- For E1: measure against `54d666f`; `Instr` is 20 B with `loc[i]` in a second
  stream — folding `loc` into the instruction encoding is adjacent to E1's
  linearization scope. Contention protocol that worked: per-config prebuilt
  bench binaries run round-robin, min over rounds.

### E3 — chain-B draw work-sharing

On live-draw rows each point pays one `eval_amp2` + one `set_alpha_s` *before* the
`eval_m2` on the same momenta — ≈1.0 µs and ~21% of the per-point budget on
`gu_to_epemu`/`gux_to_epemux` (note 30 §6). The draw is not removable (it is what
makes σ agree; note 29 chain B), but the two evaluations share externals, the
momentum pool, and every helicity-independent current. Session question: can
`eval_amp2` run as a prefix/byproduct of `eval_m2` (or cache its arena state for
reuse) without changing the draw's value stream? Gate: the draw is a pure function
of `(channel, u)` — the σ gates plus the `AMP2_c`-share partition census must be
byte-stable; any change to *what* is drawn (not just when it is computed) is out
of scope.

### E4 (stretch) — accept/reject allocator traffic

The unweighting profile is 18.4% allocator + 5.1% `BTreeMap`;
`ScaleChoice::clustered` heap-allocates its beam–leg candidate list per event
(backlog, `coupling/scales.rs`) and kT clustering re-derives the scale per trial.
First cut: per-event allocations → reused scratch. Only if a session slot frees up;
the samples category is half the size of integrals in wall time.

## 4. What this sprint does not do

- No egraph/extraction work (still NO-GO per note 15 §4.1's three prerequisites).
- No SIMD-lane or per-helicity stratified-parallel axes (backlog items (b), (e))
  beyond what I3/I4 build; they stay catalogued. Note 30's per-point *ratios*
  remain single-thread numbers — I3 changes wall time, not per-point cost, and
  every throughput comparison keeps its CPU-time denominator.
- No 2→6 σ row un-skipping, no `mg-single-helicity-bench` (still no consumer).
- No cross-host claims; every number is M3-Max-relative against note 30's tables.

## 5. Sequencing, gates, bookkeeping

- **Order**: dispatch **I1** and **P1** first (independent, biggest levers);
  **I3** (parallel integrate + `-j`) and **E2** next (both fully scoped, both
  independent of everything else); **E1** and **I2** when dev slots free;
  **I4** last in Track I (hard prerequisites: I1 for a trustworthy stopping
  error, I3 for the parallel substrate); **E3** after I1 lands (its probe rows'
  budgets must be stable while measuring); E4 stretch. E2 item 2 and E1's
  opcode-blocked order overlap — whichever session runs second measures against
  the other's result, not the pre-sprint baseline.
- **Per-session gates**: `pixi run --skip-deps validate` green with every enforced
  cell keeping its status; σ-touching sessions add the ≥5-seed sweep at two
  budgets; evaluator sessions run `validate_helas_mg` (bit-for-bit where the
  session claims bit-for-bit) and quote `eval_strategies` medians with the host
  fingerprint **plus a `scripts/mg_perf_compare.sh` before/after** (the per-point
  MATRIX1 comparison, note 15 §2.4 — the direct amplitude-component measure the
  baseline study omitted; applies to E1, E2, E3); PDF session runs
  `validate_pdf_grid` + the hadronic σ gates and states the worst oracle
  deviation before/after.
- **Measurement honesty**: layer-level claims via report `duration_s` against
  note 30 §3.2 (0.8%/3.4% noise floor — nothing sub-1% is claimable there);
  kernel-level claims via criterion; every table carries its command.
- **Close-out**: re-run the note-30 instrumented validate pass and the
  `time_stages.py` MG pass in one sitting on the same host, append the
  before/after table here, and add a sprint-level `scripts/mg_perf_compare.sh`
  before/after table (note 30 §5.3 explicitly disclaims being a substitute for
  it); update `TODO.md` and the backlog entries this sprint retires.
