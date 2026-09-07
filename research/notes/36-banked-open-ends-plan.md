# 36 — `banked-open-ends` sprint plan: closing the banked validation open ends

**Status: OPEN, planned 2026-09-07.** Wave 0 dispatched the same day (B0, B2,
B1 in parallel worktrees). Per-session landing records are appended to the
session paragraphs below ("Landed:") as they merge; §7 is the close-out slot.

The validation-slot sprint after `ufo-lorentz` (note 35). Its content is the
set of open ends the last three sprints named and banked as `info` or
`uncovered` cells, or as oracles they described but did not build. Two
sessions are feature-sized code changes that each flip banked cells to `gate`;
the rest are the cheap oracles and hygiene items around them. Nothing here
needs new MadGraph reference data, so no multi-hour regeneration is on the
critical path.

The standing rules are unchanged and binding: never a loosened tolerance; a
cell flips `info → gate` only on a recorded measurement inside the reference's
own error; a known-wrong informational comparison runs while a fix is under
construction; every convention claim is pinned by a test that would fail if it
were false; a sampler change is licensed by a seed sweep read against the
estimator's *measured* spread, never by a fixed-seed pull; and a pre-registered
may-move set precedes every change that re-rolls a sampling stream.

## 1. What the sprint should flip

Counted from `validation/manifest.toml` at `e73b158`: 176 measured cells, 166
✅ and 10 ⚠️, plus 4 ⏳ (long tier) and 24 uncovered.

| Cell | Now | Target | Session |
|---|---|---|---|
| `qqx_to_o8o8_toy_dcolor` integrals | info (σ −6.72%) | gate | B1 |
| `p3r3_to_p3r3_toy_epsilon` integrals | info (σ −5.95%) | gate | B1 |
| `p3r3_to_p3r3_toy_sextet` integrals | info (σ −6.67%) | gate | B1 |
| `ud_to_epemud_qcd0` samples | info (ICOLUP χ² ≈ 650 / 1 dof) | gate | B4 |
| `gg_to_gg_cg` integrals | uncovered (`dynamical_scale_choice = 3`) | gate or info, measured | B6 |
| `gg_to_gg_cg` samples | uncovered (same) | gate or info, measured | B6 |

New standing oracles, each with no reference data and no tolerance moved: the
incoming-leg column in `samples` (B2), the coupling-level oracle (B5), the
`SCALUP` column in `samples`, the `scale_draw_fallbacks() == 0` assertion and
the `RunningAlphaS::eval` guard (B6), and the seed-headroom census (B0). B3
moves the integrator onto MadGraph's channel set and re-gates nine σ rows.

Deliberately out, each with the reason recorded in TODO.md: the
`wpwm_to_wpwmz_cw` five-vector residual (its own sprint), the `ee_to_mumua`
+1.04% (reference-owned, chain D), the 2→6 heavy tail (performance), banking
`p p > j j j` (new coverage, hours of MadGraph), and the one-in-seven malloc
abort (not a deliverable — every brief carries the rerun-on-abort protocol).

## 2. Sequencing

- **Wave 0** — B0 (seed-headroom census), B2 (incoming-leg column), B1
  (massive fixed beams). B0 and B2 are read-only against current behaviour and
  form the baseline. B1 runs concurrently because its may-move set is exactly
  the three massive-incoming rows and nothing else; it **merges after B2**, so
  the new column is measured known-wrong on `main` before the fix flips it,
  and **after B0 has recorded**, so the census is of the pre-change streams.
- **Wave 1** — B4 (ICOLUP diagnosis), B5 (coupling oracle), B6 (hygiene
  bundle), in parallel worktrees; disjoint files.
- **Wave 2** — B3 alone. It re-rolls nine σ ladders including the capstone,
  so everything else is pinned first.
- **Close-out** — recount the manifest, re-run the collated report on a
  quiet host, fill §7, update TODO.md and the memory index.

Every session: worktree pre-created off `main` by the manager with the
reference data COW-copied in; first action is `cd` + toplevel/branch
verification; long commands backgrounded; gates run with `--skip-deps`;
one conventional commit per session with the `Assisted-by:` trailer; no
edits to `TODO.md`, `research/notes/` or memory. If the `vibegraph
integrate` child under `validate_samples_proton` aborts with libmalloc's
"pointer being freed was not allocated", rerun the binary once and report
**both** outcomes (the standing finding in TODO.md).

## 3. Wave 0

### B0 — seed-sweep headroom census (validation-dev)

**What.** AGENTS.md's ≥5-seed standard is not met by every gate statistic that
sits at a threshold. The timelike-floor cascade (note 34 §1.2) hit three such
cells one at a time behind cargo's abort chain, and B1 and B3 in this sprint
are stream-touching changes of the same class. Measure the exposure *once*,
before either lands.

**Census.** Enumerate every enforced statistic in the banked layer that is
formed on fewer than five seeds, or whose threshold was calibrated on fewer
than five, or whose recorded calibration is not in the code or manifest at all:

1. `validate_sigma.rs` — every `Plan::Gate` row runs the gate at the single
   `SEED`; the question per row is whether its `rel_tol` was set from a
   recorded ≥5-seed spread (`probe_*_seed_stability`, the SMEFT/toy sweep,
   the resonant and QCD sweeps) or from something thinner. Read each row's
   comment and the probe that backs it; where the calibration is thinner than
   five seeds or unrecorded, run five seeds at the plan budget and record.
2. `validate_samples.rs` — `GEN_SEEDS` is three; the gate statistic is the
   minimum p over rows × seeds × columns against `P_FLOOR = 1e-4`. Measure the
   two extra seeds on the columns closest to the floor (the doc comment names
   `ee_to_wpwm`'s `pt(w+)` at 1.6×) and report the five-seed minimum and how
   the floor headroom moves.
3. `validate_hadronic.rs`, `validate_unweighting.rs`, `cli_generate_proton`
   (`SIGMA_MAX_REL`), `validate_samples_proton` — the same question for each
   enforced bound: how many seeds formed it, how many calibrated it.

**Deliverable.** A headroom table, one line per enforced statistic:
`file · row · statistic · seeds forming it · seeds calibrating it · threshold ·
measured 5-seed value · headroom ×`. Written to a report file handed back to
the manager (the manager files it into §7 of this note). Where a threshold
turns out to have < 2× headroom over its five-seed value, apply the remedy
menu from note 34 — form the gate statistic over five seeds, or the
pre-registered info / reported-not-asserted prescription — **never a widened
threshold**, and record the measurement in the manifest note and the
constant's doc comment. Where the census finds nothing thin, say so with the
numbers; a "nothing to do" that is a table is the deliverable.

**Cost.** Five seeds × the rows that need them, at plan budget. Background
the sweeps; the whole census should fit inside a few hours of CPU on the
16-core host.

**Landed B0 (`44e4e04`, 2026-09-07).** Five `#[ignore]` probes, one per file
that enforces a seed statistic, and a headroom table over every enforced
banked statistic (filed in §7). Two seed counts raised to five with
before/after measured: `JJ_SEEDS` (dijet σ: 5 seeds `6.811101e8 ± 4.269e5 pb`,
rel +0.33%, χ²/dof 1.40) and the unweighting `GEN_SEEDS`. Three tolerance
cells under 2× headroom recorded, none widened: `ddx_to_epemg` 1.6×,
`gux_to_epemux` 1.9×, `pp_to_jj` 1.5×. Findings: every σ calibration comment
written before `e73b158` no longer reproduces while all thirteen written in
it do — the note-34 draw-performance commits moved the streams and the older
comments were never re-recorded (a re-recording session, after B3);
`ee_to_mumua`'s reported-not-asserted exemption is load-bearing (gate-seed
|pull| 3.56 > 3.5); six SM σ rows had no seed calibration at all, now
measured at 9.9×–27.9×; standardised thresholds (pulls, p-floors, χ²/dof)
are false-positive rates, and "form over five seeds" would raise the flag
rate on an extremum-vs-floor statistic, so they are reported, not judged.

### B2 — the incoming legs in the `samples` gate (validation-dev)

**What.** `lhef::observables::kinematics` builds every column from
`STATUS_OUTGOING` legs, so no `samples` cell compares the beam four-momenta or
their masses against MadGraph's record. The three massive-incoming toy rows
carry a 6–7% σ error and clear the KS floor comfortably (worst p 9.4e-4)
because every statistic is a normalised distribution of the outgoing state.

**Build.** A per-event comparison of the incoming legs — masses and `pz`,
which MadGraph writes in every banked `.lhe` — as a column set in
`validation/samples.rs`'s comparison. These are not distributions: on a
fixed-beam run they are constants of the process, so the right statistic is a
per-event maximum absolute deviation of each incoming leg's `(E, pz, m)`
against the banked record's, at a tolerance set by the record's own printed
precision (MadGraph's Fortran dialect prints eleven significant digits on
momenta, seven on masses — read the dialect off the file, `lhef/` already
does). On a proton run the incoming legs vary per event (`x₁`, `x₂`), so
there the column is the weighted KS on `E` per beam and the mass check alone.
Report the column in `SamplesRow` beside the KS and χ² cells so the collator
renders it.

**Expected reading on `main`.** The three massive-incoming rows **fail** the
new column by construction (this side writes `E = √ŝ/2`, `m` as the model's
mass, `pz = ±√ŝ/2`; MadGraph writes `E = 248.69635 / 251.29639`,
`pz = ±241.35011` on `p3 r3` at 250 + 250 GeV). Land those three as `info`
on the new column with the measurement in the manifest note — the
known-wrong comparison that B1 flips. Every massless row must pass it at the
printed-precision tolerance; a massless row that does not is a finding, stop
and report.

**Pins.** A hermetic unit test on a hand-built event pair that fails the
column when one incoming `pz` is perturbed by more than the tolerance and
passes under it; the `the_gate_rejects_a_sample_from_a_different_process`
negative control extended to the new column. Update
`docs/src/guide/12-validation.md`'s "what it cannot see" table row for event
samples, since the incoming legs are no longer in the blind spot.

**Landed B2 (`e96dbd9`, 2026-09-07).** `beam_columns` in `validation/samples.rs`:
per beam `E`, `pz`, `m`, exact at half the last printed digit read off the
file's own spelling where the field is constant, weighted KS where it varies
(proton rows). The brief's expected-fail set was wrong: **seven** rows carry
massive beams, not three — the three named plus `ll_to_qqx_toy_*` (10 GeV,
pz off by 2.0008e-1) and `tata_to_ttx_tensor4f` (1.777 GeV, 6.3145e-3) — and
their records were internally off-shell (model mass beside light-cone
momenta). All seven landed `info` on the column, every massless row at
deviation exactly 0, proton beam-KS minimum p 0.013. Ten SMEFT manifest
notes lost the now-false "blind to the incoming legs" claim.

### B1 — massive fixed beams (feature-dev)

**Defect.** `FixedBeamIntegrand` (`hadronic.rs`) builds both incoming legs on
the light cone at `beam_e = √ŝ/2` (`beams()`, `externals()`, `probe_scale`,
`point_scales_of` — four sites) and `prefactor()` takes the flux as
`1/(2ŝ)`, with `ŝ = (ebeam1 + ebeam2)²` from the CLI (`integrate.rs:631`,
`generate.rs:578`). MadGraph puts each beam on its own mass shell at the run
card's energy and boosts to the partonic centre of mass. The three
massive-incoming toy rows read σ low by 6.72 / 5.95 / 6.67 %, budget-flat, and
a 200-node Gauss–Legendre quadrature of the same compiled amplitudes lands on
the bank under MadGraph's convention (note 35 §10.9; the manifest notes carry
the numbers). The channel maps are **already right**:
`DiagramChannel` reads `beam_masses` off the diagram's legs and
`beams_at` → `beam_momenta` builds `(E_a*, 0, 0, ±k)` from the Källén
function. Only the integrand's own beams, the flux, the flat RAMBO path's
frame and the record are wrong.

**The kinematics, which this session also writes into the walkthrough.**
Beam `a` has run-card energy `E_a` and mass `m_a`; likewise `b`. In the
laboratory frame the beams are on shell along `±z`:

```
p_a = (E_a, 0, 0, +√(E_a² − m_a²)),   p_b = (E_b, 0, 0, −√(E_b² − m_b²)).
```

The partonic invariant is the square of their sum,

```
ŝ = (p_a + p_b)² = m_a² + m_b² + 2 (E_a E_b + |p_a| |p_b|),
```

which is MadGraph's `stot = m1² + m2² + 2 (pi1(0) pi2(0) − pi1(3) pi2(3))`
(`genps.f:676`, the second beam's `pz` negative). For massless beams this is
`4 E_a E_b`, which equals `(E_a + E_b)²` only at equal energies (B1 pinned the
unequal-energy case by a unit test after finding the earlier wording here
wrong); for `p3 r3` at 250 + 250 GeV it is `499.99275²`, and that is
the number the banked events carry, not 500².

In the centre-of-mass frame the two beams share one momentum magnitude and
split the energy by their masses:

```
E_a* = (ŝ + m_a² − m_b²) / (2√ŝ),   E_b* = (ŝ − m_a² + m_b²) / (2√ŝ),
|p*| = λ^{1/2}(ŝ, m_a², m_b²) / (2√ŝ),
λ(x, y, z) = x² + y² + z² − 2xy − 2yz − 2zx.
```

The flux factor in `σ = (1/F) ∫ dΦ_n |M|²` is the Møller invariant

```
F = 4 √((p_a·p_b)² − m_a² m_b²) = 2 λ^{1/2}(ŝ, m_a², m_b²) = 4 |p*| √ŝ,
```

which reduces to `2ŝ` when both masses vanish and is exactly what MadGraph
applies: `flux = 1/(2 √(LAMBDA(s, m(1)², m(2)²)))` (`genps.f:427`). The
relative error of the massless convention at `E ≫ m` is
`−(m_a² + m_b²)/ŝ · (1 + O(m²/ŝ))` on the flux alone, which is the O(m²/2E²)
figure note 35 §10.9 named on the rows that escaped by accident of scale.

The laboratory and centre-of-mass frames differ by a boost along `z` of
rapidity

```
y_cm = ½ ln((p⁰ + p³)/(p⁰ − p³)),   p⁰ = E_a + E_b,   p³ = |p_a| − |p_b|,
```

zero for equal masses at equal energies and `≈ 5.4e-3` on `p3 r3`. MadGraph
generates in the centre of mass, **writes the event record in the centre of
mass** (the banked `p3 r3` events have `Σ pz = 0` with unequal beam
energies), and applies its rapidity cuts in the laboratory frame by shifting
every rapidity by `y_cm` (`cuts.f`'s `rap()` reads `cm_rap`, set in
`genps.f:388`).

**Build.**

1. `FixedBeamIntegrand::new` takes the beam masses (or the two `ExternalLeg`s)
   alongside `sqrt_s`; the CLI derives `ŝ` from `(ebeam1, m_a, ebeam2, m_b)` by
   the formula above — a new small function beside `process_external_legs`,
   unit-tested against the banked `p3 r3` numbers (`√ŝ = 499.99275`,
   `E* = 248.69635 / 251.29639`, `|p*| = 241.35011`) and the `qt qt~`
   numbers (`|p*| = 244.94897` at 250 + 250 with `m = 50`).
2. The four beam-construction sites collapse onto one method that returns
   `beam_momenta(√ŝ, m_a, m_b)` — reuse the `diagram_channel.rs` helper
   (lift it to `phasespace/` if visibility needs it), so the integrand and
   the channel maps cannot disagree.
3. `prefactor()` takes `1/(2 λ^{1/2}(ŝ, m_a², m_b²))`.
4. `RamboChannel::new(sqrt_s, final_masses)` already produces the outgoing
   set summing to `(√ŝ, 0)`; confirm the flat path and the multichannel path
   both hand the amplitude `[beams*, outgoing]` in the centre of mass with the
   beams from step 2.
5. Cuts: the integrand's cut filter sees laboratory-frame rapidities, i.e.
   `y_lab = y_cm + y_cm(beams)`. Check whether any banked fixed-beam card
   with massive beams carries a rapidity cut (`etaj`, `eta_max_pdg`, …). If
   none does, still implement the shift, pin it by a unit test against the
   formula on a hand-built point, and say in the report that no banked cell
   reaches the branch.
6. The record: `SubprocessRecord` / `generate.rs`'s fixed-beam path writes
   the incoming legs with their masses and the centre-of-mass momenta from
   step 2, and the `<init>` block's `EBMUP` stays the run card's `ebeam1/2`
   (MadGraph: `ebmup(i) = ebeam(i)`, `setrun.f:194`). The B2 column is the
   oracle for this.
7. The scale prescription's beams (`point_scales_of`, `probe_scale`) take the
   same momenta; the clustering's beam-crossing measure reads them. Any change
   in `SCALUP`/`AQCDUP` on the three rows shows up in `validate_scales`'
   replay — those two colour-toy runs are in its clustered inventory (note 35
   §10.2), so a moved scale is loud.

**Falsifier, pre-registered.** After the change: the three massive-incoming
`integrals` cells land inside their references' errors (reference relative
errors 1.0e-3 / 4.3e-4 / 4.6e-4; five seeds at plan budget, χ²/dof read),
and **every massless-beam row is bit-identical** — same σ, same error, same
per-channel grids at the pinned seed — because for `m = 0` every formula
above reduces to the old one exactly (`λ^{1/2}(ŝ,0,0) = ŝ`, `E* = √ŝ/2`,
`y_cm = 0` at equal energies). `ll_to_qqx_toy_*` (`m_in = 10` GeV) and
`tata_to_ttx_tensor4f` (1.777 GeV) are the controls that *move* by
`O(m²/ŝ)` — record their before/after σ and confirm the move is inside their
gates; `ee_to_ttx_smlimit` is the control that must not move at all. The
may-move set is therefore: the three massive rows (large), the four
light-massive rows (tiny, inside gate), and nothing else.

**Flip.** With the falsifier met, the three `integrals` cells go
`info → gate` at a `rel_tol` set from the five-seed spread with headroom
(the toy-row convention in `validate_sigma.rs`'s `plan_for`), with the
manifest notes rewritten to what is measured. B2's incoming-leg column on
those rows goes `info → gate` in the same commit.

**Walkthrough.** Add a "Fixed beams" section to
`docs/src/guide/07-phase-space.md` (beside "Frames") carrying the kinematics
above in the chapter's own KaTeX style — the laboratory construction, `ŝ`,
the centre-of-mass energies and momentum, the flux, the massless limit, and
the boost that separates the cut frame from the record frame — and correct
`01-pipeline.md`'s bullet on the flux factor, which currently states the
massless `2ŝ` only. Cite `genps.f` for the flux and `stot` in the "MadGraph
compatibility" register the chapters use. The docs build
(`pixi run docs`) must pass; the CLI reference is unaffected.

**Report.** The before/after table for every fixed-beam σ row (the may-move
set with its measured moves and every other row's bit-identity), the five-seed
sweep on the three flipped rows, the `validate_scales` replay result on the
two colour-toy runs, the B2 column readings on the three rows, and the docs
diff.

**Landed B1 (`3469e7a` + `7d4b9e8`, 2026-09-07, rebased onto `d65d585`).**
`FixedBeams` derives the initial state from the run card's energies and the
legs' pole masses; `kallen`/`beam_momenta` moved to `phasespace/beams.rs` and
shared by integrand and channel maps; flux `1/(2λ^{1/2})`; lab-rapidity cut
boost when `lab_beta ≠ 0`; `EBMUP` = the run card's energies. Falsifier met:
27 of 34 fixed-beam σ rows byte-identical, the three massive rows land at
rel −3.19e-4 / +7.95e-4 / +3.80e-4 (five seeds χ²/dof 0.98 / 0.99 / 0.68,
ladders converging) and flip `info → gate` at `rel_tol 0.005`; the four
light-massive rows move by 2.5e-7–1.7e-6. The integrand's beams reproduce
all eleven of MadGraph's printed digits; B2's column is enforced on every
fixed-beam row (`MASSIVE_BEAMS` retired). Docs: "Fixed beams" section in
`07-phase-space.md`, the flux bullet in `01-pipeline.md`. Brief corrections:
the note's massless `ŝ = (E_a+E_b)²` holds only at equal energies (fixed in
§3); step 7 was wrong — `validate_scales` replays MadGraph's own record
momenta and is structurally insensitive to this change. Finding for B6: on
αs-free rows with a dynamical-scale card the record writes `SCALUP` =
`dsqrt_q2fact` (91.188) where MadGraph writes the clustered scale.
Report after the flip: 169 ✅ / 7 ⚠️ / 4 ⏳ / 24 uncovered.

## 4. Wave 1

### B4 — `ud_to_epemud_qcd0`'s `ICOLUP` (validation-dev)

**Standing finding.** The row's event sample fails its `ICOLUP` χ² at
642–664 on 1 dof (p ≈ 0) on every one of three seeds while kinematics and
`SPINUP` clear their floors. This is the **fixed-beam** record path
(`SubprocessRecord::new` reading `evaluator.color_flow_tags()`), not chain A's
relabelled-member mechanism (fixed and gated on the hadronic rows). The row
has two mixed (initial↔final) fermion lines and NCOLOR = 2; the standing
hypothesis is a colour-flow convention gap in the mixed-line topology's
flow → `ICOLUP` dictionary.

**Method.** Note 12's bit-exact-first discipline. Start from the banked run's
`SubProcesses/P1_qq_llqq/leshouche.inc` — MadGraph's own `ICOLUP` table per
flow — against `color_flow_tags` for this basis, entry for entry, before any
event is generated. Then the χ² detail: `Chi2Column::detail` carries per-key
counts (`MAX_CATEGORY_DETAIL = 32`), so the failing comparison already says
*which* connectivity is over- and under-populated; read that against the two
flows' `JAMP2` shares. Candidate causes, in order of cheapness to test:
the two flows' tags transposed for this leg ordering (a `u d` initial state
with both quarks continuing to the final state is the first banked row where
both colour lines are mixed); the `SELECT_COLOR` draw reading the flow index
against the wrong `ICOLAMP` row; a tag dictionary that is right per flow but
written with slot 1/2 swapped on one of the two incoming legs.

**Falsifier.** With the fix, the three-seed `ICOLUP` χ² p rises above
`P_FLOOR` with the same kinematics and `SPINUP` readings, **and** every
hadronic row's `ICOLUP` cell (dijet at p 0.105–0.263 and the rest) is
unmoved — the hadronic rows go through per-member tables and must not see
a change to the fixed-beam dictionary. If the fix would move a hadronic
cell, the diagnosis is wrong; stop and report. Flip the cell `info → gate`
on the measurement.

**Landed B4 (`96b0096`, 2026-09-07) — diagnosis only, the brief's cause
falsified.** `color_flow_tags` is byte-identical to `leshouche.inc`, labels
included, the `ICOLAMP` mask partitions 24/11 as MadGraph's `JAMP` rows do,
and a transposition would give χ² ≈ 17 000, not ~640 — all three candidates
above are out. Root cause: MadEvent's `SELECT_COLOR` takes `ICONFIG` from
the *integration channel* the point was generated in, so the written
`ICOLUP` marginal is the multichannel weight share, and this row's card sets
`sde_strategy = 2`, under which `AMP2(J) = GET_CHANNEL_CUT(P, I)` — a
product of propagator denominators with no amplitude in it — over
MadGraph's 21 *merged* configurations. Our `select_color_flow` draws
`∝ AMP2` (the `sde_strategy = 1` weight) over one configuration per
diagram. Flow-2 fraction on MadGraph's own events: written 0.81850 ±
0.00386; `results.dat` channel share 0.82074; `GET_CHANNEL_CUT` over the 21
merged configs 0.82181 (0.86σ); our per-diagram `AMP2` share 0.91731
(25.6σ), which our three seeds realise to three digits. Neither ingredient
alone suffices (`GET_CHANNEL_CUT` over 35 per-diagram configs gives 0.782,
χ² ≈ 54); both together predict χ² ≈ 0.5. The only banked run that is both
`sde_strategy = 2` and NCOLOR > 1, which is why only this row shows it.
`EventScaleSource::draws_configuration()` already encodes the
`sde_strategy == 1 && tmin_for_channel == -1` conjunction on the scale path;
the colour path is the asymmetry. **The fix is B3's** (below). Also
recorded: the row's χ² drifted 642–664 → 590–671 on the same seeds before
this sprint; `pp_to_jj`'s quoted `ICOLUP` band 0.105–0.263 is stale (reads
0.123/0.265/0.020); `R_SDE_STRATEGY` in `runcard/classes.rs` understates
the field's reach (close-out bookkeeping).

### B5 — a coupling-level oracle ahead of the amplitude gate (validation-dev)

**What.** For every banked `mg_amplitude` row, compare this crate's coupling
values on the row's own `param_card.dat` against MadGraph's *Fortran runtime*
values, read from the f2py module `build_amplitude.sh` already builds (the
`couplings` common block after `SETPARA`), bit-level per coupling. When the
two disagree, evaluate the same card through MadGraph's Python
`model_reader` as the arbiter. This turns the `ee_to_zh_smeft` diagnosis
(note 35 §3 E1: the Fortran writer prints the UFO's `11/24` literal at seven
significant digits, a 1.2e-8 on `GC_303`) into a standing check that names
the coupling rather than a tenth-digit spread in per-diagram constants.

**Build.** A generator (`validation/madgraph/gen_couplings.py`, run by the
`generate-amplitude-tables` chain or its own task) dumping
`{coupling name: (re, im)}` per row from the Fortran module and from
`model_reader`; a committed table under `validation/madgraph/couplings/`;
a hermetic test `coupling_oracle.rs` comparing this crate's evaluated
couplings for each row's model + restriction + card. Two tolerances: exact
(≤ 1 ulp) against Python — the arbiter — and reported against Fortran, with
the known `GC_303` class listed by name and value. A Fortran/Python
disagreement not in the known list fails the gate; a crate/Python
disagreement fails the gate.

**Blind spot, recorded.** A rounding both sides share is invisible here as it
is to the amplitude gate; say so in the test's doc comment.

**Landed B5 (`a8a19e0`, 2026-09-07).** `gen_couplings.py` banks, per
`mg_amplitude` row (41), the couplings from MadGraph's Python `model_reader`
and from the f2py module's `COMMON/COUPLINGS/` after `SETPARA`; hermetic
`coupling_oracle.rs` compares the crate's couplings against both
(`PYTHON_REL_TOL 1e-13`, `FORTRAN_REL_TOL 1e-14`, measured worst agreeing
gaps 8.85e-15 crate-vs-Python on the SM, 3.0e-16 Fortran-vs-Python);
`[[standalone]] couplings-mg`, `pixi run -e madgraph generate-couplings` /
`validate-couplings`. Findings: (1) the `GC_303` writer rounding (1.197e-8)
is on **two** rows, `ee_to_zh_smeft` and `wpwm_to_wpwmz_cw`, and nothing
else on any row deviates Fortran-vs-Python; (2) **a crate-side parser
precedence bug** — `ufo/expr.rs` binds unary `-` tighter than `**`, so
`-ee**2/(2.*cw)` reads as `(-ee)**2/…` (sign flip plus a 2.4e-16 spurious
imaginary part from `powc` on a negative base). Reach, machine-checked over
every expression in every model the repo loads: SM `GC_7`/`GC_54` (Goldstone
vertices, unreachable at tree level in unitary gauge) and SMEFTsim's `dWT`
(reached only through the `T1` custom propagator). No banked cell affected;
landed as `KNOWN_CRATE_DEFECTS`, required present so the entry cannot
outlive its cause. **B7 below fixes it.**

### B6 — hygiene bundle (validation-dev; Sonnet is adequate)

Four independent items, one commit each:

1. **`SCALUP` column in `samples`.** Add `SCALUP` (and `AQCDUP`) as a
   compared column — KS on a fixed-beam clustered row, exact-constant on a
   fixed-scale row. Finding it will surface: rows that compile no scale
   prescription emit the run card's `SCALUP` and `AQCDUP = 0`
   (`generate.rs`'s `static_scale` arm) while MadGraph's own `ee_to_mumua`
   events carry a clustered channel-dependent `SCALUP`. Land the column
   with those rows `info` on it and the discrepancy recorded in the manifest
   note; do **not** change what the record writes — that is a decision for
   the close-out (it is the "no scale prescription" convention).
2. **`scale_draw_fallbacks() == 0` on the gated rows.** One assertion in
   `validate_sigma.rs` / `validate_hadronic.rs` after each gated integration
   that the counter is zero, so the silent NaN-`AMP2` path becomes loud.
3. **`RunningAlphaS::eval` below ~0.5 GeV.** The two-loop `newton1` seed
   takes `ln` of a negative argument and returns NaN silently. Match
   MadGraph's own behaviour at that scale (read `alfas_functions.f`; it
   either clamps or stops) and pin it by a unit test at `q = 0.3 GeV`.
4. **`dynamical_scale_choice` 1–5 on the fixed-beam path.** The closed forms
   are transcribed from `setscales.f` and unit-tested through
   `ScaleChoice::compile`; `from_run_card` refuses them
   (`UnhonouredScaleChoice`) because nothing reads them. Wire them into
   `FixedBeamIntegrand::use_running_coupling` via `compile_scale_source`, keep
   the refusal for the proton path (no oracle there), and fill
   `gg_to_gg_cg`'s two `uncovered` cells: σ at `= 3` (H_T/2) against the
   bank, and its `samples` cell. Land both `info` first; flip on measured
   agreement with a five-seed sweep. `validate_scales`' `declined_runs_decline_
   for_the_declared_reason` names `gg_to_gg_cg` as declined for exactly this
   reason — promote it into the replay inventory in the same change, since
   the blocker lifting is what that test exists to notice.

**Landed B6 (`dbf2fd1`, `302a4c9`, `f745cf3`, `eca0d15`, 2026-09-07).**
(1) `SCALUP`/`AQCDUP` join the `samples` comparison as scalar field columns
(B2's beam machinery generalised to `FieldColumn`), filled through a new
`FixedBeamIntegrand::record_scales` so the column measures what the shipped
binary writes. 18 rows gate, **27 are informational**: at fixed beams no
prescription is compiled when the matrix element carries no `αs`, so the
record writes the card's `dsqrt_q2fact` (91.188) and `AQCDUP = 0` where
MadGraph writes its clustered scale (`ee_to_mumu` 91.2, `ee_to_ee` 250,
`ee_to_ttx` 500, `p3r3` 251.2964) and a running coupling; at proton beams
the prescription is compiled and only `AQCDUP` falls back (waiver asserts
`alpha_qcd == 0`). The convention decision — what an αs-free fixed-beam
record should write — is a close-out item. Two self-corrections: the KS on a
printed field must round both sides onto the reference's grid (MadGraph
piles events on one printed value); the `<event>` line carries seven
significant digits whatever the dialect's width. Pre-registered watch:
`ee_to_wpwm`'s `pt(w+)` unmoved at 1.5728e-4. (2) `scale_draw_fallbacks()
== 0` asserted on every gated fixed-beam and hadronic integration; zero
everywhere. (3) `RunningAlphaS::eval` refuses a non-positive or non-finite
result; brief correction: `alfas_functions.f` neither clamps nor stops
there — it returns a `9d98` sentinel below the Landau condition, and the
NaN here starts higher (0.40 GeV at two loops) from the Newton iterate
going negative, which MadGraph does too. (4) `dynamical_scale_choice` 1–5
honoured at fixed beams (`ClosedForms::{Honour,Refuse}`); `gg_to_gg_cg`
replays 10 000 events / 20 000 scale comparisons at worst 0.999 of budget in
`validate_scales`, its `samples` cell flips `uncovered → gate` (worst KS p
2.7e-2, ICOLUP χ² p 7.4e-2), and its `integrals` cell lands `uncovered →
info`: five seeds mean rel −2.21e-3 at χ²/dof 1.01, a converged offset 2.6×
the reference's 8.5e-4 error, ladder settling not shrinking; `gg_to_gg`
under the card differing in that one field sits at +9.8e-6, so the offset is
localised to the coupling this `SCALE_FALLBACK_ROWS` member runs at across
the cut region — filed, not gated. Every other σ row identical to the
printed digit. Report after wave 1: 178 measured, 170 ✅ / 8 ⚠️ / 4 ⏳ / 22
uncovered.

### B7 — UFO expression precedence: unary minus under `**` (feature-dev; added after B5)

**Defect** (B5's finding). `vibegraph-lib/src/ufo/expr.rs`'s PEG grammar has
`power = unary "**" power / unary` with `unary = "-" primary / …`, so unary
minus binds tighter than exponentiation. Python — the language UFO
expressions are written in — has `factor: ('+'|'-') factor | power` and
`power: primary ['**' factor]`: `-a**2` is `-(a**2)`, and the exponent itself
is a `factor`, so `a**-b` parses. Three expressions in the repo's models hit
it (SM `GC_7`, `GC_54`; SMEFTsim `dWT`), none reachable by a banked row.

**Fix.** Adopt Python's grammar: `unary = "-" unary | "+" unary | power`,
`power = primary "**" unary / primary`, with `multiplicative` calling
`unary`. Pin with unit tests on `-a**2` (= `-(a²)`), `(-a)**2`, `a**-b`,
`-a**-b`, `2**3**2` (right-associative, 512), and the three model
expressions evaluated against Python's values from the banked coupling
tables.

**Falsifier.** `coupling_oracle`'s `KNOWN_CRATE_DEFECTS` entries for `GC_7`
and `GC_54` must stop deviating — the gate is built to fail when a listed
defect disappears, so delete both entries in the same commit and the test
must pass with the list empty. Everything else in the banked layer is
bit-identical: `pixi run --skip-deps validate` cell-for-cell against the
sprint branch, and `amplitude_oracle` byte-identical.

**Landed B7 (`f3425e2`, 2026-09-07).** Python's `factor`/`power` grammar
adopted verbatim (`unary = "-" unary | "+" unary | power`, `power = primary
"**" unary`), five unit tests incl. the three model expressions against
Python's arithmetic, and `KNOWN_CRATE_DEFECTS` emptied with the falsifier
firing as designed (`GC_7` 2.0 → 3.87e-16, `GC_54` 2.0 → 2.07e-15, imaginary
parts exactly 0). Brief corrections: the interned SM blob caches parsed ASTs,
so `sm_parsed.bin.zst` had to be regenerated in the same commit (any future
`ufo/expr.rs` edit must do the same; `sm_interned_blob` catches it only under
`extended-validation`); `dWT` has no banked Python value and is 0 under both
banked restrictions, so its pin is expression-level. Two further defects
found: `-x` left `im = −0.0`, sending `(−a)**0.5` to the wrong branch (fixed,
tested); a non-negative real base still goes through `exp(e·log b)` (`3**2 =
9.000000000000002`), and switching to `powf` moves 335 of 3254 model values by
~1 ulp — measured, **not landed**, filed as a follow-up needing its own
oracle before/after. Cell-for-cell report diff: 0 non-duration differences.

## 5. Wave 2

### B3 — MadGraph's channel set (feature-dev)

**What.** `config_groups` implements `IdentifyConfigTag` and
`amplitude_oracle` asserts the partition against `matrix1.f`, but
`AmplitudeEvaluator` still integrates one channel per config-carrying diagram
and `eval_amp2` (`run.rs:474`) accumulates `Σ|amp|²` per configuration where
MadGraph's merged accumulator is the coherent `|Σ AMP|²`. Correct so far only
because every configuration held one amplitude; note 35 §V2 measured that
wiring the rule changes the channel set on nine rows with gated σ.

**Build.** (1) `eval_amp2` coherent within a configuration: sum the member
amplitudes, then square — `config_amp_counts` already carries the spans. (2)
The integrator's channel set becomes `config_groups`' partition: one channel
per configuration, its propagator poles the group's shared tag, and the
per-event configuration draw (`∝ AMP2_c`) and the `ICOLAMP` mask keyed by
configuration. (3) Re-pin the capstone's channel count in
`cli_ufo_model.rs` (36 today; the new count is a measurement, assert it).

**May-move set, pre-registered.** The nine rows note 35 §V2 named:
`ee_to_ee`, `ud_to_epemud_qcd0`, `ee_to_ttx_smeft`, `ee_to_mumu_4f`,
`ee_to_ttx_dipole`, `gg_to_gg_cg`, `bbx_to_h_identity`, `gg_to_h_cpeven`,
`gg_to_h_cpodd` — plus every proton row whose subprocesses merge (list them
from `config_groups` before touching code; `p p > j j` is the one to watch).
Every row outside the set is bit-identical. Inside it: σ lands inside its
gate at five seeds with χ²/dof read, and the `samples` cells hold. The
amplitude cells are unaffected by construction (the oracle already folds
through the same partition) — assert that with a byte-identical
`amplitude_oracle` run.

**Report.** Per-row channel counts before/after, the five-seed table on the
nine rows, the capstone's new pin, and the bit-identity list.

**Added after B4 — the colour-flow draw under MadEvent's channel rule.**
Once the integrator runs on MadGraph's merged configurations, make
`select_color_flow`'s configuration draw MadEvent's: the configuration is
the sampled integration channel's (the `ICONFIG` MadEvent hands
`SELECT_COLOR`), whose marginal is the multichannel weight share; under
`sde_strategy = 2` (or `tmin_for_channel ≠ -1`) the per-configuration
weight is `GET_CHANNEL_CUT`'s propagator-denominator product
(`genps.f:1817`), not `AMP2`; under `sde_strategy = 1` with
`tmin_for_channel = -1` it stays `AMP2` as today. Reuse the
`draws_configuration()` conjunction so the scale and colour paths read one
rule. **Falsifier (B4's numbers):** `ud_to_epemud_qcd0`'s `ICOLUP` flow-2
fraction moves from ≈0.917 to ≈0.822 and its three-seed χ² from ~600 to
O(1) (predicted 0.5, p ≈ 0.48), and its cell flips `info → gate` with B2's
beam column enforced; every other `ICOLUP` cell is unmoved (all other runs
are `sde_strategy = 1`, `tmin_for_channel = -1`, so the rule reduces to the
present one). Decision recorded (manager, 2026-09-07): MadGraph's written
flow under its own card is the reference this suite reproduces, as
everywhere else; a `--madgraph-compat`-off alternative is the feature
backlog's, not this sprint's.

## 6. Risk register

- **B1's cut frame.** No banked massive-beam card may reach the rapidity
  shift; the branch would then be pinned by a formula test only. Recorded as
  such if so — not a reason to skip the shift.
- **B1 and B2 both touch the record's incoming legs.** B2 reads; B1 writes.
  B2 merges first; B1 rebases onto it before its flip commit.
- **B3's channel move changes the per-event configuration draw** on the
  merged rows, which feeds `SCALUP` on clustered rows. `validate_scales`
  replays MadGraph's own events and is unaffected; the `samples` `SCALUP`
  column from B6 is the oracle that sees our draw — B6 lands before B3.
- **Concurrent gates on one host.** Three worktrees running
  `pixi run --skip-deps validate` at once is a noisy host; timings from
  inside the sprint are not layer measurements (note 32 §5.3). Only the
  close-out's quiet-host run is.
- **The malloc abort.** Unattributed, 1 in 7 under the proton-sample suite's
  load. Rerun-once-and-report-both is in every brief.

## 7. Close-out

(filled at close)
