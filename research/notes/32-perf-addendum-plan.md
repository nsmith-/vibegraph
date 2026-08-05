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
