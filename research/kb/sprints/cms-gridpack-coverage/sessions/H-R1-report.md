---
type: Session Report
title: "H-R1 report: the mirrored beam ordering now follows the proc card, as MadGraph's has_mirror_process does"
description: "Subprocess::has_mirror was incoming[0] != incoming[1], so a card written in one beam ordering was integrated in both: u u~ > e+ e- gave 1556.8 pb against MadGraph's 778.1, u g > u z 2877.0 against 1436.5. The enumeration now records, per subprocess, whether its own process line generated the exchanged ordering (MadGraph collects mirrors within one process definition only), and only then does the mirror term get luminosity. After the change the single-ordering rows match MadGraph at +4.8e-4 and +9.6e-4 (pulls +1.1, +1.0), both-orderings/single ratios are 1.99975 ± 0.00079 (MadGraph 2.00010 ± 0.00087) and 2.00166 ± 0.00112 (2.00088 ± 0.00205), every event of a single-ordering card has the card's parton on beam 1, and p p > e+ e- is bit for bit unchanged. Two lines writing the two orderings are now two unmirrored subprocesses (MadGraph's two P directories), not a refusal; at fixed-energy beams they are refused. The massive b b~ row waits for the massive-incoming branch."
status: draft
tags: [proton, beams, mirror, madgraph-parity, sprint-h, report, validation]
verified: [{by: claude-code/claude-opus-5-5, at: 2026-11-04}]
generated: {by: claude-code/claude-opus-5-5, at: 2026-11-04}
sources:
  - {id: mg-collect, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/core/diagram_generation.py#L1835", title: "MadGraph diagram_generation.py generate_multi_amplitudes, the collect_mirror_procs branch"}
  - {id: mg-iface, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/interface/madgraph_interface.py#L3359", title: "MadGraph madgraph_interface.py, one MultiProcess per process line, collect_mirror_procs from group_subprocesses"}
  - {id: mg-mirrorprocs, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L6612", title: "MadGraph export_v4.py write_mirrorprocs"}
  - {id: mg-dsig, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc#L86", title: "MadGraph super_auto_dsig_group_v4.inc, DO IMIRROR=1,2 under MIRRORPROCS"}
  - {id: hm1, resource: "H-M1-report.md", title: "H-M1 report, Found 1"}
---
Session H-R1 of sprint H, item `explicit-initial-state-ordering-counted-twice`.
Worktree `/home/user/wt-mirror`, branch `sprinth-mirror`, verified before any
action (`git rev-parse --show-toplevel` → `/home/user/wt-mirror`,
`git branch --show-current` → `sprinth-mirror`). The branch started at
`sprinth-plan` `b4fa65b`; on the manager's correction it was brought up to
`origin/sprinth-plan` `40244b4` (which contains `origin/main` `58ab186`;
`git merge --no-edit origin/sprinth-plan` fast-forwarded, the work in progress
stashed across it, and `git merge-base --is-ancestor origin/main HEAD` held).
**Every vibegraph measurement below was redone on the merged base**, with a
fresh baseline binary built from it and a fresh fixed binary. The pre-merge
numbers (`H-R1-vg/H-R1-vg-sigma-premerge.txt`) are identical digit for digit to
the post-merge ones, so #86 and #87 do not move these rows; only the post-merge
runs are quoted. MadGraph is the pinned checkout (`b768706`, 3.7.1) at
`/home/user/vibegraph/research/refs/mg5amcnlo`, run with gfortran, `nn23lo1`.

Scratch artefacts, none committed (scratchpad
`/tmp/claude-0/-home-user-vibegraph/0bdd8e0a-85fe-5422-8ff6-63eab43fffee/scratchpad/`,
`$SP` below): `H-R1-mg/` (MadGraph scripts `out_*.mg5`, `launch_*_<seed>.mg5`,
`driver.sh`, outputs, `H-R1-sigma.txt`), `H-R1-vg/` (proc and run cards,
`driver.sh`, logs, `H-R1-vg-sigma.txt`, the binaries `vibegraph-base2` and
`vibegraph-fix2`), `H-R1-stats.py` → `H-R1-stats.out`, `H-R1-beams.py`,
`H-R1-mutate.py` → `H-R1-mutations.log`, `H-R1-gates.sh` → `H-R1-gates.log`,
`H-R1-test.log`, `H-R1-validate.log`. Event files were deleted after their
tallies were recorded.

## 1. MadGraph's rule (pinned source)

- **Mirror collection is per process line.** `madgraph_interface.py:3359`
  builds one `diagram_generation.MultiProcess(myprocdef, collect_mirror_procs=…)`
  per `generate` / `add process` line, with `collect_mirror_procs = True` under
  the default `group_subprocesses = Auto`. Inside it,
  `generate_multi_amplitudes` (`diagram_generation.py:1835-1852`) checks each
  new concrete process against the list of processes *this line* has already
  generated: if `[id2, id1, outgoing…]` is there, it sets
  `has_mirror_process = True` on that amplitude and drops the new one
  (`continue`). The list (`non_permuted_procs`, `amplitudes`) is local to the
  call, so nothing crosses lines.
- **What follows.** `HelasMatrixElement` copies the flag
  (`helas_objects.py:3523`); `write_mirrorprocs` (`export_v4.py:6612`) writes it
  as `MIRRORPROCS`; `super_auto_dsig_group_v4.inc:86-87` runs
  `DO IMIRROR=1,2 / IF(IMIRROR.EQ.1.OR.MIRRORPROCS(IPROC))`, so the exchanged
  ordering is integrated only for a flagged matrix element.
- **Measured on the outputs of this session's cards** (`cat
  SubProcesses/P*/mirrorprocs.inc`):

  | card | P directories | `MIRRORPROCS` |
  |---|---|---|
  | `generate u u~ > e+ e-` | `P1_qq_ll` | `.FALSE.` |
  | `generate u u~ > e+ e-` + `add process u~ u > e+ e-` | `P1_qq_ll`, `P2_qq_ll` | `.FALSE.`, `.FALSE.` |
  | `generate p p > e+ e-` | one | `.TRUE.,.TRUE.` |
  | `generate u g > u z` | one | `.FALSE.` |
  | `generate u g > u z` + `add process g u > u z` | two | `.FALSE.`, `.FALSE.` |

  The two-line `u u~` card's `<init>` lists two processes, IDPRUP 1 and 2:
  `7.781200e+02 … 1` and `7.795200e+02 … 2`.
- **Decay chains.** `DecayChainAmplitude` passes `collect_mirror_procs` to its
  core (`diagram_generation.py:1362, 1387`), so a chain's core carries its
  line's mirror.

## 2. vibegraph before the change

- `generate_sets_inner` (`diagrams/mod.rs`) deduplicated candidates on the
  unordered initial and final content (`subprocess_key`), so `u~ u` of a
  `p p` line was dropped as a duplicate of `u u~`, and
  `Subprocess::has_mirror` (`proton.rs`) restored the dropped ordering for
  *every* unequal pair. A single-ordering line got the same treatment, which is
  the bug.
- The cross-line check (`enumerate`, `diagrams/mod.rs`) compared
  `summed_states` with the initial legs sorted, so two lines writing the two
  orderings were refused as a duplicate.

Measured on the merged base (`vibegraph-base2`), seed 1 shown, all five in
`H-R1-vg-sigma.txt`:

```
$ ./driver.sh $V/vibegraph-base2 base uueesym 1 2 3 4 5   # and ugzsym
ERROR failed to enumerate process: subprocess 'u~ u > e+ e-' is produced by two process lines ('u u~ > e+ e-' and 'u~ u > e+ e-'); both would be added to the cross section, so the card is refused rather than counting it twice
ERROR failed to enumerate process: subprocess 'g u > u Z' is produced by two process lines ('u g > u z' and 'g u > u z'); both would be added to the cross section, so the card is refused rather than counting it twice
```

## 3. What was changed

- `DiagramSet` gains `pub mirrored: bool`: the set's own process line also
  generated it with the two incoming legs exchanged.
- `generate_sets_inner`: a candidate that is a duplicate of an enumerated set
  and whose incoming legs (with their polarizations) are that set's exchanged
  — and differ from it — sets `mirrored` on the set (`is_exchanged_ordering`).
  This is MadGraph's per-line rule: `u u` is never mirrored, `p p` marks every
  unequal pair, a line written in one ordering marks nothing.
- `summed_states`: the beams are kept in order, and a mirrored set contributes
  both orderings. So `u u~ > X` + `add process u~ u > X` is two unmirrored
  subprocesses (one per line, each with its own `@N`, as MadGraph's two P
  directories), while `p p > e+ e-` + `add process u~ u > e+ e-` is still
  refused (the `u~ u` state is counted by both lines).
- The flag is carried through `with_final_order` (the cross-line leg reorder),
  the decay-chain stitch (`chain.rs`, from the core), and the compile-only
  subsets (`onshell.rs`, `renumbering.rs`, `rooting_soundness.rs`, two tests).
- `proton.rs`: `Subprocess` gains `mirrored`, set from its `DiagramSet` in
  `derive_flavor_groups`; `Subprocess::has_mirror` returns it; `BeamSlots.mirrored`
  reads `has_mirror()`. `FlavorGroup::has_mirror` (any over members) and every
  consumer of the member flag (luminosity, rewgt, event records via
  `flavor_records` in `generate.rs`) are unchanged and now read the card's flag.
  Functions touched in `proton.rs`: `Subprocess` (struct, `has_mirror`),
  `derive_flavor_groups` (member and `BeamSlots` construction), doc comments of
  `BeamSlots`, `FlavorGroup::has_mirror`, `FlavorGroup::luminosity`; one test
  literal and one new test. Nothing in the grouping rule or beam kinematics.
- **Fixed-energy beams** (`multiplicity::refuse_exchanged_beam_orderings`,
  called beside `refuse_mixed_multiplicity` in `integrate_fixed_energy` and
  `generate_sample`): two subprocesses that are one initial state in its two
  orderings are refused there, since both would integrate the same collision.
  Before the change the enumeration refused that card for every beam type; the
  refusal now lives where it is true.

### Deviation from the brief, with justification

The brief asks that two lines `u u~ > X` and `u~ u > X` give "one subprocess
with the mirror". MadGraph does not do that: §1 shows two matrix elements, two
P directories, `MIRRORPROCS` false on both, and two IDPRUPs. One mirrored
subprocess would label the exchanged-ordering events with the first line's
`@N`, and it would need the two lines' diagram sets to be proven mirror images
(they may differ in orders, `$`, `/` or polarizations). Two subprocesses, one
per line, is MadGraph's structure and gives the same σ: on the merged base the
two-line card integrates bit for bit to the old one-line value (same digits,
all five seeds, §4), because `|M(u~ u)|²(q)` and `|M(u u~)|²(R q)` agree
exactly for these 2 → 2 amplitudes and the phase-space map is shared.

## 4. σ against local MadGraph, 5 seeds per side, unweighted mean

Settings: 13 TeV, MadGraph `nn23lo1` against vibegraph LHAPDF 247000, fixed
`μR = μF = 91.188`, `maxjetflavor = 4`, `mmll = 20`, `ptl = 0`, `etal = −1`,
`drll = 0`; the `u g > u z` cards take MadGraph's default `ptj = 20`,
`etaj = 5`. vibegraph's run cards are MadGraph's own `Cards/run_card.dat` after
launch, with `pdlabel = lhapdf`, `lhaid = 247000`. MadGraph seeds 11-15, 10 000
events, σ from the banner's `Integrated weight`; vibegraph seeds 1-5, default
0.1% target. The PDF difference is the known ~+0.3% offset class, so the
single/both ratio is the primary oracle.

```
$ python3 $SP/H-R1-stats.py
uuee
  mg   n=5 mean 778.108  sd 0.53 (6.81e-04)  se(scatter) 0.237  se(quoted) 0.254  chi2/dof 0.92
  base n=5 mean 1556.77  sd 0.8876 (5.70e-04)  se(scatter) 0.397  se(quoted) 0.535  chi2/dof 0.49
  fix  n=5 mean 778.483  sd 0.5258 (6.75e-04)  se(scatter) 0.235  se(quoted) 0.28  chi2/dof 0.66
  base-mg: +778.7 (+1.00e+00); pull +1684.28 (scatter), +1315.38 (quoted)
  fix-mg: +0.3747 (+4.82e-04); pull +1.12 (scatter), +0.99 (quoted)
uueesym
  mg   n=5 mean 1556.3  sd 1.085 (6.97e-04)  se(scatter) 0.485  se(quoted) 0.494  chi2/dof 1.01
  base: (none)
  fix  n=5 mean 1556.77  sd 0.8876 (5.70e-04)  se(scatter) 0.397  se(quoted) 0.535  chi2/dof 0.49
  fix-mg: +0.475 (+3.05e-04); pull +0.76 (scatter), +0.65 (quoted)
ppee
  mg   n=5 mean 2805.5  sd 1.306 (4.65e-04)  se(scatter) 0.584  se(quoted) 0.865  chi2/dof 0.45
  base n=5 mean 2807.29  sd 0.8327 (2.97e-04)  se(scatter) 0.372  se(quoted) 0.935  chi2/dof 0.17
  fix  n=5 mean 2807.29  sd 0.8327 (2.97e-04)  se(scatter) 0.372  se(quoted) 0.935  chi2/dof 0.17
  base-mg: +1.787 (+6.37e-04); pull +2.58 (scatter), +1.40 (quoted)
  fix-mg: +1.787 (+6.37e-04); pull +2.58 (scatter), +1.40 (quoted)
ugz
  mg   n=5 mean 1436.47  sd 2.505 (1.74e-03)  se(scatter) 1.12  se(quoted) 1.04  chi2/dof 1.31
  base n=5 mean 2876.97  sd 2.961 (1.03e-03)  se(scatter) 1.32  se(quoted) 1.21  chi2/dof 1.17
  fix  n=5 mean 1437.85  sd 1.719 (1.20e-03)  se(scatter) 0.769  se(quoted) 0.568  chi2/dof 1.96
  base-mg: +1441 (+1.00e+00); pull +830.41 (scatter), +902.36 (quoted)
  fix-mg: +1.378 (+9.59e-04); pull +1.01 (scatter), +1.16 (quoted)
ugzsym
  mg   n=5 mean 2874.2  sd 4.247 (1.48e-03)  se(scatter) 1.9  se(quoted) 1.97  chi2/dof 0.95
  base: (none)
  fix  n=5 mean 2878.08  sd 1.038 (3.61e-04)  se(scatter) 0.464  se(quoted) 1.18  chi2/dof 0.15
  fix-mg: +3.886 (+1.35e-03); pull +1.99 (scatter), +1.69 (quoted)
ratio uueesym/uuee mg: 2.00010 +- 0.00087
ratio uueesym/uuee fix: 1.99975 +- 0.00079
ratio ugzsym/ugz mg: 2.00088 +- 0.00205
ratio ugzsym/ugz fix: 2.00166 +- 0.00112
```

Per-seed values (`H-R1-mg/H-R1-sigma.txt`, `H-R1-vg/H-R1-vg-sigma.txt`), pb:

```
uuee     MG 778.44 778.68 777.30 778.18 777.94
         base 1556.594019 1555.391635 1557.459707 1557.632781 1556.776758
         fix  778.706513 777.732686 778.940887 778.891593 778.141771
uueesym  MG 1557.64 1556.32 1554.69 1556.03 1556.80
         base refused ×5
         fix  1556.594019 1555.391635 1557.459707 1557.632781 1556.776758
ppee     MG 2806.2 2805.7 2803.2 2806.3 2806.1
         base 2806.384813 2807.087373 2808.178353 2808.135838 2806.650893
         fix  2806.384813 2807.087373 2808.178353 2808.135838 2806.650893
ugz      MG 1438.39 1438.28 1432.25 1436.39 1437.04
         base 2872.508140 2877.236645 2876.753010 2880.805082 2877.558119
         fix  1436.741785 1439.163368 1436.001032 1440.100024 1437.233351
ugzsym   MG 2876.78 2879.73 2869.58 2874.38 2870.52
         base refused ×5
         fix  2878.059696 2879.251915 2876.921715 2877.201048 2878.987934
```

Reading:

- **Fails before.** The single-ordering cards were exactly ×2 (`uuee` base/MG
  = 2.0007, `ugz` 2.0028), and the both-orderings cards were refused.
- **After.** Single-ordering rows +4.8e-4 and +9.6e-4 against MadGraph (pulls
  +1.1, +1.0 on the scatter error), both-orderings rows +3.1e-4 and +1.35e-3
  (+0.8, +2.0). The ratios agree with MadGraph's to 1.8e-4 and 3.9e-4, within
  their combined errors (0.3σ and 0.3σ). The `ugzsym` vibegraph χ²/dof is 0.15
  and its 5-seed spread is 4× below MadGraph's; that row's +2.0 pull is read on
  MadGraph's spread, and five seeds do not calibrate a 1e-3 statement. None of
  these rows is a gate.
- **Control.** `p p > e+ e-` is the same digits on both binaries for all five
  seeds (§6 for the events). Its +6.4e-4 against MadGraph is unchanged by this
  session.

## 5. Per-event: which parton is on beam 1

`$SP/H-R1-beams.py` tallies `(beam-1 PDG, beam-2 PDG)` per event, beam 1 being
the incoming line with `pz > 0`. 10 000 events each; vibegraph `generate`
seed 1 on each card's seed-1 grid.

```
MG uuee: 10000 events; (beam1, beam2): (2, -2): 10000
MG ugz: 10000 events; (beam1, beam2): (2, 21): 10000
MG uueesym: 10000 events; (beam1, beam2): (-2, 2): 5033, (2, -2): 4967
MG ugzsym: 10000 events; (beam1, beam2): (2, 21): 4989, (21, 2): 5011
VG base uuee: 10000 events; (beam1, beam2): (-2, 2): 4969, (2, -2): 5031
VG base ugz: 10000 events; (beam1, beam2): (2, 21): 5022, (21, 2): 4978
VG fix uuee: 10000 events; (beam1, beam2): (2, -2): 10000
VG fix ugz: 10000 events; (beam1, beam2): (2, 21): 10000
VG fix uueesym: 10000 events; (beam1, beam2): (-2, 2): 4986, (2, -2): 5014
VG fix ugzsym: 10000 events; (beam1, beam2): (2, 21): 4995, (21, 2): 5005
```

On the two-line cards each ordering carries its own line's IDPRUP, as in
MadGraph (`(IDPRUP, beam-1 PDG)` counts):

```
fix_uueesym.lhe (IDPRUP, beam1): {(1, 2): 5014, (2, -2): 4986}
fix_ugzsym.lhe (IDPRUP, beam1): {(1, 2): 4995, (2, 21): 5005}
```

## 6. Multiparticle control, bit for bit

`p p > e+ e-`, seed 1 grid, `generate --seed 1 --nevents 10000` on both
binaries:

```
$ diff base_ppee.lhe fix_ppee.lhe
5c5
< artifact run_base_ppee_1/grid.bin.zst
---
> artifact run_fix_ppee_1/grid.bin.zst
```

Only the artifact path line differs. The σ values match to every printed digit
on all five seeds (§4).

## 7. Fixed-energy beams

```
base2 ee exit=0 σ = 0.428161 ± 0.000166 pb
base2 eesym exit=1 ERROR failed to enumerate process: subprocess 'e- e+ > mu- mu+' is produced by two process lines ('e+ e- > mu+ mu-' and 'e- e+ > mu- mu+'); both would be added to the cross section, so the card is refused rather than counting it twice
fix2 ee exit=0 σ = 0.428161 ± 0.000166 pb
fix2 eesym exit=1 ERROR 'e+ e- > mu+ mu-' and 'e- e+ > mu- mu+' are one initial state in its two beam orderings; at fixed-energy beams both would integrate the same collision, so the card is refused rather than counting it twice
```

(`lpp1 = lpp2 = 0`, `ebeam = 250`; `eesym` is `generate e+ e- > mu+ mu-` +
`add process e- e+ > mu- mu+`.)

## 8. Tests, and mutation evidence

New tests:

- `proton::tests::mirror_luminosity_follows_the_orderings_the_card_generated`
  (lib): on synthetic flavour rows, a single-ordering card's member has
  `[f1[a]·f2[b], 0]` and no group mirror; each line of a two-line card has its
  own direct term and no mirror; every `p p > e+ e-` member has
  `[f1[a]·f2[b], f1[b]·f2[a]]`.
- `tests/diagrams.rs::a_subprocess_has_a_mirror_only_where_its_line_generated_both_orderings`:
  the flag on single-ordering lines (including `u u > u u` and a decay chain),
  on multiparticle lines (`p p`, a `define`d label, `p p > z, z > e+ e-`, a line
  with `u u`, `u g`, `g g`), and the two-line cards giving two unmirrored sets.
- `tests/diagrams.rs::two_lines_in_exchanged_orderings_are_refused_at_fixed_energy`.
- `a_subprocess_reached_from_two_lines_is_refused` gains
  `p p > e+ e-` + `add process u~ u > e+ e-` and a three-line card repeating
  `u u~`; its old lepton exchanged-ordering card became a same-ordering one
  (`e+ e- > mu+ mu-` + `e+ e- > mu- mu+`), since the exchanged one is now a
  fixed-energy refusal, tested above.

One existing test changed its card:
`proton::tests::a_polarized_group_is_evaluated_in_the_partonic_centre_of_mass`
enumerated the explicit `u u~ > z{0} g QCD=1 QED=1` and built its oracle with
the mirrored ordering added by hand. Under the change that card has no mirror,
and the first `cargo test --workspace` run failed exactly there
(`$SP/H-R1-test-run1.log`):

```
163 points: the integrand is 5.223e-1 from the centre-of-mass assembly and 2.951e0 from the laboratory one
thread 'proton::tests::a_polarized_group_is_evaluated_in_the_partonic_centre_of_mass' panicked at vibegraph-lib/src/proton.rs:4954:9:
centre-of-mass assembly off by 5.223e-1
test result: FAILED. 1196 passed; 1 failed; 13 ignored; 0 measured; 0 filtered out; finished in 24.50s
```

That is the bug seen from the other side: the oracle assumed a single-ordering
card is integrated in both orderings. The test now enumerates
`define q = u u~` / `generate q q > z{0} g QCD=1 QED=1`, which generates both
orderings, asserts `g.has_mirror()`, and keeps its oracle and its frame check
unchanged; it passes.

The pre-change code cannot compile these tests (no `mirrored` field), so the
"fails before" evidence is the mutation sweep, each mutation reverting one
piece of the change (`python3 $SP/H-R1-mutate.py`, log
`$SP/H-R1-mutations.log`, the tree restored after each):

```
M1 has_mirror back to incoming[0] != incoming[1] | --lib mirror_luminosity_follows | exit=101 | mirror_luminosity_follows_the_orderings_the_card_generated FAILED
M1                                               | --test diagrams | exit=0
M2 enumeration never records the mirror  | --lib | exit=101 | mirror_luminosity_follows… FAILED
M2                                       | --test diagrams | exit=101 | a_subprocess_has_a_mirror_only…, a_subprocess_reached_from_two_lines_is_refused FAILED
M3 cross-line check sorts the beams again | --lib | exit=101 | mirror_luminosity_follows… FAILED
M3                                        | --test diagrams | exit=101 | a_subprocess_has_a_mirror_only…, two_lines_in_exchanged_orderings_are_refused_at_fixed_energy FAILED
M4 cross-line check ignores the mirror   | --lib | exit=0
M4                                       | --test diagrams | exit=101 | a_subprocess_reached_from_two_lines_is_refused FAILED
M5 decay chain drops the core's mirror   | --lib | exit=0
M5                                       | --test diagrams | exit=101 | a_subprocess_has_a_mirror_only… FAILED
M6 fixed-energy refusal never fires      | --lib | exit=0
M6                                       | --test diagrams | exit=101 | two_lines_in_exchanged_orderings_are_refused_at_fixed_energy FAILED
```

Every mutation is caught by at least one test. M1 is exactly the bug.

**What these tests cannot see.** They pin the flag and the luminosity it
selects, not σ: a wrong `|M|²` on the mirror argument would pass them (the
existing `the_mirrored_beam_ordering_needs_the_reflected_matrix_element` covers
that). The σ rows in §4 are scratch measurements, not gates.

## 9. Gates

All on the final tree, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2`, driven by `$SP/H-R1-gates.sh`
(log `$SP/H-R1-gates.log`):

```
== fmt          cargo fmt --all --check                                   fmt exit=0
== clippy       cargo clippy --workspace --all-targets -- -D warnings     clippy exit=0
== test         cargo test --workspace --no-fail-fast                     test exit=0
                (H-R1-test.log: 51 targets, 1813 passed, 0 failed, 25 ignored)
== validate     bash validation/validate.sh                               validate exit=0
                (H-R1-validate.log: 81 test-result lines, 2264 passed, 0 failed, 110 ignored)
```

The collated report's verdict (`target/validation-report/report.md`):

```
77 rows × 4 categories = 308 cells: 218 measured in the layers this run drove (213 ✅, 5 ⚠️, 14 ⏳, 76 — / uncovered).
The measured cells are exactly the cells the manifest declares, every gate cell
passed, and every measurement agrees with the manifest about what it is.
```

The five ⚠️ cells are informational by the manifest ("measured and
deliberately not enforced"): the diagram census of `gg_to_gg` and
`gg_to_gg_cg`, the amplitude cells of `ee_to_wpwm_cw` and `ee_to_zh_smeft`, and
the samples cell of `ee_to_mumua`. None is a proton-beam row, so none reads
the mirror flag. No banked row's card is a single-ordering proton card (the
proton rows are all `p p > …`; the explicit-initial-state rows are at fixed
energy), so no banked cell was expected to move, and none failed. A
before-the-change `validate.sh` was not run, so "unchanged" for the ⚠️ cells
rests on their being informational, not on a side-by-side comparison.

## Backlog item status

`closes_when` asks for σ rows "massless and massive". The massless rows hold
(§4); the massive `b b~` row cannot be run on this branch, which refuses a
massive incoming leg (`ProtonError::MassiveInitialState`; `b` is 4.7 GeV in
`sm`) until `sprinth-massive-incoming` lands. The item file is therefore **not
deleted**: once both branches are on `main`, rerun H-M1's explicit
`generate b b~ > mu+ mu-` card (MadGraph 40.79 pb) and the explicit
`g b > z b` card (497.2 pb), and delete it then.

## Found

1. **The massive single-ordering rows are unmeasured after this fix.**
   Evidence: §"Backlog item status". To close: after `sprinth-massive-incoming`
   and this branch merge, rerun `generate b b~ > mu+ mu-` and `generate g b > z b`
   against H-M1's MadGraph numbers (40.79 pb, 497.2 pb), 5 seeds each.
   H-M1's `pbzb` vibegraph proc card is `generate g b > z b` while its MadGraph
   card is `define p = g b; generate p p > z b`: after this change those two
   cards no longer describe the same σ, so H-M1's §5 `g b + b g` row must be
   rerun with `define p = g b` / `generate p p > z b` on the vibegraph side too,
   and its `bbsym`/`bb0sym`/`ccsym` vibegraph cards (`generate b b~ > mu+ mu-`
   alone) need the `add process b~ b > mu+ mu-` line to stay symmetric.
2. **A polarized incoming leg and MadGraph's mirror key.** MadGraph's mirror
   check (`diagram_generation.py:1838`) compares PDG ids only, so on a line with
   a polarized beam leg (`p{L} p > …`) it would mirror `u{L} u~` onto
   `u~ u{L}`-shaped candidates that are a different physical state.
   vibegraph's key includes the polarizations, so it does not. Not measured,
   and whether vibegraph accepts a polarized beam leg at proton beams at all was
   not checked. To close: run such a card on both sides, or confirm it is
   refused.
3. **Two lines writing the two orderings make two groups where one mirrored
   group would do.** Correct and MadGraph's structure, but it doubles the
   compiled evaluators and channels of such cards (the log shows
   `4 (group, diagram) channels` for `uueesym` against 2 for `uuee`). Only a
   cost, and only if such cards become common.
