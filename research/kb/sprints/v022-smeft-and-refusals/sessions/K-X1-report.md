---
type: Session Report
title: "K-X1 report: split subprocess groups by outgoing masses"
description: "Flavour grouping now splits subprocesses by their outgoing masses slot by slot, after the content pairing of legs, as MadGraph's find_process_classes does; cards that grouped before are bit for bit, and ZPrimeToTT integrates, with σ against MadGraph at 5 seeds each."
status: draft
tags: [flavour-groups, phase-space, cms-gridpacks, madgraph-parity, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-17}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-17}
---
Worktree `/home/user/wt/sprintk-mass-split`, branch `sprintk-mass-split`, based
on `main` at `703f57f`. Brief: [K-X1](K-X1.md). MadGraph is the pinned 3.7.1
submodule, run from a private copy with `topBSM_UFO` installed, through
`validation/madgraph/mg5_pinned.sh` in the `madgraph` pixi environment. Every
heavy job used 2 of the host's 4 cores.

| commit | what |
|---|---|
| `42f3495` | feat(proton): split subprocesses by outgoing masses slot by slot |
| (final) | kb: this report, the closed item deleted, the census link repointed |

## The cause

`split_by_external_masses` keyed a multiplicity's subprocesses on their
incoming masses and the *multiset* of their outgoing masses. The decomposition
(`derive_flavor_groups`) compares outgoing masses *slot by slot*, after
`content_ordered` pairs each subprocess's legs with the first one's by
(mass, cut class). The two agree when equal multisets can be paired, and
disagree when they cannot. On ZPrimeToTT, `u u~ > ta+ vt b d u~ b~` and
`u u~ > e+ ve b ta- vt~ b~` hold the same masses. But the τ sits after the
`t` in one and after the `t~` in the other, and the light legs differ in cut
class (`j j` against `l` plus `ν`). No pairing lines the two up, so they
landed in one part and the part was refused.

## The change

- **`proton.rs`.** `content_ordered` is factored into `final_keys` (each leg's
  mass bits and cut class) and `content_order` (the greedy pairing), with
  unchanged behaviour. `final_masses_against(reference, set, ..)` returns the
  outgoing masses `set` presents after that pairing against `reference`.
- **`multiplicity.rs`, the split.** `split_by_external_masses` (it now takes
  the run card, for `maxjetflavor`) puts a subprocess in the first part whose
  lead subprocess has the same incoming masses and the same outgoing masses
  after pairing. Otherwise the subprocess starts a part of its own.
- **`multiplicity.rs`, the sum.** `MultiplicitySum::new` refuses two parts of
  one phase space. Its check now keys on outgoing masses in slot order, since
  two parts can share a multiset. The multiset key would have panicked on the
  new parts.
- **MadGraph** draws the same line. `madgraph/iolibs/group_subprocs.py`
  `find_process_classes` (3.7.1, lines 454–513) keys each leg's
  `(mass, spin, …)` in the process's leg order, so such subprocesses never share
  a `P` directory. ZPrimeToTT's 9 `P` directories split the same way:
  `P1_qq_ttx_t_wpq_wp_lvl_tx_wmq_wm_qq` beside
  `P1_qq_ttx_t_wpq_wp_tapvl_tx_wmq_wm_qq`, and four `P4_*` directories by
  lepton flavour.
- **No change for cards that grouped before.** Equal masses after pairing imply
  an equal multiset. A subprocess therefore never joins a part it would not
  have joined before, and a card the decomposition accepted gets the same parts
  in the same order. The bit-for-bit runs below measure this.

## Oracles

### The split on a small card (unit test)

`multiplicity::tests::equal_masses_in_different_slots_split_into_parts_of_their_own`
uses `p p > w+ w-, w+ > l+ vl, w- > j j` plus
`add process p p > w+ w-, w+ > l+ vl, w- > l- vl~`, with `l` holding `ta`. It
checks:

- the parts have the slot masses `[0,0,0,0]`, `[ta,0,0,0]`, `[0,0,ta,0]` and
  `[ta,0,ta,0]`;
- their union is the enumerated set, each subprocess once and in enumeration
  order;
- every part decomposes;
- two parts share a multiset in different slots;
- a `MultiplicitySum` over the parts builds, with distinct channel keys;
- the whole multiplicity is still refused.

```
$ cargo test -p vibegraph-lib --lib multiplicity:: -- --nocapture
parts: [32, 16, 8, 4]
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1358 filtered out; finished in 1.52s
```

The test fails first. With the split's key temporarily sorted back to a
multiset:

```
$ cargo test -p vibegraph-lib --lib equal_masses_in_different_slots
parts: [32, 24, 4]
panicked at vibegraph-lib/src/multiplicity.rs:1343:75:
a part decomposes: UnequalFinalMasses { a: "u u~ > ta+ vt d u~", b: "u u~ > e+ ve ta- vt~" }
test result: FAILED. 0 passed; 1 failed
```

*Blind to* whether the parts integrate to the right σ: the split only moves
subprocesses between decompositions. The ZPrimeToTT sweep is what sees the
sum.

### Cards that grouped before: bit for bit

Release builds of `main` 703f57f (a `git archive` in a separate target dir,
deleted afterwards) and of the branch. Each run uses `integrate --seed 11
--neval 40000 --fixed-budget --niter 4 -j 2`, and `grid` is the sha256 prefix
of `grid.bin.zst`:

```
main:                                                     branch:
ee_mumu rc=0 σ = 419.60 ± 0.38 fb grid=d025bd28b24531df   ee_mumu rc=0 σ = 419.60 ± 0.38 fb grid=d025bd28b24531df
dy13    rc=0 σ = 938.2 ± 4.6 pb   grid=f4a9a7e9d6c48876   dy13    rc=0 σ = 938.2 ± 4.6 pb   grid=f4a9a7e9d6c48876
llj_mlm rc=0 σ = 267.9 ± 1.3 pb   grid=ce8a8a69ac756b24   llj_mlm rc=0 σ = 267.9 ± 1.3 pb   grid=ce8a8a69ac756b24
dy_lll  rc=0 σ = 2.795 ± 0.014 nb grid=ad2ed72e9ee5ebab   dy_lll  rc=0 σ = 2.795 ± 0.014 nb grid=ad2ed72e9ee5ebab
```

The cards:

- `ee_mumu`: fixed beams, `config_share_ee_mumu_run_card.dat`.
- `dy13`: hadronic, the dy13 default cards.
- `llj_mlm`: MLM, `p p > e+ e- j` on `pp_to_llj_mlm_run_card.dat`.
- `dy_lll`: `p p > l+ l-` with `ta`, on the dy13 default run card. Its two mass
  parts were split before as well.

The first three rows match K-P1's independent `main` runs.

*Blind to* cards that were refused before, which are the ones whose parts
change.

### ZPrimeToTT_M2000GeV_W200GeV against MadGraph

The card is rebuilt, not the CMS gridpack: cvmfs is not reachable from this
host.

- **Source.** The cms-sw/genproductions 2017 13 TeV reference card
  `ZPrimeToTT_M500GeV_W50GeV`, from which its `generateCards.py` derives the
  other masses, at MS1 = 2000 GeV and WS1 = 200 GeV.
- **Model.** `topBSM_UFO-ckm_no_b_mass` from the CMS model store.
- **Run card.** That card's own: 6500 GeV beams, lhaid 315200
  (NNPDF31_lo_as_0130), dynamical scale `-1`. Gridpack mode and `use_syst` are
  off.
- **Parameters.** Both sides read MadGraph's generated param card
  (`--param-card`).
- **Not identical to the census sample.** That gridpack is the 13.6 TeV build,
  and the census lists lhaid 325300 for it.

MadGraph produces 490 subprocesses in 9 `P` directories. vibegraph splits the
6-leg multiplicity into 4 parts:

```
6 outgoing legs (masses [0.0, 0.0, 0.0, 0.0, 0.0, 0.0] GeV): 2 channels, …, 76.4% of the budget
6 outgoing legs (masses [1.777, 0.0, 0.0, 0.0, 0.0, 0.0] GeV): 2 channels, …, 20.1% of the budget
6 outgoing legs (masses [0.0, 0.0, 0.0, 1.777, 0.0, 0.0] GeV): 1 channels, …, 2.3% of the budget
6 outgoing legs (masses [1.777, 0.0, 0.0, 1.777, 0.0, 0.0] GeV): 1 channels, …, 1.1% of the budget
```

Per-seed results:

- MadGraph: `generate_events`, nevents = 50000, `nb_core = 2`, `iseed` = seed.
- vibegraph: release `integrate --seed N -j 2` at the default 0.1% target.

| seed | MadGraph σ (fb) | vibegraph σ (fb) |
|--:|---|---|
| 1 | 18.906 ± 0.026 | 18.946 ± 0.019 |
| 2 | 18.900 ± 0.024 | 18.954 ± 0.018 |
| 3 | 18.959 ± 0.024 | 18.954 ± 0.019 |
| 4 | 18.900 ± 0.024 | 18.909 ± 0.019 |
| 5 | 18.928 ± 0.024 | 18.877 ± 0.018 |

```
$ python3 kx1-stats.py kx1-zp-results.txt kx1-vg-zp-results.txt
MadGraph:  mean=18.9186 fb  seed sd=0.0254 fb (0.134%)  sd/sqrt(n)=0.0113  mean quoted err=0.0245  chi2/dof=1.127
vibegraph: mean=18.9280 fb  seed sd=0.0341 fb (0.180%)  sd/sqrt(n)=0.0152  mean quoted err=0.0186  chi2/dof=3.471
vg - mg = +0.0094 fb (+0.050%), combined sd/sqrt(n) 0.0190 fb, pull +0.50
```

Each row:

- Means are unweighted, and the pull is on the seed spreads (`sd/√n` on each
  side). It comes out at +0.50, so the two agree at 0.05%.
- MadGraph's χ²/dof across seeds is 1.13.
- Each vibegraph run's own iteration χ²/dof lies between 1.01 and 1.21. Across
  seeds, though, χ²/dof is 3.47: the seed spread (0.18%) is about 1.9 times the
  quoted 0.1%. Five seeds cannot calibrate that ratio, so it goes under Found
  rather than into the comparison.
- vibegraph needs 81–99 iterations, 9.7–11.9 M points and 13–15 min per seed on
  2 cores. MadGraph takes 5.2–5.5 min per seed.

A first pair at MadGraph nevents = 10000 read 18.839 ± 0.055 fb against
18.946 ± 0.019 fb (+0.57%, 1.8σ). The 50000-event seeds put MadGraph's
seed 1 at 18.906, so that offset was MadGraph's own statistics.

*Blind to*:

- the τ-mass effect on its own. τ subprocesses are under a quarter of σ, and
  the τ mass moves their σ by well under 1%, which is inside this sweep's
  resolution. A τ part integrated on massless kinematics would pass here; the
  bit-for-bit `dy_lll` row and the unit test pin the mass key instead;
- anything the 13.6 TeV gridpack's own run card would add.

### Fixed beams already refuse the mixture

This was a suspected silent wrong answer: `generate_sample` takes
`final_masses` from the first subprocess alone. It is refused before that
point. `e+ e- > l+ l-` with `l` holding `ta` (both `e+ mu+ ta+` and
`ta+ e+ mu+` orderings), dev build, `config_share_ee_mumu_run_card.dat`:

```
fwd rc=1  ERROR failed to compile subprocesses: fixed-energy beams require every subprocess to share the same external particle content, but the generated subprocesses differ
rev rc=1  ERROR failed to compile subprocesses: fixed-energy beams require every subprocess to share the same external particle content, but the generated subprocesses differ
tau rc=0  σ = 419.63 ± 0.45 fb
mu  rc=0  σ = 419.63 ± 0.45 fb
```

The refusal is `HadronicError::InconsistentExternals`, raised in
`compile_subprocesses` (`hadronic.rs`), which compares external particle IDs.
`generate` has no artifact to start from, and `generate_sample` calls the same
`compile_subprocesses`.

## Gates

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: rc=0.
- `cargo test --workspace`: rc=0, 61 test binaries, 0 failed. The lib binary
  reported `1356 passed; 0 failed; 15 ignored`. All three were rerun on the
  final tree, with the same results.
- `validate.sh` was not run locally; CI's `banked` job is that gate.

## Corrections to the brief

- **The suggested reproduction card does not reproduce the refusal.** With
  `p p > t t~, t > b w+, w+ > l+ vl`, the τ always sits in one slot, so the
  multiset split already separated τ from e/μ. The refusal needs two process
  lines that put a massive lepton in different slots, as the unit test's card
  does.
- **The refusal count is 2, not 35.** The sprint table says 35; the item and the
  202a540 census say 2.

## Found

- **vibegraph's quoted error understates its seed spread on ZPrimeToTT.** Over
  5 seeds the spread of σ is 0.18% against a quoted 0.10% (χ²/dof 3.47), while
  each run's own iteration χ²/dof is 1.0–1.2. Five seeds cannot calibrate the
  ratio: the multi-seed criterion warns that few-seed scatter can be off by 2–5×.
  Twenty or more seeds at the default target would measure it. If it holds, the
  0.1% target is not what a run delivers on this card.
- **Fixed beams refuse any flavour mixture, not only mixed masses.**
  `compile_subprocesses` (`hadronic.rs`, `InconsistentExternals`) requires
  identical external particle IDs, so `e+ e- > l+ l-` is refused even with
  `l = e mu`, where every mass agrees. MadGraph runs such a card. A split by
  external content, or by masses as on the proton path, with one summed
  integrand per part, would lift it. It is a named refusal, not a wrong answer.
