# MadEvent MLM: `rewgt` evaluates a non-jet parton's line with a stale flavour

A MadGraph-only reproducer. It needs a stock MadGraph5_aMC@NLO with a Fortran
compiler, and nothing from this repository.

## The defect

Under MLM matching (`ickkw = 1`), `rewgt` (`Template/LO/SubProcesses/reweight.f`,
`:1534-1538`, the same in 3.7.1 and 3.8.1) labels the clustering tree with the
drawn flavour combination's codes. It does so for jets only:

```fortran
      do i=1,nexternal
         j=ishft(1,i-1)
c        Set jet identities according to chosen subprocess
         if(isjet(idup(i,ipsel,iproc)))
     $        ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)
```

`ipdgcl` lives in a common block, keyed by (leg, graph, IPROC) and never by
combination.
- `initcluster` fills it from the IPROC's **first** combination
  (`cluster.f:373`).
- After that, a leg's entry changes only when some event labels that leg a jet.
- A final-state parton that is not a jet therefore keeps the code that an
  earlier event, or the first combination, left there.

That code is then used to:
- transmit flavours along the clustering (`ipartupdate`);
- choose the parton density in the PDF-ratio chain (`pdg2pdf`);
- decide which lines are parton lines (`goodjet`).

The first `setclscales` call of the next event reads the same table.

Whenever one IPROC holds combinations in which a final-state leg is a jet in
some and a non-jet parton in others, the event weight depends on that leg's
true flavour and also on the events generated before it.

## The test

The process is `p p > z j` with a five-flavour proton and massless b
(`sm-no_b_mass`, b in `p` and `j`). The cards are `ickkw = 1`, `xqcut = 20`
and `maxjetflavor = 4`. MadGraph defaults to 5 for this process, and 4 is the
user's choice of a four-flavour jet definition.
- The b is then a parton but not a jet.
- `P1_gq_zq` puts `g d > z d`, `g s > z s` and `g b > z b` in one IPROC (and
  their antiquarks in another), with `d` first.

`use_syst = T` writes the chain each event used (`<mgrwt><pdfrwt>`) and its
α_s ratios (`<asrwt>`), with `systematics_program = none`.

| run | what it is |
|---|---|
| `mixed` | `p p > z j`: b shares its IPROC with d and s |
| `alone` | `g b > z b` and `g b~ > z b~` only, same cards; the b is its IPROC's first combination, so its label is right |

For every event with exactly one incoming gluon and an outgoing quark, the
script tallies the codes on the gluon beam's chain and the number of α_s
ratios.

```
MG5=/path/to/MG5_aMC/bin/mg5_aMC ./repro.sh /tmp/ip
```

It takes about two minutes.

There is no one-line fix to run as a control. Giving every leg the drawn
combination's code (dropping the `isjet` guard) was tried on 3.7.1, and the
survey stops with `Error: Failed despite same graph: 2`. With the right
labels the b is a non-jet in `setclscales` too, so the number of jets per
config then varies with the flavour combination. The per-config jet memo
(`njetstore`/`iqjetstore`, `reweight.f:985-1018`) assumes it does not. The
stale label is what lets those runs complete.

## Results

These runs used the conda toolchain from `pixi run -e madgraph`.
- **3.7.1** is tag `v3.7.1`, commit `b7687064`.
- **3.8.1** is tag `v3.8.1`, commit `a5b17913`, the latest release on 2026-10-16.

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

- **Jet quarks.** `g q > z q` with `q = u, c, d, s` takes the ratio of its own
  line (`21,-q`) and one α_s ratio, or no ratio at all.
- **b in `alone`.** No PDF ratio (chain `21`) and one α_s ratio, on every event.
- **b in `mixed`.** Most events take one α_s ratio plus the PDF ratio of a `d`
  or `s` line (`21,±1` or `21,±3`): 93 of 97 b events (96 %) on 3.7.1, and 87
  of 97 (90 %) on 3.8.1. Which of the two depends on the events before it.

So the b events' weights in `mixed` include a PDF ratio that `alone` does not
take, evaluated for a quark flavour that is not in the event.
- The effect on σ is not measured. It is limited to the b channels: 97 of
  this sample's 2000 events.
- The same table drives the first `setclscales` call's jet tagging (the
  `xqcut` rejection) for the next event. That use is not measured separately.
