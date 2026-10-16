# MadEvent: `scalefact` is skipped for beam 2 when only beam 1's μF is fixed

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from this repository.

## The defect

`Template/LO/SubProcesses/reweight.f:1138`, in `setclscales` (the same line in
3.7.1 and 3.8.1), reads:

```fortran
      if(.not. fixed_fac_scale1.or. fixed_fac_scale2) then
         if(.not.fixed_fac_scale1) q2fact(1)=scalefact**2*q2fact(1)
         if(.not.fixed_fac_scale2) q2fact(2)=scalefact**2*q2fact(2)
         if (.not.keepq2bck)then
            if(.not.fixed_fac_scale1) q2bck(1)=q2fact(1)
            if(.not.fixed_fac_scale2) q2bck(2)=q2fact(2)
         endif
```

`.not.` binds tighter than `.or.`, so the guard is
`(.not.fixed_fac_scale1) .or. fixed_fac_scale2`. With
`fixed_fac_scale1 = T, fixed_fac_scale2 = F` it is false, and the block is
skipped. The clustering scale of `q2fact(2)` is then not multiplied by
`scalefact²`, and `q2bck(2)` is not set. The same file's vectorised path
(`:1903`) writes the intended `.not.fixed_fac_scale1.or..not.fixed_fac_scale2`.
That form is the fix.

The block is reached with `dynamical_scale_choice = -1` (the default), whether
or not MLM matching is on.

## The test

The process is `p p > z`. With 2 → 1, the clustering scale is √ŝ = M_Z. The run
card is the default with these changes:
- `dynamical_scale_choice = -1`;
- `scalefact = 2`;
- the fixed scale (`dsqrt_q2fact1`, `dsqrt_q2fact2`) is 10 GeV.

`SCALUP = sqrt(max(q2fact(1), q2fact(2)))` is therefore the dynamical beam's
scale. It must equal `scalefact · √ŝ = 2 √ŝ` on every event.

| run | `fixed_fac_scale1,2` | what it is |
|---|---|---|
| `tf` | T, F | the defect |
| `ft` | F, T | control |
| `ff` | F, F | control |
| `tf_fixed` | T, F | `tf` with the fix applied to the generated `SubProcesses/reweight.f` |

```
MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/ffs
```

It takes about one minute.

## Results

These runs used the conda toolchain from `pixi run -e madgraph`.
- **3.7.1** is tag `v3.7.1`, commit `b7687064`.
- **3.8.1** is tag `v3.8.1`, commit `a5b17913`, the latest release on 2026-10-16.

```
== 3.7.1
run       fixed_fac_scale1,2    events     SCALUP/sqrt(s_hat)             sigma [pb]
tf        T,F                     1000      1.000000-1.000000 3.831000e+04+-5.461400e+01
ft        F,T                     1000      2.000000-2.000000 2.934600e+04+-3.218000e+01
ff        F,F                     1000      2.000000-2.000000 4.691600e+04+-5.812200e+01
tf_fixed  T,F                     1000      2.000000-2.000000 4.228400e+04+-6.175700e+01
expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)
== 3.8.1
run       fixed_fac_scale1,2    events     SCALUP/sqrt(s_hat)             sigma [pb]
tf        T,F                     1000      1.000000-1.000000 3.850000e+04+-9.249800e+01
ft        F,T                     1000      2.000000-2.000000 2.944200e+04+-5.945300e+01
ff        F,F                     1000      2.000000-2.000000 4.711600e+04+-1.063000e+02
tf_fixed  T,F                     1000      2.000000-2.000000 4.250300e+04+-1.055300e+02
expected: SCALUP/sqrt(s_hat) = 2 (scalefact) in every run, and sigma(tf) = sigma(tf_fixed)
```

- **Per event.** In `tf`, every event has SCALUP = √ŝ: `scalefact` is missing
  from beam 2's scale. In every control, and in `tf` with the fix, SCALUP = 2 √ŝ.
- **Cross section.** The fix raises `tf` by 10.4 % on both versions
  (3.7.1: 38 310 → 42 284 pb; 3.8.1: 38 500 → 42 503 pb). Each figure is one
  seed, with quoted errors of 0.15–0.25 %.
- **`tf` against `ft`.** These two are not expected to agree. In a mirrored
  subprocess (`IMIRROR = 2`), `DSIG1` evaluates leg 1's parton from beam 2 but
  at `Q2FACT(1)`. So `fixed_fac_scale1` fixes the scale of process leg 1 (the
  quark of `q q~ > z`), not of beam 1 as the run card's comment says. That
  labelling is a separate question and is not part of this report.
