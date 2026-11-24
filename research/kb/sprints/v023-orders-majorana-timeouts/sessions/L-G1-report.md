---
type: Session Report
title: "L-G1 report: the g2HDM high-mass σ excess is the released chirality bug"
description: "On card 894 (M1000) the chirality fix of main a5bf8fca lowers σ by 3.0 ± 0.4% at fixed momenta, which takes v0.2.2's +2.8% over MadGraph 3.7.1 to −0.3 ± 0.5%; on card 889 (M300) it lowers σ by 0.6 ± 0.4%. The shift comes from subprocesses whose amplitude vanishes, or nearly so, once the charged-Higgs vertex has the right chirality; per diagram and per helicity, the fixed code matches MadGraph 3.7.1 standalone to 2e-15."
status: draft
tags: [g2hdm, chirality, sigma, madgraph-parity, cms-gridpacks, oracle, report, validation]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Worktree `/home/user/wt/sprintl-g2hdm`, branch `sprintl-g2hdm`, head `bb111efb`
(main a5bf8fca with the brief merged in). It has no source changes. The only
commit is this report. The item `g2hdm-charged-higgs-high-mass-sigma-reads-high`
was not deleted by this session: its closing condition asks the high-mass cards
to match MadGraph by the multi-seed criterion, which is request V1 below. V1 ran
after the merge, and the close-out that deletes the item adds its result as the
last addendum ("V1: five seeds on 889 and 894 after the fix").

## The answer

| card | v0.2.2 vg/MG | σ_fix/σ_pre (fixed momenta, this session) | predicted post-fix vg/MG |
|---|---|---|---|
| 889 (M300) | 1.0039 (3.7.1 and 2.9.18) | **0.9944 ± 0.0039** | 0.998 ± 0.004 |
| 894 (M1000) | 1.0279 ± 0.0028 against 3.7.1 (1.0237 against 2.9.18) | **0.9697 ± 0.0040** | **0.997 ± 0.005** against 3.7.1 |

Within its error, the chirality bug accounts for the whole M1000 excess. The
bug is v0.2.2's reading of the bare-projector `FFS1`/`FFS3` charged-Higgs
vertices with g_L and g_R swapped, fixed on main by
`conjugate_standalone_projectors`. Most of the shift is not in the dominant
`c b~ > h+` channels, whose helicity-summed |M|² the swap cannot change. It
comes from subprocesses with a light-quark line, such as `s b~ > h+ d u~`, where
the charged Higgs comes off a quark line that a W also couples to. With the
right chirality those amplitudes vanish, or nearly vanish, and MadGraph agrees.
v0.2.2 gave them a spurious cross section. That share grows with the H± mass:
0.6% of σ at M300, 2.2% at M1000. This is why M300 and M450 passed and M ≥ 900
failed. MadEvent's default-survey bias is not needed to explain the excess. If
R3 shows MadGraph 3.7.1 at `hard_survey = 2` reading 1–2% above its default,
fixed vibegraph would read that much **below** MadGraph on 894. That would be a
new discrepancy, not part of this one (see "What remains").

## Inputs

**Cards (reconstructed; no card files were handed over).**
- `cards/proc_card.dat`: the six process lines from the dispatch notes, with
  `define p = p b b~` and `define j = p`.
- `run_card.dat`: written by MadGraph 3.7.1's own `banner.RunCardLO` with the
  dispatch values, the MLM block displayed:
  - ebeam 6800, lhaid 325300;
  - `dynamical_scale_choice -1`;
  - `ickkw 1`, `xqcut 30`, `auto_ptj_mjj T`, `ptj 20`, `drjj 0`;
  - `maxjetflavor 5`, `nhel 0`, `sde_strategy 1`;
  - `bwcutoff 15`, `cut_decays F`;
  - everything else at 3.7.1's defaults (`etaj 5`; unknown for the gridpack).
- Param cards: the UFO's default card with every `Yukawa*` entry zeroed except
  `YukawaGUR(3,2) = 0.4` and `(3,3) = 0.6`. Masses are 25 = 125 and 35/36/37 = M.
  Widths:
  - 894: DECAY 37/35/36 as in the dispatch notes (63.5971, 184.9265, 70.24848).
  - 889: H± 6.086866 (dispatch); H2 and H3 take MadGraph 3.7.1's two-body
    `compute_widths` values, 2.412979 and 1.673927. The gridpack's own values for
    889's H2 and H3 were not handed over. They enter only through the `h± j j`
    diagrams with an H2 or H3 propagator.

**The reconstruction checks out.** MadGraph 3.7.1 `compute_widths h+
--body_decay=2` on these cards reproduces the gridpacks' numbers:
- 889: Γ(H±) = 6.086866 with BR(tb) 0.4766801, BR(cb) 0.4704199, BR(h1 W)
  0.05290005. These are the dispatch notes' numbers to every printed digit.
- 894: partial width Γ(tb) = 20.2325 GeV against the notes' 0.31814 × 63.5971 =
  20.233, and Γ(h1 W) = 29.583 against 0.46516 × 63.5971 = 29.583.

So "others 0" means every extra Yukawa is zero, and the Higgs block keeps the
UFO defaults (`mixh` 0.3).

Other inputs:
- The UFO: a scratch copy of `/home/user/wt/dl-g2hdm/x/g2HDM_UFO`, not committed.
- The PDF: NNPDF31_nnlo_as_0118_mc_hessian_pdfas member 0, from
  lhapdfsets.web.cern.ch (archive SHA-256 `c090add3…a9531ad`).
- MadGraph 3.7.1: a `cp -al` copy with real `models/` and `input/`.
- Scratch area: `/home/user/wt/sprintl-g2hdm-mg/`, holding the harness, cards,
  tables, runs and scripts.

**The quark–H± vertices.** The relevant UFO vertices are:
- `V_62 = t~ b h+` and `V_71 = b~ t h-`, each `[FFS1 = ProjM(2,1), FFS3 = ProjP(2,1)]`;
- couplings `GC_43 ∝ I1a33 ∝ GD` and `GC_52 ∝ I2a33 ∝ GU`, and their conjugates
  `GC_61` and `GC_70`.

With every `GD` zero, each of these vertices has **one** chirality.

## Oracle 1: per diagram and per helicity against MadGraph 3.7.1 standalone

`g b > t h-` uses `b~ t h-` (V_71) and `g b~ > t~ h+` uses `t~ b h+` (V_62). Both
were run with `output standalone` and the 894 param card, all widths zero on
both sides (the top is spacelike in one diagram).
- 4 RAMBO points at √ŝ = 2.5 TeV, 8 helicities, AMP() banked.
- Banked with `gen_standalone_jamps.bank()` from a scratch driver
  (`harness/bank_g2hdm.py`).
- Compared with `tests/standalone_jamps.rs`'s `check` on a scratch copy
  (`lg1_jamps.rs`, not committed), with graph pairing `[0, 1]`.

```
cargo test -p vibegraph-lib --test lg1_jamps -- lg1_ --nocapture --test-threads=1
# fix build (bb111efb):
gb_to_thm: 32 flow entries, G = (1.6e-16 - 1.0000000000000007i), worst flow 2.321e-15, worst |M|² 2.183e-15
gb_to_thm: 64 graph entries, worst graph 1.948e-15
gbx_to_txhp: 32 flow entries, G = (-3.2e-16 + 1.0000000000000007i), worst flow 3.372e-16, worst |M|² 1.691e-16
gbx_to_txhp: 64 graph entries, worst graph 3.372e-16
test result: ok. 2 passed
# pre-fix build (c18042e), same file with the old table_model:
gb_to_thm: G = (2.1e-20 - 0.000602i), worst flow 1.000e0, worst |M|² 1.000e0
  point 3, helicities [-1, 1, -1, 0]: vibegraph (-2.303 + 1.910i) against G·MadGraph (-2.1e-7 + 1.8e-7i)
gbx_to_txhp: G = (-6.8e-22 + 0.000565i), worst flow 1.000e0
```

On the pre-fix build every helicity amplitude sits in the helicity MadGraph
leaves at zero, so the fitted constant collapses to 6e-4.

**The summed |M|² is blind to this bug on these processes.** At the same four
points the helicity-summed |M|² of `g b > t h-` is identical on both builds and
in MadGraph to every printed digit (3.3863818721, 0.71257009043, 0.25334227552,
42.506000482). A single-chirality scalar vertex between two external quarks
gives the same `Tr[(p̸_b+m_b) P_χ (p̸_t+m_t) P_χ̄] = 2 p_b·p_t` for either χ. A
|M|²-level check on the obvious process would have passed v0.2.2.

**Where the bug changes |M|².** On a light-quark line the H± vertex's chirality
meets a W's (`harness/bank_van.py`; test `lg1_sums_van`, the per-point helicity
sum):

```
fix build:
sbx_to_hpdux point 0..3: vibegraph 0, MadGraph 0                    (60 diagrams)
sbx_to_hpscx point 0: vibegraph 2.8064016306e-8, MadGraph 2.8064016306e-8   (68 diagrams)
sbx_to_hpscx point 3: vibegraph 1.4834856942e-5, MadGraph 1.4834856942e-5
pre-fix build:
sbx_to_hpdux point 2: vibegraph 1.7653819755e-5, MadGraph 0
sbx_to_hpscx point 0: vibegraph 6.3940328305e-5, MadGraph 2.8064016306e-8  (2300x)
```

Blind to:
- the global phase;
- kinematic regions and helicities the four points miss;
- every other g2HDM vertex not on these four processes. The light-quark
  `h+ d u~` / `h+ s c~` rows check a sum over 60–68 diagrams at |M|² level
  only, so they are blind to per-diagram sign errors that cancel there.

## Oracle 2: σ_fix/σ_pre at fixed momenta

1. A pre-fix sample on c18042e (detached worktree `/home/user/wt/lg1-pre`, own
   target directory).
   - `vibegraph integrate cards/proc_card.dat --run-card cards/run_card.dat
     --param-card cards/param_<c>.dat --ufo-dir ufo --seed 1 --fixed-budget
     --niter 1 -j 4`, giving σ = 46.5 ± 1.6 fb (894, 19m54s) and 10.41 ± 0.37 pb
     (889, 24m28s).
   - Then `vibegraph generate … --seed 2 --nevents 2000`.
   - Only the warm-up iteration ran. The default convergence run needs about 16
     minutes per iteration here (3.3M points across 2176 channels at the
     512-accepted-point floor), so it was stopped. Unweighting is exact given
     each event's XWGTUP, which the estimator uses, so a coarse grid costs
     efficiency, not bias. 894's sample has weights from 0.041 to 0.60.
2. **`generate` refuses card 894 on c18042e**: `the resonance 2 of g u > e+ ve b
   b~ d has no flavour in the subprocess with outgoing [-11, 12, 5, -5, -2]
   that shares its matrix element`.
   - Resonance 2 is the massless s-channel u line, and the group member has a
     `d~` in its place.
   - To get events, the pre-fix build was patched locally to fall back to the
     representative's code for that record line
     (`harness/pre_generate_label_fallback.patch`). The patch changes only the
     PDG label of an intermediate record line, never a momentum or a weight, and
     the harness ignores intermediate lines.
   - `lhef/resonance.rs` is unchanged between c18042e and main, so main should
     refuse the same way. This was not run on main (Found F1).
3. The harness `lg1_fixed_momenta.rs` (identical file in both trees, test-only,
   `#[ignore]`, not committed) does the following:
   - It enumerates the proc card with each build's own `generate_from_proc_card`.
   - For every LHE event it finds the diagram set by incoming flavours in order
     and sorted outgoing flavours (a beam-swapped key when the set is `mirrored`),
     and assigns legs by PDG.
   - It calls `AmplitudeEvaluator::compile` + `BoundAmplitude::eval_m2` at the
     param card's couplings.
   - It writes, per event, the momenta as f64 bit patterns and |M|².
   - It checks that every permutation of identical outgoing flavours gives the
     same |M|² (largest deviation 9.2e-13 on 889 and 4.5e-14 on 894), so
     assigning by PDG is sound.
4. `harness/ratio.py` asserts identical labels, beam order and momentum bits on
   both builds before forming any ratio.

```
CARDS=894 /home/user/wt/logs/lg1-ratio.sh      # and SKIPGEN=1 CARDS=889
2000 events: labels, beam order and momentum bit patterns identical on both builds
```

**Per event r = |M|²_fix/|M|²_pre**, at the momenta exactly. The spread is very
large because the swap flips the top's helicity, and V−A top decay is a 100%
spin analyser:

| card | weighted mean r | median | 5–95% | std | r > 1 | r < 1 |
|---|---|---|---|---|---|---|
| 889 | 0.995 ± 0.070 | 0.394 | 0.022–3.30 | 3.31 | 23.7% | 76.2% |
| 894 | 0.959 ± 0.044 | 0.370 | 0.013–4.02 | 1.94 | 23.7% | 76.2% |

At 2000 events this direct estimator cannot resolve a 3% shift.

**The rotation-averaged estimator.** Each build's |M|² is also averaged over
216 rotations of the top-decay system (the charged lepton, its neutrino and the
b nearest m_t) in its rest frame, on a Haar midpoint grid of Euler angles that
is the same in both trees.
- What the rotation preserves:
  - every quantity the rest of the event weight reads: PDFs, the core partons
    the scale and MLM clustering see, m(t), m(W), the H± momentum;
  - every cut, at `cut_decays = F`;
  - the phase-space measure.
- So the per-event ratio of orbit averages ⟨f_fix⟩/⟨f_pre⟩ is a function of the
  orbit only, and its mean over events drawn from f_pre is exactly σ_fix/σ_pre.
- Control: E_pre[⟨f_pre⟩/f_pre] must be 1.

```
889: control E_pre[<f_pre>/f_pre] = 1.02260 +- 0.03674
     E_pre[<f_fix>/<f_pre>]       = 0.99436 +- 0.00386   (quantiles 0/16/50/84/100%: 0 0.98254 0.99954 1.01406 7.34)
894: control E_pre[<f_pre>/f_pre] = 1.01635 +- 0.02665
     E_pre[<f_fix>/<f_pre>]       = 0.96974 +- 0.00402   (quantiles: 0 0.97880 0.99947 1.01505 2.56)
```

Attribution by event class (`harness/outliers.py`), as a contribution to
σ_fix/σ_pre − 1:

| class | 889 events | 889 | 894 events | 894 |
|---|---|---|---|---|
| ⟨f_fix⟩/⟨f_pre⟩ < 0.01 (fix vanishes) | 12 | −0.0060 | 44 | **−0.0220** |
| 0.01–0.95 | 118 | −0.0078 | 125 | −0.0123 |
| > 1.05 | 89 | +0.0083 | 82 | +0.0052 |
| core 0.95–1.05 | 1781 | −0.0005 | 1749 | −0.0001 |

Where the fixed amplitude vanishes, the subprocesses are `s b~ > l+ ν b b~ d
u~`, `s b~ > … s c~`, `s~ b > … c s~`, `d b~ > … d u~` and their conjugates.
These are the `h± j j` line with H± radiated off a light-quark line that a W
also couples to. `sbx_to_hpdux` above is the bare form of the first: zero in
MadGraph and in the fixed build. The two tail classes come mostly from the
dominant `c b~ > h+` channels (one b candidate). They are the off-shell top,
where `(p̸+m)γ⁵(p̸+m) ∝ (p² − m²)` lets the top spin survive the decay average
with the sign of `p² − m²`. They nearly cancel, and they set the estimator's
error.

Blind to:
- anything in the event weight outside |M|², if the fix moves it. The MLM
  clustering and colour configuration are drawn from AMP2, so a scale or
  configuration change caused by the fix is not in r. The direct check is V1.
- The rotation average assumes the top-decay b is identifiable. In 512 of 2000
  894 events (365 on 889), two b's have the lepton's sign. There the choice
  "nearest m_t" is not exactly invariant under the rotation, and the pairing
  cut treats that b as a core jet in the other pairing.
- The ratio is c18042e against a5bf8fca. Between v0.2.2 and c18042e the source
  diff is UFO parsing (Goldstone flag, list/dict propagators, parameter
  refusals) and run-card placeholders. g2HDM flags its Goldstones `goldstone =
  True`, which v0.2.2 already read, so c18042e is taken to compute v0.2.2's |M|²
  for this card. This was not measured, because no v0.2.2 artifact was at hand.
- Between c18042e and a5bf8fca, main also landed squared orders (#168) and Auto
  widths (#179). Neither acts on this card (no squared orders, explicit widths),
  so the ratio is attributed to #169's chirality fix. That attribution is an
  inference, not a third build.

## What remains of the excess, and its attribution so far

- **Chirality bug:** −3.0 ± 0.4% on 894 and −0.6 ± 0.4% on 889. Predicted
  post-fix vg/MG(3.7.1) is 0.997 ± 0.005 on 894 and 0.998 ± 0.004 on 889.
  - The prediction is v0.2.2's measured σ (vg-validator, 5 seeds: 0.047645 ±
    0.000119 pb) × σ_fix/σ_pre, against MadGraph 3.7.1 at 0.04635 ± 0.000061 pb.
  - Errors are combined in quadrature.
- **Survey share:** pending R3. The two causes are independent, so R3's MadGraph
  `hard_survey = 2` σ on 894, divided by its default σ, is the survey share on
  its own.
  - If R3 lifts MadGraph by the 1.3–1.9% seen on the VBS cards, fixed vibegraph
    reads about that much below MadGraph on 894.
  - That is not the excess this item records. It would make the survey bias
    process dependent, or point to something else in fixed vibegraph.
  - V1 measures it directly.
- **Breit–Wigner window, threshold, `small_width_ratio`, scale and merge:** not
  needed to explain anything. With the chirality share removed, 894's residual
  is −0.3 ± 0.5%. The H± window (M ± 954 GeV, lower edge below the t b
  threshold) and the top and W windows were not compared against MadGraph's
  events, because nothing is left for them to explain at this precision. V3 is
  the per-event check if R3 or V1 leaves a residual.

## Release note: per-card verdicts, v0.2.2 on 889–895

The spurious share grows with M_H±: 0.6% of σ at M300 and 2.2% (vanishing
class) or 3.0% (total) at M1000. It comes from `h± j j` diagrams whose
relative weight rises with mass. The excess the item records follows the same
pattern: +0.4% at M300, +0.6% at M450, +2.4–2.5% at M900–M1000, and +3.5–4.4%
at M1250–M1750.

| card | mass | v0.2.2 vg/MG | can its verdict move? |
|---|---|---|---|
| 889 | M300 | 1.0039, pass | Yes, slightly: by −0.6 ± 0.4%. It stays a pass, at 0.998. |
| 893 | M450 | 1.0064, pass | Yes, slightly. 889 and 894 bracket it (−0.6% to −3.0%), so it is expected to stay a pass. Its own number is not measured. |
| 894 | M1000 | +2.4–2.8%, fail | Yes. Expected to pass after the fix (0.997 ± 0.005 against 3.7.1). Confirmation is V1. |
| 890–892, 895 | M900, M1250, M1500, M1750 | +2.5% to +4.4%, fail | Yes, all four. The fix lowers each, and the v0.2.2 failures are attributable to the bug. 889 and 894 bound only M900 (with 894). M1250–M1750 lie above the measured range, where the share grows, so **each needs its own number** (V2). |

The dispatch gives masses for 889, 893 and 894 only. Which of 890–892 and 895 is
M900, M1250, M1500 or M1750 is in vg-validator's campaign table.

For the release note: v0.2.2's σ on every g2HDM charged-Higgs card (889–895)
carries the charged-Higgs chirality bug. Its effect is −0.6% at M300 and about
−3% at M1000, and it grows with mass. Each card's v0.2.2 verdict is superseded
by a post-fix run.

## What each oracle cannot see

- **Per helicity against standalone:**
  - a global phase;
  - regions the four points miss;
  - vertices off the banked processes;
  - in the summed-|M|² rows, compensating per-diagram errors.
- **Summed |M|² on `g b > t h-`:** this bug, entirely: single-chirality vertex,
  external quarks.
- **σ_fix/σ_pre from |M|²:**
  - the scale, MLM clustering and colour-configuration draw, if the fix moves
    them;
  - c18042e standing in for v0.2.2;
  - attributing the c18042e..a5bf8fca difference to #169 alone.
- **The rotation average:**
  - an exact invariant only where the top-decay b is unambiguous (74% of 894's
    events);
  - with a 6³ grid it averages a linear spin dependence exactly, and higher
    harmonics only approximately; the per-event spread around 1 is ±1.5%.
- **Not checked here:**
  - any per-event field (scale, merge, window) against MadGraph's records;
  - card 967 (ALP → ττ). It has no charged-scalar FFS vertex, so this bug cannot
    be its cause, and its +1.6% stays open in its own right.

## Requests for vg-validator

Branch `sprintl-g2hdm`, head `bb111efb` (code identical to main a5bf8fca).

- **V1. Post-fix σ on 894 and 889.** 5 seeds each, the campaign's command and
  budget, on main a5bf8fca or later.
  - Expected against MadGraph 3.7.1 (default survey): 894 at 0.997 ± 0.005
    (σ ≈ 46.2 fb) and 889 at 0.998 ± 0.004.
  - Read 894 also against R3's `hard_survey = 2` σ. A post-fix vibegraph
    1–2% below hard-survey MadGraph is a new finding, not this item.
- **V2. Post-fix σ on 890–893 and 895.** 5 seeds each, against the campaign's
  MadGraph σ.
  - Expected: each v0.2.2 excess reduced by a mass-growing share, about 0.6%
    at M450 and over 3% from M1250.
  - The cheaper alternative is the fixed-momenta harness on each card (below,
    about 45 minutes per card on 4 cores).
- **V3. Per-event check on R3's MadGraph events.** For 894 (e.g.
  `scale-oracle-records/894_hs2_s1.unweighted_events.lhe.gz`), run
  `lg1_fixed_momenta.rs` (scratch copy `/home/user/wt/sprintl-g2hdm-mg/harness/`)
  from a checkout of a5bf8fca.
  - Command: `LG1_UFO=<g2HDM_UFO> LG1_PROC=<proc_card> LG1_PARAM=<param_894>
    LG1_LHE=<events, gunzipped> LG1_OUT=m2.tsv cargo test -p vibegraph-lib
    --test lg1_fixed_momenta -- --ignored --nocapture`.
  - Expected: every event finds a diagram set (MadGraph's subprocesses are
    ours), and no event has |M|² = 0 where MadGraph drew it. An event in a
    subprocess whose fixed |M|² vanishes would mean the two programs disagree
    there.
  - Only needed if V1 leaves a residual. Its `SCALUP`, the H± lineshape below
    the t b threshold and the top and W windows can be histogrammed from the
    same file against `vibegraph generate` output.

## Found

- **F1. `vibegraph generate` refuses card 894** (c18042e; the code is unchanged
  on main):
  - The message: "the resonance 2 of g u > e+ ve b b~ d has no flavour in the
    subprocess with outgoing [-11, 12, 5, -5, -2] that shares its matrix
    element".
  - `member_line_pdg` maps a group member's resonance line only to the
    representative's flavour or its conjugate. Here the line is a massless
    s-channel quark (u in `g u`, d~ in the member `g d~`), a different flavour
    in a member whose |M|² the grouping found equal.
  - So `integrate` passes and `generate` refuses before drawing an event. A
    named refusal, not a panic, but it blocks events for the g2HDM cards and
    any card with such a group.
  - To close: reproduce on main, then derive a member's line flavour from the
    member's own legs below the line (charge and flavour flow), or skip a record
    line for a non-resonant massless propagator, with a test on this group.
- **F2. Every v0.2.2 σ on a card with a bare-projector, non-self-conjugate
  vertex is suspect beyond g2HDM.**
  - This session confirms the census's concern with a size: up to −3% of σ, and
    growing with mass, from subprocesses whose amplitude the right chirality
    switches off.
  - On the dominant 2 → 2 channels the summed |M|² is identical before and
    after the fix, so a σ comparison on a simple channel, or a |M|²-only
    oracle, cannot clear a card.
  - To close: the census's card list re-run post-fix (σ or the fixed-momenta
    ratio).
- **F3. Proposed feature: evaluate |M|² at given momenta.** A CLI subcommand (or
  a `reweight`-like mode of `generate`): read an LHE file and a proc, param and
  run card set, and write per-event |M|², or the |M|² ratio against an artifact's
  build.
  - This session's harness did that in about 300 lines of test code, plus the
    decay-rotation average that turns it into a precise σ ratio.
  - It would have let census 2b size this bug on every card. It is also the
    natural per-event oracle against MadGraph's LHE (V3).
- **F4. The local per-card σ cost is dominated by the channel floor.** The g2HDM
  card spends 1.17M points per warm-up iteration and 3.3M afterwards, 2145 of
  its 2176 channels sitting at the 512-point floor, at about 0.3 ms per
  evaluation. One iteration takes 6–16 minutes on 4 cores. This sets the 2.5–8 h
  per seed. Worth a look by whoever owns budget allocation.
- **F5. Brief corrections.**
  - The brief's `CARGO_TARGET_DIR=/home/user/wt/sprintl-g2hdm/target`
    disagrees with the dispatch's `/home/user/wt/target-g2hdm` (the dispatch
    was followed).
  - The brief calls the CLI package `vibegraph-cli`; it is `-p vibegraph`.
  - No card files existed in `cards-sprintL`. The cards were reconstructed and
    verified through the widths above.

## Files and commands

Scratch, all under `/home/user/wt/sprintl-g2hdm-mg/`:
- `harness/`:
  - `lg1_fixed_momenta.rs` (the harness);
  - `lg1_jamps_fix.rs` and `lg1_jamps_pre.rs` (the per-helicity probes);
  - `bank_g2hdm.py` and `bank_van.py`;
  - `ratio.py`, `outliers.py`, `runcard.py`;
  - `pre_generate_label_fallback.patch`.
- `cards/` and `tables/`.
- `runs/pre889`, `runs/pre894`: events, `m2_pre.tsv`, `m2_fix.tsv`.
- Logs: `/home/user/wt/logs/lg1-*`.

The extra worktree `/home/user/wt/lg1-pre` and its target directory are
removed at the end of the session.

## Addendum: F1 on main, and R3 seed 1

**F1 reproduces on main b4e284a3** (`vibegraph --version`: `v0.2.2-17-gb4e284a3`;
detached worktree, own target directory, reconstructed cards).

Card 894, the full card:

```
vibegraph integrate cards/proc_card.dat --run-card cards/run_card.dat --param-card cards/param_894.dat \
    --ufo-dir ufo --seed 1 --fixed-budget --niter 1 --neval 20000 -j 4      # 28m34s
σ = 51.8 ± 4.2 fb
vibegraph generate runs/main_proc_card/grid.bin.zst --seed 2 --nevents 300 -j 4    # exit 1
ERROR the resonance 2 of g u > e+ ve b b~ d has no flavour in the subprocess with outgoing [-11, 12, 5, -5, -2] that shares its matrix element
```

It is a named refusal with exit code 1, not a panic, and it comes before any
event is drawn.

The minimal reproduction takes seconds: one vanishing line beside one live line.

```
import model g2HDM_UFO
define q = u d~
define qq = d u~
generate g q > h+ qq QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
add process c b~ > h+ QCD=99, (h+ > t b~, (t > w+ b, w+ > e+ ve))
```

- `integrate` (same flags, 894 param card): σ = 2.37 ± 0.20 fb, and it logs
  "1 of 1 flavour groups (2 subprocesses) have a matrix element that vanishes at
  the card; they carry no cross section and are not clustered".
- `generate` then refuses with the same message.
- The first line on its own is refused by `integrate`, with "every one of the
  20000 points … has zero weight", because with only ρ_tc and ρ_tt non-zero and
  V_td = V_ts = 0 the H± u d̄ coupling vanishes.

**Diagnosis.** The two subprocesses `g u > h+ d` and `g d~ > h+ u~` form one
flavour group: their matrix elements are equal because both are zero at the
card. The group's diagrams carry a massless s-channel quark line, `u` in the
representative and `d~` in the member. Neither is the other's flavour or
conjugate, so `member_line_pdg` (`vibegraph-lib/src/lhef/resonance.rs:356`)
returns `None`. `group_resonances` (`vibegraph-lib/src/generate.rs:1514`; the
call is at `:1539` and the message at `:1542`) then refuses the whole run. It
does so for a group that integration already set aside as carrying no cross
section.

**Open question for whoever takes it.** This session did not test whether a
non-vanishing group with such a line is also refused. That needs a model where
the two members' amplitudes are equal and non-zero.

**To close.** Skip resonance resolution for groups whose matrix element vanishes
at the card, since they never produce an event. For live groups, derive a
member's line flavour from the member's own legs below the line. Add a test on
the five-line card above. No fix was made in this session.

**R3 seed 1 (vg-validator; one seed).** MadGraph 3.7.1 at `hard_survey = 2` on
894 gives σ = 0.046501 ± 0.000129 pb.
- Against the default-survey reference 0.04654 ± 0.00013 that is −0.09%: no
  survey lift on this card.
- The predicted post-fix vibegraph, 0.9697 × 0.047645 = 0.046201 pb, reads
  0.9935 against it, with about 0.3–0.6% one-seed MadGraph error and this
  session's ±0.4%.
- Seeds 2–5 and V1 are pending.

## Addendum: V1, five seeds on 889 and 894 after the fix

vg-validator ran both cards on main 7b36722c, which contains the chirality fix.
The argv is the 3-iteration fixed budget (`--neval 120000 --niter 3 -j 4
--parallel-diagrams`), seeds 1–5, with a 7200 s cap. No seed timed out: walls
were 5833–5884 s on 894 and 5460–5558 s on 889. Means are unweighted over the
five seeds. The error on a mean is the seed spread over √5.

**894 (M1000)**

| seed | σ (pb) | reported error |
|---|---|---|
| 1 | 0.0461046 | 1.53% |
| 2 | 0.0459666 | 1.64% |
| 3 | 0.0459727 | 1.73% |
| 4 | **0.0480612** | **3.23%** |
| 5 | 0.0460138 | 2.02% |

- Mean 0.0464238 ± 0.88%. Seed sd 1.98%, mean reported error 2.03%, χ²/dof 0.55.
- Against MadGraph (reference 0.046464 pb): vg/MG = **0.9991 ± 0.0088**.
- Against the EW-recombined MadGraph σ (0.046518 pb): 0.9980 ± 0.0088.
- The prediction above (0.9935 against R3 seed 1) and the predicted post-fix
  vg/MG(3.7.1) of 0.997 ± 0.005 both agree.
- Against v0.2.2's CERN mean of 0.04764 (a different argv): 0.9745 ± 0.009.
  That is the drop the chirality fix predicts.

**The five seeds do not calibrate 894 at the 1% level.**
- Seeds 1, 2, 3 and 5 sit within 0.14% of each other, while each reports
  1.5–2.0%. Among them χ²/dof is far below 1.
- Seed 4 sits +4.4% above them and reports twice their error.
- Without seed 4, vg/MG is 0.9903. With it, 0.9991.

The excess is in the two 6-body multiplicities, read from each run's
per-multiplicity results:

| multiplicity | other four seeds, mean (sd) | seed 4 | share of seed 4's excess |
|---|---|---|---|
| ℓν bb̄ + j + b (412 maps, 3520 channels) | 0.005961 pb (6.5%) | 0.007302 ± 19% | 65% |
| ℓν bb̄ + 2 light (167 maps, 2562 channels) | 0.004108 pb (7.5%) | 0.004601 | 24% |

- The 4- and 5-body rows agree within about 1%.
- The 6-body rows carry about 22% of σ.
- From each run's support line, the effective sample size out of about 1.31M
  weighted points is:
  - seed 1: 3830
  - seed 2: 3376
  - seed 3: 3088
  - seed 4: 939
  - seed 5: 2298

  That is n_eff/N of 0.07–0.29% on every seed.
- The weight distribution is extremely heavy on all of them. On seed 4, a
  handful of events dominate.
- Per-channel w_max and variance are not recoverable: the run script deleted
  each run's output directory.

This tail is a sampler defect separate from the excess this report attributes.
It is filed as `g2hdm-894-six-body-rows-heavy-tailed`, whose first step is to
rerun 894 seed 4 with its output kept.

**889 (M300)**

| seed | σ (pb) | reported error |
|---|---|---|
| 1 | 10.1316 | 2.02% |
| 2 | 9.8859 | 0.97% |
| 3 | 9.9533 | 1.09% |
| 4 | 10.0909 | 1.13% |
| 5 | 10.1146 | 1.05% |

- Mean 10.0352. Seed sd 1.09%, error on the mean 0.49%, χ²/dof 1.00.
- The 1/σ²-weighted mean is 10.0104 ± 0.0513.
- The MadGraph reference is the gpdb survey value, 10.04 ± 0.0385 pb, as
  reported. There is no fresh `hard_survey = 2` or EW-recombined MadGraph run
  for 889, as there is for 894.
- vg/MG = **0.9995 ± 0.0062** (pull −0.08). On the 1/σ² mean: 0.9971 ± 0.0064.
- The seed spread matches the reported errors, and no seed is an outlier.
- Against v0.2.2's gpdb proxy runs (mean 10.0791, not the same argv): 0.9957.

**Verdict.** The item asked whether vibegraph reads high on the high-mass cards,
and why. After the fix, neither card reads high, under any reading of its mean:
- 894: 0.9991 with seed 4, 0.9903 without it.
- 889: 0.9995.

v0.2.2's excess is the chirality bug, attributed per diagram and per helicity
above and fixed on main. The item is deleted with this addendum.

What this does not claim: agreement with MadGraph at the 1% level on 894. Five
seeds cannot calibrate its 6-body tail, and the tail item carries that.

V2 (the remaining high-mass cards at a 7200 s cap) is post-merge release-note
validation, not part of this closure.

