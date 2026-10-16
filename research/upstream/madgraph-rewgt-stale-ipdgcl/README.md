# MadEvent MLM: `rewgt` evaluates a non-jet parton's line with a stale flavour

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from vibegraph.

## What it shows

Under MLM matching, a final-state parton that is not a jet keeps whatever PDG
code an earlier event left in the clustering table `ipdgcl`. When one IPROC
mixes such a leg with jet flavours, its PDF ratio is evaluated for the wrong
flavour, and which wrong flavour depends on the events generated before it.

Take a five-flavour `p p > z j` at `maxjetflavor = 4`. On 3.7.1, 93 of 97
`g b > z b` events take the PDF ratio of a `d` or `s` line; on 3.8.1, 87 of 97
do. The same events generated without the other flavours take no PDF ratio.

## Status

**Present in 3.8.1** (ran). It is also present in 3.7.1. The report draft is
[`report-draft.md`](report-draft.md). No one-line fix exists (see "Root
cause").

## Where

`Template/LO/SubProcesses/reweight.f:1534-1538`, in `rewgt`; the same lines in
3.7.1 and 3.8.1. The table is first filled at `cluster.f:373` (`initcluster`).

```fortran
      do i=1,nexternal
         j=ishft(1,i-1)
c        Set jet identities according to chosen subprocess
         if(isjet(idup(i,ipsel,iproc)))
     $        ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)
```

## Root cause

- *Verified* (runs below):
  - `ipdgcl` is keyed by (leg, graph, IPROC), never by flavour combination.
  - `rewgt` writes the drawn combination's code for jet legs only.
  - A non-jet parton (a `b` at `maxjetflavor = 4`) therefore carries the code
    of the IPROC's first combination (`d`), or of the last event that labelled
    that leg a jet (`s`).
  - The PDF-ratio chain written by `use_syst` (`<pdfrwt>`) shows these codes
    on the b events.
- *Verified* (tried on 3.7.1, not kept as a variant): dropping the `isjet`
  guard, so that every leg takes the drawn code, makes the survey stop with
  `Error: Failed despite same graph: 2`.
  - With the right labels, the b is a non-jet in `setclscales` too.
  - The number of jets per config then varies with the flavour combination,
    which the per-config jet memo (`njetstore`/`iqjetstore`,
    `reweight.f:985-1018`) assumes it does not.
  - The stale label is what lets stock runs complete.
- *Inferred from source:* the next event's first `setclscales` call reads the
  same table for its jet tagging (the `xqcut` rejection). That use is not
  measured separately.

## The test

There are two stock process directories with the same cards:
- `sm-no_b_mass`, with b in `p` and `j`;
- `ickkw = 1`, `xqcut = 20`, `maxjetflavor = 4`;
- `use_syst = T`, `systematics_program = none`;
- `nevents = 2000`, `iseed = 1`.

MadGraph defaults to `maxjetflavor = 5` for this process; 4 is the user's
choice of a four-flavour jet definition.

| run | process | role |
|---|---|---|
| `mixed` | `p p > z j` | the defect: `P1_gq_zq` puts `g d > z d`, `g s > z s` and `g b > z b` in one IPROC, `d` first |
| `alone` | `g b > z b`, `g b~ > z b~` | control: the b is its IPROC's first combination, so its label is right |

For every event with exactly one incoming gluon and an outgoing quark, the
script tallies the codes on the gluon beam's `<pdfrwt>` chain and the number of
α_s ratios (`<asrwt>`).

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

Both ran on 2026-10-16 (Linux x86-64), with Python 3.11 and gfortran from
vibegraph's `madgraph` pixi environment (conda-forge). Exact commands, from
this directory, with `$MG` the directory holding the two clones:

```
pixi run -e madgraph env MG5="python $MG/mg-3.7.1/bin/mg5_aMC" ./repro.sh /tmp/ip
pixi run -e madgraph env MG5="python $MG/mg-3.8.1/bin/mg5_aMC" ./repro.sh /tmp/ip
```

On a stock install, use `MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/ip`.


It takes about two minutes per version.

## Expected against actual

**Expected:** the b events of `mixed` read as in `alone`, with no PDF ratio
(chain `21`) and one α_s ratio. A jet quark `q` reads `21,-q` (the spacelike
line after `g → q q~`) with one α_s ratio.

```
#### 3.7.1
== mixed
  g -5 > z -5   gluon-beam chain 21     alpha_s ratios 0      3 events
  g -5 > z -5   gluon-beam chain 21,1   alpha_s ratios 1     30 events
  g -5 > z -5   gluon-beam chain 21,3   alpha_s ratios 1     17 events
  g -4 > z -4   gluon-beam chain 21     alpha_s ratios 0      6 events
  g -4 > z -4   gluon-beam chain 21,4   alpha_s ratios 1     63 events
  g -3 > z -3   gluon-beam chain 21     alpha_s ratios 0      8 events
  g -3 > z -3   gluon-beam chain 21,3   alpha_s ratios 1     78 events
  g -2 > z -2   gluon-beam chain 21     alpha_s ratios 0     10 events
  g -2 > z -2   gluon-beam chain 21,2   alpha_s ratios 1    108 events
  g -1 > z -1   gluon-beam chain 21     alpha_s ratios 0     12 events
  g -1 > z -1   gluon-beam chain 21,1   alpha_s ratios 1    132 events
  g  1 > z  1   gluon-beam chain 21     alpha_s ratios 0     26 events
  g  1 > z  1   gluon-beam chain 21,-1  alpha_s ratios 1    346 events
  g  2 > z  2   gluon-beam chain 21     alpha_s ratios 0     29 events
  g  2 > z  2   gluon-beam chain 21,-2  alpha_s ratios 1    359 events
  g  3 > z  3   gluon-beam chain 21     alpha_s ratios 0      6 events
  g  3 > z  3   gluon-beam chain 21,-3  alpha_s ratios 1     86 events
  g  4 > z  4   gluon-beam chain 21     alpha_s ratios 0      3 events
  g  4 > z  4   gluon-beam chain 21,-4  alpha_s ratios 1     63 events
  g  5 > z  5   gluon-beam chain 21     alpha_s ratios 0      1 events
  g  5 > z  5   gluon-beam chain 21,-1  alpha_s ratios 1     41 events
  g  5 > z  5   gluon-beam chain 21,-3  alpha_s ratios 1      5 events
== alone
  g -5 > z -5   gluon-beam chain 21     alpha_s ratios 1    957 events
  g  5 > z  5   gluon-beam chain 21     alpha_s ratios 1   1043 events
expected: the b events of 'mixed' read as in 'alone' (where the label is right): no PDF ratio (chain 21) and one alpha_s ratio
observed in 'mixed': most b events take the PDF ratio of a d or s line (chain 21,+-1 or 21,+-3), whichever an earlier event left
#### 3.8.1
== mixed
  g -5 > z -5   gluon-beam chain 21     alpha_s ratios 0      7 events
  g -5 > z -5   gluon-beam chain 21,1   alpha_s ratios 1     28 events
  g -5 > z -5   gluon-beam chain 21,3   alpha_s ratios 1     12 events
  g -4 > z -4   gluon-beam chain 21     alpha_s ratios 0      2 events
  g -4 > z -4   gluon-beam chain 21,4   alpha_s ratios 1     53 events
  g -3 > z -3   gluon-beam chain 21     alpha_s ratios 0      8 events
  g -3 > z -3   gluon-beam chain 21,3   alpha_s ratios 1     82 events
  g -2 > z -2   gluon-beam chain 21     alpha_s ratios 0      4 events
  g -2 > z -2   gluon-beam chain 21,2   alpha_s ratios 1     90 events
  g -1 > z -1   gluon-beam chain 21     alpha_s ratios 0     11 events
  g -1 > z -1   gluon-beam chain 21,1   alpha_s ratios 1    135 events
  g  1 > z  1   gluon-beam chain 21     alpha_s ratios 0     22 events
  g  1 > z  1   gluon-beam chain 21,-1  alpha_s ratios 1    333 events
  g  2 > z  2   gluon-beam chain 21     alpha_s ratios 0     44 events
  g  2 > z  2   gluon-beam chain 21,-2  alpha_s ratios 1    400 events
  g  3 > z  3   gluon-beam chain 21     alpha_s ratios 0      6 events
  g  3 > z  3   gluon-beam chain 21,-3  alpha_s ratios 1     79 events
  g  4 > z  4   gluon-beam chain 21     alpha_s ratios 0      3 events
  g  4 > z  4   gluon-beam chain 21,-4  alpha_s ratios 1     61 events
  g  5 > z  5   gluon-beam chain 21     alpha_s ratios 0      3 events
  g  5 > z  5   gluon-beam chain 21,-1  alpha_s ratios 1     37 events
  g  5 > z  5   gluon-beam chain 21,-3  alpha_s ratios 1     10 events
== alone
  g -5 > z -5   gluon-beam chain 21     alpha_s ratios 1    967 events
  g  5 > z  5   gluon-beam chain 21     alpha_s ratios 1   1033 events
expected: the b events of 'mixed' read as in 'alone' (where the label is right): no PDF ratio (chain 21) and one alpha_s ratio
observed in 'mixed': most b events take the PDF ratio of a d or s line (chain 21,+-1 or 21,+-3), whichever an earlier event left
```

**Actual**, on both versions:
- `u`, `c`, `d` and `s` events read their own flavour.
- The b events of `mixed` mostly read `21,±1` or `21,±3`, a `d` or `s` PDF
  ratio: 93 of 97 on 3.7.1, and 87 of 97 on 3.8.1.
- `alone` reads `21` on every b event.

The effect on σ has not been measured; it is limited to the b channels (97 of
2000 events).
