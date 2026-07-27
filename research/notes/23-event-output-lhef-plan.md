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
span the same space without being a 1:1 labelling**. An index-keyed transcription
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
