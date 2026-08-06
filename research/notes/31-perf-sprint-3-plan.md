# 31 — Performance sprint 3: integration budget, PDF interpolation, evaluator schedule (plan)

**Status:** PLAN, drafted 2026-08-04 against the note-30 baseline (per-stage timings,
samply profiles, host block — all taken on `main` @ `45a7d62`, M3 Max). Predecessor
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

**I1 — MERGED 2026-08-04 (`ea58ab9`, merge `e99b05c`): measured results.**

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

**I2 — MERGED 2026-08-04 (`ad54c8f`, merge `152efb1`): the premise was false.**

- **The maxima never converge — there is no "own schedule".** On the llj
  grids, `Σⱼ w_maxⱼ ∝ n^0.508` over 2.4 decades of scan budget (10³–2.56·10⁵
  draws/channel, no plateau) and the σ-share above the maxima falls only as
  `n^−0.455`: one statement — a Pareto weight tail of index ≈ 2, confirmed by
  a Hill estimator (α = 2.08–2.40 on the twelve σ-carrying channels, which
  also own the maxima; the heavy-tailed tiny channels never do). The budget
  buys a point on an acceptance-vs-overweight-tail curve, nothing more.
- **Delivered**: `ScanBudget::{PerChannel(n), IntegrationShare}` +
  `--scan-points <N|share>` on `generate`, plus a `scan:` stdout line.
  **Default `share` is bit-identical to the pre-change binary** (artifact md5
  equality on llj two seeds + dy13; a live `--scan-points 20000` negative
  control differs). Nothing banked moved; census character-identical.
- **Allocation is a weak lever**: flat-vs-share sit on the same
  overweight-vs-efficiency curve within the 5-stream spread (±7%); at equal
  draws flat is +21% acceptance / +10% overweight share.
- **For I4**: the σ-carrying channels' weight-tail index ≈ 2.1–2.4 sits at
  the variance-existence boundary — empirical variances (the `N_j ∝ α_j σ_j`
  input AND the χ²/dof stopping guard) converge slowly and bias low in
  exactly the regime I1 showed the quoted error underestimates. Size the
  χ²/dof precondition with that in mind.
- **The real lever is the maxima rule, not the budget** (backlog): MadGraph's
  `unwgt.f` sets the maximum from a percentile and re-normalises, capping
  `Σⱼ w_maxⱼ` instead of chasing an extremum. That is the session that would
  move llj's ~5e-3 σ-share. Also free and provably inert: `Unweighter::scan`
  is a pure per-channel function on its own stream — a rayon `par_iter` over
  channels cannot move a number (relevant only for large `--scan-points`).

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

**I3 — MERGED 2026-08-04 (`b612253`, merge `17fd612`): measured results.**

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
  vs the pre-change serial binary at `7938409`, on partonic, `dy13_default`
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

**I4 — MERGED 2026-08-04 (`a62df73`, merge `bd16311`): measured results.**

- **Brief corrections**: the hard split already existed (`adapt_grids` was
  per-channel deterministic with a 512 floor; the multinomial survives only in
  the undivided comparison estimator and the α-survey — neither the production
  σ path); `α_j` is already inside `value_in_channel`, so the implemented rule
  is `N_j ∝ s_j^term` (literal `α_j σ_j` would double-count); and convergence
  mode is **opt-in `--target-rel`, not the CLI default** — a default flip
  changes `integrate`'s default artifact bytes, which CLI gates and
  `generate_samples.sh` pin (small follow-up: flip + re-pin, see backlog).
- **Design**: `budget.rs` owns allocation (`ByAlpha`/`Neyman`; iterative floor
  pinning, exact not clamped) + stopping (quoted error × √max(1, χ²/dof)
  **per channel**; `min_iters ≥ warmup+2`; `Budget::Target` panics on
  `InverseVariance`, pinned by `should_panic`); `vegas::adapt_blocks_iteration`
  runs every channel's iteration in one rayon region keyed `(channel, chunk)`,
  preserving `adapt_parallel_seeded`'s seek + point-order contracts
  (per-channel `first_point` is a running total, enabling varying `N_j`).
- **Gates**: fixed-budget bit-for-bit (artifact md5s vs the `3ea0402` binary;
  inert across `-j 1/5/16`, chunk sizes and bases); census unchanged;
  807 workspace tests. Convergence calibration: 2 processes × 2 targets ×
  2 allocations × 8 seeds — **64/64 met target**, seed χ²/dof 0.44–1.14 (all
  inside the 7-dof 95% band), sd/quoted ≤ 1.07 (llj over-covers ~25%,
  conservative).
- **Matched accuracy vs MG (note 30 §5.3 denominators)**: llj at MG's banked
  accuracy = **CPU parity** (8.2–9.9 CPU-s vs 9.93); dy13 = **4.2–4.5× less
  CPU** (19.5–22.3 vs 92.9). Vs our own pinned budgets, llj reaches
  better-calibrated accuracy in ~2× fewer points.
- **The offline Neyman prediction (1.00–1.22×, would have killed the lever)
  was wrong for targets**: the KP α-survey already allocates near-Neyman for
  a *fixed* budget, but under a live target the win is 2.18× fewer
  evaluations — the mechanism is feeding the starved channels whose χ²/dof
  blowup (one floor channel hit 23) inflates the stopping scale factor.
  `--allocate` defaults to `neyman` under a target, `by-alpha` otherwise.
- **Scheduling**: −7.2% wall at byte-identical CPU on llj fixed budget
  (6.30× → 6.79× at `-j 16`; 3 dead-stable rounds; the identical-CPU
  signature is the discriminator under load). Chunk size stays untuned — the
  scan was load-dominated and a knob was not changed on unresolvable
  evidence; inertness (identical md5 at every setting) is what survived.
- **For close-out**: wall times here were taken under load — CPU-s and point
  counts are the bankable columns; the α-survey is now ~27% of a fixed-budget
  llj `-j 16` run and remains the named serial lever.

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

### 2.4 P1 — MERGED 2026-08-04 (`865828a`, merge `c999c16`): measured results

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
  `c999c16`, not note 30 §3.2's table.
- Ops note for future worktree sessions: `git worktree add` leaves the
  `research/refs/mg5amcnlo` submodule checkout empty, which aborts
  `cargo test --workspace` before most gates run; COW-copy the checkout in and
  point its `.git` file at the shared module.

**P1b follow-up — MERGED 2026-08-04 (`5f953b9`, merge `be42df2`)**, user-directed:
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

**E1 — MERGED 2026-08-04 (`94ed907`, merge `b9bb758`): verdict GO** — study
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

**E1b — MERGED 2026-08-04 (`052a00e`, merge `01ba9cd`): the pass is production.**

- `Program::build` defaults to op-blocked-within-ASAP-levels. One-pass without
  rule duplication: the lowering `match` was split out as `lower_node` and run
  against a null slot map to read each node's true `Instr` discriminant — the
  grouping key structurally cannot drift from the variant it groups.
- **Guardrail**: 16 MiB arena-footprint limit (allocated bytes at f64,
  lane-independent), falling back **only if interning order is actually
  smaller**. Measured footprints: pruned 2→6 0.31→0.46 MB, unpruned
  3.66→5.84 MB (E1's 2.84→4.64 figure was `live_bytes_peak`, a different and
  also-correct metric; the guardrail uses allocated bytes) — the limit is
  ~36× the largest production program; it fires on nothing measured.
- **Bit-for-bit**: all 100 banked category row files digest-identical
  (`e2c4299a…` before and after, per-file diff count 0) — including every σ
  row, so `eval_amp2`'s unexpanded program is byte-stable too. Bit-identity
  tests pass with production as default (anti-vacuity: order ≠ arena).
  Census character-identical.
- **Performance**: eval geomean **−17.34%** (14/14 rows; reproduces E1's
  −17.9%/18-row study number inside noise). **MATRIX1 geomean 1.21× → 1.00×**;
  processes beating MadGraph 3 → **8 of 14**; the 2→6 rows 1.14×→0.89× and
  1.34×→1.06×. Compile cost of the pass: +1.06 ms = **+0.2%** of evaluator
  construction on the biggest production program (+4.8% on the study-only
  unpruned 2→6).
- Row `duration_s` from this sitting is not bankable (same-build back-to-back
  spread reached −57% under sibling load; row *contents* byte-identical) —
  close-out re-measures on a quiet host.
- **E3 re-baselines against `052a00e`**: note 30 §6's "≈1.0 µs / ~21%"
  chain-B draw cost predates a ~17% evaluator win and needs re-measuring
  before E3 sizes its payoff.

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

**E2 — MERGED 2026-08-04 (`82b68d1`, merge `9ac8858`): measured results.**
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
- For E1: measure against `82b68d1`; `Instr` is 20 B with `loc[i]` in a second
  stream — folding `loc` into the instruction encoding is adjacent to E1's
  linearization scope. Contention protocol that worked: per-config prebuilt
  bench binaries run round-robin, min over rounds.

**E2b (item-4 follow-up) — MERGED 2026-08-05 (`31640a8`, merge `854c049`).**
The fermion-current CSE landed: the four spinor products in
`left_current`/`right_current` named once (`cmul_add` fusion had blocked CSE
of the sibling `cmul`). **Reassociating**: worst oracle deviation
1.7760e-11 → 1.7761e-11, every row pass→pass, census unchanged, no bar
touched. Win survives the scheduling pass: fermion rows −2.4 to −3.6%
(8-round alternating control; the code-identical `gg_to_gg` control at
+0.03%), MATRIX1 geomean **1.00× → 0.98×** — the evaluator fleet is now
marginally ahead of MadGraph. (Session hit two API drops + a watchdog stall;
the manager assembled the final gate evidence from its on-disk logs and
verified the branch directly.)

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

**E3 — MERGED 2026-08-05 (`f3d6e8b`, merge `cf2d489`): measured results.**

- **Re-baselined draw cost post-E1b**: ≈870 ns / ≈18% on the partonic
  live-draw rows (was ≈1 µs / ≈21% in note 30 §6); `llj_dyn` still a few
  per cent.
- **The plan's prefix design is a NO-GO by measurement**: only ≤28% of the
  live-draw rows' program nodes are αs-invariant (and they are the cheap
  ones), and a stream partition would perturb the op-blocked order worth
  −17.3%. Premise correction: on strong-coupling drawing rows the
  per-event *clustered* prescription exists precisely to move the coupling
  between `AMP2` and `|M|²`, so the two evaluations share only momenta —
  the absorbable case is Drell–Yan.
- **Landed instead: arena-reuse cache** — `fill_token` + bit-compared
  momenta stamp on `ScratchSpace`; all six helicity-summed read-outs reuse
  a matching fill; any writer or `set_pools`/`set_alpha_s` retires the
  stamp; per-thread token blocks keep I3's parallel path uncontended.
  **Order-preserving, bit-for-bit**: 100 row files digest-identical
  (`7e1cae69…` both sides), census unchanged; three reuse tests with an
  anti-vacuity fills counter (a pass provably *skipped*, not merely
  agreeing) including a 4-thread cross-thread isolation test.
- **Wins**: the biggest beneficiary is `select_event` (4 passes → 1):
  event readout −37.5%/−36.2% on `gu_to_epemu`/`gux_to_epemux`;
  `pp_to_ll` draw cost −83% (193.6 → 32.5 ns/pt), total −8.4%.
- **Known cost, accepted**: `pp_to_llj_dyn` +1.0–1.6% (consistently
  signed) — the shape where the cache can never hit still pays the stamp;
  the mitigation would trade away the exactness that makes this safe.
- **Handed back, session-worthy (E3b candidate)**: `FixedBeamIntegrand`
  runs the draw *before* the cut — 22% of `gu/gux_to_epemu(x)` points pay
  ~190 ns of provably dead draw work; `ProtonIntegrand::shape` already
  cuts first. Also corrects note 30 §6's "points the cuts reject return
  before the draw" (true hadronic, false partonic) — fix the doc comments
  with it.

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

---

## 6. Close-out: the sprint measured end to end

**Status:** measurement record, taken 2026-08-05 on `perf3-closeout` @ `62d78e4`
— the merged sprint tree, all eleven sessions in — on the same M3 Max note 30
was taken on. Every table carries the command that produced it and the load
average at its own start.

### 6.1 A correction to the close-out protocol

The close-out was briefed to take the per-row table under `RAYON_NUM_THREADS=1`,
reasoning that note 30's rows were single-threaded and pinning rayon to one
thread restores that condition. **The premise is right and the prescription is
wrong**, and it is worth recording because it will catch anyone who compares a
row duration across the I3 boundary.

The premise: before the sprint nothing in the library reached rayon on a gate
path. The only call site in `vibegraph-lib/src` at `e951045` is
`run_iter_parallel`, the private helper of the never-reached `adapt_parallel` —
note 30 §7's "~16 rayon workers parked in `__psynch_cvwait`" is the same fact
seen from the profile side.

The prescription: after I3/I4 every σ row's integrand submits to the **global**
rayon pool, and `validate.sh` runs the gates under the harness's default test
parallelism. With one global worker every concurrently-running row funnels
through it, and each row's `Stopwatch` charges itself the whole cohort's wall
time. The signature is unmistakable — every hadronic integrals row lands at the
same ~305–320 s whether it spends 4.32M, 4.5M or 9M points:

| row | `RAYON_NUM_THREADS=1`, cohort | points |
|---|--:|--:|
| `pp_to_bb_fixed` | 305.1 s | 9 000 000 |
| `pp_to_bb` | 308.8 s | 9 000 000 |
| `pp_to_bb_qcd2` | 312.2 s | 9 000 000 |
| `pp_to_jj` | 315.9 s | 9 000 000 |
| `pp_to_ll` | 319.3 s | 4 320 000 |
| `pp_to_llj_dyn` | 310.3 s | 4 500 000 |
| `pp_to_llj_fixed` | 307.3 s | 4 500 000 |
| `pp_to_llj` | 390.6 s | 4 500 000 |

Isolating one row settles it. `sigma_bb_fixed_scale_vs_mg` alone, same binary,
same 9M points:

```
RAYON_NUM_THREADS=$rt cargo test -p vibegraph-lib --profile release-debug \
  --features extended-validation --test validate_hadronic -- \
  --nocapture --test-threads=1 sigma_bb_fixed_scale_vs_mg
```

| `pp_to_bb_fixed` integrals | duration |
|---|--:|
| in the cohort, `RAYON_NUM_THREADS=1` | 305.1 s |
| alone, `RAYON_NUM_THREADS=1` | **37.1 s** |
| alone, `RAYON_NUM_THREADS=16` | **11.1 s** |
| note 30 §3.2 | 38.59 s |

So the 8.2× was cross-row contention on one worker, not work. The corrected
protocol pins the harness as well as the pool — `RUST_TEST_THREADS=1
RAYON_NUM_THREADS=1`, one row at a time on one thread.

One trap inside that correction is worth naming, because it nearly went into
this note as a finding: **a row measured on its own is not the same measurement
as the same row inside a pass.** Alone, `pp_to_bb_fixed` reads 37.1 s; inside
§6.3's pass it reads **26.0 s**. The difference is one-time process setup — the
interned SM, and on a hadronic row the PDF grid — which an isolated run charges
to the only row present while a full pass charges it to whichever row runs
first. So 37.1 s sitting 3.9% from note 30's 38.59 s is a coincidence between
two different contaminations, not evidence that the protocols agree, and the
same caveat caps the isolated `37.1 → 11.1 s` thread-count ratio at "≥3.3×, both
arms carrying setup" rather than a clean measurement of what I3/I4 bought. The
clean parallel numbers are §6.7's, taken through the CLI with no harness in the
way.

What licenses §6.3 against note 30 §3.2 is therefore narrower, and is argued
there rather than here: on the two categories carrying 97.9% of the layer's
timed work, each row's duration is dominated by its own integration or
generation work, which both protocols measure alike.

### 6.2 What is and is not comparable

- **Per-row `duration_s`: comparable**, on the corrected protocol, subject to the
  few-per-cent caveat above.
- **Elapsed wall: comparable for the *default* command, and then only as a lower
  bound on the gain.** §6.5's 691 s → 391 s runs note 30's exact invocation on
  both sides, so it is like-for-like — but the suite grew this sprint (`#[test]`
  861 → 905, `#[ignore]` 32 → 34 between `e951045` and `62d78e4`: +42
  newly-running tests, several deliberately expensive — E1's alternative-order
  bit-identity runs, I3's thread-count identity runs, I4's calibration), so the
  391 s buys strictly more work than the 691 s did. The wall of the *corrected
  per-row* protocol (§6.3) is comparable to nothing in note 30, because that
  protocol serialises the harness; it is reported per binary and used for
  nothing else.
- **MadGraph's side: untouched by the sprint.** Nothing here wrote to the
  reference bank, so §6.6's job is to show the *host* still reproduces note 30
  §4.2 — which is what licenses the our-side comparison at all.
- **`diagrams` is a protocol artifact; `amplitudes` is not.** These two are worth
  separating carefully, because a partial reading of the first close-out pass
  suggested both had moved and neither had. Measured properly, category totals:

  | category | note 30 | close-out, default command | close-out, one row at a time |
  |---|--:|--:|--:|
  | `diagrams` | 14.2 s (26 rows) | **13.59 s** | **1.29 s** |
  | `amplitudes` | 3.0 s (20 rows) | **2.59 s** | **2.24 s** |

  Under note 30's own command both reproduce. But `diagrams` collapses **10.5×**
  when the rows are run one at a time, and its residue is concentrated in exactly
  the two rows that have real enumeration work to do (`bbx_to_ccx` 0.62 s,
  `uux_to_ccx` 0.59 s; every other row ≤ 0.05 s). So note 30 §3.2's reading of
  that column — "every `diagrams` row costs ~0.51–0.68 s whatever the process,
  because each trial re-loads the interned SM" — has the mechanism inverted:
  `sm_model()` is a **process-wide interned** model, so a trial does not re-load
  it; under the harness's default parallelism the 26 rows race its lazy
  initialisation and each `Stopwatch` spans the contention. Run sequentially only
  the first pays, and the column becomes what it should always have been, a
  per-process enumeration cost.

  `amplitudes` shows no such protocol dependence (2.59 s vs 2.24 s) and its shape
  is honest work: the two 2→6 rows carry 1.0–1.1 s of the total and every other
  row is ≤ 0.02 s. Note 30's *numbers* for it are right; only its explanation is
  wrong, since `amplitude_oracle::measure` does run enumeration and
  `AmplitudeEvaluator::compile` per row rather than "never build an evaluator" —
  which is precisely why the 2→6 rows cost fifty times what a 2→2 does. The file
  is untouched since note 30's own tree (`git log 45a7d62..62d78e4 --
  vibegraph-lib/tests/amplitude_oracle.rs` shows only the instrumentation commit
  note 30 itself measured), so this is a wording defect in note 30, not a change.

  Either way the two categories together are ~17 s of note 30's ~1 700 s, so
  nothing rests on them; §6.3 compares **integrals and samples**, which note 30
  §3.2 itself puts at **97.9% of the layer's timed work**.

### 6.3 Per-row durations: integrals and samples

```
RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 cargo test -p vibegraph-lib \
  --profile release-debug --features extended-validation --test <target> \
  -- --nocapture --test-threads=1
```

run for the six targets that write report rows — `validate_madgraph_diagrams`,
`amplitude_oracle`, `validate_sigma`, `validate_hadronic`, `validate_samples`
(vibegraph-lib) and `validate_samples_proton` (vibegraph-cli, `-p vibegraph`) —
then the collator. Running only these six is deliberate: the protocol also pins
the CLI subprocesses the *non*-row tests spawn to one thread, which costs over an
hour of wall clock measuring nothing. **No census is quoted from this pass** — it
ran a subset of the layer, so its report has missing cells by construction; the
census is §6.5's, from the full default-parallel run. Per-binary wall:
diagrams 3 s, amplitudes 3 s, `validate_sigma` 38 s, `validate_hadronic` 357 s,
`validate_samples` 133 s. Host load at start 3.42.

Two effects the table keeps apart, and they must not be added:

- **Budget.** I1 re-pinned `LLJ_NEVAL` 300k → 150k and `pp_to_llj`'s recarded row
  600k → 150k, so the three llj rows spend a quarter to a half of note 30's
  points. The points column makes that visible rather than folded in.
- **Per-point cost.** Every other row spends *exactly* the points note 30 spent,
  so its whole move is per-point work: P1's all-flavour PDF kernel on the
  hadronic rows, and E1b's op-blocked schedule + E2's arena hoisting + E2b's
  current CSE + E3's arena-reuse cache everywhere.

#### integrals

| row | n30 pts | pts | n30 s | close-out s | Δ |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 180,000 | same | 0.17 | 0.14 | -17.2% |
| `ee_to_ee` | 800,000 | same | 0.96 | 0.78 | -18.5% |
| `ee_to_ttx` | 180,000 | same | 0.23 | 0.18 | -22.9% |
| `ee_to_wpwm` | 320,000 | same | 0.80 | 0.59 | -26.5% |
| `ee_to_zh` | 180,000 | same | 0.13 | 0.11 | -14.3% |
| `uux_to_mumu` | 180,000 | same | 0.16 | 0.15 | -5.9% |
| `uux_to_uux` | 240,000 | same | 1.34 | 1.25 | -6.6% |
| `gg_to_ttx` | 480,000 | same | 3.01 | 2.67 | -11.2% |
| `gg_to_gg` | 240,000 | same | 2.47 | 2.07 | -16.1% |
| `ee_to_mumua` | 640,000 | same | 1.28 | 1.11 | -13.0% |
| `ee_to_tatah` | 480,000 | same | 0.85 | 0.67 | -21.4% |
| `uux_to_epemg` | 480,000 | same | 3.16 | 2.93 | -7.3% |
| `ddx_to_epemg` | 480,000 | same | 3.40 | 2.89 | -15.1% |
| `gu_to_epemu` | 480,000 | same | 3.35 | 3.08 | -8.2% |
| `gux_to_epemux` | 480,000 | same | 3.32 | 3.07 | -7.5% |
| `ee_to_mumu_tata_qcd0` | 800,000 | same | 5.83 | 4.54 | -22.1% |
| `ud_to_epemud_qcd0` | 960,000 | same | 9.40 | 7.57 | -19.5% |
| `pp_to_ll` | 8,640,000 | same | 15.82 | 11.25 | -28.9% |
| `pp_to_bb` | 9,000,000 | same | 62.94 | 46.52 | -26.1% |
| `pp_to_bb_qcd2` | 9,000,000 | same | 78.45 | 56.99 | -27.4% |
| `pp_to_bb_fixed` | 9,000,000 | same | 38.59 | 26.04 | -32.5% |
| `pp_to_jj` | 9,000,000 | same | 174.81 | 86.17 | -50.7% |
| `pp_to_ll_scalefact2` | 9,000,000 | same | 41.78 | 26.01 | -37.8% |
| `pp_to_llj_fixed` | 9,000,000 | 4,500,000 | 87.87 | 27.75 | -68.4% |
| `pp_to_llj` | 18,000,000 | 4,500,000 | 186.19 | 34.40 | -81.5% |
| `pp_to_llj_dyn` | 9,000,000 | 4,500,000 | 116.32 | 40.85 | -64.9% |
| **total** | | | **842.6** | **389.8** | **-53.7%** |

The note-30 column sums to 842.6 s, reproducing §3.2's own integrals total
exactly, which is the check that the rows were transcribed right.

**The partonic block is the cleanest read on the evaluator stack alone** — no PDF
in the integrand, no budget change, 17 rows, every one improved, **39.9 s →
33.8 s (−15.2%)**, spread −5.9% to −26.5%. That is what a −17.3% evaluator win
(E1b) plus E2/E2b/E3 looks like once diluted by the phase-space map, the cuts and
the clustering, which note 30 §7.1 put at roughly half the partonic integrand.

**The hadronic block is 802.8 s → 356.0 s (−55.7%)**, and it splits cleanly. At
unchanged budget: `pp_to_jj` −50.7%, `pp_to_ll_scalefact2` −37.8%,
`pp_to_bb_fixed` −32.5%, `pp_to_ll` −28.9%, `pp_to_bb_qcd2` −27.4%, `pp_to_bb`
−26.1% — all per-point, and all far above the partonic rows' −15%, which is the
PDF kernel showing up exactly where note 30's profiles said the PDF was
(14.5–19.4% of self time on proton paths, nowhere else). At reduced budget the
three llj rows fall −64.9% to −81.5%, of which roughly half is I1's re-pin.

#### samples: this protocol does not measure that category comparably

The same construction applied to `samples` does not hold together, and the honest
move is to say so rather than publish the table. Three protocols, one row
(`pp_to_llj_dyn` samples), same tree except where noted:

| protocol | `pp_to_llj_dyn` samples |
|---|--:|
| note 30: default command, pre-sprint tree | 127.78 s |
| `RAYON_NUM_THREADS=1`, default test parallelism | 93.2 s |
| `RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1`, target run alone | 210.41 s |

The middle figure comes from the aborted cohort pass of §6.1 — a pass whose
*integrals* rows are invalid for the reason given there, but whose samples rows
are untouched by it, because accept/reject stayed serial through the sprint (I3
says so explicitly) and so never reaches the rayon pool.

Serialising the harness should make a row faster, never 2.3× slower, so this is
not contention. Two candidates, neither established here: these rows drive
`vibegraph generate` as a **subprocess**, so the row times a process launch plus
its work rather than in-process work; and a lone thread on a 12P+4E hybrid can be
placed on an efficiency core, which note 30 §1 already flags as unquantified
("no affinity is set on either side … a real uncertainty on a hybrid CPU"). The
partonic samples rows — in-process, spawning nothing — behave under the same
protocol: all 17 improved, −6.5% to −42.4%, **128.2 s → 106.7 s (−16.8%)**. That
contrast is what makes the subprocess-driven rows the suspect half.

The layer-level samples comparison is therefore taken from §6.5, which runs note
30's exact command and is like-for-like; no samples claim is made from this
protocol beyond the partonic block.

### 6.4 Throughput against MadGraph, recomputed

Note 30 §5.3's construction, unchanged: our points are `seeds × neval × niter`
from each row's own report record; MadGraph's are field 4 of
`SubProcesses/results.dat` divided by that file's `<cumulated_time>`, which is the
**summed CPU seconds of its Fortran jobs** and so takes the 16-way job farm out of
the comparison. Our column is single-thread wall time, from §6.3.

MG's two columns are carried over from note 30 verbatim, because this sprint did
not touch the reference bank. They were spot-checked against it rather than
trusted: `gg_to_gg` reads 434 257 points and `<cumulated_time> 7.098467` and
`pp_to_llj` 176 272 and `10.939257`, matching note 30's 434 257/7.0 and
176 272/11.0. As a second check on the arithmetic, recomputing the *before* column
from note 30's own numbers returns **6.84×**, against the 6.8× note 30 quotes.

| row | our kpts/s (n30) | our kpts/s (now) | MG kpts/CPU-s | ours/MG (n30) | ours/MG (now) |
|---|--:|--:|--:|--:|--:|
| `ee_to_mumu` | 1059 | 1278 | 34 | 30.9x | 37.3x |
| `ee_to_ee` | 833 | 1023 | 42 | 19.7x | 24.1x |
| `ee_to_ttx` | 783 | 1016 | 40 | 19.5x | 25.3x |
| `ee_to_wpwm` | 400 | 544 | 39 | 10.1x | 13.8x |
| `ee_to_zh` | 1385 | 1615 | 47 | 29.6x | 34.6x |
| `uux_to_mumu` | 1125 | 1195 | 40 | 28.2x | 29.9x |
| `uux_to_uux` | 179 | 192 | 49 | 3.7x | 3.9x |
| `gg_to_ttx` | 159 | 180 | 49 | 3.3x | 3.7x |
| `gg_to_gg` | 97 | 116 | 62 | 1.6x | 1.9x |
| `ee_to_mumua` | 500 | 574 | 54 | 9.3x | 10.7x |
| `ee_to_tatah` | 565 | 719 | 36 | 15.7x | 19.9x |
| `uux_to_epemg` | 152 | 164 | 66 | 2.3x | 2.5x |
| `ddx_to_epemg` | 141 | 166 | 63 | 2.3x | 2.7x |
| `gu_to_epemu` | 143 | 156 | 31 | 4.7x | 5.1x |
| `gux_to_epemux` | 145 | 156 | 29 | 4.9x | 5.3x |
| `ee_to_mumu_tata_qcd0` | 137 | 176 | 13 | 10.7x | 13.7x |
| `ud_to_epemud_qcd0` | 102 | 127 | 22 | 4.6x | 5.8x |
| `pp_to_ll` | 546 | 768 | 33 | 16.6x | 23.3x |
| `pp_to_bb` | 143 | 193 | 48 | 3.0x | 4.0x |
| `pp_to_bb_qcd2` | 115 | 158 | 46 | 2.5x | 3.5x |
| `pp_to_bb_fixed` | 233 | 346 | 28 | 8.4x | 12.5x |
| `pp_to_jj` | 51 | 104 | 23 | 2.3x | 4.6x |
| `pp_to_ll_scalefact2` | 215 | 346 | 27 | 8.0x | 12.9x |
| `pp_to_llj_fixed` | 102 | 162 | 18 | 5.8x | 9.1x |
| `pp_to_llj` | 97 | 131 | 16 | 6.0x | 8.2x |
| `pp_to_llj_dyn` | 77 | 110 | 16 | 4.7x | 6.8x |
| **geomean over 26 rows** | | | | **6.84×** | **8.76×** |

**Geometric mean 6.84× → 8.76× over the same 26 rows.**

What this column can and cannot see: it is points per second, so I1's budget
re-pin is invisible here by construction — halving llj's points halves §6.3's
duration and leaves throughput alone. Everything in this table is per-point cost.

The shape note 30 called "the finding" has moved where it matters. Its complaint
was that our advantage was 20–30× on the cheapest leptonic rows and "collapses to
1.6–6× on the ones a sprint cares about". Those rows now read: `gg_to_gg`
1.6× → **1.9×**, `uux_to_epemg` 2.3× → **2.5×**, `ddx_to_epemg` 2.3× → **2.7×**,
`pp_to_jj` 2.3× → **4.6×**, `pp_to_llj_dyn` 4.7× → **6.8×**, `pp_to_llj`
6.0× → **8.2×**, `pp_to_llj_fixed` 5.8× → **9.1×**. The floor is still
`gg_to_gg` — a pure-gluon 2→2 with no PDF work to win back and the densest colour
algebra in the census — and it moved least, which is the consistency check on the
attribution: the row with nothing for P1 to speed up gained only what the
evaluator sessions gave it.

Note 30's two caveats stand unchanged and are not re-litigated here: whether
MadEvent's `results.dat` point count includes the survey pass was never
established, so a systematic factor of order unity sits on every MG column; and
our per-point work is not MadGraph's per-point work. The ratio is integrand
throughput, not a matrix-element ratio — §6.8 is the matrix-element ratio.

### 6.5 The layer as a user runs it: wall time and census

```
$ pixi run --skip-deps validate      # note 30 §3.1's exact command, nothing pinned
VALIDATE_EXIT=0
ELAPSED_S=391
29 rows × 4 categories = 116 cells: 98 measured in this layer (96 ✅, 2 ⚠️, 4 ⏳, 14 — / uncovered).
```

**691 s → 391 s (6 min 31 s), −43.4%**, on the identical command, and the census
is cell-for-cell what note 30 recorded and what the manager's independent gate run
on this tree recorded: **98 measured, 96 ✅ / 2 ⚠️ / 4 ⏳**. Load average at start
3.1. This is the one figure that is like-for-like by construction, and it is the
only place the sprint's parallelism counts, since from I3 onward the rows use the
machine internally.

It also *understates* the sprint, because the suite grew: `#[test]` 861 → 905 and
`#[ignore]` 32 → 34 between `e951045` and `62d78e4`, so 391 s buys 42 more running
tests than 691 s did.

**Its per-row durations are not used for per-row claims, in this note or note 30.**
Note 30 §2 says why in its own words — "`duration_s` is not a benchmark … their
durations overlap" — and this pass shows the failure mode plainly: eight hadronic
integrals rows land within 31.15–36.85 s of each other (`pp_to_bb` 31.65,
`pp_to_bb_qcd2` 31.15, `pp_to_jj` 31.15, `pp_to_ll_scalefact2` 32.34,
`pp_to_llj_fixed` 32.34, `pp_to_llj_dyn` 32.34, `pp_to_bb_fixed` 32.53,
`pp_to_llj` 36.85) despite spending between 4.5M and 9M points — the same
everything-finishes-together signature §6.1 diagnosed, here from ordinary test
parallelism rather than from a starved thread pool. `pp_to_ll` reads 64.31 s
against note 30's 15.82 s for the same reason and is not a regression: run alone
(§6.3) that row takes 11.25 s.

For the record, the category totals as the collator sums them — overlapping spans
on both sides, so a loose comparison only: integrals **842.6 s → 336.7 s**,
samples **840.1 s → 585.7 s**.

### 6.6 MadGraph's side: the control that says the host has not drifted

```
$ pixi run -e madgraph python validation/madgraph/time_stages.py \
      --out target/closeout-mg-timing <the 31 processes of note 30 §4, same order>
PASS START 2026-08-05T10:54:38Z
PASS END   2026-08-05T11:44:33Z
MG_EXIT=0
```

31 processes, **all exit 0, every stage boundary parsed (no nulls)** — the same
clean-parse condition note 30 §4.1 reports. The sprint touched neither MadGraph,
the reference bank, nor the pinned submodule, so nothing here should move; the
point of running it is that an our-side before/after taken across a sprint is
worth nothing if the machine drifted underneath it.

| stage | note 30 §4.2 | close-out | ratio |
|---|--:|--:|--:|
| `generate` | 1.2 | 1.2 | 1.00× |
| `output` | 62.8 | 62.3 | 0.99× |
| `compile` | 63.4 | 66.6 | 1.05× |
| `integrate` | 685.8 | 703.3 | 1.03× |
| `events` | 151.1 | 149.1 | 0.99× |
| **sum of per-process totals** | **1001.8** | **1028.3** | **1.03×** |

Per process, 27 of 31 sit within ±8% and most within ±3%. Three things worth
naming rather than averaging away:

- **`ee_to_mumu` reads 19.9 s against note 30's 9.2 s, and that is a
  confirmation, not a drift.** It is the first MadGraph invocation of the pass,
  and this worktree's `madgraph` environment was fresh, so it paid the cold
  Python import. Note 30 §4.3 measured exactly that: "a smoke run before the pass
  measured `ee_to_mumu` at **19.0 s** against 9.2 s inside the pass". 19.9 s
  against their 19.0 s cold. Netting the cold start out, the pass sums to
  1017.6 s, **1.02×** note 30.
- **The one-off LHAPDF set install recurred on exactly one process**, `pp_to_llj`,
  which is precisely note 30 §4.2's footnote
  (`grep -l "successfully downloaded" logs/*.timed.log` matches one file, the
  same one). Fresh environment, same seam.
- `bbx_to_ccx_emmm_qcd0` +8% and `ud_to_epemud_qcd0` +28% are the only rows
  outside ±8%; both were taken while the machine's resident background agents
  (`mds_stores`, `mediaanalysisd`) were active, which is the same ±few-per-cent
  noise source note 30 §3.3 quantified from our side.

**Verdict: the host reproduces note 30 to ~2–3%.** That is what licenses reading
§6.3–§6.4 against note 30's tables at all, and it bounds how much of any our-side
move could be the machine rather than the sprint: not much.

One bookkeeping note for whoever runs this next. The wrapper measured 2 995 s
between `PASS START` and `PASS END` while `time_stages.py`'s own accounting sums
to 1 028.4 s. The gap is environment activation and the LHAPDF fetch, which sit
outside the per-process loop; note 30 had the same shape but smaller (25.4 min
wall against 16.7 min of stages). Quote the stage-accounted figure, as note 30
§4.3 does — the wrapper wall is not a measurement of MadGraph.

### 6.7 The `-j` column, and the retirement of note 30's biggest caveat

Note 30 §1 named one caveat above all others: "on the integrate and events stages
**MadGraph is a 16-way parallel job farm and our integrator is one thread**", and
§5 therefore compared CPU time rather than wall time throughout. I3 and I4 removed
the asymmetry; this measurement retires the sentence rather than arguing it away.

```
vibegraph integrate <proc card> --run-card <run card> --out <dir> --force -j {1,16}
```

on `dy13_default` (the committed `dy13_proc_card.dat` + `dy13_default_run_card.dat`)
and on `pp_to_llj` (the `generate` line of its `.mg5` driver, which is exactly a
proc card, + the banked run's `run_card.dat`), at the CLI's default budget
(`--neval 120000 --niter 12`). Four rounds round-robin over the four
configurations, minimum over rounds. Host quiet: load average 3.5–3.9 at every
round start, no bench or cargo processes. The interpreter that times each run
starts *outside* the timed region — a `-j 16` dy13 run is under half a second, so
bracketing it with two `python3` startups would have been a tens-of-per-cent error
on the very number being measured.

| card | `-j 1` | `-j 16` | speedup | artifact md5 |
|---|--:|--:|--:|:--|
| `dy13_default` | 2.077 s | 0.442 s | **4.70×** | `c42bc53e6679f79802b9be78cc9dd78a` |
| `pp_to_llj` | 12.088 s | 2.256 s | **5.36×** | `788c2ec5e6eae85d3f44b54d59d53e44` |

Round-to-round spread is 1–2% on every cell (dy13 `-j 1`: 2.077/2.085/2.086/2.116;
llj `-j 16`: 2.256/2.279/2.283/2.296), so these are tight.

**The md5 column is the point as much as the timing is.** All eight runs of each
card — four rounds × two thread counts — produced one artifact digest. I3's
bit-identity contract is that thread count moves no bit, and this is that contract
checked at the CLI rather than inside a test: `-j 16` is not an approximation of
`-j 1`, it is the same number computed faster. That is what makes §6.3's
single-thread row durations and the parallel CLI numbers descriptions of one
program.

Against I3's own scaling numbers (dy13 4.46×, llj 4.87×), both improve — I3
measured under sibling-session load 7–8 and said its speedups were biased down, and
on a quiet host they are.

Two things this does not claim. It is a *wall-time* number, so it moves no ratio in
§6.4 — those keep their CPU-time denominators exactly as note 30 required. And the
serial floor is real: I4 measured the α-adaptation survey at ~27% of a fixed-budget
llj run at `-j 16`, budget-independent and still sequential. That is the remaining
Amdahl term and is now the leading parallel-scaling item in the backlog.

### 6.8 Sprint-level `mg_perf_compare`: the per-point matrix element

Note 30 §5.3 explicitly disclaimed being a substitute for this — its throughput
column is *integrand* throughput, carrying the multichannel map, the VEGAS grid,
the cuts and the clustering along with the amplitude. `scripts/mg_perf_compare.sh`
is the narrow measurement: criterion medians of the `eval_m2/forward/*` rows
divided by the bench's points-per-iteration, against MATRIX1 ns/eval from
`validation/madgraph/output/mg_timings.json`. Same 14 processes both arms.

```
bash scripts/mg_perf_compare.sh          # run in each tree
```

- **after arm**: this worktree at `62d78e4`.
- **before arm**: `e951045`, the pre-sprint tip, checked out with
  `git worktree add --detach` into this worktree's gitignored `target/` — never the
  shared checkout — with `mg_timings.json` and the `mg_*.so` modules symlinked in
  so both arms read an identical MadGraph column.

The arms alternate round by round and each row takes its minimum over two rounds; a
daemon spike then lands on a round rather than on an arm, which mattered here — the
after arm's whole-table geomean read 0.98× on round 1 and 1.09× on round 2. Both
arms are the `bench` profile, `RUSTFLAGS` unset, same rustc, same host, one
sitting. The after arm's fingerprint reads `62d78e4-dirty`: at measurement time the
worktree carried the uncommitted `TODO.md` and note-31 edits and nothing else
(`git diff --name-only` returns exactly those two files), so no source went into
that binary which this commit does not also contain.

| process | MG ns/eval | before ns/eval | after ns/eval | before/MG | after/MG | after/before |
|---|--:|--:|--:|--:|--:|--:|
| `ee_to_zh` | 192 | 223 | 189 | 1.16× | 0.99× | −15.1% |
| `pp_to_ll_qcd0` | 267 | 274 | 243 | 1.03× | 0.91× | −11.3% |
| `uux_to_uux` | 268 | 465 | 398 | 1.74× | 1.48× | −14.5% |
| `ee_to_mumu` | 285 | 273 | 237 | 0.96× | 0.83× | −13.0% |
| `ee_to_ttx` | 330 | 451 | 362 | 1.37× | 1.10× | −19.7% |
| `gg_to_ttx` | 650 | 1040 | 834 | 1.60× | 1.28× | −19.8% |
| `ee_to_ee` | 680 | 686 | 573 | 1.01× | 0.84× | −16.5% |
| `ee_to_wpwm` | 769 | 1315 | 950 | 1.71× | 1.24× | −27.7% |
| `ee_to_tatah` | 835 | 912 | 677 | 1.09× | 0.81× | −25.8% |
| `gg_to_gg` | 936 | 1707 | 1285 | 1.82× | 1.37× | −24.7% |
| `ee_to_mumua` | 1389 | 1341 | 1004 | 0.97× | 0.72× | −25.1% |
| `ee_to_mumu_tata_qcd0` | 6846 | 6025 | 4368 | 0.88× | 0.64× | −27.5% |
| `uux_to_ccx_emmm_qcd0` | 102885 | 124886 | 88946 | 1.21× | 0.86× | −28.8% |
| `bbx_to_ccx_emmm_qcd0` | 144616 | 211454 | 148598 | 1.46× | 1.03× | −29.7% |

**MATRIX1 geomean 1.25× → 0.98× over 14 processes; the evaluator itself −21.6%;
processes beating MadGraph 3 → 8.** Every row improved, −11.3% to −29.7%.

Three independent corroborations of the sessions' own claims fall out of this,
which is the value of measuring the pair in one sitting rather than trusting the
chain of per-session baselines:

- E1b recorded "processes beating MadGraph 3 → **8** of 14". Reproduced exactly.
- E1b's −17.3% plus E2's −4.2% plus E2b's ~−1% predicts about −21.5% end to end;
  measured **−21.6%**.
- E2 measured its own starting point at 1.29× and E1b at 1.21×; the pre-sprint tip
  measures **1.25×** here, inside that pair.

The remaining gap is where it was always going to be: `gg_to_gg` at 1.37× and
`uux_to_uux` at 1.48× are the rows with the densest colour algebra and no PDF work,
and they are what a further evaluator session would have to move.

### 6.9 Where this record disagrees with something already written

Recorded, not quietly reconciled. Nothing here changes a gate or a tolerance.

1. **The close-out brief's own per-row protocol was wrong** (§6.1). Pinning only
   `RAYON_NUM_THREADS=1` funnels every concurrently-running σ row through one
   global rayon worker, and each row's `Stopwatch` then spans the whole cohort.
   It would have published an ~8× phantom regression on every hadronic integrals
   row. The tell was that the rows landed within 305–320 s of one another while
   spending 4.32M, 4.5M and 9M points.

2. **Note 30 §3.2's `diagrams` column is not per-row work** (§6.2). Its stated
   mechanism — "each trial re-loads the interned SM" — is inverted: `sm_model()`
   is process-wide interned, and the uniform ~0.55 s is 26 rows racing its lazy
   initialisation under default test parallelism. One row at a time, the category
   is 1.29 s against 14.2 s.

3. **Note 30 §3.2's `amplitudes` column is fine; only its explanation is wrong**
   (§6.2). "That gate compares committed tables and never builds an evaluator"
   does not describe `amplitude_oracle::measure`, which runs enumeration and
   `AmplitudeEvaluator::compile` per row — which is why its two 2→6 rows cost
   ~1.1 s and everything else ≤ 0.02 s. The numbers reproduce under both
   protocols. **This item corrects an earlier reading inside this same close-out**:
   a partial first pass showed every amplitudes row at ~0.57 s and looked like a
   regression; that pass was the pathological one of item 1, and the effect
   vanished once complete passes were measured.

4. **A row measured alone is not the same measurement as that row inside a pass**
   (§6.1). `pp_to_bb_fixed` reads 37.1 s alone and 26.0 s in-pass; the difference
   is one-time PDF-grid and interned-model setup charged to whichever row runs
   first. Any future session isolating a row to time it has to subtract that.

5. **The `samples` category has no uncontended protocol in this sitting** (§6.3).
   `pp_to_llj_dyn`'s samples row reads 127.78 s (note 30), 93.2 s and 210.41 s
   under the two close-out protocols. The rows that misbehave are the ones driving
   `vibegraph generate` as a subprocess; the in-process partonic samples rows are
   well behaved under every protocol (all 17 improved, −16.8% as a block). Layer-
   level samples numbers are therefore taken from §6.5's like-for-like run only.

6. **E3's `pp_to_llj_dyn` cost is confirmed with its sign.** E3 recorded
   "+1.0–1.6%, consistently signed" as the accepted price of the arena-reuse stamp
   on the shape where the cache can never hit. It is the one row in the layer that
   did not improve — an independent confirmation from a different measurement.

7. **E3's "−37.5% event read-out on `gu_to_epemu`" and this note's −7.0% on that
   row's `samples` cell are not in conflict.** The cell also carries event
   generation and the weighted-ECDF KS and χ² comparisons against MadGraph's
   banked sample, so the read-out is only a fraction of it. Flagged because the
   two numbers look contradictory side by side.

8. **Build-fingerprint caveat.** §6.3's per-target invocations
   (`-p vibegraph-lib --features extended-validation`) trigger a rebuild relative
   to `validate.sh`'s workspace invocation even though the resolved
   `vibegraph-lib` features are identical. Far below the moves reported, but it is
   a difference from note 30's build, which used the workspace command.

### 6.10 What the sprint moved, in one place

| measure | note 30 baseline | close-out | change |
|---|--:|--:|--:|
| `pixi run --skip-deps validate` wall (default command) | 691 s | **391 s** | **−43.4%** |
| integrals category, one row at a time | 842.6 s | **389.8 s** | **−53.7%** |
| ── partonic block (no PDF, no budget change) | 39.9 s | 33.8 s | −15.2% |
| ── hadronic block | 802.8 s | 356.0 s | −55.7% |
| integrand throughput vs MG, geomean over 26 rows | 6.84× | **8.76×** | +28% |
| MATRIX1 per-point geomean over 14 processes | 1.25× | **0.98×** | −21.6% evaluator |
| processes beating MadGraph per point | 3 of 14 | **8 of 14** | |
| `integrate` wall, `dy13_default` | 2.077 s (`-j 1`) | **0.442 s** (`-j 16`) | 4.70× |
| `integrate` wall, `pp_to_llj` | 12.088 s (`-j 1`) | **2.256 s** (`-j 16`) | 5.36× |
| census | 96 ✅ / 2 ⚠️ / 4 ⏳ | **96 ✅ / 2 ⚠️ / 4 ⏳** | unchanged |
| MadGraph side (control) | 1001.8 s | 1028.3 s | +2.6% |

The last two rows are the ones that make the rest mean anything: the gate did not
move, and neither did the machine.
