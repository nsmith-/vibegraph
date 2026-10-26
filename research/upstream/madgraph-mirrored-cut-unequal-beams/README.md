# MadEvent: at unequal beam energies the mirrored beam ordering is cut in the wrong frame

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

With `ebeam1 ≠ ebeam2`, grouped MadEvent cuts a subprocess's mirrored beam
ordering (`IMIRROR = 2`) on the phase-space point it sampled, not on the event
it writes. The two differ by `ln(ebeam1/ebeam2)` in every rapidity. On
`p p > e+ e-` at 6500 × 4000 GeV with the default `etal = 2.5`, a mirrored
event (the antiquark on the `+z` beam) is accepted when its leptons lie in
`−2.014 < η < 2.986`, not `|η| < 2.5`:
- **Per event:** the written sample carries leptons up to `η = 2.986`, all of
  them on mirrored events.
- **Cross section:** σ depends on which beam is called beam 1. Exchanging the
  two energies is a reflection of the same collision, so σ should not change.

## Status

**Not filed.** Filing is the user's step.

**Present in 3.7.1 and in the 3.5.17 LTS release; fixed in 3.8.1.** All three
are *read*, from source at the commits below; the reproducer has not been run
on any of them, because the session that wrote it had no MadGraph install. The
3.7.1 measurement below comes from this project's banked MadGraph run and a
vibegraph emulation of the defect, not from `repro.sh`.

The report draft, [`report-draft.md`](report-draft.md), asks for the 3.8.1 fix
on the LTS line.

## Where

**3.7.1** (`b7687064`):
- `Template/LO/Source/dsample.f:181`: `passcuts(p, VECSIZE_USED)` once per
  phase-space point, before `select_grouping` chooses `IMIRROR` (`:222`).
  `genps.f:345` has set `cm_rap = ½ ln(xbk(1)·ebeam(1) / (xbk(2)·ebeam(2)))`
  for the sampled point; `rap()` adds it (`kin_functions.f:131`) and
  `cuts.f:426` cuts on `|rap|`.
- `madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc:815-829`: for
  `IMIRROR = 2` the momenta are rotated by π about x, `IB = (2,1)`, `XBK`
  swapped and `CM_RAP = −CM_RAP`, after the cut. The `PASSCUTS` call at `:835`
  is commented out, and `export_v4.py:5103` sets `passcuts_begin = ""`.
- `unwgt.f:623-660` writes the mirrored event at its own boost,
  `½ ln(x₂·ebeam(1) / (x₁·ebeam(2)))`.

**3.5.17** (`4492b3e3`): `DSIGPROC` rotates the momenta and sets
`CM_RAP = −CM_RAP` (`super_auto_dsig_group_v4.inc:452-468`), then cuts them
(`IF (PASSCUTS(P1))`, `:472`). The mirrored event's own `cm_rap` is
`ln(ebeam(1)/ebeam(2)) − CM_RAP`, so the cut sees each rapidity reflected and
displaced by `ln(ebeam(1)/ebeam(2))`: the same window as 3.7.1.

**3.8.1** (`a5b17913`): `PASSCUTS` (`cuts.f:24-98`) evaluates the cuts once per
orientation when `MIRROR_CUTS.AND.EBEAM(1).NE.EBEAM(2)`. The mirrored one is
evaluated on the rotated momenta with `XBK` swapped and
`CM_RAP = DLOG(EBEAM(1)/EBEAM(2)) − CM_RAP`, and each orientation's result is
kept in `CUTS_ORIENT`. `DSIG` zeroes an orientation whose cut failed
(`super_auto_dsig_group_v4.inc:94`, `:599`). That is the event's own frame.

## Root cause

- *Verified* (by measurement, against this project's banked 3.7.1 run): a
  vibegraph integrand that cuts the mirrored term on MadEvent's sampled
  configuration reproduces MadGraph's σ to 0.05 % (below); with each ordering
  cut on its own lab frame it is 5.3 % above.
- *Inferred from source:* the 3.5.17 path gives the same window; 3.8.1 is
  fixed. At equal beam energies the frames coincide and no card is affected.
  The effect needs a mirrored group, unequal beam energies and a cut that is not
  invariant under boosts along z (`eta*`, `etaonium`, rapidity cuts, the jet
  cuts in `η`).

## The test

`p p > e+ e-` with the default cards (`ptl = 10`, `etal = 2.5`,
`drll = 0.4`, the default dynamical scale and PDF), `nevents = 10000`,
`iseed = 1`:

| run | `ebeam1, ebeam2` | role |
|---|---|---|
| `ab` | 6500, 4000 | the defect |
| `ba` | 4000, 6500 | the same collision reflected in z: σ must agree with `ab` |

The script prints σ and, for each ordering (which parton sits on the `+z`
beam), the events with an electron or positron at `|η| > 2.5` and the largest
`|η|`.

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.5.17 (latest LTS release, 2026-10-09) | `git clone --depth 1 --branch v3.5.17 https://github.com/mg5amcnlo/mg5amcnlo mg-3.5.17` | `4492b3e3` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

None has been run yet. Exact commands, from this directory, with `$MG` the
directory holding the clones:

```
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/mirror
pixi run -e madgraph env MG5="python $MG/mg-3.5.17/bin/mg5_aMC" ./repro.sh /tmp/mirror
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/mirror
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/mirror`.

## Expected against actual

**Expected:** no event with `|η(e)| > 2.5`, and σ(`ab`) = σ(`ba`).

**Predicted** for 3.7.1 and 3.5.17, until the script is run:
- `ab`: mirrored (antiquark on `+z`) events with `|η(e)| > 2.5`, all at
  `2.5 < η ≤ 2.986`; none among the direct ones.
- `ba`: the same at `−2.986 ≤ η < −2.5`.
- σ(`ab`) about 10 % below σ(`ba`).

**Predicted** for 3.8.1: no event outside the cut, and σ(`ab`) = σ(`ba`).

**Measured on 3.7.1** by this project, on the same process with
`NNPDF23_lo_as_0130_qed` and both scales fixed at `m_Z` (the `dy13` card with
`ebeam2 = 4000`), not with `repro.sh`:

| | σ at 6500 × 4000 (pb) |
|---|---|
| MadGraph 3.7.1, seed record | 717.25 ± 0.145 |
| vibegraph, each ordering cut on its own lab frame | 754.96 ± 0.35 |
| vibegraph, the mirrored ordering cut as MadEvent cuts it | 717.575 ± 0.309 |

vibegraph gives 755.16 ± 0.36 at 4000 × 6500, unchanged under the exchange, and
792.28 ± 0.42 with MadEvent's cut emulated. That is the prediction for
MadGraph 3.7.1's σ(`ba`) on that card.
