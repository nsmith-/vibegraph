# MadEvent: `AQCDUP`/`AQEDUP` are written with π truncated to 3.1415926

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

Each LHE event line carries the strong and electromagnetic couplings MadEvent
used (`AQCDUP`, `AQEDUP`). MadEvent computes them by dividing `g²` by
`4·3.1415926` instead of 4π. The written value is therefore α·(1 + 1.7e-8).

That is about a sixth of the last printed digit. On a run where αs varies per
event, it pushes the printed `AQCDUP` one unit high on about 12 % of events.

## Status

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran). It is also present
in 3.7.1. The report draft is [`report-draft.md`](report-draft.md).

## Where

`Template/LO/SubProcesses/unwgt.f`:

| version | lines |
|---|---|
| 3.7.1 | 760–761 |
| 3.5.17 | 719–720 |
| 3.8.1 | 768–769 |

```fortran
      aaqcd = g*g/4d0/3.1415926d0
      aaqed = gal(1)*gal(1)/4d0/3.1415926d0
```

## Root cause

- *Verified* (the runs below): on every event, the written `AQCDUP` equals
  αs(μR)·π/3.1415926 rounded to the printed digits. It does not always equal
  αs(μR).
- *Inferred from source:*
  - `g` is `sqrt(4·π·αs)` with full double-precision π, so the two π values do
    not cancel.
  - `AQEDUP` carries the same factor. Its constant value happened not to sit
    near a rounding boundary here, so the runs below cannot show it.

On a fixed-scale run the bias never shows in the printed field. MadEvent
compiles `aS` into `Source/param_card.inc` at 7 significant digits, so αs is
then a 7-digit decimal, and the +1.7e-8 shift cannot cross a rounding boundary.
The test therefore needs a per-event scale.

## The test

The process is `e+ e- > u u~ g` at √s = 1 TeV (`lpp = 0`), with
`dynamical_scale_choice = 3`, which sets μR = Σ m_T / 2. That makes αs vary per
event. The run has 2000 events, `iseed = 1` and the default `aS = 0.118`.

For each event, the script:
1. rebuilds μR from the event's momenta;
2. evaluates αs(μR) with MadGraph's own `alphas()` (the process's
   `Source/alfas_functions.f`, 2-loop running, `asmz = aS`, as MadEvent does
   without PDFs), using a small Fortran driver;
3. prints αs and `g*g/4d0/3.1415926d0` in the LHE format (`e15.7`);
4. counts which of the two equals the written `AQCDUP`.

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.5.17 (latest LTS release, 2026-10-09) | `git clone --depth 1 --branch v3.5.17 https://github.com/mg5amcnlo/mg5amcnlo mg-3.5.17` | `4492b3e3` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

All three versions ran on 2026-10-19 (Linux x86-64),
with Python 3.11 and gfortran from vibegraph's `madgraph` pixi environment (conda-forge). Exact commands, from
this directory, with `$MG` the directory holding the clones:

```
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/aq
pixi run -e madgraph env MG5="python $MG/mg-3.5.17/bin/mg5_aMC" ./repro.sh /tmp/aq
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/aq
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/aq`.


It takes about one minute per version.

## Expected against actual

**Expected:** the written `AQCDUP` equals αs(μR) on every event.

```
== 3.7.1
  e.g. mu_R 429.7320 GeV: written 0.9572415E-01, alpha_s 0.9572414E-01, alpha_s*pi/3.1415926 0.9572415E-01
  e.g. mu_R 452.6562 GeV: written 0.9512460E-01, alpha_s 0.9512459E-01, alpha_s*pi/3.1415926 0.9512460E-01
  e.g. mu_R 488.7164 GeV: written 0.9425418E-01, alpha_s 0.9425417E-01, alpha_s*pi/3.1415926 0.9425418E-01
aS(MZ) = 1.180000e-01, events = 2000
written AQCDUP equals alpha_s(mu_R)                : 1759 of 2000
written AQCDUP equals alpha_s(mu_R)*pi/3.1415926   : 2000 of 2000
expected: AQCDUP = alpha_s(mu_R) on every event
== 3.5.17
  e.g. mu_R 406.8135 GeV: written 9.63648100e-02, alpha_s 0.9636480E-01, alpha_s*pi/3.1415926 0.9636481E-01
  e.g. mu_R 487.9769 GeV: written 9.42712200e-02, alpha_s 0.9427121E-01, alpha_s*pi/3.1415926 0.9427122E-01
  e.g. mu_R 456.7704 GeV: written 9.50210000e-02, alpha_s 0.9502099E-01, alpha_s*pi/3.1415926 0.9502100E-01
aS(MZ) = 1.180000e-01, events = 2000
written AQCDUP equals alpha_s(mu_R)                : 1752 of 2000
written AQCDUP equals alpha_s(mu_R)*pi/3.1415926   : 1999 of 2000
expected: AQCDUP = alpha_s(mu_R) on every event
== 3.8.1
  e.g. mu_R 439.2732 GeV: written 0.9546988E-01, alpha_s 0.9546987E-01, alpha_s*pi/3.1415926 0.9546988E-01
  e.g. mu_R 431.1569 GeV: written 0.9568573E-01, alpha_s 0.9568572E-01, alpha_s*pi/3.1415926 0.9568573E-01
  e.g. mu_R 489.7639 GeV: written 0.9423010E-01, alpha_s 0.9423009E-01, alpha_s*pi/3.1415926 0.9423010E-01
aS(MZ) = 1.180000e-01, events = 2000
written AQCDUP equals alpha_s(mu_R)                : 1762 of 2000
written AQCDUP equals alpha_s(mu_R)*pi/3.1415926   : 2000 of 2000
expected: AQCDUP = alpha_s(mu_R) on every event
```

**Actual:** the written value equals αs·π/3.1415926 on 2000 of 2000 events
(3.7.1, 3.8.1) and 1999 of 2000 (3.5.17). It equals αs on only 1752–1762, so
about 12 % of events are one unit high in the last digit.

The one 3.5.17 event that matches neither (written `9.69951400e-02`) sits on a
rounding boundary. Its recomputed αs·π/3.1415926 is 0.0969951349999972, which
is 3e-15 below the rounding point. That is inside the precision of a μR
rebuilt from printed momenta.

3.5.17 writes the event line in another format (`9.63648100e-02`). Its values
still carry only 7 significant digits.
