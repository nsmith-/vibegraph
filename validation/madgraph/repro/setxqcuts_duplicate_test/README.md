# MadEvent: `setxqcuts` tests the same daughter twice

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from this repository.

## The defect

`setxqcuts` (`Template/LO/SubProcesses/setcuts.f`) fills `xqcutij`: the xqcut
grid hint for each final-state pair joined at an s-channel vertex. A pair gets
the hint when the vertex has a jet daughter.

| version | lines |
|---|---|
| 3.7.1 | `setcuts.f:939-942` |
| 3.8.1 | `setcuts.f:948-951` |

```fortran
                  if(iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21.or.
     $                 iabs(idup(iforest(2,k,j),1,1)).le.maxjetflavor.or.
     $                 idup(iforest(2,k,j),1,1).eq.21)then
```

The test is written twice for `iforest(2,k,j)` and never for `iforest(1,k,j)`.
- MadGraph stores the higher leg number first.
- A (jet, non-jet) pair therefore gets the hint only when the jet has the lower
  leg number.
- The same physics written in another leg order gets different grids.

The hint feeds `myamp.f`'s s-channel minimum (`xm(i) = max(xm(i), xqcutij(l1,l2))`).

The fix reads `iforest(1,k,j)` in the second pair of tests. That matches
`setclscales`, which applies `xqcut` to every clustering vertex with a jet
daughter (`reweight.f:1066-1085`).

## The test

There are two leg orderings of one process, `u u~ > g b b~` and
`u u~ > b b~ g`. The cards are the defaults plus `ickkw = 1` and `xqcut = 20`.
At the default `maxjetflavor = 4` the `b` quarks are not jets and the gluon is.
The first call to `passcuts` prints the `xqcutij` table to each channel's
`G*/run1_app.log`.

| run | what it is |
|---|---|
| `gbb` | `u u~ > g b b~` |
| `bbg` | `u u~ > b b~ g` (the defect) |
| `bbg.fixed` | `bbg` with the fix applied to the generated `SubProcesses/setcuts.f` |

```
MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/sc
```

It takes about two minutes.

## Results

These runs used the conda toolchain from `pixi run -e madgraph`.
- **3.7.1** is tag `v3.7.1`, commit `b7687064`.
- **3.8.1** is tag `v3.8.1`, commit `a5b17913`, the latest release on 2026-10-16.

```
##### 3.7.1
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
##### 3.8.1
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

- In `gbb`, the g–b and g–b~ pairs, `(4,3)` and `(5,3)`, carry 20.
- In `bbg`, the same pairs, now `(5,3)` and `(5,4)`, carry 0.
- The fix restores 20 in `bbg.fixed`.

σ is the same in all three runs within its 0.9 % quoted error. That is expected:
the table only shapes the integration grids. Whether a missing hint can change
a cross section somewhere has not been measured. Here, the kT cut on the
gluon already implies m(b g) ≳ xqcut by reading (m² ≥ min(p_T²)·ΔR²); that is not measured.
