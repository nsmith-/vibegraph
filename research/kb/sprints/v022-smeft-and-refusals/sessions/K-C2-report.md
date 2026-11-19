---
type: Session Report
title: "K-C2 report: jet and lepton pT-rank cuts"
description: "ptj1min–ptj4max and ptl1min–ptl4max implemented as MadGraph's cuts.f rank cuts, with σ against MadGraph 3.5.7 on two DY1j pT bins and a biting ZH lepton cut, the ZH SMEFTsim gate, a single-leg control that separates the rank cut from the ℓℓj offset, and bit for bit on three banked cards."
status: draft
tags: [run-card, cuts, refusal, report, validation, madgraph-parity, cms-gridpacks, smeft]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-18}
---
Branch `sprintk-runcard` after K-C1 (488295c). Brief: [K-C2](K-C2.md). The two
closed items' files, `rank-jet-pt-cuts-unimplemented` and
`rank-lepton-pt-cuts-unimplemented`, are deleted in the final commit; no other
page linked them.

## What changed

**Rank cuts** (`vibegraph-lib/src/cuts.rs`, `RankCut`, two `Cuts` fields
`jet_rank` and `lepton_rank`). Citations are to the pinned
`research/refs/mg5amcnlo` (3.7.1). The same blocks in the local MadGraph 3.5.7
(`Template/LO/SubProcesses`) are identical by `diff`, 14 lines earlier in
`cuts.f`.

| convention | MadGraph | vibegraph |
|---|---|---|
| ranked objects | jets: `is_a_j` (`cuts.f:525-529`), gluon and quarks up to `min(maxjetflavor, 6)` (`setcuts.f` class flags), so a b above `maxjetflavor` is not ranked; leptons: `is_a_l`, `e mu ta` only (`cuts.f:925-930`) | `Letter::Jet` / `Letter::Lepton` members |
| `do_cuts` | not read by either block; the template says "sorted cuts always apply" (`Cards/run_card.dat:102`) | decay products under `cut_decays = F` and legs above 20 GeV are ranked and cut |
| ordering | by pT, hardest first (`cuts.f:826-835`, `:943-952`) | four hardest by insertion |
| fewer objects than the rank | a positive `pt{N}min` fails every point with fewer than N members (`cuts.f:817-820`, `:934-937`); a maximum alone requires nothing | same |
| bounds | N-th hardest, N ≤ min(n, 4): `<` min fails, `>` max fails when max ≥ 0 (`cuts.f:848-864`, `:957-965`); thresholds copied at `setcuts.f:488-496`, `:514-522` | same |
| `cutuse` | `jetor = cutuse.eq.0` (`setcuts.f:509`); "all" mode at `cuts.f:858-861`; the leptons have no `cutuse` and are always "any" | `cutuse = 0` implemented; `cutuse ≠ 0` inert with every jet bound off, refused by name with one on |
| τ floor | rank minima folded per member, in leg order, into `smin` (`setcuts.f:534-566` jets, `:639-670` leptons, loop to `nexternal-1`), applied at `myamp.f:569-570` | own provable bound: the k-th hardest member is above `max(t_k, a_(k))`, `t_k` the largest minimum at rank k or above, `a_(k)` the k-th largest single-leg pT minimum of the members |

Two cases where MadEvent's floor cuts on its own are refused by name
(`CutError::ValueRefused`), not reproduced:
- **`cutuse = 1` with a jet bound on.** "All fail" keeps points whose leading
  jet is below `ptj1min`, but `smin` still adds `ptj1min` as though it held.
- **A ranked class mixing cut and uncut members under a positive `pt{c}` or
  `e{c}`** (decay products under `cut_decays = F`, or a jet above 20 GeV, beside
  cut members). MadGraph's leg-order terms can put the rank minimum on an uncut
  member while the hardest is a cut one, counting it twice. With every member cut,
  or none, the sum is a valid bound and the card compiles. ZH 832/837 have both
  leptons as decay products: none mixed.

Also: a rank floor on every member of a class (`pt{n}min` for n ≤ 4 members)
regulates that class's soft and beam-collinear limits;
`rejects_zero_transverse_momentum` and `admits_every_point` read the rank cuts.
A process with no member and a positive minimum (`e+ e- > mu+ mu-` with
`ptj1min = 30`) cuts every point, as MadGraph does, and ends with the existing
named error "the cuts reject every point the sampler reached".

**Lists, report and table.** The 16 rank names left `UNIMPLEMENTED_CUTS`,
`NONPOSITIVE_IS_OFF` and `SINGLE_PT_MAXIMA` (now `missetmax` alone). `classes.rs`
classes them `Consumed(R_CUT_RANK)` and `cutuse` `Consumed(R_CUTUSE)`;
`report.rs` lists `cutuse` in `VALUE_REFUSALS` and carries its refusal on its own
key in `vibegraph run-card`. Tests that used `ptj1min` as the example of an
unimplemented cut use `ht2min` or `misset`.

**Concept.** [run-card/madgraph-cut-conventions](../../../run-card/madgraph-cut-conventions.md)
describes the rank cuts and the two refusals.

## Oracles

All on 2 cores (`-j 2`, MadGraph `nb_core 2`). vibegraph: release build of this
branch, `integrate --target-rel 0.001 -j 2 --seed 1..5`. MadGraph: 3.5.7 from the
main checkout's pixi env, seeds `iseed` 1–5, 50 000 events, σ and error from
`SubProcesses/results.dat`. Statistics: unweighted mean of 5 seeds, sample sd
across seeds, χ²/dof of the seeds about their mean with their quoted errors, and
the pull of the means with the quoted errors (`√Σσᵢ²/n`) and with the seed
spread (`sd/√n`). Script and per-seed logs:
`/home/user/wt/sprintk-runcard-mg/kc2/` (`mg/sweep_results.txt`,
`vg/sweep_results.txt`, `ctrl/`, `b4b/`, `seven/`, `logs/kc2-*`).

**Substitutions.**
1. **DY1j cards.** cms-sw/genproductions (a7fbf78) holds only the
   `DY1jToLL_M-1to10` Pt-0to70 and Pt-600toInf directories, and
   `makeDatacards.py`, which writes the six bins. The 70to100 and 200to400 run
   cards were rebuilt from the 600toInf card with `ptj1min`/`ptj1max` at the bin
   edges, `lhaid = 315200` (NNPDF31_lo_as_0130, local) in place of the
   `$DEFAULT_PDF_SETS` placeholder (both sides read the same set),
   `$DEFAULT_PDF_MEMBERS` dropped, `gridpack`, `use_syst` False, `nevents` 10000
   (50 000 on MadGraph's side). Process as the card: `sm-ckm_no_b_mass`,
   `l = e mu ta`, `p p > l+ l- j`.
2. **Local `sm` model for MadGraph.** 3.5.7 does not ship
   `restrict_ckm_no_b_mass.dat`; MadGraph imported a copy of its `models/sm`
   carrying the CMS restriction vibegraph bundles
   (`vibegraph-lib/src/ufo/sm_assets/restrict_ckm_no_b_mass.dat`).
3. **`NPprop=0` dropped** from the ZH generate line: 3.5.7 refuses it as 3.7.1
   does ("model order NPprop not valid for this model"). vibegraph runs the cards
   unpatched.
4. **`symmetry.f` patched** in the two MadGraph process directories: this env's
   gfortran stops `gensym` on the `$` edit descriptors (`'(I1$)'`, `'(a$)'`:
   "Missing comma between descriptors"); written `',$)'`. Output formatting only.
   The CERN host needed the same patch.

**Jet family, DY1j `Pt-70to100`** (pb):

| side | seeds | mean | sd | χ²/dof |
|---|---|---|---|---|
| MadGraph | 15.730 15.764 15.747 15.772 15.755 | 15.7536 | 0.0162 | 0.42 |
| vibegraph | 15.835 15.806 15.800 15.770 15.776 | 15.7974 | 0.026 | 2.65 |

Δ = +0.278%, pull 3.29 (quoted), 3.20 (spread).

**Jet family, DY1j `Pt-200to400`** (pb):

| side | seeds | mean | sd | χ²/dof |
|---|---|---|---|---|
| MadGraph | 0.36544 0.36611 0.36568 0.36465 0.36472 | 0.36532 | 0.00063 | 1.77 |
| vibegraph | 0.36642 0.36644 0.36605 0.36571 0.36570 | 0.366064 | 0.00036 | 1.11 |

Δ = +0.204%, pull 2.84 (quoted), 2.29 (spread).

**Single-leg control, `Pt-70to100`.** With one jet, `ptj1min = 70`,
`ptj1max = 100` is the same cut as `ptj = 70`, `ptjmax = 100` with the rank cut
off, compiled through different code on each side (MadGraph's rank block against
its single-leg block; `RankCut` against `SingleLegCut`), with the same MadGraph
`smin`. The unit test `a_single_jet_rank_cut_is_its_single_leg_cut` shows the two
compiled filters accept the same points over a pT scan through both boundaries
and give the same ŝ floor. Same seeds, same budgets:

| comparison | rank mean | single-leg mean | Δ | pull (quoted / spread) |
|---|---|---|---|---|
| MadGraph, rank vs single-leg | 15.7536 (sd 0.0162) | 15.7626 (sd 0.0173, χ²/dof 0.47) | +0.057% | 0.56 / 0.85 |
| vibegraph, rank vs single-leg | 15.7974 (sd 0.026) | 15.7888 (sd 0.0087, χ²/dof 0.30) | −0.054% | −0.86 / −0.70 |
| single-leg, vibegraph vs MadGraph | | 15.7888 vs 15.7626 | +0.166% | 1.95 / 3.03 |

Each side's rank σ equals its own single-leg σ within its seed spread, and the
single-leg card reads +0.17% high on its own. The DY1j offset is therefore not
the rank cut: it is present without it, the
`llj-family-reads-high-vs-seed-records` offset (+0.15–0.23% on every ℓℓj row,
flat in budget), attributed here by measurement on these cards. The DY1j rows
close the jet item on that reading.

**Lepton family, biting: ZH 832 at `ptl1min = 50`.** The 832 inputs with
`ptl1min` raised to 50. One vibegraph seed at 0 and 50: 74.720 ± 0.090 fb and
55.867 ± 0.082 fb, a −25% shift. Both leptons are Z decay products under
`cut_decays = F`: `ptl`, `etal` and `mmll` do not reach them, the rank cut does.
In pb:

| side | seeds | mean | sd | χ²/dof |
|---|---|---|---|---|
| MadGraph | 0.055913 0.055955 0.055994 0.055930 0.055900 | 0.0559384 | 3.7e-5 | 0.46 |
| vibegraph | 0.055967 0.055881 0.056005 0.056018 0.056003 | 0.0559748 | 5.6e-5 | 1.00 |

Δ = +0.065%, pull 1.04 (quoted), 1.21 (spread).

**ZH gate at the cards' own `ptl1min = 10`**, own param cards (837 has
`chw = 0.4`), vibegraph 5 seeds against the CERN host's MadGraph 3.7.1 runs (50k
events, `iseed` 1, one run each). In pb:

| card | vibegraph seeds | mean | sd | χ²/dof | reference | Δ | pull (quoted / spread) |
|---|---|---|---|---|---|---|---|
| 832 SM | 0.074616 0.074717 0.074870 0.074943 0.074928 | 0.0748148 | 1.43e-4 | 3.81 | 0.07481 ± 5.06e-5 | +0.006% | 0.08 / 0.06 |
| 837 HW0p4 | 0.099514 0.099546 0.099730 0.099846 0.099793 | 0.0996858 | 1.48e-4 | 2.42 | 0.0997 ± 5.97e-5 | −0.014% | −0.19 / −0.16 |

The references are single runs, so their spread is unmeasured: a pull near 0 is
consistent but weakly constrained. The reference is MadGraph 3.7.1 at one seed,
where the other rows compare with local 3.5.7 at five.

**vibegraph's quoted error undercovers its seed spread** at `--target-rel 0.001`
on these rows: χ²/dof 2.65 (DY1j 70to100), 3.81 and 2.42 (ZH gates). Every pull
above is also given against the seed spread. This is the symptom recorded under
`zprime-tt-quoted-error-below-seed-spread`; nothing here distinguishes it.

**Seven cards unpatched.** 832 and 837 pass `integrate --check` with their own
run, param and proc cards (`check ok: p p > z h SMHLOOP=0 NPprop=0 NP<=1, z > l+
l-, h > b b~`) and integrate in the gate. The other five stand in as the 832
cards with their SMEFT point in the param card (833 `chb = 0.45`, 834 `chb =
0.18`, 835 `chw = 0.16`, 836 `chb = 0.9`, 838 `chb = 1.35`); each passes
`--check`, and so does 833 with its `maxjetflavor = 4` (its `lhaid` 325500 is not
local and was not substituted). One short integrate per family
(`--fixed-budget --neval 20000 --niter 3 --seed 1`): 833 HB (`maxjetflavor 4`)
78.16 ± 0.45 fb, 835 HW 84.27 ± 0.48 fb.

**Bit for bit**, main 0340a89 (separate worktree and target, deleted afterwards)
against this branch, `--fixed-budget --neval 20000 --niter 3 --seed 7 -j 2`,
`LHAPDF_DATA_PATH=validation/pdf`, default rank cuts:

| card | σ main = branch | grid.bin.zst sha256 (main = branch) |
|---|---|---|
| `config_share_ee_mumu_run_card.dat`, `e+ e- > mu+ mu-` | 419.72 ± 0.75 fb | 934120b117f2e527edb2559840fb60bdf3b5843742aed0d169b732d00a72b29b |
| `dy13_default_run_card.dat`, `p p > e+ e-` | 935 ± 11 pb | b5321668ddc818289629ddb98404ce77e9d78046c6fdf2467223bc925d3300f0 |
| `pp_to_llj_mlm_run_card.dat`, `p p > e+ e- @0; add process p p > e+ e- j @1` | 932 ± 12 pb | 1c274c427d46a9ff288086f3dae013c29598b0213c98639e6fa71af84e1c4075 |

`cmp` of the three pairs of grid files: identical.

## What each test cannot detect

- The unit tests (`cuts.rs`, "pT-ranked jet and lepton cuts") pin ordering, fewer
  members than the rank, min and max boundaries, the fifth jet, `maxjetflavor`,
  lepton membership, exempt decay products, both refusals, `cutuse`, IR
  regulation and the ŝ floor against my reading of `cuts.f`; a guard I misread
  is reproduced in both the code and the test.
- The DY1j rows exercise one jet: rank 1 only, no reordering between jets. The
  multi-jet ordering and ranks 2–4 are tested only on hand-built momenta. The
  control shows equality with a single-leg cut, so it cannot see an error that
  both of vibegraph's cut paths share.
- The ZH lepton rows exercise ranks 1 of two leptons that are both decay
  products, `ptl1min` only; no lepton maximum and no lepton rank above 1 is
  measured against MadGraph.
- No oracle measures the τ floor's effect: a valid floor changes no σ, and the
  refusals keep the cards where MadEvent's floor would cut out of every run.
- The ZH gate reads against single reference runs of unmeasured spread.
- Bit for bit covers default rank cuts only.

## Gates

`CARGO_TARGET_DIR=/home/user/wt/sprintk-runcard/target` with the lean
variables.
- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo test --workspace -j 2 -- --test-threads 2`, run last, after the
  sweeps: 66 test binaries `test result: ok`, none failed, `DONE rc=0` (log
  `kc2-test.log`).
- `pixi run --skip-deps kb-lint`: 0 errors.
- The `banked` job is CI's; `validate.sh` was not run.

## Corrections to the brief

- The pinned submodule is MadGraph 3.7.1, not 3.5.7; the local 3.5.7 has the
  same rank and `smin` code.
- genproductions has two of the six DY1j bins; the rest are generated by
  `makeDatacards.py`.
- MadGraph 3.5.7 needs the `symmetry.f` format patch with this env's gfortran.
- The brief's list of conventions misses one case: a ranked class mixing cut and
  uncut members, where MadEvent's floor cuts on its own. It is refused.

## Found

- **MadGraph's class flags are per directory, not per subprocess.** `setcuts.f`
  sets `is_a_j`, `is_a_b`, `is_a_l` inside `do iproc=1,maxsproc` without an iproc
  index, so in a grouped directory the last subprocess's leg classes apply to
  all of them (for example a b against a u at `maxjetflavor = 4` under
  `no_b_mass` grouping). This touches every class cut, not only the rank cuts.
  Closing it: a census of grouped directories where one leg position changes
  class across iproc, and a σ comparison on one.
- **`resolve_mom` is dead code in a release build.** `cargo build --release -p
  vibegraph` warns "method `resolve_mom` is never used"
  (`vibegraph-lib/src/helas/eval/analysis.rs:208`); clippy over all targets does
  not, so CI misses it. Closing it: gate the method on the configuration that
  uses it, or remove it.
