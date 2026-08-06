# 32 — Performance-sprint addendum: budget alignment, bias closure, the serial tail (plan)

Planned 2026-08-05 from the user's triage of note 31's open items. This is a
cleanup addendum, not a fourth full sprint: every session either takes a win the
close-out already measured, zeroes a bias whose mechanism is already understood,
or aligns a cost with a precision argument. The triage's own framing, folded in
verbatim:

1. **Reduce the validation point budget to the precision MadGraph defaults to** —
   no sense burning CPU re-validating against something already weaker, and the
   systematic uncertainties will always dominate these integration errors.
2. **Any easy wins per-point should be taken on** (E3b, E4).
3. **Zero out any bias we understand how to** (I5; I1's iteration fix was the
   same defect one level down).
4. **Turn on the 2→6 rows in validation** now that the evaluator is faster.
5. **Re-visit `mg_perf_compare`**: manifest single-source-of-truth violation and
   the timing's missing artifact story (both confirmed below, §1.2), plus the
   bench arm's unrepresentative process sample (§1.2 finding 3).
6. **Why does `-j 16` only see 4–5×?** (answered §1.1; relief is a session).
7. **Undo the scalar cost of the lane-FMA commit** (`3dab3a1`): SIMD lanes are
   not in production, but the shared complex-FMA path that bought them −22–35%
   also took the packed complex idiom away from the production scalar
   evaluator — the commit's own message records "scalar forward regresses
   ~+3.5%, shipped as-is since forward is the least-used path", a premise that
   inverted the moment lanes stayed out of production. Workaround now: an
   in-house `MulAdd` trait whose `Complex<f64>` impl defers to the
   `num_traits::MulAdd` path the scalar evaluator used before; long-term,
   `MulAdd` support lands in `numeric_array` itself — necessarily **upstream**,
   since both `num_traits::MulAdd` and `NumericArray` are foreign to this
   crate and the orphan rule forbids the impl in-tree. (Session S9.)

Ground rules carried over from note 31 unchanged: no tolerance is relaxed
anywhere; every budget move is licensed by a ladder plus a ≥5-seed sweep with
χ²/dof read, never by the reference's error alone; the may-move set of every
session is pre-registered in its brief; census cells may be added, never
removed.

## 0. What the close-out licenses this plan to believe

From note 31 §6, all one host (M3 Max), one sitting: `validate` 391 s wall;
integrals 389.8 s one-row-at-a-time; per-point MATRIX1 0.98× geomean;
`integrate` 4.70×/5.36× at `-j 16` on `dy13_default`/`pp_to_llj` with a
byte-identical artifact; the α-adaptation survey ~27% of a fixed-budget llj run
at `-j 16`, budget-independent and sequential.

Reference-precision facts the budget session starts from (banked values):

| row | MG σ ± err | MG rel err | our gate err | our budget |
|---|---|--:|--:|---|
| `dy13_default` | 933.110 ± 0.447 | 0.048% | ±0.537 (0.058%) | 120k × 12, 3 seeds |
| `dy13_mmll` | 644.420 ± 0.315 | 0.049% | ±0.367 (0.057%) | 120k × 12, 3 seeds |
| `pp_to_llj` (fixed) | 422.840 ± 1.805 | **0.43%** | ±0.248 (**0.06%**) | 600k × 10, 3 seeds |
| `pp_to_jj` | 6.7885e8 ± 1.4726e6 | **0.22%** | ±2.511e5 (**0.037%**) | 300k × 10, 3 seeds |
| 2→6 rows (each) | — | 0.30% | not integrated | `Plan::Skip` |

The DY rows are already matched to the reference; llj and jj overshoot it by
7× and 6× in precision — which at 1/σ² scaling is CPU spent at ~40× the rate
the comparison can use. That is the whole budget-alignment argument: the gate's
resolving power is `√(σ_ours² + σ_MG²)`, and once `σ_ours ≲ σ_MG` further points
buy nothing the pull can see. The floor under any cut is never MG's error alone:
chain B's draw raises low-budget seed scatter (χ²/dof 6.38 at 75k on llj_dyn,
clean ≥150k, note 31 §I1), and a ladder still climbing is a convergence problem
no precision argument may paper over.

## 1. Two questions answered at planning time

### 1.1 Why `-j 16` yields only 4.7–5.4×: it is Amdahl, not stalled threads

Fit the two-parameter serial/parallel model to note 31 §6.7's own numbers
(`T₁ = S + P`, `T₁₆ = S + P/16`):

| card | T₁ | T₁₆ | ⇒ serial S | S/T₁ | S/T₁₆ | speedup ceiling |
|---|--:|--:|--:|--:|--:|--:|
| `dy13_default` | 2.077 s | 0.442 s | 0.33 s | 16% | **75%** | 6.2× |
| `pp_to_llj` | 12.088 s | 2.256 s | 1.60 s | 13% | **71%** | 7.6× |

So the parallel region itself scales essentially cleanly — threads are not
stalled or contending; at `-j 16` the run is simply ~70–75% serial floor, and
no thread count fixes that. The floor decomposes into:

- **The α-adaptation survey** — the largest single term, measured at ~27% of a
  fixed-budget llj `-j 16` run (note 31 §I4), budget-independent
  (`neval.clamp(10k, 40k) × 6` iterations). `survey_variance`
  (`proton.rs:1920`) is a sequential point loop whose inner loop visits **every
  channel's density per point** — O(n_survey × n_channels) — and
  `adapt_alphas` runs six such surveys back to back.
- **One-time setup**: model parse/intern, diagram enumeration, evaluator
  compile (0.05–0.29 s, note 23), PDF grid parse, artifact serialization.
  On dy13 this is most of the 0.33 s — the run is under half a second.
- **Sequential structure of the adapt phase**: each VEGAS/α iteration is a
  barrier; per-channel grids have a 512-point chunk floor, so small channels
  parallelize poorly within an iteration.
- A smaller, structural term: `-j 16` on an M3 Max maps onto 12 P-cores +
  4 E-cores, so even perfectly parallel work cannot reach 16×.

Relief, in measured-size order: **parallelize `survey_variance`** (session S3
below — the same deterministic chunk-keyed reduction I3 used, so thread count
still moves no bit); **shorten the sequential adapt critical path** via the
batch-size-vs-iteration-count sweep (backlog item (e), folded into S3 as a
measurement); **chunk-size tuning** (measured inert, never tuned — free);
setup caching (the compiled-program bundle) stays a backlog feature, out of
scope here.

### 1.2 `mg_perf_compare`: both halves of the triage suspicion are confirmed

Checked 2026-08-05 against the tree at `1e5b950`:

1. **The manifest is not the source of the timing registry.**
   `validation/madgraph/gen_amplitude.py` carries its own hardcoded `PROCESSES`
   list (line 77: name, process string, PDGs, grid energies, seeds,
   `profile_npoints`) and never reads `validation/manifest.toml` — whose own
   header promises it is "the single source of truth the reference generators
   … read from, so a process is added here once rather than in five places."
   A process added to the manifest today gets no amplitude CSV and no
   `mg_timings.json` row unless separately registered in the Python file.

2. **The timing reaches no artifact.** `mg_timings.json` is written only to the
   gitignored work area `validation/madgraph/output/`; `assemble_bundle.sh`'s
   member selection (Cards, Events, `results.dat`, `leshouche.inc`,
   `matrix*_orig.f`, run logs, `*_amplitude.csv`) never picks it up, no
   validation-report row carries it, and the JSON itself records no host
   identity. On a fetched checkout `scripts/mg_perf_compare.sh` hard-errors at
   its first existence check (line 42). The precedent for the fix is already
   in-tree: `validation/madgraph/timings.json` (note 30's per-stage table) is a
   **host-labelled committed file** — host-specific data stays out of the
   host-independent refdata bundle but lives in git with provenance.

3. **The vibegraph arm of the comparison is not a representative sample of the
   amplitudes it could compare** (user, 2026-08-05 follow-up; checked the same
   day). `eval_strategies.rs` carries a *third* hand-synced copy of the
   registry — `const PROCESSES: [(&str, &str); 14]` at line 110, whose own
   comment says "keep in sync with `gen_amplitude.py` PROCESSES and
   `amplitude_oracle.rs`" — while `gen_amplitude.py` registers **19** processes,
   every one with a compiled MATRIX1 module and a timing row. The five the
   bench silently drops from the join are `uux_to_epemg`, `ddx_to_epemg`,
   `gu_to_epemu`, `gux_to_epemux` and `ud_to_epemud_qcd0`: exactly the
   QCD llj-class rows — radiated-gluon and gluon-beam 2→4 amplitudes, the
   class nearest production hadronic work and nearest the sprint's named
   remaining gap (`gg_to_gg` 1.37×, `uux_to_uux` 1.48× are the densest-QCD
   rows in the 14). The MATRIX1 table is therefore biased toward the
   EW/leptonic rows where we already win. The `VIBEGRAPH_BENCH_EXTRA_PROCESSES`
   escape hatch exists but is study-only by design — the joined set "stays
   exactly `PROCESSES` unless the variable is set".

Session S4 closes all three.

## 2. Sessions

Eight sessions, two waves. Wave 1 is dispatch-parallel (disjoint surfaces);
wave 2 is sequenced because budgets must be re-pinned after the bias and
sampler changes land, and σ re-recorded once at the end. Worktree, dispatch and
long-command discipline per `AGENTS.md` "Sprint & Subagent Operations".

### Wave 1

#### S1 — E3b + I5: dead work off the fixed-beam path, seed combination unweighted

*(performance-dev, light — Sonnet eligible)*

Two small, adjacent fixes:

- **E3b (cut before the draw)**: `FixedBeamIntegrand` runs the
  scale-configuration draw before the cut, so 22% of
  `gu_to_epemu`/`gux_to_epemux` points pay ~190 ns of provably dead draw work;
  `ProtonIntegrand::shape` already cuts first (note 31 §E3). The brief must
  decide the stream semantics explicitly and pre-register the consequence:
  either the cut path still consumes the draw's uniforms (bit-identical σ, the
  clean version — preferred if the dead 190 ns is the mapping/selection work
  rather than the uniform draw itself) or the stream realigns (statistically
  equivalent σ; every affected gate re-run and the shift recorded). Also fixes
  note 30 §6's "points the cuts reject return before the draw" — true on the
  hadronic path, false on the partonic one — wherever doc comments repeat it.
- **I5 (`combine_seeds` unweighted)**: `validate_hadronic.rs:395` still
  combines seeds by inverse variance — the same defect I1 removed from the
  iteration combination, one level up. Replace with the unweighted mean,
  `err = √(Σᵢ σᵢ²)/n`, χ² about the unweighted mean. Seeds run equal budgets,
  so the expected move is second-order; pre-register that every hadronic gate's
  combined value moves, assert all stay inside their unchanged tolerances, and
  record the per-row deltas in the session report for S8's re-record.

#### S2 — I6: `w_max` from a percentile, not an extremum

*(performance-dev)*

The real overweight lever, per I2's own falsification of the budget premise:
the maxima never converge (`Σⱼ w_maxⱼ ∝ n^0.508` over 2.4 decades — a Pareto
weight tail of index ≈ 2, Hill α = 2.08–2.40), so no scan budget fixes llj's
~5e-3 overweight σ-share. Implement MadGraph `unwgt.f`'s rule: set each
channel's maximum from a **percentile** of the scanned weights and
re-normalise, capping `Σⱼ w_maxⱼ`. The estimator is unbiased either way
(overweights kept at weight > 1), so the deliverable is measured **unweighting
efficiency and overweight σ-share before/after** at matched budgets, plus the
sample-lumpiness consequence on the llj rows. Match the percentile and
re-normalisation to `unwgt.f`'s actual rule, cited by line, not paraphrased.
Also take the free adjacent win: `Unweighter::scan` is a pure per-channel
function on its own stream, so a rayon `par_iter` over channels cannot move a
number (assert the artifact digest to prove it).

Pre-registered may-move set: every `samples` cell downstream of `generate`
(event composition changes; distributions unchanged in expectation — KS/χ²
gates re-run and read, `SIGMA_MAX_REL` unaffected in expectation). σ cells must
not move: the unweighter is downstream of every σ gate.

#### S3 — J1 + I4b: the serial tail, then the convergence default, one re-pin wave

*(performance-dev)*

Two items folded into one session because both change `integrate`'s artifact
bytes and the byte gates should be re-pinned once, not twice:

1. **Parallelize `survey_variance`** over its point loop with I3's
   deterministic chunking (fixed chunk size independent of thread count,
   per-chunk partials reduced in chunk order), keyed `(iteration, chunk)`.
   Thread count must still move no bit — assert one digest across
   `-j {1, 4, 16}`. The serial→chunked re-association is a **one-time byte
   change**, pre-registered. Measure: `-j 16` wall on `dy13_default` and
   `pp_to_llj` against §1.1's table — the survey term (~27% of llj T₁₆) should
   mostly vanish; predict and check the new speedup against the fitted model
   (llj ceiling moves from ~7.6× toward ~11× if the survey fully
   parallelizes). While in there, run the two free measurements: chunk-size
   sweep (known inert, never tuned) and the batch-size-vs-iteration-count
   sweep on the adapt phase (backlog item (e)) — measurements only, adopt
   nothing that moves a gate.
2. **I4b**: flip `--target-rel` convergence mode to the CLI default
   (`integrate` reaches MG's banked accuracy at CPU parity on llj, 4.2–4.5×
   less CPU on dy13 — note 31 §I4). Library callers and validation tests pass
   explicit budgets and are untouched.

Then re-pin the CLI byte gates and `validation/pythia/generate_samples.sh`'s
pinned artifacts **once**, after both changes. Pre-registered may-move set:
artifact bytes (once), `-j` wall times; no σ tolerance, no census cell.

#### S4 — M1: `mg_perf_compare` onto the manifest, the MG timing into an artifact, the bench made representative

*(validation-dev, light — Sonnet eligible)*

Close all three §1.2 findings:

1. Move `gen_amplitude.py`'s per-process registry into
   `validation/manifest.toml` (grid energies, PDG legs, npoints, seeds,
   `profile_npoints` — the legacy seed exceptions become explicit fields) and
   make the Python read the manifest. Migration gate: a dry-run dump of the
   generation parameters before and after the move must be identical, so no
   reference CSV would regenerate differently; the committed CSVs are not
   regenerated.
2. Give the timing provenance and a home: `gen_amplitude.py` writes host
   identity (the same fields `host.json` records) into `mg_timings.json`, and
   a host-labelled copy is committed beside `validation/madgraph/timings.json`
   under the same convention (in git, never in the refdata bundle).
   `scripts/mg_perf_compare.sh` falls back to the committed copy when the work
   area lacks one — keeping its existing other-host warning, now driven by the
   JSON's own host field rather than filename inference — and stamps the MG
   column's host + date into the emitted `.md`/`.tsv` fingerprint.
3. **Retire the bench's hand-synced process list.** `eval_strategies.rs`
   derives its row set from the same manifest registry at runtime — the
   machinery already exists (`vibegraph-lib` depends on `toml`, and
   `src/validation.rs` already reads `manifest.toml`), so this is reuse, not
   new plumbing. The bench then benches **all 19** MATRIX1-comparable
   processes; `mg_perf_compare.sh` reports (rather than silently drops) any
   row present on only one side of the join, so the next registry addition
   fails loudly instead of shrinking the sample. `amplitude_oracle.rs`, the
   third name in the "keep in sync" comment, gets the same treatment if it
   carries its own list, or the comment corrected if it does not.

Pre-registered: no reference bytes change, no gate moves; the only new
committed files are the manifest fields and the host-labelled timing JSON.
**The MATRIX1 headline is re-based, not moved**: note 31's 0.98× geomean is
defined over 14 rows, and the widened table is a different series — the
session records the 14-row geomean (must reproduce ~0.98× within bench noise,
the continuity check) *and* the new 19-row geomean side by side, and the
19-row figure becomes the standing baseline. Expect the wider geomean to read
worse than 0.98×, since the five added rows are QCD-dense — that is the
honest number, not a regression.

#### S5 — E4: accept/reject allocator traffic (stretch)

*(performance-dev)*

The one untouched note-31 item. The unweighting profile is the most
allocation-bound of the four note-30 profiles: 18.4% allocator + libc mem,
5.1% `BTreeMap`, kT clustering at 9.2% because accept/reject re-derives the
per-event scale every trial and `ScaleChoice::clustered` heap-allocates its
beam–leg candidate list per event (~100 ns on a 0.5–1.7 µs matrix element; +6%
`gg_to_gg`, +21% `uux_to_uux`). First cut: per-event allocations → reused
scratch (`coupling/scales.rs`; measure with `probe_scale_cost`). Sized
honestly: the samples category is about half the integrals category in wall
time, so the layer-level win is bounded — the session's kill criterion is a
measured per-event improvement under 5% on the profiled rows. Bit-for-bit on
event bytes at fixed seed is the correctness gate (scratch reuse must not
reorder any draw).

#### S9 — E5: restore the scalar packed-complex idiom without giving lanes back

*(performance-dev)*

Commit `3dab3a1` routed the hot complex primitives (`cmul`/`cmul_add` in
`helas/repr/lorentz.rs`, feeding `dot`/`dot_lorentz`, `slash_bispinor`, the
currents and `scalar_bilinear`) through element-wise real `F::mul_add`, shared
between the scalar and lane fields because `Complex<NumericArray<f64, N>>`
has no `num_traits::MulAdd` impl. Lanes won −22–35%; the scalar forward path —
which **is** the production evaluator, lanes being out of production — paid
~+3.5% because plain `Complex<f64>` `*`/`+` had been compiling to the packed
two-doubles complex idiom.

The session, in order:

1. **Re-measure the regression on the current tip first.** The +3.5% was
   measured before E1b's scheduling, E2's arena hoist and E2b's current-CSE;
   the A/B is `cmul`/`cmul_add` via the trait below vs the shared path, on the
   (post-S4, manifest-driven) bench, forward rows. **Kill criterion,
   pre-committed**: if the packed idiom no longer wins ≥2% forward geomean,
   record the numbers and land nothing — the workaround exists to recover a
   measured loss, not on principle.
2. **The workaround**: an in-house complex multiply-add trait — most naturally
   associated functions on `Real` with a default body equal to today's shared
   real-FMA construction — overridden for `f64` to defer to the
   `Complex<f64>` `num_traits::MulAdd`/operator path the scalar evaluator used
   before `3dab3a1`. Lane fields keep the default, so no lane row may move a
   bit (asserted). The real-valued `p3_squared`/`m2` single-rounding fixes are
   orthogonal and stay.
3. **Re-scope the bit-identity contract consciously.** `lanes.rs`'s
   `eval_m2_lanes_bit_identical` pins lanes against the scalar path, and that
   contract breaks at ulp level the moment the scalar path diverges. Preferred
   re-scope: pin lanes `N` against `LaneField<1>` (the scalar carried through
   the shared path) — the oracle stays **bit-exact** and the packed-vs-shared
   scalar difference is then covered by the ordinary amplitude tolerance gates,
   which is exactly where AGENTS.md's "ULP exactness is never the target" says
   such a difference belongs. Falling back to a tolerance on the existing test
   is acceptable only if the `LaneField<1>` instantiation is impractical;
   either way the module doc's "bit-identical to the scalar `eval_m2`" sentence
   is updated to say what is actually pinned.
4. **Record the long-term path in the backlog**: `impl num_traits::MulAdd for
   NumericArray` is an **upstream** `numeric_array` contribution (orphan rule);
   when it lands, `Complex<NumericArray>` gets `MulAdd` for free, the in-house
   trait's default body collapses to one deferral, and the scalar/lane split
   disappears again — this time in the fast direction.

Pre-registered may-move set: every scalar amplitude and σ at ulp level —
tolerance gates absorb this, but **artifact bytes move**, so S9 must merge
before S3 executes the byte re-pin (sequencing note, §3). Lane evaluations,
bit-for-bit unmoved (asserted). The MG amplitude gates re-run as the standing
end-to-end signal.

### Wave 2 (sequenced: S6 after S1+S2 merge, S7 after S6, S8 last)

#### S6 — B1: validation budgets aligned to reference precision

*(validation-dev)*

The triage's headline item. Rule: size each σ gate's budget so
`σ_ours ≈ σ_MG` — beyond that the pull cannot see the points — subject to two
floors that always win: the seed-scatter floor (≥5-seed sweep, χ²/dof clean at
the proposed budget — chain B's draw is dirty at 75k on llj_dyn, clean ≥150k)
and the convergence floor (the ladder must be flat across the cut; a climbing
ladder is a bias, and the answer to bias is never a wider tolerance — `info`
at the affordable budget is the fallback, per the standing decision).

Pre-registered targets, from §0's table:

- **`pp_to_jj`**: 300k → 75k (4× cut). The ladder is already measured flat to
  0.08% across 75k–600k and MG's own error is 0.22%; ~65 s of single-thread
  integrals time at the post-sprint cost. Confirm with the sweep at 75k, move.
- **`pp_to_llj` (banked integrals gate)**: the 600k budget was set because the
  ladder was still climbing at 300k — **before** I1's unweighted iteration
  combination collapsed the llj ladders (span 2.09% → 0.46%). Re-run the
  ladder under the current combination first; if flat, cut toward 150k–300k
  (MG's 0.43% error licenses a large cut *if and only if* the climb is gone).
  If it still climbs, the standing alternative applies: `info` at 300k, never
  a wider tolerance — escalate to the user before flipping any mode.
- **`BB_NEVAL` 300k and the recarded rows**: same rule against their
  refdata-5 references; cut where the sweep stays clean.
- **DY rows**: already matched (0.058% vs 0.048%) — untouched.
- **Partonic rows**: the whole block is 33.8 s; touch nothing unless an
  overshoot is both large and free to cut.

Every constant change re-records the adjacent ladder/sweep evidence in the
same comment block it amends (fresh numbers, not edited-in-place old ones).
Pre-registered may-move set: the named rows' gate σ/err (within unchanged
tolerances), `validate` wall (predict −60–90 s), no tolerance, no census flip.
S1's I5 must be merged first so the sweeps are run under the unweighted
combination.

#### S7 — V26: the 2→6 rows turned on

*(validation-dev)*

`uux_to_ccx_emmm_qcd0` / `bbx_to_ccx_emmm_qcd0` are `Plan::Skip` with a stale
premise: "24-dim flat RAMBO at ~1 ms/eval" predates three evaluator sprints —
the bench now reads **89/149 µs per eval** for these rows (note 31 §6.8). The
manifest already declares their `integrals` and `samples` cells
`tier = "long", mode = "gate"` — these are the census's 4 ⏳ cells. The session:

1. **Measure the real per-eval cost in the gate's own harness** (the
   extended-validation build, not the bench) — the first number in the report.
2. **Ladder + ≥5-seed sweep** for a flat-RAMBO σ against the banked references
   (both 0.30% MG error; budget rule per S6). If flat RAMBO's variance makes
   the budget absurd, measure the multichannel path (579/615 diagram channel
   trees) before concluding; the decision is whichever the ladder licenses.
3. **Flip `integrals` to a real long-tier gate** with its own pixi task named
   in `validate-deep`'s long-tier text — closing, for these rows, the same
   hygiene gap `probe_recarded_budget_ladder` sits in (no task, undocumented);
   give that probe its task in the same pass.
4. **`samples`**: cost an unweighted-event run at the measured eval price. Flip
   if affordable at the long tier; if not, the cell keeps ⏳ with the measured
   cost recorded as the blocker — a number, not a guess.

Pre-registered: census 98 measured → 100 (or a recorded-cost ⏳ on the two
samples cells); no existing cell moves.

#### S8 — C0: close-out and the σ re-record

*(validation-dev, light — Sonnet eligible; runs after everything merges)*

The debt I1's close-out could not pay because its charter forbade code edits:
`BB_NEVAL`'s ladder comments, the DY row docs and `validate_sigma.rs`'s
per-process numbers still quote σ from before the 300k → 150k re-pin — and
after S1/S2/S6 they will be doubly stale. One pass, explicitly licensed to
edit comments and docs only: re-record what a current run actually prints
(values, errors, χ²/dof, pulls) at every place a gate source quotes a number,
using S1/S6's recorded deltas as the cross-check. Then the addendum's own
close-out measurements: `validate` wall on the identical note-30 command,
census, `-j 16` wall against §1.1's prediction, and this note's close-out
section. TODO.md updated to the post-addendum state.

## 3. Sequencing and bookkeeping

```
wave 1 (parallel):  S1 (E3b+I5)   S2 (I6)   S3 (J1+I4b)   S4 (M1)   S5 (E4, stretch)   S9 (E5)
wave 2 (serial):    S6 (B1, after S1+S2)  →  S7 (V26)  →  S8 (C0, after all)
```

- S3 owns the **only** byte re-pin wave; nothing else may re-pin an artifact
  digest. Because S9's scalar ulp shift also moves artifact bytes, S3 executes
  its re-pin step only after S9 has merged (S3's code work is independent and
  can proceed in parallel; only the re-pin waits).
- S6 runs its sweeps only after S1 (unweighted seed combination) and S2 (the
  percentile rule does not touch σ, but its scan-parallelism assert should be
  in before budgets shrink scan inputs) are merged.
- Every session brief carries the dev-agent worktree/long-command discipline
  verbatim, pre-registers its may-move set, and reports command + output for
  every claim, per `AGENTS.md`.
- The v0.1 tag decision is the user's and is not blocked by this plan: the tag
  can precede the addendum (it is a cleanup, not a correctness sprint) or
  follow S8's re-recorded numbers — flagged, not decided here.

## 4. What this addendum does not do

- No tolerance moves, anywhere, in either direction.
- No new samplers or physics: the percentile rule changes unweighting
  efficiency, not the estimator; the budget session changes CPU, not claims.
- No compiled-program/PDF bundling (the self-contained-artifact feature owns
  setup cost), no per-flow α, no stratified-parallel axes beyond the survey
  loop, no `feyngraph-perf`, no egraph work.
- No SIMD-lane production promotion and no upstream `numeric_array` PR: S9
  recovers the scalar path's codegen and leaves the lane path bit-for-bit
  untouched; the upstream `MulAdd` impl is recorded as the long-term
  replacement, not attempted here.
- No refdata regeneration: S4's migration is gated on producing byte-identical
  generation parameters, and S7 integrates against already-banked references.

## 5. Close-out (S8, 2026-08-05)

Eight of the nine planned sessions merged; S9 was killed by its own
pre-registered criterion and landed nothing. `main` @ `225657a` carries every
merge below. This section is the addendum's own close-out — the last debt
note 31's close-out could not pay because its charter forbade code edits.

### 5.1 Per-session outcomes

- **S1 (E3b + I5, `824bfc8`/`a684f4d`)**. `FixedBeamIntegrand` drew the scale
  configuration before checking the cut, paying `eval_amp2` + `set_alpha_s`
  for a channel selection a rejected point would discard (~190 ns on 22% of
  `gu_to_epemu`/`gux_to_epemux` points). `scale_u` is always a slice of the
  point's own already-drawn `u`, never an independently advanced RNG stream,
  so cutting first is a pure dead-work skip — bit-identical for every accepted
  point, checked rather than assumed. `combine_seeds` in
  `validate_hadronic.rs` moved to the unweighted mean (`err = √(Σᵢσᵢ²)/n`),
  the same fix I1 already made one level up to VEGAS's own iteration
  combination; every hadronic row moved, all ≤0.02% and within its unchanged
  tolerance.
- **S2 (I6, `49dc87d`/`dc2e581`)**. Read `w_max` off MadGraph's own `unwgt.f`
  truncation-ladder rule (the lowest scanned weight leaving under 1% of a
  channel's scanned cross section above it, re-normalised) rather than the
  scan's extremum, which a Pareto tail of index ≈ 2 never lets converge.
  Unweighting efficiency on the five gating rows rose from
  22.2/20.6/23.3/10.6/4.21% to 54.1/52.1/52.9/38.9/9.98% at matched budgets;
  `p p > l+ l- j` at 300k×8 needed 2 269 051 trials for 20 000 events before
  the fix, 477 125 after (4.36× cheaper per effective event). `Unweighter::scan`
  now runs its per-channel scans on a `rayon::par_iter`, bit-identical at
  `--max-truncation 0`.
- **S3 (J1 + I4b, `e8cd61e`/`43a9f51`)**. `survey_variance` now runs its point
  loop in one rayon region over I3's deterministic chunking — each chunk
  addresses its own substream by point index and seeks straight to its first
  point, so the split changes only the summation order (per-chunk partials
  reduced in chunk order), asserted bit-identical at `-j {1, 4, 16}`. This
  session's re-measurement corrected note 32 §1.1's own decomposition: the
  survey was 41–52% of a fixed-budget `-j 16` wall (`llj` 1.04/2.51 s = 41%,
  `dy13` 0.24/0.46 s = 52%), not the ~27% first quoted from a different card
  and budget. `integrate --target-rel` (0.1% default) is now the CLI's
  convergence mode unless `--fixed-budget` asks for a fixed
  `--neval × --niter` spend — every caller that actually wanted a fixed budget
  (CLI tests, `generate_samples.sh`, `scripts/acceptance.sh`) had to say so
  explicitly, which is the footgun the flip could have been without that
  audit. The pre-registered byte re-pin step "resolved empty": once the
  fixed-budget callers were made explicit, no pinned CLI/Pythia artifact
  actually depended on the old default-budget path, so there was nothing left
  to re-pin.
- **S4 (M1, `db33913`/`767bb2d`)**. Closed all three §1.2 findings:
  `validation/manifest.toml` gained a per-row `mg_amplitude` table and
  `gen_amplitude.py` reads it instead of its own hardcoded registry (dry-run
  parameter dump byte-identical across the migration); `mg_timings.json`
  carries host identity and a host-labelled copy is committed beside
  `timings.json`, with `mg_perf_compare.sh` falling back to it and reporting
  one-sided rows instead of dropping them; `eval_strategies.rs` derives its
  bench row set from the manifest at runtime, covering all 19 MATRIX1-comparable
  rows instead of 14. The session's own finding inverted its brief's implicit
  premise: the five previously-dropped rows are QCD-dense, so the wider table
  reads *worse* (19-row geomean 0.95×) than the narrower one it replaces
  (14-row geomean 1.06×, kept as the continuity figure) — that is the honest
  number the biased 14-row sample was hiding, not a regression from this
  session's own work.
- **S5 (E4, `143e8e9`/`3ffcf7e`)**. Scoped as "accept/reject allocator
  traffic," but the actual allocation hoisted out was
  `ScaleChoice::cluster_scales`'s per-event rebuild of three `BTree`
  containers — a different, adjacent piece of the same profile from the one
  the session name names (`ScaleChoice::clustered`'s per-event beam–leg
  candidate `Vec`, still unaddressed — see the follow-ups below).
  `MergeTablesByOrder` builds one table set per coupling order at setup
  instead of one per event;
  `probe_scale_cost` — which had been unrunnable since an earlier session
  widened the scale-aware integrand's dimension out from under it, and had to
  be fixed before it could measure anything — now reads **−16.9%** (`gg_to_gg`),
  **−21.6%** (`gg_to_ttx`), **−22.3%** (`uux_to_uux`) ns/point, with
  `validate_unweighting` **−16.8%** and the partonic σ gate **−11.1%**
  end to end, byte-identical throughout (2000 events byte-for-byte on the
  clustering-scale card, before and after).
- **S6 (B1, `551b3f7`/`59d1865`)**. Sized `pp_to_jj`, `pp_to_bb_fixed`,
  `pp_to_bb`, `pp_to_bb_qcd2` and `pp_to_ll_scalefact2` 300k→75k under a
  ladder+sweep license; `pp_to_llj` (both the fixed-scale and re-carded rows)
  stayed at 150k on the floor that always wins over a precision argument — the
  fixed-scale ladder is flat but its 75k rung carries dirty seed scatter
  (χ²/dof 1.66/2.59), and the re-carded row's ladder still climbs
  monotonically across the whole 75k–600k range. Same host, one sitting:
  `validate` 360.6 s → 342.6 s wall, 1608.9 s → 1321.3 s CPU — the session's
  own first demonstration that these cuts move CPU far more than wall, because
  `validate` runs its rows concurrently (§5.3 below repeats this at the
  addendum's full scale). `probe_bb_budget_ladder` had no committed ladder at
  all before this session — `BB_NEVAL` cited a three-seed scan with no
  instrument behind it — so it was added alongside the other three.
- **S7 (V26, `d850b57`/`225657a`)**. The 2→6 rows' `Plan::Skip` blamed a
  "~1 ms/eval" matrix element; the gate's own harness reads **64/71 µs** —
  stale by more than an order of magnitude — and even the flat-RAMBO map the
  premise offered as the affordable alternative is wrong for an unrelated
  reason: six outgoing legs put the physical poles on a set of vanishing flat
  measure, so flat RAMBO misses these cross sections by **eleven and fifteen
  orders of magnitude** despite a respectable 46% cut-survival rate. The real
  cost floor is `MIN_CHANNEL_NEVAL` (512, `budget.rs:78`) times the per-diagram
  channel count — 579/615 channels put 296 448/314 880 evaluations under every
  iteration whatever budget is asked, a mechanism the original brief did not
  name. Under the multichannel the physics agrees (five-seed means inside 1.1%
  of a 0.30%-precision bank at 300k/600k/1.2M) but the estimator is
  heavy-tailed — single seeds swing +4.8%/−4.5%/+3.5% at both ends of that
  ladder and do not shrink with budget — so both rows are `Plan::Long`,
  measured and reported (`info`) on every `validate-sigma-2to6` run rather than
  tolerance-bound. `ladder-2to6`, `ladder-bb` and `ladder-recarded` are now
  named pixi tasks, all three listed in `validate-deep`'s long-tier text.
  Census 98 → 100 measured; the two `samples` cells stay ⏳ with their cost
  recorded (117/45 trials/event, inside the 400-trial budget, but ~40
  unparallelisable minutes for the pair) rather than assumed.
- **S9 (E5, no merge)** — **killed clean**. The plan's kill criterion was "if
  the packed idiom no longer wins ≥2% forward geomean, record the numbers and
  land nothing"; the session found something sharper than a null result — the
  packed complex idiom `3dab3a1` traded away is **x86-specific** codegen, and
  forcing it back via the in-house `MulAdd` trait on this ARM host (M3 Max)
  cost **8–9%**, the opposite of a win. The kill criterion fired and nothing
  merged: `add-s9-packed-complex`'s worktree is clean and its branch carries no
  commit past the note-32 planning doc itself (verified directly, this
  session — `git diff --stat HEAD` empty, `git log main..add-s9-packed-complex`
  empty). The design — an in-house complex multiply-add trait, default body
  the shared real-FMA construction, `f64` override deferring to
  `Complex<f64>`'s `num_traits::MulAdd`/packed path, lanes left at the default
  and therefore bit-for-bit untouched — stays at §2 S9 above as the resume
  point for whoever revisits this on an x86 host, where the original +3.5%
  scalar-forward toll may still be worth recovering.
- **S8 (C0, this session)** — the re-record and the close-out measurements
  below.

### 5.2 Brief-correction ledger

Every session corrected something in its own brief or a predecessor's,
consistent with this sprint cycle's running pattern:

- **S2** corrected both the rule's mechanism (MadGraph's truncation ladder, not
  literally a percentile) and confirmed the σ-share direction the plan
  predicted (no scan budget fixes it; the rule has to change).
- **S3** corrected §1.1's serial-floor decomposition (41–52% of the `-j 16`
  wall, not ~27%) and surfaced a CLI default-flip footgun the plan did not
  name: every caller relying on the old fixed-budget default had to be found
  and made explicit, or it would have silently changed behavior.
- **S4** corrected the framing carried into the sprint by its own §1.2
  finding 3: widening the bench sample was expected to be neutral bookkeeping,
  and instead inverted the "we're already ahead" reading by exposing that the
  omitted five rows were exactly the QCD-dense ones.
- **S5** corrected two things: `probe_scale_cost` itself was broken (unrunnable
  since an earlier dimension change), so no measurement in this area was
  possible before it was fixed; and the session's own name (E4 = "accept/reject
  allocator traffic") pointed at `ScaleChoice::clustered`'s per-event `Vec`,
  while what actually got hoisted was the adjacent `cluster_scales` merge-table
  rebuild — a related but distinct allocation from the one E4 was scoped
  around.
- **S6** corrected two stale figures inherited from the perf sprint: `pp_to_llj`
  quoted a 600k×10 budget that had not existed since I1's unweighted iteration
  combination collapsed the ladder (already re-pinned to 150k before this
  addendum even started), and the reference σ/error figures S6 was handed were
  pre-`refdata-5`.
- **S7** corrected the "~1 ms/eval" premise (stale by more than an order of
  magnitude) and named the actual cost mechanism (`MIN_CHANNEL_NEVAL` × channel
  count) that the original brief did not identify.
- **S9** corrected the premise that the packed-complex win would transfer to
  this host: it is x86-specific, and forcing it here cost 8–9% rather than
  saving 3.5%.
- **S8 (this session)** found the assignment brief's claim that
  `probe_bb_budget_ladder` "has no pixi task and is unnamed in `validate-deep`'s
  long-tier text" was itself stale — S7's merge (`d850b57`) added `ladder-bb`
  alongside `ladder-2to6` and `ladder-recarded` and named all three in
  `validate-deep`, so no action was needed there.

### 5.3 Close-out measurements

All measured 2026-08-05, same worktree (`vibegraph-addendum/s8` @ `225657a`
plus this session's docs-only commit), same M3 Max. The host was not quiet in
the note-30/31 sense (`mds_stores`/`mediaanalysisd` background indexing held
load average in the 6–25 range through most of this session, against note 31
§6.5's load average 3.1 at start) — every timing below is reported with that
caveat rather than re-run on a host this session could not obtain.

**`validate` wall, the identical note-30 command**:

```
$ pixi run --skip-deps validate
...
29 rows × 4 categories = 116 cells: 98 measured in the layers this run drove (96 ✅, 2 ⚠️, 4 ⏳, 14 — / uncovered).
real  7m23.316s   (443.3 s)
user  40m53.826s  (2453.8 s)
sys   2m0.669s
EXIT=0
```

**443.3 s against note 31 §6.5's 391 s is a +13.4% wall regression on the
identical command, and it is host noise, not the addendum going backward.**
S6's own within-session before/after already showed why wall is the wrong
instrument for these particular changes: its budget cuts alone moved CPU by
−288 s (1608.9 s → 1321.3 s) against only −18 s of wall (360.6 s → 342.6 s),
because `validate` runs its rows concurrently — a CPU saving only shows up in
wall to the extent the run was CPU-bound rather than scheduler- or
contention-bound, and a host at load average 17–25 from unrelated background
processes is neither. Restated in both terms: the addendum's CPU total this
session measured, 2453.8 s, is not directly comparable to a note-31 CPU figure
(none was recorded for the 391 s run), but S6's own −288 s CPU / −18 s wall
split on one licensed cut is the mechanism to read this session's wall number
through — the addendum's CPU-time saving is real and larger than its wall-time
saving, and the wall-time saving on this run was masked entirely by
background load neither S6 nor this session controls.

**Quiet-host re-run (post-close-out, same day, main @ `cb2436f`)**: with no
sibling work and background daemons idle (load average 1.8), two back-to-back
rounds of the identical command. The first paid rustc recompilation of the two
test binaries this session's own comment edits had dirtied (381.3 s wall,
1635.9 s user); the second, fully warm, is the steady-state measurement:

```
real  5m41.403s   (341.4 s)
user  21m45.646s  (1305.6 s)
sys   1m15.110s
EXIT=0            census unchanged
```

**341.4 s against note 31 §6.5's 391 s: −49.6 s wall (−12.7%).** It also
reproduces S6's within-session "after" (342.6 s wall / 1321.3 s CPU) to ~1% on
both axes from a different checkout — confirming the 443.3 s reading above was
entirely load + recompile, and giving the addendum's wall saving a clean
measurement after all: roughly −50 s wall and −300 s CPU on the standard
banked run.

**Census, two numbers, not blurred**:

```
$ pixi run --skip-deps validate            # drives hermetic + banked layers
29 rows × 4 categories = 116 cells: 98 measured (96 ✅, 2 ⚠️, 4 ⏳, 14 — / uncovered).

$ pixi run validate-sigma-2to6 && pixi run validation-report   # + the oracle layer
29 rows × 4 categories = 116 cells: 100 measured (96 ✅, 4 ⚠️, 2 ⏳, 14 — / uncovered).
```

The two numbers are not in tension. `validate` drives only the `hermetic` and
`banked` dependency layers (note 25's layering), so its own line has never
included the `oracle`-tier cells — the two 2→6 `integrals` cells S7 turned on
are `oracle`, not `banked`, exactly like `probe_bb_budget_ladder` and its
siblings. `validation-report` renders whatever is on disk, so its 100-measured
line is only true once `validate-sigma-2to6` has populated those two cells in
the same tree; run cold, `validation-report` alone still reads 98. The four
⚠️ cells (up from 2) are the two 2→6 rows' `integrals` cells landing as
`info`-not-`gate` (S7's design, not a regression) plus the two `samples` cells
already counted ⏳ moving nowhere — the two ⏳ in the 100-measured line are
`bbx_to_ccx_emmm_qcd0`/`uux_to_ccx_emmm_qcd0`'s `samples` cells, cost recorded,
not run.

**`-j 16` wall, `dy13_default` and `pp_to_llj`, `--fixed-budget --neval 120000
--niter 12`, min of 5 rounds, host load 6–19 across the sweep**:

| card | `-j 1` (min) | `-j 16` (min) | speedup | artifact md5 (all 10 runs) |
|---|--:|--:|--:|:--|
| `dy13_default` | 2.0446 s | 0.2357 s | **8.68×** | `cb8d12e354a426d00286a3c67739fdb0` |
| `pp_to_llj` | 11.0434 s | 1.1645 s | **9.48×** | `e135706599bd07e59447bea9205336ef` |

Every one of the 20 runs (5 rounds × 2 thread counts) for a given card wrote
the identical artifact digest — thread count moved no bit, checked at the CLI
rather than inferred from the unit-level assertion.

Against §1.1's original prediction (dy13 ceiling 6.2×, llj ceiling 7.6×, both
already superseded by S3's own fitted-model update to 18.2×/20.1× after the
survey parallelised): the measured `-j 16` speedups sit well below those
fitted asymptotic ceilings, which is expected — a ceiling from a two-term
serial/parallel fit is a `-j → ∞` limit, not a `-j 16` prediction, and this
run's ceiling is further suppressed by real contention from
`mds_stores`/`mediaanalysisd`. Against S3's own re-measurement under a
similarly noisy host (its commit message: `-j 16` walls "1.44 s and 0.23 s"
for llj/dy13) and the dispatch brief's citation of that session's own
correction (dy13 0.225 s/9.16×, llj 1.443 s/8.78×): this session's 0.2357 s/
8.68× (dy13) and 1.1645 s/9.48× (llj) sit within a few percent of both,
consistent with run-to-run noise on a host neither session could quiet, and
not with a regression in either direction. The dy13/llj asymmetry in which
session read the higher ratio (S3 read llj lower than dy13; this session reads
llj higher) is itself a symptom of that noise rather than a real effect —
llj's ~11 s `-j 1` run averages over more wall-clock-seconds of contention
than dy13's ~2 s one, so it is the noisier of the two ratios on either
session's host.

### 5.4 Standing follow-ups

- **Heavy-tail multichannel fix, with S7's falsifier.** The 2→6 rows' single-seed
  swings (+4.8%/−4.5%/+3.5%, both signs, top and bottom of a 300k–1.2M ladder)
  are diagnosed as a heavy-tailed multichannel estimator rather than a
  convergence defect, on AGENTS.md's own rule: "if extra budget makes a failure
  migrate between seeds instead of shrinking, it is a bug, not statistics" — S7
  measured the swings *not* shrinking with budget, which is the heavy-tail
  signature, not the bug one. The falsifier for any future fix: it must make
  the single-seed swings shrink as budget grows, not merely move where they
  land: a fix that reduces the swing magnitude at fixed budget without changing
  its budget-scaling is a variance-reduction win, not a resolution of this
  finding.
- **E4's scratch-through-`setclscales` continuation.** S5 hoisted the
  per-event merge-table rebuild out of `ScaleChoice::cluster_scales`, but
  `ScaleChoice::clustered`'s own per-event beam–leg candidate `Vec`
  (`coupling/scales.rs:376`) and `setclscales.rs`'s several per-call `Vec`s
  (`attempts`, `traces`, `pt2`, `mt2`, `lines`) are untouched — the allocation
  profile E4 was named for still has a remainder.
- **`adapt_alphas` is still serial at the seed level** — `ProtonIntegrand::
  adapt_alphas` (`proton.rs:1905`, itself calling down into
  `phasespace::channel::Channels::adapt_alphas` at `channel.rs:322`) is invoked
  once per seed inside the caller's own seed loop in every σ gate
  (`validate_hadronic.rs`'s `run_seed*` family). S3 parallelised the
  point-loop `survey_variance` runs *inside* one call to `adapt_alphas`, not
  the seed-level loop that calls `adapt_alphas` itself; a multi-seed gate
  still adapts its seeds one after another.
- **llj batch-shape candidate (relayed as "240k×6" in this session's dispatch
  brief; not independently verified here), pending a ≥5-seed sweep.** S3's
  batch-size-vs-iteration-count measurement (§1.1 relief item, backlog (e))
  found the adapt phase's sequential critical path shrinks with fewer, larger
  iterations at fixed total budget — a free measurement, adopted nowhere. This
  session found no "240k" or "240k×6" figure in S3's commit message or diff;
  the specific batch shape is either recorded only in that session's own
  (unavailable to S8) report, or was a manager-side inference from the
  measurement's direction. Whoever picks this up should re-derive the
  candidate shape from S3's raw sweep data before sizing a sweep around it.
- **VEGAS per-iteration χ²/dof overflow on wide channel splits**
  (`budget.rs:252`) — confirmed still present and unclamped this session
  (the 2→6 `integrals` re-run above printed a χ²/dof over 10^250 on both rows).
  S7's decision to pass the value through rather than clamp it, with the
  manifest note saying it is not a statistic, stands; a future session touching
  `budget.rs`'s combination code should know the overflow is expected on any
  row with hundreds of channels, not a bug to fix reactively.
