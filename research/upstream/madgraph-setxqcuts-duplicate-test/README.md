# MadEvent: `setxqcuts` tests the same daughter twice

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

MadEvent's xqcut grid hint for a final-state (jet, non-jet) pair depends on the
order in which the process lists its legs:
- `u u~ > g b b~` gives its g–b pairs the hint (20).
- `u u~ > b b~ g`, the same physics with the legs in another order, gives them 0.

## Status

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran). It is also present in
3.7.1. It affects the
integration grids only. The report draft is [`report-draft.md`](report-draft.md).

## Where

`Template/LO/SubProcesses/setcuts.f`, subroutine `setxqcuts`: `:948-951` in
3.8.1, `:939-942` in 3.7.1, and `:946-949` in 3.5.17.

```fortran
                  if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21.or.
     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21)then
```

## Root cause

- *Verified* (the runs below):
  - The jet test is written twice for `iforest(2,k,j)` and never for
    `iforest(1,k,j)`.
  - The generated `configs.inc` stores the higher leg number first. For
    `u u~ > b b~ g`, the g–b vertices are `IFOREST(·,-1,2) = 5,3` and
    `IFOREST(·,-1,3) = 5,4`, with the gluon (leg 5) in `iforest(1)`.
  - So a (jet, non-jet) pair gets the hint only when the jet has the lower
    leg number.
  - Testing `iforest(1,k,j)` in the second pair restores the hint (`bbg.fixed`).
- *Inferred from source:*
  - The intended condition is "either daughter is a jet". That matches
    `setclscales`'s `xqcut` test on vertices with a jet daughter
    (`reweight.f:1066-1085`).
  - The hint is used only as the s-channel minimum in `myamp.f`
    (`xm(i) = max(xm(i), xqcutij(l1,l2))`).

## The test

The cards are the defaults plus `ickkw = 1` and `xqcut = 20`. At the default
`maxjetflavor = 4` the `b` quarks are not jets and the gluon is. The first
`passcuts` call prints the `xqcutij` table (row `j` lists `xqcutij(i,j)` for
`i = 3..5`) to each channel's `G*/run1_app.log`.

| run | process | role |
|---|---|---|
| `gbb` | `u u~ > g b b~` | control |
| `bbg` | `u u~ > b b~ g` | the defect |
| `bbg.fixed` | `u u~ > b b~ g` | the fix applied to the generated `SubProcesses/setcuts.f` |

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.5.17 (latest LTS release, 2026-10-09) | `git clone --depth 1 --branch v3.5.17 https://github.com/mg5amcnlo/mg5amcnlo mg-3.5.17` | `4492b3e3` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

3.7.1 and 3.8.1 ran on 2026-10-16 and 3.5.17 on 2026-10-17 (Linux x86-64),
with Python 3.11 and gfortran from vibegraph's `madgraph` pixi environment (conda-forge). Exact commands, from
this directory, with `$MG` the directory holding the clones:

```
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/sc
pixi run -e madgraph env MG5="python $MG/mg-3.5.17/bin/mg5_aMC" ./repro.sh /tmp/sc
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/sc
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/sc`.


It takes about two minutes per version.

## Expected against actual

**Expected:** the gluon–quark pairs carry 20 in every run.

```
== 3.7.1
== gbb  (legs 3 4 5 = g b b~; sigma 5.464680e+03 +- 5.032902e+01 pb)
xqcutij # 3>     0.0    20.0    20.0
xqcutij # 4>    20.0     0.0     0.0
  xqcutij(4,3) = 20.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 0.0
== bbg  (legs 3 4 5 = b b~ g; sigma 5.500190e+03 +- 5.099376e+01 pb)
xqcutij # 3>     0.0     0.0     0.0
xqcutij # 4>     0.0     0.0     0.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 0.0  xqcutij(5,4) = 0.0
== bbg.fixed  (legs 3 4 5 = b b~ g, fixed; sigma 5.500500e+03 +- 5.154950e+01 pb)
xqcutij # 3>     0.0     0.0    20.0
xqcutij # 4>     0.0     0.0    20.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 20.0
expected: the gluon-quark pairs carry xqcut = 20 in every run
  gbb: legs g b b~ -> xqcutij(4,3) = xqcutij(5,3) = 20, xqcutij(5,4) = 0
  bbg: legs b b~ g -> xqcutij(5,3) = xqcutij(5,4) = 20, xqcutij(4,3) = 0
and sigma the same in all three runs (the table only shapes the grids)
== 3.5.17
== gbb  (legs 3 4 5 = g b b~; sigma 5.551880e+03 +- 4.605996e+01 pb)
xqcutij # 3>     0.0    20.0    20.0
xqcutij # 4>    20.0     0.0     0.0
  xqcutij(4,3) = 20.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 0.0
== bbg  (legs 3 4 5 = b b~ g; sigma 5.558220e+03 +- 4.417230e+01 pb)
xqcutij # 3>     0.0     0.0     0.0
xqcutij # 4>     0.0     0.0     0.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 0.0  xqcutij(5,4) = 0.0
== bbg.fixed  (legs 3 4 5 = b b~ g, fixed; sigma 5.564170e+03 +- 4.573061e+01 pb)
xqcutij # 3>     0.0     0.0    20.0
xqcutij # 4>     0.0     0.0    20.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 20.0
expected: the gluon-quark pairs carry xqcut = 20 in every run
  gbb: legs g b b~ -> xqcutij(4,3) = xqcutij(5,3) = 20, xqcutij(5,4) = 0
  bbg: legs b b~ g -> xqcutij(5,3) = xqcutij(5,4) = 20, xqcutij(4,3) = 0
and sigma the same in all three runs (the table only shapes the grids)
== 3.8.1
== gbb  (legs 3 4 5 = g b b~; sigma 5.481800e+03 +- 5.061572e+01 pb)
xqcutij # 3>     0.0    20.0    20.0
xqcutij # 4>    20.0     0.0     0.0
  xqcutij(4,3) = 20.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 0.0
== bbg  (legs 3 4 5 = b b~ g; sigma 5.501640e+03 +- 5.115608e+01 pb)
xqcutij # 3>     0.0     0.0     0.0
xqcutij # 4>     0.0     0.0     0.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 0.0  xqcutij(5,4) = 0.0
== bbg.fixed  (legs 3 4 5 = b b~ g, fixed; sigma 5.495570e+03 +- 5.157480e+01 pb)
xqcutij # 3>     0.0     0.0    20.0
xqcutij # 4>     0.0     0.0    20.0
  xqcutij(4,3) = 0.0  xqcutij(5,3) = 20.0  xqcutij(5,4) = 20.0
expected: the gluon-quark pairs carry xqcut = 20 in every run
  gbb: legs g b b~ -> xqcutij(4,3) = xqcutij(5,3) = 20, xqcutij(5,4) = 0
  bbg: legs b b~ g -> xqcutij(5,3) = xqcutij(5,4) = 20, xqcutij(4,3) = 0
and sigma the same in all three runs (the table only shapes the grids)
```

**Actual**, on all three versions: `bbg` has 0 for both g–b pairs, and `bbg.fixed`
has 20. σ agrees across the three runs within its 0.9 % error, as expected for
a grid hint. No σ effect has been looked for. Here, by reading only, the kT cut
on the gluon already implies m(b g) ≳ xqcut.
