# MadEvent: a decayed `b` switches the `ktdurham` cut on

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

The Durham-kT merging cut (`ktdurham`) should apply only to events with a
massless parton subject to the merging cut. A `b` quark from a decay
nevertheless switches it on. On `e+ e- > t t~ z, z > b b~`, whose only merging
partons are the massive `t` and `t~`, `ktdurham = 500` removes 34 % of σ.

## Status

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran). It is also present in
3.7.1. The report draft is
[`report-draft.md`](report-draft.md).

## Where

`Template/LO/SubProcesses/cuts.f`, the `NMASSLESS` count. The same expression
appears twice:

| version | Durham kT | ptlund |
|---|---|---|
| 3.7.1 | `:565-566` | `:667-668` |
| 3.5.17 | `:551-552` | `:653-654` |
| 3.8.1 | `:643-644` | `:745-746` |

```fortran
C       COUNT NUMBER OF MASSLESS OUTGOING PARTICLES, SINCE WE DO NOT WANT
C       TO APPLY A CUT FOR JUST A SINGLE MASSIVE PARTICLE IN THE FINAL STATE.
        NMASSLESS = 0
        DO I=NINCOMING+1,NEXTERNAL
          if(is_pdg_for_merging_cut(i) .and. .not. from_decay(I) .and.
     &                        is_a_j(i).or.is_a_b(i)) then
            NMASSLESS = NMASSLESS + 1
```

## Root cause

- *Verified* (the runs below):
  - `.and.` binds tighter than `.or.`, so the condition is
    `(… .and. is_a_j(i)) .or. is_a_b(i)`.
  - Every `is_a_b` parton counts. At `maxjetflavor = 4` that includes `b`
    quarks with `from_decay` set.
  - Parenthesising `(is_a_j(i).or.is_a_b(i))` makes the `ktdurham = 500` run
    bit-identical to `ktdurham = -1`.
- *Inferred from source:*
  - A `b` outside `pdgs_for_merging_cut` counts in the same way.
  - The `ptlund` count has the same effect. It was not run.

## The test

The process is `e+ e- > t t~ z, z > b b~` at √s = 1 TeV
(`lpp1 = lpp2 = 0`, `ebeam1 = ebeam2 = 500`). Otherwise the cards are the
defaults: `maxjetflavor = 4`, `cut_decays = F`, and `pdgs_for_merging_cut`
including 5 and 6. Each run has `nevents = 2000` and `iseed = 1`.

| run | `ktdurham` | role |
|---|---|---|
| `off` | −1 | control |
| `on` | 500 | the defect |
| `on_fixed` | 500 | both counts parenthesised in the generated `SubProcesses/cuts.f` |

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
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/kt
pixi run -e madgraph env MG5="python $MG/mg-3.5.17/bin/mg5_aMC" ./repro.sh /tmp/kt
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/kt
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/kt`.


It takes about two minutes per version.

## Expected against actual

**Expected:** the three cross sections agree.

```
== 3.7.1
run       ktdurham  sigma [pb]
off       -1        6.818183e-04 +- 2.870782e-06
on        500       4.512858e-04 +- 1.552665e-06
on_fixed  500       6.818183e-04 +- 2.870782e-06
expected: the three cross sections agree (no massless parton is subject to the merging cut)
== 3.5.17
run       ktdurham  sigma [pb]
off       -1        6.820050e-04 +- 4.831283e-06
on        500       4.509180e-04 +- 2.342569e-06
on_fixed  500       6.820050e-04 +- 4.831283e-06
expected: the three cross sections agree (no massless parton is subject to the merging cut)
== 3.8.1
run       ktdurham  sigma [pb]
off       -1        6.838672e-04 +- 4.308265e-06
on        500       4.469083e-04 +- 2.794296e-06
on_fixed  500       6.838672e-04 +- 4.308265e-06
expected: the three cross sections agree (no massless parton is subject to the merging cut)
```

**Actual:** `on` is 33.8 % below `off` on 3.7.1, 33.9 % below on 3.5.17 and
34.6 % below on 3.8.1, and
`on_fixed` equals `off` to every printed digit.

By reading, the same applies to any process whose only `is_a_b` partons are
decayed, or are not merging partons, while a massive merging parton is
undecayed. One example is the zero-jet multiplicity of a CKKW-L
`p p > t t~ h, h > b b~` merge at `maxjetflavor = 4`.
