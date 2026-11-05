---
type: Session Report
title: "H-K1 report: per-group cut indicators, and cut agreement as a grouping join condition"
description: "D4 and item 8 implemented. A subprocess whose cut filter differs from a group's starts a group of its own (CutIndicatorDiffers is gone), and every flavour group carries its own cut indicator in ProtonIntegrand (GroupCutsDiffer is gone): a point's value is Σ_g Θ_g·term_g and its event is drawn among the passing groups. Every channel is built under the weakest of the groups' floors (UnionBounds): built under its own group's floors, the mixture read g g > z j j, z > j j 4% low, because a channel's density below its floors is positive. MadGraph refuses 5F p p > j j at maxjetflavor 4 whenever the b cuts differ from the jet cuts (setcuts.f consistency check), so the backlog item's 'MadGraph runs this card' holds only for equal b and jet cuts. σ: p p > j j at maxjetflavor 4 with ptb 30/etab 2.5 against MadGraph's three class-split runs, −6.1e-4, pull −0.98, five seeds each; g g > z j j, z > j j at cut_decays T +4.3e-3 (pull +2.7); at cut_decays F −6.8%, all of it in the four same-flavour subprocesses (an existing decay-product rule, Found); the twelve others agree to 1e-4. Cards with one filter are bit for bit."
status: draft
tags: [cuts, flavour-groups, five-flavour, decay-chains, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-05}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-05}
sources:
  - {id: setcuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setcuts.f#L196", title: "MadGraph setcuts.f: per-IPROC cut classes from IDUP(i,1,iproc) and the grouping consistency check"}
  - {id: groupsub, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/group_subprocs.py#L453", title: "MadGraph group_subprocs.py find_process_classes"}
  - {id: survey-bias, resource: "../../../validation/madevent-default-survey-bias.md", title: "MadEvent's default survey settings bias σ low"}
  - {id: hd2, resource: "H-D2-report.md", title: "H-D2 report"}
---
Session H-K1 of sprint H: the user's decision D4 (per-group cut indicators) and
item 8, `five-flavour-ppjjj-refused-cut-indicator-differs`. Worktree
`/home/user/wt-gc`, branch `sprinth-group-cuts` off `origin/main` `d56871e9`:

```
$ cd /home/user/wt-gc && git rev-parse --show-toplevel && git branch --show-current && git log --oneline -1
/home/user/wt-gc
sprinth-group-cuts
d56871e9 Sprint G (2/3): diagnose the one-point σ, VEGAS under-coverage and the heap aborts (#89)
```

`$S` below is the scratch directory
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/H-K1`.
`vibegraph-before` is a release build of `d56871e9`, and `vibegraph-after2` a
release build of this branch's final code. MadGraph is the pinned checkout
(3.7.1, `b768706`), run with LHAPDF 6.5.4 (`pdlabel = lhapdf`, `lhaid = 247000`),
`hard_survey = 2`, a fresh copy of the output directory per seed (`$S/mg_run.py`),
and `nevents = 10000`. MadGraph σ values are the first line of
`SubProcesses/results.dat`, kept in `$S/mgkeep/`.

## 1. What MadGraph does

Read on the pinned source, then measured.

**Cut classes are per matrix element, read off its first flavour row.**
`setcuts.f:196` loops over the matrix elements of a P directory (`iproc`), and
`setcuts.f:217` classes leg `i` from `idup(i,1,iproc)`, the first flavour row of
that matrix element only. Matrix elements that differ only by flavour are merged
into one with several rows: `p p > j j` under `sm-no_b_mass` puts
`g g > u u~, …, g g > b b~` into one matrix element (`P1_gg_qq/leshouche.inc`,
`IDUP(I,1,1) = 21,21,2,-2`, `IDUP(I,5,1) = 21,21,5,-5`). The b row then takes
the u row's classes.

**Within a P directory every matrix element must compile to the same cuts.**
After each `iproc`, `setcuts.f:712-808` compares the per-leg thresholds
(`etmin`, `r2min`, `s_min`, …) with the first matrix element's. On any
difference it writes `Grouping of subprocesses not consistent with setcuts.f` and
stops (`stop 1`, `setcuts.f:808`). The comparison is on threshold values, not
classes. `smin` is not compared: it is recomputed per `iproc` and the last one
stands (`setcuts.f:708`).

**P directories are integrated apart.** `find_process_classes`
(`group_subprocs.py:453`) keys a P directory on masses, spins, colour and
on-shell flags, not on PDG codes, so a massless b shares a P directory with the
light quarks. Each P directory runs its own `passcuts` (`dsample.f:181`) and its
own channels, and the total is the sum of the P directories' σ.

So the cuts are uniform within a P directory, which is the unit MadGraph
integrates, and they differ between P directories. The forced-resonance windows
(`cut_bw`, `myamp.f`) belong to each configuration of a P directory. `smin`
belongs to the P directory and enters its own `τ` floor.

**Measured: MadGraph refuses 5F `p p > j j` at `maxjetflavor 4` when the b cuts
differ from the jet cuts.** Same card, `ptj 20`, `etaj 5`, `drjj 0.4`, fixed
scales at 91.188, seed 1:

```
$ $S/mg_run.py H-K1-ppjj-probe H-K1-mg-ppjj-mjf4-s1 1 maxjetflavor=4 ptb=0.0 etab=-1.0 drbb=0.0 drbj=0.0 mmbb=0.0 …
$ cat H-K1-mg-ppjj-mjf4-s1/error
 Error: grouping of subprocesses not consistent with setcuts.f. Either change your cuts and/or turn grouping of subprocesses off.
$ cd H-K1-mg-ppjj-mjf4-s1/SubProcesses/P1_qq_qq/Hel && ../madevent_forhel < input_app.txt
 Define smin to   1600.0000000000000   (×7)
 Define smin to   400.00000000000000
 Grouping of subprocesses not consistent with setcuts.f. Either change your cuts and/or turn grouping of subprocesses off:r2min
STOP 1
```

The eighth matrix element of `P1_qq_qq` is `u~ b > b u~` (`processes.dat`), whose
first row puts a b on leg 3, where the other seven put a light quark. With b cuts
equal to the jet cuts (`ptb 20`, `etab 5`, `drbb 0.4`, `drbj 0.4`) MadGraph runs,
and gives the `maxjetflavor 5` number to the bit:

```
H-K1-mg-ppjj-mjf4eq-s1  0.54615E+09 0.24162E+07
H-K1-mg-ppjj-mjf5-s1    0.54615E+09 0.24162E+07
```

**This corrects the backlog item.** It says "MadGraph runs this card". That holds
only for equal b and jet cuts, and vibegraph already ran that card before this
change (`vibegraph-before … rc_ppjj_mjf4eq.dat`: `σ = 549.62 ± 0.65 µb`; seed 3,
§4: bit for bit before and after). The card the item reproduces, with the
template's `ptb = 0`, `etab = −1`, is one MadGraph refuses. With per-group
indicators vibegraph cuts each subprocess with its own classes, which is what
MadGraph's rule gives on every card it accepts.

**What vibegraph reproduces.**

- The cut filter belongs to a flavour group, the unit vibegraph evaluates. A
  subprocess joins a group only if it compiles to the group's filter. Where
  MadGraph would refuse, vibegraph cuts each subprocess with its own classes.
  It does not use MadGraph's first-row rule, which only matters where MadGraph
  refuses.
- Each group's term is cut by its own filter: Σ_g Θ_g(p)·term_g, the per-point
  form of MadGraph's sum over P directories.
- The `ŝ` floor (`shat_min_hint`) and the `τ` range: the hull over the groups'
  ranges. Each group's bound holds on its own accepted region, so the minimum
  holds on the union.
- Forced-resonance windows stay in each group's filter (`Cuts::windows`) and in
  each diagram's channel. The windows' `bwcutoff` is the run card's and is the
  same for every filter.
- An event's cut record: the event is drawn among the groups whose filter
  accepts it. A rejecting group's scales are recorded as `None`, and
  `select_event` gives a term with no scale no weight.

## 2. Every consumer of the decomposition-wide `Cuts`

| consumer | before | now |
|---|---|---|
| `derive_flavor_groups` | `CutIndicatorDiffers` refusal | join condition, `compiled[head].2 != compiled[i].2` → own group; separation floor exempts a cut-only split |
| `ProtonIntegrand::build` | `GroupCutsDiffer` refusal | `filters` (distinct) and `group_filter` |
| channel maps (spacelike floor, timelike floors, energy floors, windows' `bwcutoff`) | group 0's cuts | `UnionBounds(&filters)`: the weakest of each bound (§3) |
| `process_shape` (map rule) | group 0's cuts | all diagrams under `UnionBounds` |
| `τ` range, `TauMixture` | `tau_range(group 0)` | hull of the filters' ranges |
| `shape()` cut test | one `pass` | `cut_point`: one `pass` per distinct filter, stored per filter in the scratch |
| `shared_scale_sum` | every group | a rejecting group's scales `[None, None]`, its term skipped |
| `per_group_sum` (MLM clustering, `rewgt`) | every group clustered | a rejecting group is not clustered and records `None` everywhere |
| `select_event`, event writing, unweighting | groups by term | unchanged: a rejecting group's term is zero, so it is never drawn |
| `#89` census | passed = one filter | passed = some group's filter accepts |
| `probe_scale` | one filter | some filter |
| MadEvent mirrored-cut emulation | one `[direct, mirrored]` pair | one pair per filter, read per group |
| `$` on-shell vetoes | per group already | unchanged |
| `FixedBeamIntegrand` (`hadronic.rs`) | one subprocess content | unchanged: fixed beams refuse different contents before any grouping |
| `edge_probe.rs` (test) | `integ.cuts.pass` | `any_group_passes` |

The multiplicity sum (`multiplicity.rs`) reads `process_shape` per part and is
otherwise untouched. MLM clustering reads each group's representative colours
and channels. It needs no global cut: the only cut it reads is `xqcut`, which
reaches the filter through the rewritten `ptj` (`runcard/matching.rs`).

## 3. Channels must cover the union, and the first version did not

The first version built each group's channels under that group's own cuts. On
`g g > z j j, z > j j` at `cut_decays = F`, the four same-flavour subprocesses
cut all four quarks, and the twelve others cut only the two core quarks. That
version read 137.62 ± 0.17 pb (five seeds, `$S/vg/vgsig_ggzjj_s*.log`), against
140.81 for the two halves run apart:

```
$ vibegraph-after integrate ggzsame4.dat --run-card rc_ggzjj.dat … --target-rel 0.003    σ = 28.517 ± 0.081 pb
$ vibegraph-after integrate ggzdist12.dat --run-card rc_ggzjj.dat … --target-rel 0.003   σ = 112.29 ± 0.27 pb
```

A larger budget left it in place (`--neval 600000`: 137.47 ± 0.18; `--alpha-floor
0.02`: 137.47 ± 0.37, `$S/H-K1-ggzjj-budget.out`). A scratch library test, since
removed, showed that the integrand itself is linear. Over 20000 points drawn from
the full mixture, the full decomposition's value equalled the sum of the values
of the same-flavour-only and distinct-only decompositions, with 0 mismatches at
1e-12. With the synthetic probe PDF and 200k × 10:

```
sigma full  seed 1: 7.72919e-1 +- 1.76e-3     (same4 1.61720e-1, dist 6.45620e-1: sum 8.0734e-1)
all channels under the strict filter:  7.60413e-1, 7.62883e-1
all channels under the loose filter:   8.06032e-1, 8.06120e-1
```

The mechanism is documented on `DiagramChannel::with_timelike_floors` and
`with_split_angles`: below a floor a channel keeps a positive density rather
than zero. That is consistent only while every configuration below the floor has
integrand zero. With per-group cuts it does not: a configuration below the
same-flavour groups' pair floor passes the distinct groups' cuts. The
strict channels then add density there, and the mixture's `g` is overstated.
The fix builds every channel under `UnionBounds`, the weakest of each filter's
floors (`phasespace/maps.rs`). On one filter these are that filter's floats, so
single-filter cards do not move (§4). The final σ is 140.39 (§5b).

## 4. Bit for bit on cards with one filter

Before and after binaries, `--seed 3 --fixed-budget --neval 40000 --niter 6`,
then `generate --seed 7 --nevents 2000`. The hashes are of the artifact and of
the LHE from the first `<event>` on. The headers differ only in the artifact
path line. (`$S/H-K1-rerun.out`, `$S/H-K1-bitforbit-events.out`)

```
bfb ppjj:  before σ = 551.6 ± 1.1 µb after2 σ = 551.6 ± 1.1 µb grid-sha 0568456ee0ec215d 0568456ee0ec215d events-sha 4fb9975010183bc5 4fb9975010183bc5
bfb llj:   before σ = 212.23 ± 0.69 pb after2 σ = 212.23 ± 0.69 pb grid-sha 86793e56d63ebf3b 86793e56d63ebf3b events-sha 1b554ce0cda81568 1b554ce0cda81568
bfb ppjj5: before σ = 551.6 ± 1.1 µb after2 σ = 551.6 ± 1.1 µb grid-sha 294e436431725b76 294e436431725b76 events-sha 4fb9975010183bc5 4fb9975010183bc5
```

The first attempt at this check, `$S/H-K1-bitforbit.out`, called `generate` on
the artifact directory instead of `grid.bin.zst`. No LHE was written, and every
events hash there is `e3b0c442…`, the SHA-256 of empty input. That attempt
compares σ and grids only. The rows above come from the rerun. The files are
real:

```
bfb_before_ppjj.lhe events=2000 bytes=1325358  events-sha256=4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
bfb2_ppjj.lhe       events=2000 bytes=1325352  events-sha256=4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
bfb_before_llj.lhe  events=2000 bytes=12672672 events-sha256=1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
bfb2_llj.lhe        events=2000 bytes=12672666 events-sha256=1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
bfb_before_ppjj5.lhe events=2000 bytes=1325359 events-sha256=4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
bfb2_ppjj5.lhe      events=2000 bytes=1325353  events-sha256=4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
(empty input: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855)
```

The byte counts differ only by the length of the artifact path in the header.

**Repeated on the merge with `origin/main` (`9601bfe0`, which includes #103).**
`vibegraph-main` is a release build of a detached worktree at `origin/main`
`9af360ac`, the merge's second parent (`$S/H-K1-build-main.log`: `main
9af360ac…`, `Compiling vibegraph-lib v0.1.0 (/home/user/wt-gc-main/vibegraph-lib)`,
`BUILD-EXIT 0`, and `cmp` against `vibegraph-merged`: `DIFFERENT`). Its
`--version` line reads `9601bfe0`, a stale build-script stamp, not its source.
`vibegraph-merged` is a release build of `9601bfe0`. Same flags (`$S/H-K1-bfb3.sh`,
`$S/H-K1-bfb3.out`):

```
main ppjj:    σ = 551.6 ± 1.1 µb grid-sha 0568456ee0ec215d events=2000 bytes=1325357 events-sha 4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
main llj:     σ = 212.23 ± 0.69 pb grid-sha 86793e56d63ebf3b events=2000 bytes=12672671 events-sha 1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
main ppjj5:   σ = 551.6 ± 1.1 µb grid-sha 294e436431725b76 events=2000 bytes=1325358 events-sha 4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
merged ppjj:  σ = 551.6 ± 1.1 µb grid-sha 0568456ee0ec215d events=2000 bytes=1325359 events-sha 4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
merged llj:   σ = 212.23 ± 0.69 pb grid-sha 86793e56d63ebf3b events=2000 bytes=12672673 events-sha 1b554ce0cda81568ef2d79f1e3eed1320c324946e8ee719ec978d4e3db6715b5
merged ppjj5: σ = 551.6 ± 1.1 µb grid-sha 294e436431725b76 events=2000 bytes=1325360 events-sha 4fb9975010183bc5fb399c5b273a4ceeb03dbd491061601d15230dc52ac7e5c7
```

The two bytes are the artifact path in the LHE header, `bfb3_main_…` against
`bfb3_merged_…`; nothing else in the header differs:

```
$ cd $S/vg && diff <(sed '/<event>/,$d' bfb3_main_ppjj.lhe) <(sed '/<event>/,$d' bfb3_merged_ppjj.lhe)
5c5
< artifact bfb3_main_ppjj/grid.bin.zst
---
> artifact bfb3_merged_ppjj/grid.bin.zst
```

(`llj` gives the same one-line diff.)

`ppjj` is 5F `p p > j j` at `maxjetflavor 4` with equal b and jet cuts, `ppjj5`
the same at 5. Their events agree with each other, as MadGraph's σ does.
`llj` is `p p > e+ e- j` under the banked fixed-scale `llj_card`. The banked MG
layer (§7) is the third check.

## 5. σ against MadGraph

`$S/H-K1-stats.py`, unweighted means over five seeds per side.
`se(scatter)` is sd/√5.

### (a) 5F `p p > j j`, `maxjetflavor 4`, b cuts apart from jet cuts

Run card `$S/vg/rc_ppjj_mjf4diff.dat`: `ptj 20`, `etaj 5`, `drjj 0.4`, `ptb 30`,
`etab 2.5`, `drbb 0.4`, `drbj 0.4`, fixed scales 91.188, LHAPDF 247000. MadGraph
refuses this card as one process (§1), so its reference is three outputs, one per
outgoing class pattern, each of which MadGraph accepts:

- `H-K1-ppjj-A`: `p p > j4 j4`, with `j4` the light partons;
- `H-K1-ppjj-B`: `p p > b j4`, `p p > b~ j4`;
- `H-K1-ppjj-C`: `p p > b b~`, `p p > b b`, `p p > b~ b~`.

Together they hold the same subprocesses as `p p > j j`. The total is A+B+C per
seed:

```
A s1..s5: 0.53087E+09 0.52902E+09 0.52994E+09 0.53030E+09 0.53005E+09
B s1..s5: 0.32150E+07 0.32209E+07 0.32175E+07 0.32089E+07 0.32125E+07
C s1..s5: 0.38463E+06 0.38523E+06 0.38465E+06 0.38508E+06 0.38461E+06
vibegraph  n=5 mean 5.333100e+08 pb sd 3.290e+05 (6.17e-04) se(scatter) 1.471e+05 se(quoted) 2.248e+05 chi2/dof 0.43
           seeds: 5.33620e+08±5.3e+05, 5.33330e+08±5.0e+05, 5.33150e+08±4.7e+05, 5.33610e+08±5.3e+05, 5.32840e+08±4.8e+05
madgraph   n=5 mean 5.336358e+08 pb sd 6.690e+05 (1.25e-03) se(scatter) 2.992e+05 se(quoted) 3.535e+05 chi2/dof 0.72
vg-mg: -3.2580e+05 pb (-6.105e-04); pull -0.98 (scatter), -0.78 (quoted)
```

Known-wrong control: the same card with the b cut as a light jet, MadGraph's
`maxjetflavor 5` row, is 546.15 µb. That is +2.3% from the class-split reference,
or about 40 combined standard errors, so the row can tell the b classes apart. A
vibegraph-high offset of the size of MadEvent's survey bias would not appear here:
vibegraph reads 6e-4 low.

Before the change the card was refused (`vibegraph-before`, seed 1):

```
ERROR failed to decompose into flavour groups: subprocesses g g > u u~ and g g > b b~ share a matrix element but compile to different cut filters, so no single cut indicator covers the sum
```

After: `INFO 13 flavour groups are cut by 3 distinct filters, each group by its own`.

**`p p > j j j` (item 8's process), same card, `--target-rel 0.02`**
(`$S/H-K1-ppjjj.out`):

```
== before   ERROR failed to decompose into flavour groups: subprocesses g g > g u u~ and g g > g b b~ share a matrix element but compile to different cut filters, …
== after2   INFO 28 flavour groups are cut by 4 distinct filters, each group by its own
            σ = 47.79 ± 0.16 µb
```

There is no MadGraph row for `j j j` (§1), only the `j j` analogue above.

### (b) `g g > z j j, z > j j`, 13 TeV

`$S/vg/rc_ggzjj.dat`: the MadGraph output's card with `ptj 20`, `etaj 5`,
`drjj 0.4`, fixed scales 91.188, LHAPDF 247000. MadGraph seeds 41–45, vibegraph
seeds 1–5 at `--target-rel 0.003`. H-D2's banked 149.70 ± 0.29 pb (`nn23lo1`)
is consistent with the LHAPDF value here.

```
### cut_decays = T
vibegraph  n=5 mean 1.106140e+02 pb sd 2.901e-01 (2.62e-03) se(scatter) 1.297e-01 se(quoted) 1.289e-01 chi2/dof 1.00
madgraph   n=5 mean 1.101380e+02 pb sd 2.712e-01 (2.46e-03) se(scatter) 1.213e-01 se(quoted) 1.902e-01 chi2/dof 0.37
vg-mg: +4.7600e-01 pb (+4.322e-03); pull +2.68 (scatter), +2.07 (quoted)
### cut_decays = F
vibegraph  n=5 mean 1.403860e+02 pb sd 4.619e-01 (3.29e-03) se(scatter) 2.066e-01 se(quoted) 1.692e-01 chi2/dof 1.46
madgraph   n=5 mean 1.506120e+02 pb sd 4.147e-01 (2.75e-03) se(scatter) 1.855e-01 se(quoted) 2.563e-01 chi2/dof 0.54
vg-mg: -1.0226e+01 pb (-6.790e-02); pull -36.84 (scatter), -33.29 (quoted)
```

Before, both cards stopped at `groups 0 and 1 compile to different cut filters`.

**The `cut_decays = F` offset is entirely the four same-flavour subprocesses**
(single seeds, `$S/H-K1-*.out`):

| subset | MadGraph (s41) | vibegraph (s1) |
|---|---|---|
| twelve distinct-flavour sets (`ggzdist12`) | 112.29 ± 0.22 | 112.29 ± 0.27 |
| four same-flavour sets (`ggzsame4`) | 38.14 ± 0.13 | 28.52 ± 0.08 |
| `g g > z u u~, z > u u~`, `cut_decays F` | 7.211 ± 0.028 | 5.348 ± 0.015 |
| `g g > z u u~, z > u u~`, `cut_decays T` | 5.283 ± 0.019 | 5.348 ± 0.015 |

vibegraph gives the same-flavour set the same σ under either `cut_decays`. It
exempts a leg only where the leg descends from a forced line in every diagram
(`cuts.rs` module docs), and with identical quarks no leg does. MadEvent exempts
the legs configuration 1 forces (`check_decay`, `setcuts.f:193`). The
distinct-flavour half agrees to 1e-4. The parts add to the whole on both sides:
vibegraph 140.81 against 140.39 ± 0.21, MadGraph 150.43 against 150.61. That
rule is older than this change and outside its scope (Found). The `cut_decays = T`
row's +0.43% is of the size of the window-union effect H-D1 measured
(+0.24% to +1.1%; +1.2% on `u u~ u u~` above).

## 6. Per event

20000 events from each seed-1 artifact (`generate --seed 7`), cut by each
group's own rule (`$S/H-K1-evcheck.py`, `$S/H-K1-evcheck.out`):

```
dijet: events 20000 legs {'j': 39852, 'b': 148} failures {}
  j pT: min 20.0001  j |y|: max 4.9997  b pT: min 30.1978  b |y|: max 2.4362
zjj:   events 20000 legs {'core': 40000, 'decay': 40000} failures {}
  core pT: min 20.0003  mZ: min 54.7703 max 127.6543 (window 91.188 ± 15·2.4414)  decay pT: min 0.1305
same-flavour events 4002: min pT over all four quarks 20.0004; other events 15998: min pT 0.1305
```

The b share of outgoing partons, 148/40000 = 0.37% ± 0.03%, matches MadGraph's
(B + 2C)/(2σ) = 0.374%.

## 7. Tests and mutations

New tests, all in `proton.rs`:

- `subprocesses_cut_differently_are_grouped_apart`: 5F `p p > j j` at
  `maxjetflavor 4` decomposes, with `g g > u u~` and `g g > b b~` in two groups
  whose `|M|²` agree to `GROUP_REL_TOL` and whose cuts differ. No group mixes a b
  and a light quark in one slot. Under equal b and jet cuts the two share a group.
- `a_point_carries_the_terms_of_the_groups_whose_cuts_accept_it`: at a dijet with
  both partons at pT 25 GeV, the full decomposition's value equals the
  light-only decomposition's (1e-12). The b-only one gives 0, no b group records a
  scale, and 200 draws of `select_event` never pick a b group. At 40 GeV the value
  is the light part plus the b part.
- `channels_are_built_under_bounds_every_group_shares`: the integrand's channel
  map keys are the `UnionBounds` keys, not each group's own.

Mutations (`$S/H-K1-mutate.py`, `$S/H-K1-mutate3.py`; outputs
`$S/H-K1-mutate.out`, `$S/H-K1-mutate3.out`):

```
== M1-no-join-condition: exit 101
test proton::tests::subprocesses_cut_differently_are_grouped_apart ... FAILED
test proton::tests::a_point_carries_the_terms_of_the_groups_whose_cuts_accept_it ... FAILED
(panicked at proton.rs:938, the debug_assert that a member shares its group's filter)
== M2-every-group-passes-when-any-does: exit 101
test proton::tests::a_point_carries_the_terms_of_the_groups_whose_cuts_accept_it ... FAILED
the point carries 2.8134616127175287e-4, the light groups alone 2.811258917673959e-4
== M3-channels-under-own-group-cuts: exit 101
test proton::tests::channels_are_built_under_bounds_every_group_shares ... FAILED
```

Before the change, M1's test fails as a refusal (`CutIndicatorDiffers`, §5a). The
σ evidence for M3 is §3: −4.3% on the decay chain.

### Gates on the merge `9601bfe0`

`cargo fmt --all --check`, then `cargo clippy --workspace --all-targets -- -D
warnings` (`$S/H-K1-gates-merged.log`):

```
HEAD 9601bfe09853c59e79f23a9f2914ec17bd12c325
FMT-OK
CLIPPY-EXIT 0
```

`CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 cargo test --workspace` (`$S/H-K1-test-merged.log`):

```
$ head -1 H-K1-test-merged.log; grep TEST-EXIT H-K1-test-merged.log
9601bfe09853c59e79f23a9f2914ec17bd12c325
TEST-EXIT 0
$ grep "test result" H-K1-test-merged.log | awk '{p+=$4; f+=$6; i+=$8} END {print NR" suites, passed "p", failed "f", ignored "i}'
55 suites, passed 1968, failed 0, ignored 25
```

VALIDATE-PLACEHOLDER

## 8. The survey's decay-chain cards

CMS-PLACEHOLDER

## Found

- **The identical-particle decay-product rule under `cut_decays = F`.** On
  `g g > z q q~, z > q q~`, vibegraph exempts no quark from the cuts. MadEvent
  exempts the two configuration 1 forces (`check_decay`, `setcuts.f:193`).
  Evidence: §5b, 7.211 (MadGraph) against 5.348 (vibegraph) on `u u~ u u~`, and
  −6.8% on the whole `g g > z j j, z > j j` card. Every hadronic `z > j j` /
  `w > j j` card with jets in the core reaches it (Zto2Q, Wto2Q). Closing it
  takes reproducing `check_decay`'s configuration-1 exemption per group. With
  per-group indicators that is one filter per pairing, or a filter that reads the
  drawn configuration. The cell to flip is §5b's `cut_decays F` row.
- **A card with no `ŝ` floor panics instead of being refused.** A massless final
  state with a leg left uncut gives `τ_min = 0`, and the first point stops with
  `PDF evaluation point (x=NaN, …)`. This holds on main for `p p > b b~` under
  `sm-no_b_mass` at `ptb = 0` (`vibegraph-before`, `$S/vg/ppbb.dat`). It is newly
  reachable from item 8's literal card (template `ptb = 0`, `etab = −1`),
  which was a clean refusal before and is now this panic (`$S/vg/a2_mjf4.log`).
  MadGraph refuses that card at `setcuts.f:808`, and its σ is collinear-divergent
  anyway. Closing it takes a refusal at integrand construction when the `τ` floor
  is zero.
- **A user question: should vibegraph refuse what MadGraph refuses here?**
  vibegraph now integrates 5F cards whose b and jet cuts differ, which MadGraph
  stops on (§1). The σ is checked against a class-split MadGraph reference (§5a),
  but no single MadGraph run reproduces it. The alternative is MadGraph parity:
  refuse a decomposition whose groups' filters differ only through leg classes.
  That would refuse item 8's card again. I did not choose. The code takes the
  brief's reading.
- **Concepts that still describe the refusals.** `hadronic/flavour-groups.md` and
  `hadronic/proton-integrand.md` describe `CutIndicatorDiffers` and
  `GroupCutsDiffer`, and the one-indicator integrand. For the close-out.
- **MadGraph's first-row rule is a deviation risk wherever MadGraph does not
  refuse.** With `group_subprocesses = False`, or with one matrix element per
  P directory, a b row merged into a light-quark matrix element takes that
  element's classes (`setcuts.f:217`). vibegraph never does this. No banked card
  reaches it; a registered-deviation note would cover it.
