# `event-output-lhef` — unweighted event output (sprint plan)

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

## E4 — `generate-cli`

`vibegraph generate <artifact> [--nevents N] [-o events.lhe]`: deserialise
`IntegrateArtifact`, **refuse** a run whose proc/run card does not match the one
that trained the grid (rather than re-taking raw flags), drive E2 → E3.

**Gate:** the emitted `.lhe` parses in a downstream tool, and σ recovered from the
event weights matches the `integrate` σ within MC error.

---

## Open decision — per-channel VEGAS grids, before or after E2?

Note 21's addendum argues vibegraph's single grid over `1 + channel_ndim` (with
`u[0]` selecting the channel) should become `Vec<VegasGrid>`, one per channel, as
MG does. Two reasons it bears on *this* sprint: a single global `w_max` over a
channel mixture is set by the worst channel (directly costing E2's unweighting
efficiency), and the change alters the `IntegrateArtifact` schema — which E2/E4
are about to harden around. Estimated ~1–2 sessions.

Deferred, not dismissed. E1 is unaffected either way; the decision is due before
E2 opens.

## Execution notes

- **`feature-dev`** agent (Opus), one session per agent, on branch
  `event-output-lhef` in the pre-created worktree
  `/Users/ncsmith/src/generators/vibegraph-event-output-lhef` (`.pixi`,
  `validation/madgraph/output` and the compiled MG probes COW-cloned from the
  main checkout; `pixi run -e madgraph` verified working there). Hard `cd`-verify
  before acting — worktree isolation has leaked into the shared checkout before.
- ff-merge to `main` at sprint close; the user decides the merge.
