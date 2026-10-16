# MadEvent: a decayed `b` switches the `ktdurham` cut on

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from this repository.

## The defect

`Template/LO/SubProcesses/cuts.f` applies the Durham-kT merging cut only to an
event with at least one massless parton that is subject to the merging cut.
The source comment says this is "SINCE WE DO NOT WANT TO APPLY A CUT FOR JUST A
SINGLE MASSIVE PARTICLE IN THE FINAL STATE". The count is:

```fortran
          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
     &                        is_a_j(i).or.is_a_b(i)) then
            NMASSLESS = NMASSLESS + 1
```

`.and.` binds tighter than `.or.`, so this is `(… .and. is_a_j(i)) .or.
is_a_b(i)`. Every `b` counts (`is_a_b`, at `maxjetflavor = 4`), including:
- one from a decay (`from_decay`);
- one whose PDG is not in `pdgs_for_merging_cut`.

When such a `b` is the only "massless" parton, the cut is applied to the
massive merging partons alone.

The same expression counts `NMASSLESS` for the `ptlund` cut further down.

| version | Durham kT | ptlund |
|---|---|---|
| 3.7.1 | `cuts.f:565-566` | `:667-668` |
| 3.8.1 | `cuts.f:643-644` | `:745-746` |

The fix parenthesises the `.or.`:
`… .and. .not. from_decay(I) .and. (is_a_j(i).or.is_a_b(i))`.

## The test

The process is `e+ e- > t t~ z, z > b b~` at √s = 1 TeV. The cards are the
defaults: `maxjetflavor = 4`, `cut_decays = F`, and `pdgs_for_merging_cut`
including 5 and 6.
- The merging-cut particles are `t` and `t~`, which are massive.
- The `b` quarks come from the decay.

`NMASSLESS` should be 0, and `ktdurham` should not change σ.

| run | `ktdurham` | what it is |
|---|---|---|
| `off` | −1 | cut off |
| `on` | 500 | the defect |
| `on_fixed` | 500 | `on` with both counts parenthesised in the generated `SubProcesses/cuts.f` |

```
MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/kt
```

It takes about two minutes.

## Results

These runs used the conda toolchain from `pixi run -e madgraph`.
- **3.7.1** is tag `v3.7.1`, commit `b7687064`.
- **3.8.1** is tag `v3.8.1`, commit `a5b17913`, the latest release on 2026-10-16.

```
== 3.7.1
run       ktdurham  sigma [pb]
off       -1        6.818183e-04 +- 2.870782e-06
on        500       4.512858e-04 +- 1.552665e-06
on_fixed  500       6.818183e-04 +- 2.870782e-06
expected: the three cross sections agree (no massless parton is subject to the merging cut)
== 3.8.1
run       ktdurham  sigma [pb]
off       -1        6.838672e-04 +- 4.308265e-06
on        500       4.469083e-04 +- 2.794296e-06
on_fixed  500       6.838672e-04 +- 4.308265e-06
expected: the three cross sections agree (no massless parton is subject to the merging cut)
```

- With the defect, `ktdurham = 500` removes **34 %** of σ (3.7.1: −33.8 %; 3.8.1:
  −34.6 %), cutting on the t–t~ pair's kT.
- With the fix, `on_fixed` is bit-identical to `off` at the same seed: the cut
  never fires.

The size depends on `ktdurham`. By reading, the same holds for any process
whose only `is_a_b` partons are decayed, or are not merging partons, while a
massive merging parton is undecayed. One example is the zero-jet
multiplicity of a CKKW-L `p p > t t~ h, h > b b~` merge at `maxjetflavor = 4`.
