# `event-output-lhef` — unweighted event output (sprint plan)

**✅ CLOSED 2026-07-28, merged to `main` (ff, HEAD `49124d5`; the model-digest
hardening continued on `main` through `cd5d075`).** Close-out with session
outcomes and the deferred table in §"Sprint close-out" below.

Sprint B of the two-sprint events program opened in note 21 (Sprint A =
`resonance-sampling`, ✅ closed + merged 2026-07-26). Four sessions, **E1 → E4**,
strictly linear. `mg-single-helicity-bench` folds into E2.

Goal: turn the converged `integrate` phase into an **unweighted event sample**
serialised as a Les Houches Event File that a downstream shower (Pythia/Herwig)
accepts — accept/reject `w(p) = |M(p)|²·J(p)/w_max` over the frozen VEGAS grid and
Sprint A's multichannel sampler, with per-event helicity and colour-flow
*selection* filling in the LHE record.

## Dependency check at open (2026-07-27)

| Needed | State |
|---|---|
| n-body final states + peak-resolving sampler (unweighting efficiency) | ✅ `resonance-sampling`, live in the production `integrate` path |
| Frozen-grid replay primitive | ✅ `vegas.rs` `sample_frozen`/`_parallel`, deterministic ChaCha8 substreams |
| Accept-gate-shaped cuts | ✅ `cuts.rs` `Cuts::pass(&momenta) -> bool` |
| Handoff format | ✅ `IntegrateArtifact` (bincode+zstd: trained grid + run metadata) |
| Per-event `SCALUP` / `AQCDUP` for the LHE record | ✅ `dynamical-scales` (D1–D4, closed 2026-07-27) — this is why running couplings were sequenced ahead of the writer |
| Multi-flow JAMPs + exact CF matrix (E1's raw material) | ✅ `color-flow` + `validation-sprint`; `ColorBasis { elements: Vec<BasisElement>, cf_matrix }` in `helas/color/colorize.rs` |
| MG oracle for the flow → LHEF colour tags | ✅ `validation/madgraph/output/*/Source/leshouche.inc` — see §E1 |

No blockers. One **open sequencing decision** carried from note 21 (per-channel
VEGAS grids) is recorded in §"Open decision" below; it does not touch E1.

---

## E1 — `jamp2-flow-select`

Two pieces: the per-flow diagonal accumulator, and the flow → LHEF colour-tag
dictionary.

### E1a — the JAMP2 diagonal

`JAMP2(i) = Σ_hel |JAMPᵢ(p)|²` — MG's `SELECT_COLOR` input. The combination loop
in `helas/eval/run.rs` `eval_m2` already walks `locs.chunks_exact(n_flows)` per
helicity combination and indexes each flow's complex JAMP out of
`scratch.scalars`; the diagonal is `norm_sqr()` on the same reads (note 15 §2.2).

Requirements:
- **σ must not move.** The accumulator is an additional output, never a change to
  the CF-contracted return value. Gate: `validate_helas_mg` stays bit-exact
  14/14, and `validate_sigma` rows are unchanged.
- Provide it as a *separate* entry point (or an opt-in out-parameter) so the hot
  integration path pays nothing — `eval_m2` is 0.5–1.7 µs and this sprint's
  perf-sensitive consumer is per-accepted-event, not per-probe.
- `n_flows == 1` short-circuits to `JAMP2(0) = Σ_hel |M|²` — trivially, but the
  NCOLOR=1 path in `eval_m2` is separate code, so it needs its own branch.

### E1b — the flow → `(colour, anticolour)` dictionary

Per external leg, LHEF wants an `ICOLUP(1..2, leg)` pair drawn from a colour-line
label pool (MG starts at 501). MG bakes exactly this table into
`Source/leshouche.inc` of every generated process, indexed
`ICOLUP(1|2, leg, iflow, iproc)`:

```text
# validation/madgraph/output/gg_to_gg/Source/leshouche.inc   (NCOLOR = 6)
DATA (ICOLUP(1,I,1,1),I=1, 4)/504,501,503,504/
DATA (ICOLUP(2,I,1,1),I=1, 4)/501,502,502,503/
...
# validation/madgraph/output/uux_to_uux/Source/leshouche.inc (NCOLOR = 2)
DATA (ICOLUP(1,I,1,1),I=1, 4)/501,  0,502,  0/
DATA (ICOLUP(2,I,1,1),I=1, 4)/  0,501,  0,502/
```

That file is the oracle — it is MG's `color_flow_decomposition` /
`get_color_flow_string` output already evaluated per process, so E1 does not need
to re-run MG's Python.

**The load-bearing hazard.** Do **not** transcribe `leshouche.inc` by flow index.
`ColorBasis::elements` is documented as MadGraph's JAMP order
(`sorted(ColorBasis.keys())`), and that holds through NCOLOR=2 — but note 16 §"JAMP-level
probe" records that at **NCOLOR=6 (`gg_to_gg`) vibegraph's basis and MG's 6 JAMPs
span the same space without being a 1:1 labelling**. (That premise turned out to be
false — see §E1c — but the conclusion below stands on its own: a transcribed table
agrees with MadGraph by construction and so cannot detect a mislabelling, whether or
not one exists.) An index-keyed transcription
would therefore be silently wrong exactly where it matters most, and — per the
AGENTS.md rule — a transposed or permuted dictionary is invisible to every
|M|²-level gate we own, because |M|² contracts the flows away.

So: **derive** the tag pair per leg from vibegraph's own basis key
(`BasisElement::structure` — a product of `T(...)`/`Tr(...)` chains over the
external colour indices, i.e. literally the colour lines), and then *check*
against `leshouche.inc`:

1. Where the bases coincide (all NCOLOR≤2 processes: `uux_to_uux`, `gg_to_ttx`,
   plus every NCOLOR=1 process trivially), assert the derived table equals MG's
   **element-wise, per flow index** — the strong form.
2. For `gg_to_gg` (NCOLOR=6), assert the weaker but still convention-sensitive
   invariant: the derived table and MG's agree **as sets of colour-line
   permutations up to relabelling of the line ids**, and the basis-change matrix
   relating the two flow bases maps one table onto the other. If establishing
   that basis change is more than a session's work, record the gg_to_gg row as
   INFORMATIONAL with an explicit statement of what it therefore cannot detect,
   and keep the strong form enforced everywhere else. **Do not** let the
   NCOLOR≤2 pass stand in for it silently.
3. Colour-line *ids* are arbitrary (any consistent relabelling is the same
   event); the test must compare the induced line **connectivity**, not the
   integers. Matching MG's 501-based numbering exactly is nice-to-have for a byte
   diff in E3, not a physics requirement.

Sampling: draw flow `i` with probability `JAMP2(i)/Σⱼ JAMP2(j)`, emit that flow's
tag pair per leg. Zero effect on σ.

**Gate:** the dictionary tests above; `validate_helas_mg` 14/14 bit-exact;
`validate_sigma` unchanged; a `JAMP2` unit check that `Σᵢ JAMP2(i)` and the
CF-contracted `|M|²` agree only where they should (they are *different* numbers —
the diagonal is not the contraction — so assert the relation that actually holds,
`Σ_hel Σᵢⱼ CF_{ij} JAMPᵢ* JAMPⱼ = |M|²`, and use the mismatch as the test that the
accumulator reads the right slots).

### E1 outcome (2026-07-27) ✅

**E1a.** `BoundAmplitude::eval_jamp2(momenta, scratch, &mut jamp2)` walks the same
`RootKind::Hels` `locs.chunks_exact(n_flows)` the CF contraction walks and
accumulates `norm_sqr()` per flow. It is a separate entry point — `eval_m2` is
untouched, so the integration path pays nothing. No `n_flows == 1` special case
was needed: the diagonal carries no CF weight, so the general loop already
degenerates correctly to `JAMP2(0) = Σ_hel |M|²` (the constant `CF(1,1)` is
deliberately left off; it cancels in the selection probability).

The correctness test (`eval_jamp2_is_the_diagonal_of_the_slots_eval_m2_contracts`)
pins the slots through one shared per-`(combination, flow)` JAMP dump: its CF
contraction reproduces `eval_m2` bit-for-bit, and its diagonal reproduces
`eval_jamp2` bit-for-bit. It also asserts `Σᵢ JAMP2(i) ≠ |M|²` on the
non-orthogonal processes, so a later "simplification" that returns the
contraction cannot pass.

**E1b.** `helas/color/flow_tags.rs` derives, per basis key, the flow's colour
lines from its `T`/`Tr` chains and emits one `(colour, anticolour)` pair per leg
(`ColorFlowTags`), plus `select_flow`/`ColorFlowTags::select` for the
`∝ JAMP2(i)` categorical draw. `AmplitudeEvaluator` computes the table at compile
time and exposes it as `color_flow_tags()`.

Two conventions came out of the derivation, both now pinned:

1. **The chain reading.** `T([a₁…aₙ], i, j)` links `(leg i, 3) — (a₁, 3̄)`,
   `(a_k, 3) — (a_{k+1}, 3̄)`, `(aₙ, 3) — (leg j, 3̄)`; `Tr` closes the same links
   cyclically.
2. **The crossing rule.** MadGraph's colour structure treats every leg as
   outgoing, so a leg's index rep is its particle's rep when outgoing and the
   conjugate when incoming, while `ICOLUP` slot 1/2 are the *physical*
   colour/anticolour. `color_flow_tags` therefore checks per flow that the
   occupied slots are exactly the ones the leg's particle rep allows; flipping
   the rule puts an incoming quark's line in its anticolour slot and the check
   fires (unit test `crossing_rule_is_not_free`).

**The `gg_to_gg` NCOLOR=6 row landed in the STRONG form, not informational.** The
plan above (following note 16) expected it not to. It was wrong about *what* note
16's caveat covers: the caveat is that the per-flow JAMP *values* do not match MG
element-wise, but the basis *keys* do — the CF oracle's ordering cross-check
reports no `ORDER-DIFF` for `g g > g g`, i.e. vibegraph's six sorted keys are
`Tr(1,2,3,4), Tr(1,2,4,3), Tr(1,3,2,4), Tr(1,3,4,2), Tr(1,4,2,3), Tr(1,4,3,2)`,
exactly MadGraph's six CF structure comments in order. So flow indices are
directly comparable and `color_flow_tags_oracle` asserts connectivity
element-wise per flow index for all 24 MG subprocesses.

What the oracle deliberately cannot detect: the colour-line **integers**. It
compares the induced connectivity (the set of `(leg, slot)` endpoint pairs
sharing a label), because any consistent relabelling is the same event. Label
equality is reported as information — 20/24 subprocesses come out identical to
MadGraph's 501-based numbering; the 4 gluon-initiated ones (`gg_to_gg`,
`gg_to_ttx`, both `gg_bbx`) are relabellings. A byte-level `.lhe` diff in E3 must
therefore normalise colour labels rather than compare them literally.

Mutation-checked: permuting flows 0↔1 in the derived table fails 7 subprocesses
including `gg_to_gg`, so the oracle really is sensitive to the flow labelling
that |M|² provably cannot see.

**Carried to E2 — resolved, see E1c below.**

**Gate observed.** `cargo test` all green (440 lib + integration suites);
`validate_helas_mg` **14/14 bit-exact/at-tolerance, unchanged** (`uux_to_uux`
5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14); `validate_sigma` 11 GATE
rows unchanged; `validate_diagrams` 16/16; `validate_helas` 2/2;
`color_cf_oracle` 24/24; new `color_flow_tags_oracle` 24/24.

### E1c — the NCOLOR=6 JAMP question, closed (2026-07-27) ✅

Diagnosis session on the caveat E1 carried forward. **There is no discrepancy.**
vibegraph's per-flow JAMPs equal MadGraph's element-wise under the identity flow
pairing — every flow, every helicity, all 5 banked phase-space points — up to a
single global phase `g` (`-i` for `uux_to_uux` and `gg_to_gg`, `+i` for
`gg_to_ttx`), the same `i`-placement convention already visible at the
per-diagram `AMP()` level. Max element-wise deviation 3.7e-16 at NCOLOR=6.

The caveat was an artifact of `compare_amps.py`'s greedy overlap matcher. At tree
level the four-gluon amplitude is MHV, so by Parke–Taylor every colour-ordered
partial carries the same helicity dependence ⟨ij⟩⁴ and the `[flow × helicity]`
JAMP matrix is **rank 1**. Every pair of rows then has overlap exactly 1, greedy
max-overlap pairs them arbitrarily, and the `|r|` values it prints are the norm
ratios of mismatched rows. (`gg_to_ttx` is rank 1 too and passed only by luck of
enumeration order — the rig was never sound for either.) The matcher now prefers
the identity pairing on ties, and all three multi-flow processes report MATCH.

**`JAMP2` is unaffected, so E2's colour selection is sound at NCOLOR=6.**
`|g| = 1` to 4.4e-16, and a phase dies in `norm_sqr`; `eval_jamp2` reproduces
`Σ_hel |JAMPᵢ^mg|²` to 1.1e-15 for `gg_to_gg`. E2 needs no compensation and no
NCOLOR≤2 restriction.

Pinned by `vibegraph-lib/tests/color_jamp_oracle.rs` (new, `extended-validation`,
3/3), against `validation/madgraph/jamp_reference.json` — MadGraph `JAMP()` values
banked by `gen_jamp_reference.py`, committed since the MG output tree is
gitignored. `g` is fitted once per process by least squares over every
(point, helicity, flow) entry, so it has nowhere to hide per-flow structure, and
`|g| = 1` is asserted separately — without that, a uniform rescaling would be
absorbed by the fit and would silently rescale every selection weight.
Mutation-checked against the four error classes |M|² cannot see: a per-flow phase
(0.1 rad → 6.7e-2), a per-flow rescale (1.001 → 6.7e-4), a flow permutation
(2↔3 → 1.1e0) and a global rescale (→ the `|g|` assertion) each trip it.

Two claims verified rather than assumed:
- CF at NCOLOR=6 is `(7/2)I + P − (1/3)J` with `P` the trace-reversal involution
  `(1,6)(2,4)(3,5)`; eigenvalues 5/2 (×4), 9/2 (×2) — positive definite, so a
  CF-null flow combination does not exist. The danger was never a null vector, it
  was CF's large automorphism group, which permutes `JAMP2` while fixing |M|².
- `color_cf_oracle` reports no `ORDER-DIFF` for `g g > g g`, so the basis ordering
  really is the identity and the element-wise comparison is meaningful.

**Known blind spot, documented not closed.** For `gg_to_gg` the trace-reversal
partners carry *identical* JAMPs (`J₁ = J₆`, `J₂ = J₄`, `J₃ = J₅`, read straight
off MadGraph's `JAMP(1,1) = JAMP(6,1) = 2·(AMP(3)+AMP(6)−AMP(1)−AMP(4))`).
Swapping such a pair is invisible to the JAMP values, to `JAMP2` and to |M|² —
but not to `color_flow_tags_oracle`, whose `leshouche.inc` connectivity comparison
distinguishes the two orientations. The pair ambiguity is covered there.

**Consequence for E3.** None beyond what E1b already recorded: the flow → `ICOLUP`
dictionary is unchanged, and the label-relabelling caveat for gluon-initiated
processes still stands (normalise colour labels before any byte-level `.lhe`
diff). What changes is that pure-gluon colour-flow *statistics* are now known
correct, not merely unfalsified — so a shower handoff on `g g > g g` is on the
same footing as NCOLOR≤2.

**Gate observed.** `cargo test` all green (440 lib tests + integration suites);
`validate_helas_mg` **14/14 unchanged** (`uux_to_uux` 5.61e-14, `gg_to_ttx`
1.89e-15, `gg_to_gg` 8.25e-14); `color_cf_oracle` 24/24;
`color_flow_tags_oracle` 24/24; `color_jamp_oracle` 3/3 (`uux_to_uux` 2.4e-16,
`gg_to_ttx` 2.7e-15, `gg_to_gg` 3.7e-16 element-wise; JAMP2 ≤ 1.8e-15).

## E2 — `accept-reject` (+ `mg-single-helicity-bench`)

Unweighting over the frozen grid + Sprint-A multichannel sampler:

- `w_max` estimation from the `integrate` phase (survey the frozen grid, record
  the max weight in `IntegrateArtifact`), **overweight bookkeeping** (fraction of
  points above `w_max`, and their weight share — a silent overweight tail is the
  classic unweighting bug), and reported **unweighting efficiency**.
- **Per-event selection, not sampling** (note 21 §"Helicity & colour handling"):
  helicity `∝ |M_hel(p)|²` (`SELECT_HEL`), flow `∝ JAMP2(i)` (`SELECT_COLOR`,
  E1). Both are categorical draws off diagonal accumulators, with **zero effect
  on σ or the integrand**.
- Single-helicity evaluation through the *unexpanded* program becomes the hot
  path here, so `mg-single-helicity-bench` lands with it: vibegraph
  `eval_amplitude` at one fixed helicity, plus the MG single-config Fortran
  timing (its MATRIX1 driver hardcodes the helicity-sum loop, so this needs a
  generated-driver + `gen_amplitude.py` harness edit — that is the session's
  reference-data cost, and the reason it was deferred until it had a consumer).

**Gate:** the unweighted sample reproduces σ *and* Sprint A's L5 distributions
(invariant-mass / angular histograms) within MC error. Per the seed-sweep lesson,
sweep seeds before trusting the σ agreement.

### E2 outcome (2026-07-28) ✅

`unweight::Unweighter` (`vibegraph-lib/src/unweight.rs`) + the
`validate_unweighting` gate. Landed in two commits (`6a81986` selection
primitives, `9195d21` accept/reject) — the session was interrupted twice by
infrastructure failures, and committing at the seam is why nothing was lost.

**Plan correction — the channel draw.** The brief said draw the channel
`∝ σⱼ`; that is wrong. The channel must be drawn **`∝ w_maxⱼ`**. Only that rule
leaves the *kept* events distributed `∝ σⱼ` without a compensating per-event
weight: a channel offered a share of trials `∝ w_maxⱼ` and accepting at rate
`σⱼ/w_maxⱼ` contributes kept events `∝ σⱼ`, which is the whole point of
unweighting. Drawing `∝ σⱼ` would require re-weighting every accepted event and
would not be an unweighted sample at all. Overall efficiency is then
`σ / Σⱼ w_maxⱼ`, exactly the quantity the per-channel-grid session measured.

**Overweight treatment.** Overweights are *kept* at weight `> 1` (not clipped)
and counted three ways: rate, cross-section share, and excess share. Clipping
would bias σ low invisibly; keeping them preserves the estimator and puts the
distortion on the record.

**Measured** (`validate_unweighting`, 5 processes × 4 seeds, ~6 s, profiling
profile). Efficiency reproduces the predicted `σ/Σⱼw_maxⱼ` to ≤1% everywhere:

| process | chan | eff (pred) | largest chan share | overwt frac | wt share | max `w/w_max` | σ(events) vs VEGAS |
|---|---|---|---|---|---|---|---|
| `ee_to_mumu` | 2 | 2.224e-1 (2.226e-1) | 50% | 6.5e-5 | 2.9e-4 | 1.014 | pull −0.49, −0.100% |
| `uux_to_uux` | 2 | 1.210e-1 (1.206e-1) | 54% | 2.3e-5 | 2.0e-4 | 1.081 | pull +0.58, +0.310% |
| `gg_to_ttx` | 3 | 2.333e-1 (2.335e-1) | 38% | 1.5e-4 | 6.3e-4 | 1.031 | pull −0.60, −0.105% |
| `ee_to_tatah` | 5 | 1.064e-1 (1.060e-1) | 100% | 2.3e-5 | 2.5e-4 | 1.575 | pull +1.17, +0.356% |
| `ee_to_mumua` | 8 | 2.872e-2 (2.854e-2) | 29% | 1.5e-4 | 9.3e-3 | **8.393** | pull +1.63, +1.024% |

Shape agreement `χ²/dof` 0.42–1.18 over 7–16 bins, worst single-bin pull 2.14.
Seed spreads 0.42%–1.30%.

**The oracle is deliberately not self-referential**: σ from events is compared
both to the VEGAS σ *and* to an independent weighted estimator over the same
grids that uses a **different channel-selection rule**, so the reference cannot
share the generator's mistake.

**`ee_to_mumua` is the one to watch.** Its overweight tail is two orders of
magnitude heavier than every other process (weight share 9.3e-3, excess share
4.0e-3, a single event at **8.4×** its channel's assumed maximum), and its σ runs
+1.0% high with the widest seed spread. This is the photon-pole process that
`validate_sigma::probe_photon_pole_is_the_instability` already flags, so the
reading is that the finite-sample `w_max` scan under-resolves the same
photon-pole region the sampler struggles with — the extremum is biased low, and
badly so where the integrand has a near-singular spike. Not a blocker (the row
gates), but E4 should surface the overweight share per run rather than bury it,
and the honest fix is better pole coverage, not a fudged `w_max`.

**`mg-single-helicity-bench` deferred, and re-sequenced away from this sprint.**
The A6 go/no-go predicted accept/reject would make single-helicity evaluation the
hot path. It did not: per-event helicity is a *selection* off the `eval_hel_m2`
diagonal — one helicity-summed evaluation per accepted event, read off the
*expanded* program — so the unexpanded single-helicity path never became hot.
E3/E4 will not create that consumer either. Re-sequence it under whatever first
needs a single fixed helicity in a loop.

**Standing gates unmoved:** `validate_helas_mg` 14/14 bit-exact, 11 σ GATE rows
unchanged (`uux_to_uux` pull −1.94, `gg_to_gg` −0.53), `color_cf_oracle` 24/24,
`color_flow_tags_oracle` 24/24, `color_jamp_oracle` 3/3.

### E2 correction (made in E4, 2026-07-28)

E2 and E3 both recorded that `IDWTUP = -4` is *needed* because a strictly unit
weight cannot represent an overweight event. **That reasoning is wrong**, and the
claim is corrected here rather than quietly rewritten, because it is the kind of
"this convention comes for free" assertion the project treats as a hypothesis.

A unit-weight file represents the overweight tail perfectly well, as
**multiplicity** instead of weight: write an event `k = floor(w) + Bernoulli(frac
w)` times and `E[k] = w` exactly, so the sample it describes is the same one. What
`-4` actually buys is that the tail stays *visible per event* — a consumer reads
`w = 8.4` rather than seeing eight or nine copies of one point — and that the file
length is exactly the number of events asked for. What it costs is that `XMAXUP`
and the normalising mean are properties of the realised sample, so a `-4` writer
cannot stream without either holding the sample or replaying it.

E4 implements both, so the choice is now a flag rather than a necessity. The
`WeightStrategy::MeanCrossSectionPb` doc comment carried the same false claim and
was corrected with it.

**For E3:** events carry subprocess, helicity combination, colour flow index and
momenta from `FixedBeamIntegrand::select_event`; colour tags come from E1's
`ColorFlowTags`. Remember the E1b caveat — 4 gluon-initiated subprocesses use
different colour *integers* than MadGraph's 501 pool for the same connectivity,
so normalise colour labels before any byte-level `.lhe` diff.

## E3 — `lhef-writer`

LHE serialiser: `<init>` (beams, PDF ids, process ids, `XSECUP`/`XERRUP`/`XMAXUP`,
`IDWTUP`) + `<event>` (`NUP`, `IDPRUP`, `XWGTUP`, `SCALUP`, `AQEDUP`, `AQCDUP`;
per leg `IDUP`, `ISTUP`, `MOTHUP`, **`ICOLUP` from E1**, momenta, mass, `VTIMUP`,
`SPINUP`). `SCALUP`/`AQCDUP` come from `coupling::scales` (note 22) — and note the
recorded MG defect that **`SCALUP` ≠ μR** in MG's own output (note 07); match MG's
actual behaviour, not the naive reading, and cite which one the code implements.

**Pin the byte-level format against a banked MG `.lhe`** —
`validation/madgraph/output/*/Events/run_01/unweighted_events.lhe.gz` are already
on disk (13+ processes) and are what `validate_scales` already replays.

### E3 outcome (2026-07-28) ✅

`vibegraph-lib/src/lhef/` in four layers — `record` (the `<init>` and `<event>`
blocks as data), `write`, `parse`, `build` (assembling a record from what the
generator produces) — plus the `validate_lhef` gate. Commit `04d5da4`.

**Plan correction — the gate the brief asked for was not achievable, and the
brief's format reference was the wrong file.** Two separate corrections:

1. *No event-content comparison against MadGraph is possible.* We do not share
   MadGraph's random number generator, so our events are not its events. A
   per-event comparison of `.lhe` **content** — byte-level or field-by-field —
   would be comparing unrelated phase-space points. The user caught this
   mid-session and superseded the brief. Statistical (distribution-level)
   agreement between the two samples is a real question and is **deferred to a
   later validation pass**, which should design those comparisons properly.
2. *The banked files are not written by the Fortran.* The brief (following
   `validate_scales.rs`'s comment and note 07) pointed at
   `Source/rw_events.f`'s `(i2,i5,e16.7e3,3e15.7)`. That specifier writes the
   *intermediate* per-channel event files. MadGraph reads those back in Python
   and rewrites the delivered `unweighted_events.lhe` through
   `madgraph/various/lhe_parser.py`, whose formats are

   ```text
   init beam line:    "%d %d %e %e %d %d %d %d %d %d"
   init process line: "%e %e %e %d"
   event line:        "%2d %6d %+13.7e %14.8e %14.8e %14.8e"
   particle line:     " %8d %2d %4d %4d %4d %4d %+13.10e %+13.10e %+13.10e %14.10e %14.10e %10.4e %10.4e"
   ```

   The two disagree on **every line**: two exponent digits rather than three,
   nine significant digits in the scale fields rather than seven. A writer built
   against the Fortran specifier would not match the file a shower receives. The
   round-trip check below is exactly what caught this, and is the reason to keep
   it. Note 07's remark stands for what it describes — the *content* of those
   fields really does carry only seven significant digits, because it came
   through the Fortran first — but the *format* is Python's, so a banked file's
   scale fields end in two padding zeros.

**What replaced the gate.** MadGraph is still usable as a **format oracle**, with
no shared sampling at all: parse its own banked events into our record types,
write them back, require the bytes to be identical.
`banked_files_round_trip_byte_for_byte` does that for all **20** banked runs —
**198 747 events / 1 020 299 particle lines, byte-identical, first try**. That
pins field order, column widths, the exponent spelling, the `px py pz E` ↔
`[E, px, py, pz]` permutation, and the sign on a negative zero (MadGraph writes
the second beam's transverse components as `-0.0`) in one shot, against files a
real shower reads. The E1b colour-label caveat does not bite here: the round-trip
re-emits the integers it read, so it never compares labels across
implementations.

A round-trip is only evidence if it is sensitive, so
`the_round_trip_is_sensitive_to_every_convention_sensitive_field` mutates a parsed
banked file eight ways and requires each to break it: `MOTHUP` dropped, `MOTHUP`
order swapped, `ISTUP` sign flipped on the incoming legs, `ICOLUP` slots 1 and 2
exchanged, incoming momenta crossed to all-outgoing, the momentum tuple rotated,
the mass field replaced by the momentum's own invariant, `SPINUP` zeroed. All
eight fire.

**XML by library, records by hand.** `quick-xml 0.32.0` (new dependency, added
offline from the cargo cache) owns the *document*: the `<LesHouchesEvents>` root,
`<header>` and its comment, `<init>`, `<event>`, and skipping past the banner's
`CDATA` cards and the per-event `<mgrwt>`/`<rwgt>` blocks. It does **not** own the
bodies of `<init>` and `<event>`, which are Fortran fixed-format numeric records
and not XML content; those are parsed and written column by column. The reader is
configured with `check_end_names = false` — LHE banners in the wild are not
reliably well-formed, and a banner defect must not cost us the events. All 20
banked headers parsed without complaint. Content the writer *authors* rather than
carries (the `<generator>` element, the header comment) goes through the XML
writer so it is escaped; content it merely carries through (MadGraph's
single-quoted `<generator>` tag, the reweighting blocks) is re-emitted verbatim,
which is what makes the round-trip byte-exact.

**Two conventions decided and pinned, each by a test built with the alternative
separated out:**

- **`SCALUP` is the larger factorisation scale** (`build::scalup` =
  `max(mu_f[0], mu_f[1])`), which is both what the accord defines and what
  MadGraph writes (`sqrt(max(q2fact(1), q2fact(2)))`). It is **not** `μR`. Note
  07's "defect" is really a misreading hazard: the two coincide on every process
  whose clustering this crate computes, which is why no banked file can separate
  them, and why the pinning test constructs `EventScales { mu_r: 91.188, mu_f:
  [200.0, 50.0] }` by hand. The renormalisation scale reaches the record through
  `AQCDUP` instead.
- **`AQCDUP` is `αs(μR)` untruncated.** MadGraph's `unwgt.f` divides by a π
  truncated to eight digits and writes `αs·(1 + 1.7e-8)`; that is a defect of the
  field, not a convention of it, so we do not reproduce it. The test asserts the
  *size* of the difference (1.7e-8 relative) rather than a tolerance, so the
  choice cannot drift silently.

**Other decisions worth carrying forward.** `IDWTUP = -4` (`XWGTUP` in pb, σ =
the **mean** of the weights) — the only strategy that can represent E2's
overweight events, which carry weight `> 1` by design; `WeightNormalisation`
turns E2's dimensionless weights into it. `MOTHUP` is `[0,0]` on incoming legs and
`[1, n_in]` on outgoing ones. `VTIMUP = 0`. `SPINUP` is the selected helicity.
Masses are the model's pole masses, not `√(p²)`. No status-2 intermediate
resonances are written: MadGraph 3.6.6's own "intermediate particles wrongly
written to the event file" defect (note 07) is a live hazard and we have no
resonance reconstruction, so the record is flat 2 → n. The record type and reader
do handle status 2, since MadGraph's banked files contain it.

`FixedBeamIntegrand` gained `event_scales(momenta)` and `running_alpha_s()`, so a
record reports the scale its matrix element actually ran at rather than a second
prescription compiled off the same run card. `None` when nothing in the matrix
element moves with `αs` and so no prescription was installed — the caller then
takes `μF` from the run card, no cross section having depended on it.

**A file is a lossy record of the run that wrote it**, and the gate had to be told
so: `<init>` cross sections get seven significant digits, `XWGTUP` eight, momenta
eleven. Re-serialising what was read reproduces the file exactly; recovering the
*run* is only good to those precisions.

**What the E3 tests provably cannot detect** (AGENTS.md rule):

- **Anything about which event we generated.** The round-trip re-emits values
  MadGraph chose, so it is blind to every physics field being filled with a wrong
  number; the end-to-end test compares our record against our own generator, so it
  is blind to a wrong matrix element, cut or sampler. Covered by
  `validate_helas_mg`, `validate_sigma`, `validate_unweighting`.
- **Whether our events are distributed like MadGraph's.** Nothing here compares
  the two samples. This is the deferred validation-pass item.
- **The colour-line integers.** Only the induced connectivity is physical, and 4
  of MadGraph's 24 banked subprocesses relabel the same connectivity (E1b), so
  nothing compares labels.
- **Which helicity an event should have carried.** `SPINUP` is a selection off a
  diagonal accumulator; the end-to-end test only checks it is one of the
  subprocess's surviving combinations.
- **`SCALUP` as `μF` rather than `μR` on real kinematics.** Every closed-form
  clustering has them equal; the distinction is pinned only by the hand-built unit
  test, and measured on MadGraph's `2 → 6` runs by `validate_scales`.
- **Four-momentum conservation is weak for `2 → 2`.** The outgoing momenta are
  exactly back-to-back, so their printed components cancel digit for digit and the
  balance comes out at exactly `0`. What the check really tests there is the sign
  convention on the incoming legs (a crossed record would give `2√s`), not the
  printing.

**Gate observed.** `validate_lhef` 3/3: round-trip 198 747 events / 1 020 299
particle lines byte-identical across 20 runs (9.2 s); 8/8 mutations detected;
end-to-end `ee_to_mumu` 8 873 events (5.9 MB) and `gg_to_ttx` 10 872 events
(7.2 MB) written and read back, momentum balance `0.0` of `√s`, `|p² − m²| ≤
1.0e-11` of `s`, mean `XWGTUP` matches σ to 2.3e-8, every event's colour lines are
the selected flow's. Standing gates **unmoved**: `cargo test` green (485 lib tests
+ every integration suite); `validate_helas_mg` **14/14** bit-exact/at-tolerance
(`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14);
`validate_sigma` 11 GATE rows unchanged (`uux_to_uux` −1.94, `gg_to_gg` −0.53);
`color_cf_oracle` 24/24; `color_flow_tags_oracle` 24/24; `color_jamp_oracle` 3/3;
`validate_unweighting` unchanged (`ee_to_mumua` eff 2.872e-2, max `w/w_max` 8.393,
pull +1.63); `validate_scales` 400 000 comparisons over 140 000 events, worst
1.000 of budget. `cargo clippy` adds no new finding (the one pre-existing error in
`coupling/alphas.rs:210` is untouched).

**What E4 inherits.**

- `LheWriter::begin(sink, &LheInit, header) → write_event(&LheEvent)* → finish()`,
  streaming. `<init>` must be filled before the first event, from the integration:
  `XSECUP`/`XERRUP` from the `IntegrateArtifact`'s banked σ, `XMAXUP` from the
  largest `XWGTUP` the run will emit, `IDWTUP = WeightStrategy::MeanCrossSectionPb`.
- `SubprocessRecord::new(evaluator, model, evaluated)` once per subprocess, then
  `record.event(&momenta, &helicity, flow, EventHeader)` per accepted event.
  `momenta` is every external leg in `[E, px, py, pz]`, **incoming first with
  physical signs** — `integrand.beams()` chained with the outgoing momenta.
  `record.pdg()` supplies the `<init>` `IDBMUP` for a fixed-beam run.
- `WeightNormalisation::new(sigma_pb, stats.mean_event_weight())` then
  `.xwgtup(point.weight)`. Note the ordering constraint this creates: the mean
  event weight is a property of the *generated sample*, so a strictly streaming
  CLI either generates first and writes second (what `validate_lhef` does), or
  accepts `mean = 1` and reports the overweight share separately. E2's warning
  about `ee_to_mumua` (weight share 9.3e-3, one event at 8.4× its channel's
  maximum) is exactly why the second option is not free.
- `EventHeader::from_scales(id, weight, &scales, alpha_qed, alpha_qcd)` puts the
  right quantity in `SCALUP`; `integrand.event_scales(&momenta)` and
  `integrand.running_alpha_s()` supply the arguments. `alpha_qed` is `1/aEWM1`
  from the param card.
- `write::generator_element(name, version, note)` for the `<init>` trailer, and
  `parse::LheFile::parse` to read a file back — which is how E4 can check its own
  output without a downstream tool, though its gate should still be a real one.
- `validate_lhef::generate_and_check` is a working end-to-end template for the CLI
  path.

## E4 — `generate-cli`

`vibegraph generate <artifact> [--nevents N] [-o events.lhe]`: deserialise
`IntegrateArtifact`, **refuse** a run whose proc/run card does not match the one
that trained the grid (rather than re-taking raw flags), drive E2 → E3.

**Gate:** the emitted `.lhe` parses in a downstream tool, and σ recovered from the
event weights matches the `integrate` σ within MC error.

### E4 outcome (2026-07-28) ✅

`vibegraph generate <artifact> <proc-card> [--run-card …] [--nevents N]
[-o events.lhe] [--strategy …] [--seed …]`, plus the generic
`lhef::emit::UnweightStrategy` the output mode is chosen through. Commit
`8234ede`.

**The proc card is an argument, not an inference.** The artifact banks the
process string and the resolved run card, but not the compiled amplitude and not
the model import, so the process has to be re-enumerated from a card the caller
supplies — which is then *checked* rather than trusted. `card_mismatches` compares
the process string and **every** run-card parameter exactly, including the ones no
physics reads, and a difference is refused with the offending names listed. The
policy is deliberately blunt: the card is the fingerprint of the run, and deciding
case by case which differences are harmless is how a mismatched grid gets sampled
anyway. Refusal is tested on four separate mismatches (beam energy, a lepton `pT`
cut, a different process, an omitted `--run-card` falling back to the MG proton
defaults), each with the matching case as the control — a check that refuses
everything would otherwise pass a "matching cards work" test just as well.

**Replay is exact, not re-derived.** `αⱼ` enters each channel's weight, so an
integrand whose channel weights were *re-surveyed* would only match the one the
grids were trained on by accident.
`FixedBeamIntegrand::use_multichannel_with_alphas` installs the banked weights
directly, and `banked_channel_weights_rebuild_the_integrand_bit_for_bit` pins
`value_in_channel` bit-for-bit against the adapted integrand. Two traps found
while writing it, both now guarded inside the test:

- On a **two-channel** process the α-adaptation converges to uniform, where `αⱼ`
  cancels between a channel's weight and the mixture density — the comparison is
  then blind to whether the weights were installed at all. It failed to detect the
  obvious mutation (drop `set_alphas`) on `e+ e- > mu+ mu-`. The test now runs on
  `e+ e- > ta+ ta- h` (5 channels) and asserts the converged weights are *not*
  uniform before comparing.
- It also installs a deliberately skewed weight set and requires the same probes
  to move, so "the weights reach the mixture" is evidence rather than assumption.
  With both in place the `set_alphas` mutation fires.

**The weight-strategy interface.** `UnweightStrategy::emit(source, plan, sink)`
owns the generation loop rather than filtering events, because that is where the
two modes differ: `<init>` precedes the first event and carries `XSECUP`,
`XERRUP`, `XMAXUP`, so a strategy whose `<init>` depends on the realised sample
cannot stream.

| | `Buffer` (default) | `StochasticRounding` |
|---|---|---|
| `IDWTUP` | `-4` | `+3` |
| `XWGTUP` | pb, σ = their mean | `1` |
| `XSECUP` | the **sample's** σ (the accept/reject estimator) | the artifact's VEGAS σ |
| `XMAXUP` | largest weight actually written | `1` |
| overweight tail | kept as a weight `> 1` | kept as `floor(w) + Bernoulli(frac w)` copies |
| passes | one, sample held (~424 B/event at 2→2, ~42 MB at 100k) | one, streaming, no buffer, no seekable sink |

`Buffer` declaring the *sample's* σ rather than the integration's is what makes
the gate below a real comparison instead of a restatement of its own input.

Stochastic rounding's variance is `f(1−f) ≤ ¼` with `f = frac(w)`, strictly below
the `Var = w` of a Poisson resample with the same mean, and exactly zero on the
integer weights that are almost the whole sample; for `w ≤ 1` it degenerates to
plain accept/reject, so it touches only the tail. That is a distributional claim,
so it is pinned on controlled weights (`the_rounding_rule_is_not_free`): mean
`1.300` against `floor`/`ceil`/`round`, observed variance against `f(1−f)`, and
`< ½·w` against Poisson. The physics-level gate cannot see it — almost every event
of a real sample carries weight exactly `1`, where all four rules agree.

**The third mode is deferred and would be nearly free.** Streaming `IDWTUP = -4`
by deterministic two-pass replay needs only `EventSource::restart`, which is on
the trait and contract-tested (`restarting_the_source_replays_the_same_events`:
40 records and weights identical after a restart, accumulators reset with the
stream, and the mutation of dropping the reseed fires). A future implementation
measures the sample on the first pass, restarts, and writes on the second, with no
change to `emit`'s signature.

**Gate — the plan's "parses in a downstream tool" was narrowed by decision.** The
reader used is our own `lhef::parse`, and the weakness that accepts is explicit:
our reader and writer share their assumptions, so a **self-consistently wrong
format is invisible to it**. What carries the format evidence is E3's byte-for-byte
round trip of MadGraph's own banked files; a run through a real shower (Pythia via
pixi) is filed as deferred work, not claimed here.

Observed on `e+ e- > mu+ mu-` at √s = 91.2 GeV, 20 000 events per strategy
(`cli_generate`, 13 s in the default `cargo test`):

| | buffered | stochastic rounding |
|---|---|---|
| efficiency | 2.2429e-1 | 2.2269e-1 |
| overweight rate / share | 6.73e-5 / 3.04e-4 | 3.34e-5 / 1.52e-4 |
| largest `w/w_max` | 1.009 | 1.008 |
| σ(sample) | 2.034961e3 pb | 2.036752e3 pb |
| declared `XSECUP` | 2.034961e3 pb | 2.022623e3 pb |

against `integrate` σ = 2.022623e3 ± 1.75 pb: the buffered file's mean `XWGTUP`
sits **+0.610%** away, inside the ±2.915% band (`4/√N` for the sample plus the
integration's own error). Mean `XWGTUP` equals the declared `XSECUP` to `< 1e-6`,
which is what `IDWTUP = -4` promises a consumer, and `XMAXUP` bounds the largest
weight written. The two strategies, run on **independent seeds** (with a shared
seed the source hands them the same accepted events and the comparison would be a
tautology), agree on the `cosθ` shape at **χ²/dof = 0.952 over 10 bins, worst pull
1.83**.

**What the E4 gate provably cannot detect.**

- **A self-consistently wrong file format** (above).
- **Anything wrong with the integrand.** A wrong matrix element, cut or sampler is
  replayed faithfully and agrees with itself. `validate_helas_mg`,
  `validate_sigma`, `validate_unweighting` cover those.
- **The event labels.** Helicity and colour-flow selection move no weight, so a
  mislabelled event is invisible to every comparison here.
- **σ from a unit-weight file.** `XSECUP` is carried from the integration, so
  recovering σ from the `+3` file is exact by construction. The buffered file is
  where the σ comparison has teeth; the `+3` half is checked on shape and on unit
  weights instead.
- **The rounding rule on real events.** Almost every weight is exactly `1`, where
  every plausible rule agrees; the rule is pinned only on controlled weights.
- **A different model behind the same process string.** The artifact banks no
  model name or restrict card, so `import model sm-no_b_mass` versus
  `import model sm` with the same `generate` line is *not* caught. Fixing it means
  banking the model import, i.e. an artifact `format_version` bump — filed, not
  done. **Closed straight after E4**; see the next section.

**Scope notes.** `generate` covers fixed-energy partonic beams (`lpp = 0`) only
and refuses `lpp = 1` by name: the Drell–Yan map is integrated but not split into
sampling channels, so there is no `ChannelIntegrand` for it. `<init>` carries one
process entry (`LPRUP = 1`) and requires every subprocess to share an initial
state, which a fixed-beam run does by construction. `EBMUP` is `√s/2` per beam —
what the integrand actually generates — rather than the run card's `ebeam1/ebeam2`,
which would disagree on an asymmetric card.

**Gate observed.** `cargo test` green — 483 lib + 6 CLI unit + 3 `cli_generate` + 2
`cli_fixed_energy` + every other suite (baseline was 477 lib; +6 = 5 `lhef::emit` +
1 `hadronic`). `validate_helas_mg` **14/14 bit-exact/at-tolerance, unchanged**
(`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15, `gg_to_gg` 8.25e-14);
`validate_sigma` 11 GATE rows unchanged (`uux_to_uux` −1.94, `gg_to_gg` −0.53,
`ee_to_mumu_tata_qcd0` still INFO at +2.2%); `color_cf_oracle` 24/24;
`color_flow_tags_oracle` 24/24; `color_jamp_oracle` 3/3;
`validate_unweighting` unchanged (`ee_to_mumua` eff 2.872e-2, max `w/w_max` 8.393,
pull +1.63); `validate_lhef` 3/3 (198 747 events / 1 020 299 particle lines
byte-identical across 20 runs, 8/8 mutations detected). `cargo clippy` adds no new
finding beyond the pre-existing error in `coupling/alphas.rs:210`.

---

## Post-E4 correctness fix — model identity in the artifact (2026-07-28) ✅

**The gap.** `generate` refused a proc card or run card that did not match the one
that trained the grid, but `IntegrateArtifact` banked no model identity, and a
process string is not a model. `import model sm-no_b_mass` with a byte-identical
`generate` line and run card passed every check and produced events from a
different model than the grids came from. E4 filed it as a known blind spot rather
than improvising the schema bump; this session closes it.

**What now catches it.** `IntegrateArtifact` is `format_version = 3`, carrying

```rust
pub struct ModelIdentity { name: String, restrict: String, digest: String }
```

— the `import model` label (`sm-no_b_mass`) for the error message, and a digest of
the bytes the model was actually built from: for the interned SM, the compressed
pre-restriction blob plus that variant's restrict-card text
(`ufo::sm::sm_assets`); for a UFO directory, every source file the parser reads
(`ufo::REQUIRED_SOURCE_FILES`, now shared between the parser and the digest, so
they cannot drift) plus the restrict card the loader resolves.

`card_mismatches` compares both. A differing **label** is reported by name, in the
same style as the run-card mismatches. When the labels agree, a differing
**digest** is reported instead — that is the case the digest exists for and a name
provably cannot see: a restrict card regenerated with different contents under an
unchanged name. Resolution and identification share one path
(`GlobalConfig::load_ufo_with_identity`), so a banked digest can never describe a
different model than the one that was loaded.

The digest is **SHA-256** (`sha2`) over the model's **own serialized form** — the
bincode encoding of the restricted `ParsedModel`, the same encoding the interned
SM blob holds — and not over the UFO source files it was parsed from. Two source
trees that parse to the same particles, couplings, vertices and parameters are
the same model whatever the Python around them said, so a reworded comment or
reformatted whitespace must not refuse an artifact. The restriction is baked in
before hashing (`ParsedModel::apply_restriction`, split out of `into_model`), so
the restrict card contributes through its *effect* rather than its text, and the
directory path no longer re-reads the source files after the parser has already
read them. `UFOModel::load_with_digest` is the single path that loads and
identifies together.

Two false starts are worth recording. It first landed as a hand-written 128-bit
FNV-1a over the raw source bytes, on the grounds that no hash crate was in the
dependency set; that was the wrong trade — a hash function is not something to
write in-tree when a standard one is a dependency away. What the digest actually
needs is stability across builds and platforms, which is what rules out `std`'s
`DefaultHasher` (documented as unstable between releases). `digest_bytes` is
pinned by known answers against `shasum -a 256`, since a digest that changed
silently would refuse every artifact written before the change.

**The serialized-model digest then exposed a real defect.** Five collections
reachable from `ParsedModel` were `HashMap`/`HashSet` — `ParsedModel::
order_hierarchy`, `Coupling::orders`, `Vertex::couplings`, `ParameterSet::rdeps`
and `ParameterSet::zeros`. Their iteration order varies per map instance and per
process, so the serialized bytes — and therefore the digest — were **not
reproducible**: `vibegraph generate` would have refused artifacts written by
`vibegraph integrate` at random, for no physical reason. All five are now
`BTreeMap`/`BTreeSet`. Every use of them is a lookup, a membership test, a
`values()` scan or an insertion into another map, so the change is
behaviour-preserving; the bincode wire format for maps is unchanged, so the
committed SM blob still deserializes without regeneration. The guard is
`the_digest_survives_a_serialization_round_trip`, which is what caught it — and
`cli_generate` covers the cross-process case end to end, since it banks a digest
in one process and compares it in another.

**Any artifact written before this must be regenerated** — the prefix version
check refuses it by name before the body is decoded, as it does for any other
schema. Nothing in the repo, in `pixi.toml`, or in a test consumes a committed
artifact: every test writes its own with `vibegraph integrate`, so there was
nothing to regenerate.

**Tests, and what each provably cannot detect.**

| Test | Pins | Blind to |
|---|---|---|
| `generate::tests::a_different_restrict_variant_is_refused` | `sm` vs `sm-no_b_mass` under an identical `generate` line and run card is refused, reported as `model` | A digest that is wrong the same way on both sides — it never reaches the digest branch |
| `generate::tests::a_restrict_card_that_changed_under_its_name_is_refused` | Same label, different digest is refused | Whether the digest's *inputs* are the right bytes — it plants the digest directly |
| `ufo::identity::a_directory_digest_follows_every_source_file` | Each source file, and the restrict card's contents under an unchanged name, reach the digest | Whether the parser reads the same file set — that is why the list is a shared const |
| `ufo::identity::every_interned_variant_has_its_own_digest` | The restrict card reaches the SM digest (a blob-only digest would make all nine variants identical) | A blob that is stale — `interned_blob_matches_submodule_exactly` covers that |
| `ufo::identity::digest_is_framed_not_concatenated` | `("ab","c")` and `("a","bc")` differ | Collisions in general; 128 bits is the argument, not a test |
| `config::identity_follows_the_resolved_variant` | The identity handed back is the variant that was resolved | A digest that is wrong the same way on both sides |
| `cli_generate::a_card_that_did_not_train_the_grid_is_refused` (new case) | End to end: a proc card importing a different restrict variant is refused by the binary | Everything the unit tests above are blind to |

**Mutation checks** (each applied, run, reverted):

| Mutation | Result |
|---|---|
| label comparison → `if false` | `a_different_restrict_variant_is_refused` FAILS (the digest still refuses the run, but by the wrong name) |
| digest comparison → `else if false` | `a_restrict_card_that_changed_under_its_name_is_refused` FAILS |
| both comparisons deleted | `cli_generate::a_card_that_did_not_train_the_grid_is_refused` FAILS — and the accepted wrong-model run wrote 10 events at **σ = 2061.99 pb against the banked 2022.62 pb, +1.95%**, which is what the check is standing in front of |

### The compiled-program cache slot — designed, not built

The motivation: if diagram enumeration or the e-graph extraction work ever gets
expensive, `generate` should load a compiled `AmplitudeEvaluator` rather than
recompiling one. **Measured today it is not expensive** — 0.05–0.29 s of setup for
every process the CLI can currently run, against ~13 s for a 20k-event `generate`.
So this is prospective, and nothing was built.

**The key such a cache matches on** is `(model digest, process string, compiler
schema version)`. The first two are already banked side by side in the artifact —
they are the whole input to compilation — and the third belongs to the *cache
entry*, not to the artifact: it says which compiler wrote the program, which the
artifact cannot know. **No field was reserved.** A reserved field would be dead
storage in every artifact ever written for a consumer that may never exist, while
the key is already derivable from what is banked, so the cache costs no schema
bump when it arrives. The derivation is recorded in the doc comment on
`IntegrateArtifact` so the next person does not re-derive it.

Three things make it more than a `derive`, and all three are recorded with it:

1. **Nothing in `helas::eval` implements serde.** `Op`, `Node<T>`, `Sym`, `Const`,
   `Folded` and the layout/constant-pool specs would all need it, plus a schema
   version of their own — the arena encoding is an internal representation that has
   changed repeatedly for performance (three sprints running), so it cannot ride
   the artifact's version.
2. **"The compiled program" is not one fixed object.** `AmplitudeEvaluator::folded_hel`
   is an `OnceLock` built lazily on first `eval_m2`, and it is the large part.
   Whether the expanded arena travels with a cached program or is rebuilt on load
   is a decision about what the cache is for, not a detail.
3. **A pruned evaluator carries a kinematic contract.** `prune_zero_helicities`
   mutates the evaluator and is correct only for partonic-CM momenta with beams
   along ±z; replayed under other kinematics it is silently wrong — no error, just
   a missing set of helicities. The `pruned` flag must not travel unless the
   contract travels with it and is rechecked on load.

**Gate observed.** `cargo test` green — **488 lib** (+5: 4 `ufo::identity`, 1
`config`) + **8 CLI unit** (+2 `generate`) + 3 `cli_generate` + 2
`cli_fixed_energy` + every other suite; `cli_integrate` 3/3 under
`extended-validation`. `validate_helas_mg` **14/14
bit-exact/at-tolerance, unchanged** (`uux_to_uux` 5.61e-14, `gg_to_ttx` 1.89e-15,
`gg_to_gg` 8.25e-14); `validate_sigma` **11 GATE rows unchanged** (`uux_to_uux`
−1.94, `gg_to_gg` −0.53, `ee_to_mumu_tata_qcd0` still INFO at +2.2%);
`color_cf_oracle` 24/24; `color_flow_tags_oracle` 24/24; `color_jamp_oracle` 3/3
(`gg_to_gg` JAMP max_rel 3.65e-16); `validate_unweighting` unchanged
(`ee_to_mumua` eff 2.872e-2, max `w/w_max` 8.393, pull +1.63); `validate_lhef` 3/3
(198 747 events / 1 020 299 particle lines byte-identical across 20 runs).
`cargo clippy` adds no new finding beyond the pre-existing error in
`coupling/alphas.rs:210`.

---

## Sprint close-out (2026-07-28) ✅

**E1 → E2 → E3 → E4 plus one post-E4 correctness fix, all complete**, on branch
`event-output-lhef`. The pipeline now runs end to end: UFO model → diagrams →
HELAS amplitudes → multichannel phase space + VEGAS → σ + banked grids
(`vibegraph integrate`) → unweighted events (`vibegraph generate`) → a Les Houches
file.

| Session | Commit(s) |
|---|---|
| sprint opened | `5dfdba6` |
| E1 `jamp2-flow-select` | `2242c13` |
| E1c NCOLOR=6 JAMP diagnosis | `5820c59` |
| per-channel VEGAS grids (detour, before E2) | `d73fbca` |
| E2 `accept-reject` | `6a81986`, `9195d21`, `071a168` |
| E3 `lhef-writer` | `04d5da4`, `63b4657` |
| E4 `generate-cli` | `8234ede`, `3d84ea6` |
| post-E4 model identity in the artifact | `8f01091` |

**What the sprint delivered.** `eval_jamp2` and the `leshouche.inc`-checked
flow → `ICOLUP` dictionary (24/24 subprocesses, strong form including
`gg_to_gg` NCOLOR=6); the `color_jamp_oracle` that closed the note-16 NCOLOR=6
caveat as an artifact of a greedy matcher rather than a real discrepancy; the
`Unweighter` and its `∝ w_maxⱼ` channel rule with overweight bookkeeping; the
four-layer `lhef/` writer/reader pinned byte-for-byte against 20 banked MadGraph
runs; the `generate` CLI with a swappable weight strategy; and
`IntegrateArtifact` `format_version = 3`, which banks the model a grid was trained
under so a replay against a different one is refused rather than silently sampled.

**Deferred out of this sprint.**

| Item | Why |
|---|---|
| **Downstream-shower validation of the emitted `.lhe`** (Pythia via pixi) | The E4 gate reads its own output with `lhef::parse`, which cannot see a self-consistently wrong format. E3's MadGraph round trip is the real format evidence; a shower actually consuming our file is the missing one. |
| **Event-sample vs MadGraph statistical comparison** | Filed in E3. Distribution-level, including the empirical `SPINUP` and `ICOLUP` frequencies that E1/E2 pin only as rules. Needs designing as a validation pass. |
| **Streaming `IDWTUP = -4` (two-pass replay)** | Interface in place (`EventSource::restart`), implementation not needed while 100k-event runs buffer comfortably. |
| **Compiled-program cache in the artifact** | Designed above, deliberately not built: compilation is 0.05–0.29 s against ~13 s for a 20k-event `generate`. **Trigger:** setup climbing to a noticeable share of a generation run — richer diagram enumeration, or e-graph extraction becoming part of compilation. The key it would match on is already banked; the three obstacles are recorded with it. |
| **Proton-beam (`lpp = 1`) event generation** | The Drell–Yan map has no channel decomposition, so no `ChannelIntegrand`. |
| **`mg-single-helicity-bench`** | Re-sequenced in E2: accept/reject never made single-helicity evaluation the hot path, so it has no consumer yet. |

---

## Decided and done — per-channel VEGAS grids (before E2)

Note 21's addendum argued vibegraph's single grid over `1 + channel_ndim` (with
`u[0]` selecting the channel) should become one grid per channel, as MG does. Two
reasons it bore on *this* sprint: a single global `w_max` over a channel mixture
is set by the worst channel (directly costing E2's unweighting efficiency), and
the change alters the `IntegrateArtifact` schema — which E2/E4 are about to
harden around.

**Done in one session**, ahead of E2. Measurements and the full close-out are in
note 21 §"Addendum — one VEGAS grid vs. MadGraph's grid-per-channel". What E2
inherits:

- **`FixedBeamIntegrand::adapt_grids(neval, niter, seed)`** returns
  `Vec<ChannelIntegration>` (grid, `αⱼ`, `nevalⱼ`, per-term `VegasResult`) plus
  their sum, and is what `vibegraph integrate` now drives. `adapt_grid` (one grid
  over the mixture) survives as the comparison point the studies use.
- **`FixedBeamIntegrand::value_in_channel(j, u)`** — the `j`-th term's integrand
  over `channel_grid_ndim()` coordinates, the channel frozen. This is the
  function accept/reject samples.
- **`VegasGrid::draw(rng, x)`** — one point against a frozen grid plus its
  Jacobian weight, so a per-point weight `jac · value_in_channel(j, x)` is
  available without going through `sample_frozen`'s internal accumulation. This
  is the accept/reject primitive.
- **`IntegrateArtifact` is `format_version = 2`**, banking `channels:
  Vec<ChannelGrid>` (grid, `alpha`, `neval`, `sigma_pb`, `sigma_err_pb`,
  `chi2_per_dof`). A file at any other version is refused by
  `ArtifactError::UnsupportedVersion` — the version is read from the payload
  prefix *before* the body is decoded. `sole_grid()` is the accessor for a
  single-grid (Drell–Yan) run. Every artifact written before this session must be
  regenerated.

**Consequences for E2.** Unweighting efficiency improves by 1.7–2.9× on four of
the five processes measured and is 0.91× on `ee_to_tatah`, where a single channel
carries the entire `Σⱼ w_maxⱼ`. So E2 should:

- draw the channel per event `∝ σⱼ` (banked per channel in the artifact) and
  unweight against that channel's own `w_maxⱼ`; the overall efficiency is then
  `σ / Σⱼ w_maxⱼ`, which is the quantity to report;
- estimate each `w_maxⱼ` by a frozen scan on that channel's grid, budgeting
  draws by `nevalⱼ` — the per-channel maximum is an extremum estimate and is
  biased low at small sample counts, which is what the overweight bookkeeping is
  for;
- expect no benefit where one channel dominates `α`; the diagnostic to print is
  the largest channel's share of `Σⱼ w_maxⱼ`, not the channel count.

## Execution notes

- **`feature-dev`** agent (Opus), one session per agent, on branch
  `event-output-lhef` in the pre-created worktree
  `/Users/ncsmith/src/generators/vibegraph-event-output-lhef` (`.pixi`,
  `validation/madgraph/output` and the compiled MG probes COW-cloned from the
  main checkout; `pixi run -e madgraph` verified working there). Hard `cd`-verify
  before acting — worktree isolation has leaked into the shared checkout before.
- ff-merge to `main` at sprint close; the user decides the merge.
