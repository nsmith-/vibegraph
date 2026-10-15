---
type: Session Report
title: "V1 report: the u u~ > w+ b w- b~ $ t t~ gap is MadEvent under-integrating the top-free continuum"
description: "Diagnosis of the ~2 % gap on the on-shell-veto t t~ row: it is not in the veto, the patch or the top windows. The same deficit appears on the top-free process (/ t t~); a non-adaptive third estimator agrees with vibegraph and sits 1.9 % above MadEvent; MadEvent's own iterations, recombined with equal weights, move it up by 0.5-1.2 %."
status: draft
tags: [process-grammar, onshell-veto, madgraph-defect, info-cell, top, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-15}
---
Diagnosis only; no source change is committed. Every number below is a run
made in this session (scratchpad, commands and logs under
`scratchpad/uux/`), except the banked `uu_tt` 10k seeds, which one fresh run
reproduced bit-for-bit (seed 20260925: 0.048922 ± 0.00018798 both times).

## Verdict

The gap is a **reference-side integration deficit in MadEvent on the
top-free continuum** `u u~ → W⁺W⁻ g*(→ b b̄)`, which carries ~94 % of this
row's σ. It is not the `$` veto, not the `FFV2P1D_1` patch and not the top
windows:

1. The row is continuum-dominated. MadEvent's channel table: the double-top
   configuration (`G2`) carries 0.00225 pb (4.6 %), the single-top `G1` 0.00013;
   the rest is the t-channel W-pair configurations (`G10`–`G16`). This side's
   `u u~ > w+ b w- b~ / t t~` (no top diagram at all) reads 0.045889 against
   the row's 0.049792.
2. **The top-free process shows the same deficit.** MadEvent (pinned 3.7.1,
   unpatched, no `$`) 0.045176 ± 0.000173 against this side's 0.045889 ±
   0.000035: −1.55 %, 4.0 σ.
3. **A non-adaptive third estimator sides with vibegraph.** At fixed μ = M_Z
   (α_s = 0.118 on all three), MadEvent 0.059867 ± 0.000227, vibegraph
   0.060942 ± 0.000055, third opinion 0.061029 ± 0.000123 (χ²/dof 0.46):
   vibegraph − third = −0.14 % (0.6 σ); MadEvent − third = −1.9 % (4.2 σ).
4. **Every window slice is low**, not one: MadEvent both-outside −0.8 %
   (3.5 σ), at-least-one-inside −1.2 % (3.9 σ), and MadEvent's unsliced run
   another −0.9 % below its own slice sum.
5. **Part of it is MadEvent's iteration weighting** (the `sigma-2to4-offset`
   mechanism): recombining each channel's iterations with equal weights moves
   MadEvent up by +0.80 % (top-free), +0.49 % (fixed μ), +0.47 % and +1.15 %
   (row, two 40k seeds).
   That closes about a third of the gap; the rest is iterations that still
   under-sample.
6. The under-sampled region, located from the third estimator's heaviest
   weights: both W's within a few degrees of the beams (W⁺ forward, W⁻
   backward, p_T ≲ 30 GeV), m_WW near √s, a light b b̄ pair (m_bb ≈ 10–25 GeV):
   the double t-channel d-quark exchange with a soft g* → b b̄. Both W's at
   |cos θ| > 0.99 hold 2.9 ± 0.25 % of σ (third estimator, 2e6 points, 8 %
   relative error on that slice): a rare, heavy-weight region as large as the
   gap. That MadEvent misses *this* region is inferred, not measured.

## 1. Patched reference at two budgets

The banked card (`onshell_uu_run_card.dat` + the row's overrides, dynamical
scale −1), patched tree (`aloha-p1d-flipped-fermion.patch` verified applied
by a reverse dry run; the generated `FFV2P1D_1` theta reads
`(p² − (M−cΓ)²)(p² − (M+cΓ)²)`), `use_syst = False` (systematics only; the
reproduced seed shows σ is unchanged).

| rung | seeds | σ (pb), unweighted mean | χ²/dof | events written |
|---|---|---|---|---|
| 10k (banked, seed 20260925 reproduced fresh) | 5 | 0.048808 ± 0.000199 | 6.30 | 1931 of 10000 (seed 25) |
| 40k | 2 | 0.049085 ± 0.000312 | 24.1 | 7463, 6101 of 40000 |
| vibegraph, `--target-rel 2e-3` | 5 | 0.049792 ± 0.000049 | 1.37 | — |
| vibegraph, `--target-rel 5e-4` | 1 | 0.049784 ± 0.000024 | — | — |

Per seed, MadEvent 10k → 40k: 20260925 0.048922 ± 0.000188 → 0.048773 ±
0.000090 (−0.30 %); 20260926 0.049214 ± 0.000157 → 0.049396 ± 0.000090
(+0.37 %). The two 40k seeds disagree by 4.9 combined quoted errors.

**Deviation:** the brief asked ≥ 5 seeds at the 4× rung. A 40k run took
4515 s on this host (2 cores, load 7–10 from siblings), so two seeds were
run. The drift with budget is therefore **not established**: the 10k rung's
own seed scatter (χ²/dof 6.3, seed 20260929 1.8 % below the others) is
larger than any 10k → 40k shift seen. What is established is that 4× budget
does not close the gap: the 40k mean is −1.42 % from this side, and its two
seeds scatter at χ²/dof 24 about each other — the signature of a quoted
error that does not see a missed region. Both seeds moved by less than the
10k rung's spread, in opposite directions.

MadEvent writes 1931 of 10000 requested events at 10k (7463 of 40000 at 40k),
while the both-outside slice alone writes 10000 of 10000 in 128 s: the
unweighting fails on the windowed full process, a symptom of a few very large
weights. Its top-free run shows the same (1704 of 10000).

## 2. Window decomposition (banked card, sliced σ, not event fractions)

Slices by cut: here an uncommitted `VG_PROBE_TOPWIN` filter in
`Cuts::pass` on m(W⁺b), m(W⁻b̄) (reverted; the build after the revert is
clean); MadEvent a `dummy_cuts` with the same test, |m − 173| < 15 × 1.4915.

| slice | vibegraph (5 seeds) | MadEvent 10k (seed 20260925) | MadEvent / here − 1 |
|---|---|---|---|
| both outside (OO) | 0.032959 ± 0.000032 (χ² 0.5, 5 seeds) | 0.032694 ± 0.000073 | −0.80 % (3.5 σ) |
| t inside only (IO) | 0.007555 ± 0.000007 | — | |
| t̄ inside only (OI) | 0.007554 ± 0.000007 | — | |
| both inside (II) | 0.001759 ± 0.000003 (χ² 3.9) | — | |
| at least one inside (IO+OI+II) | 0.016868 ± 0.000010 | 0.016660 ± 0.000053 | −1.23 % (3.9 σ) |
| sum of slices | 0.049827 | 0.049354 ± 0.000090 | −0.95 % |
| unsliced | 0.049792 ± 0.000049 | 0.048922 ± 0.000188 (same seed) | −1.75 % |

No slice carries the gap alone. The windowed slice is relatively lower than
the outside one, and MadEvent's unsliced run is 0.9 % (2.1 σ) below the sum of
its own slices. A veto defect would put the deficit in the windowed slice
only; a sampling defect in the continuum puts it in every slice and can
depend on how the space is cut up (the sliced runs also write far more of
their requested events: 10000 and 3371 of 10000, against 1931 unsliced). Both slices are dominated by the continuum
(MadEvent's per-channel table for the outside slice puts `G2` at 0.00225 of
0.0327).

## 3. Honest-error third opinion

Because the gap follows the continuum, the third opinion is on the whole
process at fixed μ = M_Z (α_s(M_Z) = 0.118 is the standalone's param-card
value; vibegraph and MadEvent run at `fixed_ren_scale = fixed_fac_scale =
True`, `scale = 91.188`). |M'|² is the patched MadGraph **standalone**
`SMATRIX` (BWCUTOFF 15), called on points from a fixed density written for
this purpose (numpy): three channels, (W⁺b)(W⁻b̄) with Breit–Wigner/flat
masses, (W⁺W⁻)(b b̄) with log-flat m_bb² and a propagator-shaped production
angle, and W⁺, W⁻ drawn in the lab with t-channel-shaped polar angles. No
grid, no adaptation; mean and standard error of iid weights. Its Jacobians
are checked against each other: |M|² = 1 gives the 4-body volume
2.4236e6 ± 0.011e6 (lab-W channel alone), 2.4312e6 ± 0.0064e6 (pairing A
alone), 2.4370e6 ± 0.0076e6 (pairing B alone).

| estimator | seeds × points | σ (pb) | χ²/dof |
|---|---|---|---|
| third opinion, 3 channels | 5 × 2e6 | 0.061029 ± 0.000123 | 0.46 |
| third opinion, 2 channels (no lab-W channel) | 5 × 2e6 | 0.060886 ± 0.000174 | 3.14 |
| vibegraph | 5 | 0.060942 ± 0.000055 | 1.09 |
| MadEvent 10k (seed 20260925) | 1 | 0.059867 ± 0.000227 | — |
| MadEvent, same run, equal-weight iterations | 1 | 0.060162 | — |

The two-channel sampler is itself the failure mode in miniature: one of its
seeds read 0.060207 ± 0.000217, 1.4 % and 4 σ low, and a 2e5-point test 4 %
low; adding the lab-frame W channel, which follows the forward-W region,
removed it. A sampler that misses that region reports a low σ with a small
error.

## 4. Paired iteration test (requested by the manager)

`G*/results.dat` kept for the runs made after the request (`iter_combine.py`,
adapted from `2to4-work/iter_seed.py`): per run, the reported σ, the
inverse-variance and the equal-weight combination of each channel's
iterations, summed over channels.

| run | reported | equal-weight | equal − reported |
|---|---|---|---|
| top-free, 10k | 0.045176 | 0.045537 | +0.80 % |
| row, fixed μ = M_Z, 10k | 0.059867 | 0.060162 | +0.49 % |
| row, banked card, 40k, seed 20260925 | 0.048773 | 0.049001 | +0.47 % |
| row, banked card, 40k, seed 20260926 | 0.049396 | 0.049962 | +1.15 % |

The pattern in the channels is the under-coverage one: in the top-free run's
W-pair channel `G8` the first three iterations read 0.0107, 0.0103, 0.0102
with the fourth results.dat column at 2–3, then later iterations find the
region and read 0.0114–0.0123 with that column at 25–70; the
inverse-variance mean (0.011267) is pulled to the early, confidently low
iterations. Its mirror `G9` reads ~0.0119 throughout with the same large
fourth column. The equal-weight sum still sits 0.8 % (top-free) and 1.3 %
(fixed μ) below vibegraph and the third opinion, so iteration weighting is
part of the mechanism, not all of it. At 40k the two seeds' equal-weight sums
are 0.049001 and 0.049962 (mean −0.62 % from this side, one of them +0.34 %
above it): the per-channel iterations themselves scatter by ~2 %, so the
equal-weight number is not a converged estimate either.

## 5. Attribution, and the veto on each side

**Attributed:** MadEvent 3.7.1, at the reference's budget and at 4× it,
under-integrates the top-free `u u~ → W⁺W⁻ b b̄` continuum by 1.5–2 %; the
`$ t t~` row inherits it because that continuum is 94 % of its σ. The fixed-μ
and unrestricted cross-checks (15.060 against 15.052) were blind to it: the
unrestricted row is 99.7 % double top, where the continuum's 2 % is 0.007 %.

**Falsified by:** (a) a MadEvent run at much higher survey/refine statistics
(or with the forward-W region sliced out and integrated separately) that
settles below the third opinion; (b) a top-free third opinion (standalone
`/ t t~`, same sampler) disagreeing with this side's 0.045889; (c) a slice
in |cos θ_W| > 0.99 showing MadEvent agrees there and the deficit elsewhere.
None of these was run; (c) is the cheapest decisive test.

**The veto on each side**, for the record (the decomposition does not point
there): MadEvent zeroes a marked line in the amplitude (`P1D` theta, so the
diagram's `AMP2` is zero on the window) and also rejects the point in any
channel whose configuration carries the line on the window (`cut_bw`,
`gForceBW = 2` at `sde_strategy = 1`); the rejection removes nothing the
amplitude has not already zeroed. The BW cut runs once per point, in
`dsample.f`'s call to `passcuts` on the unpermuted momenta with
`this_config` (the re-check inside `DSIGPROC` is commented out); for the
symmetric configuration 9 (= 1 with W⁺b ↔ W⁻b̄) the permuted term's forced t̄
line is the original's t line, so the rejection is consistent. This side evaluates the zeroed amplitude per window subset and
drops zeroed configurations from its AMP2 draws; it agrees with the
third opinion on the full row and on the top-free process.

**The sibling's mechanism** (`sigma-2to4-offset`: mean²/variance iteration
weights) fits in part: equal-weight recombination moves MadEvent up by
0.5–0.8 % here, the same sign and order as there. The brief's "MadEvent drifts
down with budget" is not supported by this session's data (10k → 40k: one
seed −0.30 %, one +0.37 %; rung means 0.048808 → 0.049085, inside the
scatter), so there is no sign conflict to explain;
the remaining 1–1.3 % is under-sampling that more budget does not fix.

## Brief corrections

- "This side reads 0.05059–0.05080" and "fixed μ = 173" are the backlog's
  fixed-μ numbers; the banked row is at the dynamical (−1) scale, where this
  side reads 0.049792 ± 0.000049 and MadEvent's banked mean 0.048808 ±
  0.000199 (+2.0 %). Both scale choices show the gap.
- "Pointwise amplitudes agree and the unrestricted process agrees" was read
  as exonerating the integrand. The unrestricted check cannot see a 2 %
  continuum error (it is 0.3 % of that row), so it was never evidence on this
  row's dominant part.
- The window decomposition does not localise the gap: it is in every slice.

## Banking request

None needed for the attribution. If the row is to be gated, the request is:
`gen_onshell_veto.sh` gains a row `uu_notop` (`generate u u~ > w+ b w- b~ /
t t~`, pinned tree, `onshell_uu_run_card.dat`, overrides `lpp1=0;lpp2=0;
ebeam1=250;ebeam2=250;nevents=10000`, seeds 20260925–20260929), and both it
and `uu_tt` keep each run's `G*/results.dat` so the equal-weight
recombination is a recorded measurement. Expected: `uu_notop` ≈ 0.0452 pb,
1.5 % below this side, the same offset as `uu_tt`.

## Gate results

- `cargo build --profile release-debug -p vibegraph` after reverting the
  probe: `Finished ... in 4m 46s`, exit 0 (one pre-existing dead-code warning,
  `helas/eval/analysis.rs:208 resolve_mom`).
- `cargo test`, `pixi run --skip-deps validate`: not run; this commit adds one
  Markdown file and no source.
- `pixi run kb-index`: `wrote 4 index files`; `pixi run kb-lint`: `566 concepts,
  193 backlog items; 0 errors, 2 warnings` (both warnings pre-existing, on
  performance backlog items).

## Found

- **MadEvent under-integrates `u u~ > w+ b w- b~ / t t~` at √s = 500 by
  ~1.5 %** (0.045176 ± 0.000173 against 0.045889 ± 0.000035 here and the
  third opinion on the `$` row). Close: the |cos θ_W| > 0.99 slice on both
  sides, and a decision whether rows dominated by double-peripheral W
  emission are gated against MadEvent at all.
- **The `$ t t~` row's `info` note is wrong about its cause.** The manifest
  note and the cli_onshell_veto doc call the 2 % unexplained after the patch;
  it is the continuum (this report). Close: rewrite the note and decide the
  gate (against the third opinion, or with an explicit reference-side
  tolerance).
- **Equal-weight iteration recombination is not recorded for banked
  MadEvent rows.** `madevent_seeds.sh` records only `results.dat`'s headline
  value and error per seed (`vg_seed_result.txt`); recording `G*/results.dat` per seed would make the paired test a
  standing measurement. Close: keep the files in the run record.
- **vibegraph's both-inside slice is hard for its own grids:** seed χ²/dof
  3.9, and one seed ran 270 iterations with an in-run iteration χ²/dof of
  8–10 (0.35 % of this row's σ, so immaterial here). Close: look at which
  channel's iterations disagree in that slice.
