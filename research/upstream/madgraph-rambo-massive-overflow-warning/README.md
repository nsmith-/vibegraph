# MadGraph: RAMBO's massive-momenta overflow warning can never print

A code-level reproducer. It needs only a stock MadGraph5_aMC@NLO Python tree.

## What it shows

`madgraph/various/rambo.py` (the Python RAMBO used by MadGraph's `check`
commands) warns when the log of the phase-space weight may overflow. For
massive particles that warning is guarded by `iwarn[4] > 5` instead of
`iwarn[4] < 5`. It can never print, because `iwarn` starts at zero on every
call.

The practical loss is small. By reading, the mass correction only lowers the
weight, so a massive weight above the threshold is
already preceded by the massless-weight warning in the same call.

## Status

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran, code level). It is
also present in 3.7.1. The report draft is [`report-draft.md`](report-draft.md).

## Where

`madgraph/various/rambo.py`, function `RAMBO`:

| version | massive overflow test | massless warning formats (`%f20.9`) |
|---|---|---|
| 3.7.1 | `:218` | `:162`, `:165` |
| 3.5.17 | `:220` | `:164`, `:167` |
| 3.8.1 | `:218` | `:162`, `:165` |

```python
    iwarn = FortranList(5)          # fresh zeros on every call
    ...
    if wt > 174 and iwarn[2] < 5:   # massless weight
        print(" RAMBO WARNS: WEIGHT = EXP(%f20.9) MAY  OVERFLOW" % wt)
    ...
    if(wt > 174  and iwarn[4] > 5): # massive weight
        print(" RAMBO WARNS: WEIGHT = EXP(%s) MAY OVERFLOW" % wt)
```

## Root cause

- *Verified* (run below): with massive particles and a log weight of 191, only
  the massless-weight warning prints. A copy of `RAMBO` with `> 5` replaced by
  `< 5` prints both.
- *Verified* (same run, a second cosmetic slip): the massless warnings format
  the weight with `%f20.9`, which prints `191.10138020.9`. That is `%f`
  followed by the literal text `20.9`; `%20.9f` was meant.
- *Inferred from source:*
  - the other three warnings in the function all test `iwarn[k] < 5`;
  - because `iwarn` is local, the "at most five warnings" counting never spans
    calls anyway.

## The test

`check_rambo_warning.py` calls `RAMBO(10, 1e6, masses)` with 10 massless and
then 10 massive (1 GeV) particles; the log weight is then about 191, above the
174 threshold. It prints the overflow warnings each call produces. It then does
the same with an in-memory copy of `RAMBO` carrying the one-character fix, and
exits 1 if the massive warning is missing.

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.5.17 (latest LTS release, 2026-10-09) | `git clone --depth 1 --branch v3.5.17 https://github.com/mg5amcnlo/mg5amcnlo mg-3.5.17` | `4492b3e3` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

All three ran on 2026-10-19 with Python 3.11 from vibegraph's `madgraph` pixi
environment. The commands, from this directory, with `$MG` the directory
holding the clones:

```
pixi run -e madgraph python -I check_rambo_warning.py $MG/mg-3.7.1
pixi run -e madgraph python -I check_rambo_warning.py $MG/mg-3.5.17
pixi run -e madgraph python -I check_rambo_warning.py $MG/mg-3.8.1
```

## Expected against actual

**Expected:** the massive call prints the massive-weight warning, as the fixed
copy does.

```
== 3.7.1
massless        log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive         log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive, fixed  log weight 191.1  overflow warnings printed: 2
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
    RAMBO WARNS: WEIGHT = EXP(191.1013799869683) MAY OVERFLOW
expected: the massive run prints the massive-weight warning after the massless-weight one (2 lines), as the fixed copy does
exit 1
== 3.5.17
massless        log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive         log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive, fixed  log weight 191.1  overflow warnings printed: 2
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
    RAMBO WARNS: WEIGHT = EXP(191.1013799864649) MAY OVERFLOW
expected: the massive run prints the massive-weight warning after the massless-weight one (2 lines), as the fixed copy does
exit 1
== 3.8.1
massless        log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive         log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive, fixed  log weight 191.1  overflow warnings printed: 2
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
    RAMBO WARNS: WEIGHT = EXP(191.10137998648733) MAY OVERFLOW
expected: the massive run prints the massive-weight warning after the massless-weight one (2 lines), as the fixed copy does
exit 1
```

**Actual:** on all three versions the massive call prints only the
massless-weight warning, and the script exits 1.
