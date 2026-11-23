---
type: Session Report
title: "L-P2 report: the pairing-weighted side bound, and PR 5 owns no precision loss on VBS card 134"
description: "Under pairing-weighted cuts a channel now reads the side transfer bound off its own pairing's filter, which fixes CI's cli_reweight_proton refusal and is bit-for-bit main on that card. Like-for-like seed pairs on card 134 show no PR 5 precision loss, and no heavy point lies in the region it opened. Card 126 stays fixed on the final head, and same-scheme MadGraph agrees with it, so the multichannel item closes."
status: draft
tags: [sampling, multichannel, spine, t-channel, cuts, pairing, vbs, cms-gridpacks, report]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-23}
---
Continuation of [L-P1](L-P1-report.md) on branch `sprintl-multichannel`
(PR 5, #184). Code head **2091a26f** (fix2). This session's closing commit
adds only this report and the kb edits. Closes
`multichannel-subdominant-channel-carries-dominant-region`; its file is
deleted.

Every σ below names the code it ran on:
- `main a5bf8fca` is the branch base. On card 134 its code equals main
  7b36722c (one seed checked bit for bit below).
- `f70ab694` is L-P1's fix.
- `2091a26f` is fix2, the code head.

The binaries `vibegraph-probe` and `vibegraph-fix1` are a5bf8fca and
f70ab694 with L-P1's env-gated per-point probe (`probe.patch`). The hook
prints and changes nothing: L-P1 showed seed 2 of card 126 bit for bit
between the probe build and a clean one.

## fix2: a channel reads its own pairing's filter

**The failure.** CI's banked job failed on ab4fac69 in
`cli_reweight_proton::reweighting_a_pairing_weighted_card_is_refused`.
Its setup integrate of `generate g g > z u u~, z > u u~ @1` (sm,
`cut_decays = F`, neval 4000) refused with "the cross section is carried
by an effective 9.4 points, below the 20".

**Cause.** The cuts there are pairing-weighted, and the integrand is
`Σ_p w_p Θ_p |M|²` (`pairing.rs`, module doc):
- `Θ_p` is the filter that exempts pairing `p`'s decay products;
- `w_p = |M_p|²/Σ_q|M_q|²` is formed over `p`'s own diagrams, and the
  diagrams of a subprocess partition exactly into its pairings.

At f70ab694, `Cuts::single_leg_transfer_bound` took the weakest bound over
all pairing filters. Each of the four quarks is a decay product in some
pairing, so every single-leg side read 0 and every rung lost its `ptj²`
bound. Each pairing's channels then spread their draws over configurations
that only another pairing's term carries.

**The rule** (`cuts.rs`, `single_leg_transfer_bound(slots, products)`):
- The caller is `MapChoices::channel` (`phasespace/maps.rs`). It passes as
  `products` the OR of the diagram's `forced_lines` slots, the same set that
  `Pairing::products` is built from.
- The bound is read off the filter whose `exempt` equals `products`.
- Where no filter matches, the bound is the weakest, which holds over the
  union of their accepted regions.
- `UnionBounds` takes the minimum over its member filters, each asked with
  the same `products`.

**Is the own-pairing bound proven?** Yes, for what the estimator needs: the
channels' supports together cover every point where the integrand is
non-zero.
1. Where pairing `q`'s term is non-zero, `Θ_q` passes. So every leg outside
   `q`'s products is above its own `pT` threshold.
2. `q`'s channels come from `q`'s diagrams. A single-leg side of their
   spine rungs is never one of `q`'s decay products. A forced decay is
   timelike, so its products sit together inside the forced line's
   subtree, and a forced line has at least two of them.
3. So every single-leg side of a `q` channel is a leg that `Θ_q` cuts.
   `with_side_transfer_bounds`' derivation (`t ≤ −pT²` for a side taking
   a light-cone fraction `x ≤ 1` of its beam) gives `|t| ≥ pT_min²` over
   the whole support of `q`'s term.
4. `p`'s channels need not cover `q`'s term, because `q`'s channels do.

Composite sides keep their heuristic scale, as on any other card. That is
the open caveat L-P1's Found 2 already records.

**Should a decay product's single side relax a production spine?** No:
- In its own pairing's channels, a decay product is never a single-leg side
  (step 2).
- A leg that is a decay product only in another pairing is cut in this
  pairing's term. Relaxing on it moves this channel's draws into
  configurations its own term gives zero.

**Evidence.** The pairing card was run with `--fixed-budget --niter 2` at
neval 4000/16000/64000, on seeds 1–4 and 20261005
(`/home/user/wt/lp1-bin/pair_sweep.sh`).

| code | ESS range | pass-cuts | refusals |
|---|---|---|---|
| main a5bf8fca | 52–5118 | 0.46–0.48 | 0 |
| f70ab694 | 9–786 | 0.37–0.42 | 1 (neval 4000, seed 20261005) |
| fix2 | equal to main in `sigma` and `sigma_err`, full f64, all 15 runs | | 0 |

- `cli_reweight_proton` on fix2: 3 passed, 0 failed (`lp1-reweight.log`).
- CI on 2091a26f is green, banked job included. The manager checked the
  check runs.
- Card 126 has no forced decays, so fix2 cannot move it.
- Unit test: `subprocesses_whose_decays_carry_different_legs_weight_one_shared_filter`
  asserts the own-filter bound (0 for the channel's own products, `ptj²`
  for its core legs) and the weakest-bound fallback (0).

## Card 126 on the code head

Command and cards as L-P1 (`/home/user/wt/lp1-bin/run_seed.sh`:
`--target-rel 0.01 --neval 120000 -j 4 --parallel-diagrams`). The binary is
a clean release build of 2091a26f.

The build prints `v0.2.2-19-g2091a26f-dirty`. The tree had no content
change: `git diff --quiet HEAD -- vibegraph-lib vibegraph-cli` was clean
before the build, and `git describe --dirty` reads clean once the index is
refreshed. The suffix is a stale index stat after a revert.

```
$ python3 /home/user/wt/lp1-bin/scatter.py /home/user/wt/logs/lp2-head-126     (2091a26f, seeds 1–8)
seed   1 sigma 2.51487e-04 err 5.30e-07 chi2 1.57 ch4share 3.6% ch4chi2 0.65
seed   2 sigma 2.53562e-04 err 6.16e-07 chi2 2.79 ch4share 1.4% ch4chi2 2.34
seed   3 sigma 2.53377e-04 err 5.38e-07 chi2 2.37 ch4share 2.0% ch4chi2 2.06
seed   4 sigma 2.52646e-04 err 6.28e-07 chi2 4.27 ch4share 1.9% ch4chi2 1.21
seed   5 sigma 2.51786e-04 err 5.26e-07 chi2 4.73 ch4share 1.9% ch4chi2 1.48
seed   6 sigma 2.51243e-04 err 5.51e-07 chi2 1.95 ch4share 1.9% ch4chi2 1.81
seed   7 sigma 2.52081e-04 err 5.48e-07 chi2 1.43 ch4share 1.5% ch4chi2 1.51
seed   8 sigma 2.52166e-04 err 5.32e-07 chi2 6.76 ch4share 2.7% ch4chi2 0.08
n 8 mean 2.52294e-04 ± 2.98e-07 (sd 8.43e-07, mean quoted err 5.59e-07) scatter chi2/dof 2.24
wall 15.6–16.7 s per seed
```

All eight seeds sit in L-P1's high population (2.512–2.536e-4; the base
code's low population was 2.384–2.41e-4). Each σ equals the f70ab694 seed
of the same number exactly, in full f64, as fix2 predicts for a card with
no forced decays. So L-P1's 20-seed and 480k sweeps carry over to the head
unchanged.

These 8 seeds alone read vg/MG 1.0032 against the same-scheme reference
below, 2.1σ on the combined error. They are the first 8 of L-P1's 20, which
read 1.0026; the full 20-seed mean is the one to quote.

## Card 134: PR 5 owns no precision loss

**The step-1 table compared different commands.** It set PR 5 (`--niter 3
--fixed-budget`, α allocation) against v0.2.2 gpdb rows run at campaign
settings (target_rel 0.01, Neyman allocation, 6 iterations). It is not
evidence either way.

**The 3-iteration pair** was run by vg-validator on its own host:
`--fixed-budget --neval 120000 --niter 3 --parallel-diagrams -j 4`, seeds
1–5, default allocation.

| code | mean (pb) | ± | seed sd | χ²/dof | mean reported err |
|---|---|---|---|---|---|
| main 7b36722c | 0.0551970 | 0.59% | 1.31% | 1.87 | 0.95% |
| #184 2091a26f | 0.0551242 | 0.60% | 1.33% | 1.89 | 0.93% |

- #184/main per seed: 1.0049, 1.0003, 0.9926, 0.9994, 0.9963.
- 2091a26f's σ values equal ab4fac69's bit for bit on all five seeds.

**The campaign-setting pair** ran in this container.

Cards: the sibling VBS session's staging, `/home/user/wt/sprintl-vbs-mg/vg/`,
copied to `/home/user/wt/lp2-card134/`:
- `proc_134.dat`, as `vbs.md` records it;
- `run_card_pol.dat`, matching every setting `vbs.md` lists: ebeam 6800,
  lhaid 325500, dynamical scale −1, bwcutoff 15, ptj 10, etaj 6.5, drjj 0.4,
  mmjj 10, maxjetflavor 4, ptb 0, etab 6.5, drbj 0.4;
- `param_card.dat`, the SM default, identical to MadGraph 3.5.7's output for
  134.

The gridpack's own run card was not available here, so the run card is a
faithful reconstruction rather than the original.

Command (`/home/user/wt/lp2-bin/run134.sh`):

```
vibegraph integrate proc_134.dat --run-card run_card_pol.dat --param-card param_card.dat \
  --fixed-budget --niter 6 --allocate neyman --neval 120000 -j 4 --parallel-diagrams \
  --seed N --output-ldjson --no-network      (LHAPDF_DATA_PATH → the nf_4 set lhaid 325500 resolves to)
```

- **This is the campaign run.** The campaign command (`--target-rel 0.01`
  in place of the three fixed-budget flags) stops at `target_met` after
  exactly 6 iterations. On seed 1, a5bf8fca gives the same σ both ways:
  0.05567511783639536 ± 3.6959e-4.
- **Evaluations: 1 548 076–1 576 980 per run on main, 1 536 706–1 577 067 on
  PR 5.** The v0.2.2 gpdb rows spent 1.55–1.58M.

| code | mean (pb) | seed sd | χ²/dof | mean reported err | per-seed err |
|---|---|---|---|---|---|
| main a5bf8fca | 0.0553847 | 0.30% | 0.23 | 0.55% | 0.66 0.51 0.57 0.50 0.48% |
| PR 5 f70ab694 | 0.0554140 | 0.57% | 0.84 | 0.61% | 0.59 0.52 0.48 0.50 0.95% |

PR 5/main per seed: 0.9981, 1.0004, 0.9957, 0.9994, 1.0091.

Which code each side is, checked on seed 1 with the same command:

```
a5bf8fca+probe  0.05567511783639536  ± 0.0003695930791613474
main 7b36722c   0.05567511783639536  ± 0.0003695930791613474   (vibegraph v0.2.2-18-g7b36722c)
f70ab694+probe  0.055570658001722936 ± 0.00032783461379962576
fix2 build      0.055570658001722936 ± 0.00032783461379962576
```

Reading:
- **The wider spread is not significant.** The seed variance ratio is
  (0.57/0.30)² = 3.6, against F(4,4)'s 5% one-sided point of 6.39.
- **Main is the tight side, not PR 5 the loose one.** Main's scatter χ²/dof
  of 0.23 on 4 dof has p ≈ 0.08 from below. PR 5's 0.84 is ordinary.
- **The mean error moves only through seed 5** (0.95%). Without it, PR 5's
  errors are 0.48–0.59%, against main's 0.50–0.66%.
- **Seed 5's extra error is one point** of 3.1% of σ (below).

**Per channel** (5 seeds each; `/home/user/wt/lp2-bin/chan.py`,
`wmax.py`):
- σ_j/σ, err_j/σ and α_j are seed means.
- χ²_j is the mean per-channel iteration χ²/dof (max over seeds in
  brackets).
- "max w" is the largest single-point contribution drawn in that channel,
  value·w_VEGAS/N_j over one iteration, as a share of σ, over all five
  seeds.
- Rows are ranked by the larger of the two err_j.
- In "configurations", s0 is `u d > W- Z u u`, s10 is `d d > W- Z u d`,
  s34 is `u~ u~ > W- Z u~ d~`, and s17 is `d u~ > W- Z b b~`.

| channel (group/ch) | configurations | σ_j/σ main / PR 5 | err_j %σ | α_j | χ²_j | max w %σ |
|---|---|---|---|---|---|---|
| g0/24 | s0.c24 … | 0.0396 / 0.0383 | 0.078 / **0.372** | 0.057 / 0.051 | 1.27 / 6.45 (29.9) | 0.59 / **3.14** |
| g6/26 | s10.c26 … | 0.0182 / 0.0173 | **0.213** / 0.067 | 0.016 / 0.016 | **8.13** / 1.21 | **1.81** / 0.19 |
| g0/13 | s0.c13 … | 0.0270 / 0.0276 | 0.095 / 0.180 | 0.030 / 0.030 | 1.00 / 2.03 (5.10) | 0.44 / 1.36 |
| g0/12 | s0.c12 … | 0.0924 / 0.0908 | 0.131 / 0.128 | 0.095 / 0.094 | 1.11 / 1.05 | 0.55 / 0.55 |
| g18/5 | s34.c5 … | 0.0071 / 0.0061 | 0.122 / 0.061 | 0.009 / 0.008 | 2.54 / 3.52 | 0.97 / 0.23 |
| g0/11 | s0.c11 … | 0.0503 / 0.0517 | 0.119 / 0.112 | 0.062 / 0.063 | 1.42 / 1.13 | 0.53 / 0.34 |
| g18/31 | s34.c31 … | 0.0066 / 0.0068 | 0.101 / 0.104 | 0.008 / 0.009 | 3.04 / 4.14 (14.4) | 0.80 / 0.80 |
| g0/27 | s0.c27 … | 0.0129 / 0.0151 | 0.077 / 0.071 | 0.015 / 0.019 | 1.16 / 0.65 | 0.30 / 0.83 |
| g6/1 | s10.c1 … | 0.0012 / 0.0009 | 0.072 / 0.009 | 0.001 / 0.001 | **65.6** / 1.35 | 0.63 / 0.04 |
| g24/27 | s17.c27 | 0.0019 / 0.0019 | 0.096 / 0.096 | 0.037 / 0.037 | 36.5 / 36.5 (178) | 0.85 / 0.85 |
| Σ err_j² (quadrature) | | | 0.550% / 0.631% | | | |

Every other channel's err_j is under 0.09% of σ on both sides, with α_j
and σ_j/σ equal to within a few per cent. The two binaries' heavy-point
populations are the same size: 4520 against 4547 points above 5e-14 GeV⁻²,
summing to 315% against 319% of σ over the 30 iterations.

**The question the planner asked:** did the widened support raise the
variance of channels that now reach the uncut b̄-collinear corner without
adding σ there?
- **No b̄-collinear corner opens on 134.** The beams are light quarks and
  the process is QCD=0. A spine rung whose emitted side is a single `b` or
  `b̄` would need a beam quark turning into a `b̄`, so `d u~ > W- Z b b~`
  (group 24) has no such rung. Group 24's channels are unchanged:
  `g24/27` reads the same in both columns.
- **What PR 5 relaxes on 134 is the W and Z single-leg sides.** Both bosons
  are uncut, so a rung that emits the W (or Z) alone off a beam quark loses
  its `t ≤ −ptj²` cap. Light-jet sides carry `ptj² = 100`, which equals the
  old cap.
- **σ is not there.** Of 4547 heavy points in PR 5 (4520 in main), **none**
  has `|t| < 100 GeV²` between either beam and the W or Z
  (`/home/user/wt/lp2-bin/region.py`). For a W, that region needs the boson
  to carry more than 98.5% of its beam side's light-cone momentum.
- **The channels that move do so both ways.** g6/26 and g6/1 improve under
  PR 5: their main χ² of 8 and 66 come from main's heaviest points (1.8%
  and 0.63% of σ). g0/24 and g0/13 get worse.
- **PR 5's heaviest point** (seed 5, 3.1% of σ) was drawn in g0/24, with
  99.98% of the mixture density on g2/6. It sits at `m_jj = 13.8 GeV`, just
  above `mmjj = 10`, at `x₂ = 1.4e-3`. All four beam transfers to the W and
  Z are above 1700 GeV².
  - So it is the γ* → jj low-mass corner, outside the region PR 5 opened.
  - Main's heaviest points are also outside it, in other corners. The
    1.8% and 0.97% points sit at `m_jj` 79–83 GeV, a W/Z → jj
    resonance; one of 0.63% sits at `m_jj` 16.9 GeV.
  - On both builds, the points above 0.5% of σ are spread over the
    resonant, low-mass and high-mass `m_jj` corners.
  - Without replaying that point under both builds, it cannot be said
    whether PR 5's density there is lower. See Found.

**Verdict under the planner's rule.** PR 5 owns no precision loss on 134:
- the 3-iteration pair agrees within noise (sd 1.31% against 1.33%, χ²/dof
  1.87 against 1.89);
- the campaign-setting pair differs by an amount not significant at 5
  seeds;
- no weight sits in the support PR 5 added.

No restriction or dedicated collinear map is warranted on this evidence, and
none is proposed for merge. The step-1 "2×/3–4×" reading came from comparing
different commands.

**A prototype exists if the long-run rows show a loss.** It was built,
never committed, and is at `/home/user/wt/lp2-bin/proto.patch`. In
`with_side_transfer_bounds` it relaxes a rung only on a single-leg side
whose own mass² is below the cap. That keeps the b̄ (m² = 22 GeV²) on 126
and holds the W and Z (m² ≥ 6467) at main's cap. On 126, seeds 1–2 equal
the fix bit for bit. It was not run on 134, because nothing called for it.

## Same-scheme MadGraph reference for card 126

vg-validator ran MadGraph 3.7.1 with `hard_survey 2` on L-P1's staged cards,
5 seeds:

| | σ (pb) | |
|---|---|---|
| A, complex-mass scheme off (vibegraph's scheme) | reported 2.51498e-4 | sem 0.09%, χ²/dof 0.66; EW-recombined 2.51522e-4 |
| B, complex-mass scheme on | 2.51388e-4 | |

- **The scheme's effect is −0.04%, measured** (B against A).
- **Against vibegraph,** #184's 20-seed means (L-P1, f70ab694) give vg/MG
  1.0026 at neval 120k (2.52144e-4) and 1.0013 at 480k (2.51820e-4). Both
  are within about 1.4σ.
- **The closing condition holds:** the mean is within MadGraph's
  same-scheme error, the scatter χ²/dof is 1.54 at 120k and 0.76 at 480k,
  and banked rows are unchanged within error (L-P1's bit-for-bit set; the
  validate below).

The gridpack's 2.504e-4 was never the comparison, and the 0.04% scheme
effect shows the scheme was not what separated the two populations.

## Gates on the code head 2091a26f

- **fmt:** `cargo fmt --all --check` gives no output, rc 0.
- **clippy:**
  ```
  cargo clippy --workspace --all-targets -- -D warnings
      Finished `dev` profile [optimized] target(s) in 36.66s          (rc 0)
  ```
- **lib tests:**
  ```
  cargo test -p vibegraph-lib --lib
  test result: ok. 1457 passed; 0 failed; 15 ignored
  ```
- **Hermetic suite.** `research/refs/mg5amcnlo` was emptied,
  `validation/madgraph/output` moved aside, and the fetched `validation/pdf`
  sets moved aside, keeping the six tracked files
  (`/home/user/wt/lp2-bin/hermetic.sh`, with a restore trap):
  ```
  hidden: mg5=0 output=0 pdf=fetch.sh fetch_cms.sh gen_oracle.cpp gen_oracle.sh oracle.json oracle_multigrid.json
  cargo test --workspace
  75 test binaries: 2307 passed, 0 failed, 28 ignored   (rc 0)
  HERMETIC EXIT rc=0 restored … mg5=18 out=132 pdf=8
  ```
- **Full banked layer.** It ran detached with an EXIT-trap marker and a PID
  watcher (`/home/user/wt/lp2-bin/validate.sh`). The untracked
  `.cargo/config.toml` (`profile.release-debug` debug = 0) was present for
  the run and deleted afterwards.
  - A first attempt died in 1 s: the profile setting through
    `CARGO_PROFILE_RELEASE_DEBUG_DEBUG` collides with `profile.release`'s
    env namespace.
  ```
  head 2091a26f code-dirty=no
  lock acquired 17:33:09
  pixi run --skip-deps validate
  validate rc=0 at 18:39:39
  test result lines: 2770 passed, 0 failed; 0 lines matching FAILED
  76 rows × 4 categories = 304 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 13 ⏳, 73 — / uncovered).
  The measured cells are exactly the cells the manifest declares, every gate cell passed
  ```
  The five ⚠️ are the cells the manifest declares measured and not
  enforced: gg_to_gg, ee_to_wpwm_cw, gg_to_gg_cg, ee_to_zh_smeft and
  ee_to_mumua.
- **kb-lint:** `pixi run kb-lint` gives `1354 concepts, 463 backlog items;
  0 errors, 5 warnings`. The warnings are on items this session did not
  touch.

## What each check cannot see

- **The 134 seed pairs** compare spread and error at one budget on one
  card.
  - Five seeds cannot calibrate a variance ratio below about 6 (F(4,4)).
    The verdict leans on the validator's pair agreeing too.
  - A loss that grows only at higher budget would not show.
- **The heavy-point probe** classifies only points above 5e-14 GeV⁻², about
  3.5e-4 of σ in one iteration.
  - Diffuse dilution of a relaxed channel's draws into the opened window,
    with f ≈ 0 there, is invisible to it.
  - It cannot show whether a given heavy point's mixture density differs
    between the two builds.
- **fix2's bit-for-bit result on the pairing card** shows it restores main
  there. It does not test a pairing card whose own decay products could
  form a single-leg side. The proof above says none can.
- **Card 126 on the head** is a sweep of 8 seeds: blind to a corner drawn
  less than about once in 8 runs, and to an offset all seeds share. The
  MadGraph comparison covers the offset to its 0.09% sem.
- **The validate run** gates the banked rows. None of them is a
  pairing-weighted card at campaign budget, and none is a VBS card.

## Long-run requests (vg-validator)

Branch `sprintl-multichannel`, code head **2091a26f** (any later commit on
this branch is kb-only). Each line of the table is one request.

Command A (campaign): `vibegraph integrate <proc> --run-card <run>
--param-card <param> --target-rel 0.01 --neval 120000 -j 4
--parallel-diagrams --seed N --no-network --output-ldjson`, seeds 1–5.

Command B (as the validator's pair): `--fixed-budget --neval 120000
--niter 3 --parallel-diagrams -j 4`, seeds 1–5.

| card | command | expected answer |
|---|---|---|
| 134 (VBSWmTZLTo4Jets) | A | σ ≈ 0.0554 pb; seed sd and mean error within noise of main's campaign rows (0.30–0.6% sd, about 0.55% error); scatter χ²/dof about 1. Bit-identical to the validator's 2091a26f B run under B |
| 700 (VBSWpZTo4Jets, `p p > w+ z j j QED=4 QCD=0`, unpolarised; cards as the sibling VBS session staged them) | A and B, both on main 7b36722c as well | PR 5 and main within noise on sd, χ²/dof and mean error; means within 1σ |
| 967 (cards as the validator staged it) | A and B, both on main 7b36722c as well | as 700 |

Both 700 and 967 have no forced decays under `cut_decays` with no decay
chain, so fix2 cannot move them. ab4fac69 and 2091a26f should be
bit-identical on them, as they are on 134.

## Found

1. **A point-replay probe would settle per-point attribution.** It would
   evaluate one recorded point's mixture density and every channel's
   density under two builds. Today a heavy point's weight can be compared
   between builds only statistically. That is why seed 5's 3.1% point on
   134 has no build-to-build answer.
   - To close it: land L-P1's probe hook (L-P1 Found 4), plus a
     `--replay-point` input that takes the printed `u` and channel index.
2. **Card 134 carries heavy single points on both builds.** The largest
   point per seed is 0.5–3.1% of σ on both sides. They sit in three
   corners: W/Z → jj resonant (`m_jj` 79–83 GeV), low-mass γ* → jj (`m_jj`
   14–17 GeV, just above `mmjj = 10`) and high-mass VBS. Main's own scatter
   χ²/dof swings between 0.23 (here, 6 iterations) and 1.87 (validator, 3
   iterations).
   - What is wrong: each of these points is drawn in a channel whose share
     of the mixture there is 1e-5 to 1e-2, so the channels that own those
     corners are thin there.
   - Evidence: the per-channel table above (g0/24, g6/26, g18/5) and
     `/home/user/wt/lp2-bin/heavy.py` over `/home/user/wt/logs/lp2-attr134/`.
   - To close it: a per-channel `max f/g` check on the VBS cards, then a
     density floor or an α floor for the γ*-pole channels.
3. **The step-1 VBS comparison mixed commands.** Any per-row "vg v0.2.2 vs
   branch" comparison should run both sides with one argv. The gpdb rows'
   argv (campaign) should be written beside the numbers.
4. **The validate profile setting has no env-var spelling.**
   `CARGO_PROFILE_RELEASE_DEBUG_DEBUG` is read as `profile.release`'s
   `debug`, and the banked run aborts in a second. The extended-validation
   skill's untracked `.cargo/config.toml` is the only route. It is easy to
   commit by accident, as the brief warned.
   - To close it: name the profile `release_debug`, or document the
     `--config` route in the skill.
