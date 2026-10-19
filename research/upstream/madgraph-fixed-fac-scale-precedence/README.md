# MadEvent: `scalefact` is skipped for beam 2 when only beam 1's μF is fixed

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

With `fixed_fac_scale1 = T`, `fixed_fac_scale2 = F` and a clustering scale
(`dynamical_scale_choice = -1`, the default), MadEvent does not multiply beam
2's factorisation scale by `scalefact`. On `p p > z` with `scalefact = 2`:
- **Per event:** SCALUP is √ŝ on every event, where 2 √ŝ is expected.
- **Cross section:** σ comes out 10.4 % below what the one-line fix gives.

## Status

**Filed** as [mg5amcnlo/mg5amcnlo#454](https://github.com/mg5amcnlo/mg5amcnlo/issues/454) on 2026-10-19.

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran). It is also present in
3.7.1. The report draft is
[`report-draft.md`](report-draft.md).

## Where

`Template/LO/SubProcesses/reweight.f`, in `setclscales`: line 1138 in 3.7.1
and 3.8.1, and line 1117 in 3.5.17.

```fortran
      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
         if(.not.fixed_fac_scale1) q2fact(1)=scalefact**2*q2fact(1)
         if(.not.fixed_fac_scale2) q2fact(2)=scalefact**2*q2fact(2)
         if (.not.keepq2bck)then
            if(.not.fixed_fac_scale1) q2bck(1)=q2fact(1)
            if(.not.fixed_fac_scale2) q2bck(2)=q2fact(2)
         endif
```

## Root cause

- *Verified* (the `tf_fixed` run below):
  - `.not.` binds tighter than `.or.`, so the guard is
    `(.not.fixed_fac_scale1) .or. fixed_fac_scale2`.
  - That guard is false only for `(T, F)`. The block is then skipped:
    `q2fact(2)` keeps the unscaled clustering scale and `q2bck(2)` is not set.
  - Writing the guard as `.not.fixed_fac_scale1.or..not.fixed_fac_scale2`
    gives SCALUP = 2 √ŝ. That is the form the same file's vectorised path uses
    at `:1903`.
- *Inferred from source:* the block is reached at `ickkw = 0` and at
  `ickkw = 1`. Only `ickkw = 0` was run here.

## The test

The process is `p p > z`. With 2 → 1, the clustering scale is √ŝ. The run card
is the default with these changes:
- `dynamical_scale_choice = -1`;
- `scalefact = 2`;
- `dsqrt_q2fact1 = dsqrt_q2fact2 = 10`;
- `nevents = 1000`, `iseed = 1`.

The fixed beam's scale is therefore below the other, and
SCALUP = `sqrt(max(q2fact(1), q2fact(2)))` is the dynamical beam's scale.

| run | `fixed_fac_scale1,2` | role |
|---|---|---|
| `tf` | T, F | the defect |
| `ft` | F, T | control |
| `ff` | F, F | control |
| `tf_fixed` | T, F | `tf` with the fix applied to the generated `SubProcesses/reweight.f` |

The script prints the range of SCALUP/√ŝ over all events (√ŝ from the two
incoming partons) and σ.

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
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/ffs
pixi run -e madgraph env MG5="python $MG/mg-3.5.17/bin/mg5_aMC" ./repro.sh /tmp/ffs
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/ffs
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/ffs`.


It takes about one minute per version.

## Expected against actual

**Expected:** SCALUP/√ŝ = 2 in every run, and σ(`tf`) = σ(`tf_fixed`).

```
== 3.7.1
run       fixed_fac_scale1,2    events     SCALUP/sqrt(s_hat)             sigma [pb]
tf        T,F                     1000      1.000000-1.000000 3.831000e+04+-5.461400e+01
ft        F,T                     1000      2.000000-2.000000 2.934600e+04+-3.218000e+01
ff        F,F                     1000      2.000000-2.000000 4.691600e+04+-5.812200e+01
tf_fixed  T,F                     1000      2.000000-2.000000 4.228400e+04+-6.175700e+01
expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)
== 3.5.17
run       fixed_fac_scale1,2    events     SCALUP/sqrt(s_hat)             sigma [pb]
tf        T,F                     1000      1.000000-1.000000 3.824600e+04+-1.358800e+02
ft        F,T                     1000      2.000000-2.000000 2.930800e+04+-4.728900e+01
ff        F,F                     1000      2.000000-2.000000 4.684000e+04+-1.034700e+02
tf_fixed  T,F                     1000      2.000000-2.000000 4.223500e+04+-1.477200e+02
expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)
== 3.8.1
run       fixed_fac_scale1,2    events     SCALUP/sqrt(s_hat)             sigma [pb]
tf        T,F                     1000      1.000000-1.000000 3.850000e+04+-9.249800e+01
ft        F,T                     1000      2.000000-2.000000 2.944200e+04+-5.945300e+01
ff        F,F                     1000      2.000000-2.000000 4.711600e+04+-1.063000e+02
tf_fixed  T,F                     1000      2.000000-2.000000 4.250300e+04+-1.055300e+02
expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)
```

**Actual**, on all three versions:
- `tf` has SCALUP/√ŝ = 1.000000 on all 1000 events. Every other run has 2.
- The fix raises `tf`'s σ by 10.4 %: 38 310 → 42 284 pb on 3.7.1, and
  38 500 → 42 503 pb on 3.8.1, and 38 246 → 42 235 pb on 3.5.17. Each is one seed, with quoted errors of
  0.15–0.25 %.

`tf` and `ft` are not expected to agree. In a mirrored subprocess
(`IMIRROR = 2`), `DSIG1` evaluates leg 1's parton from beam 2 at `Q2FACT(1)`.
So `fixed_fac_scale1` fixes the scale of process leg 1, not beam 1 (verified
by reading the generated `auto_dsig1.f`). That is a separate question.
