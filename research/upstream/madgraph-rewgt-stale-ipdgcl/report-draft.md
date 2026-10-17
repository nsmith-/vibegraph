# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** MLM `rewgt`: a non-jet final-state parton keeps the flavour code left by an earlier event, so its PDF ratio uses the wrong flavour

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested.

### Summary

Under MLM matching (`ickkw = 1`), `rewgt` relabels the clustering tree with the
selected flavour combination, but only for legs that are jets. A final-state
parton that is not a jet keeps whatever PDG code the table held before. That is
the subprocess's first combination, or the last event in which that leg was a
jet.

When one subprocess mixes jet and non-jet flavours on the same leg, the PDF
ratio for that line is taken for the wrong quark flavour. Which wrong flavour
depends on the events generated before it.

### How to reproduce

The case needs a five-flavour proton with a four-flavour jet definition:

```
import model sm-no_b_mass
define p = g u c d s b u~ c~ d~ s~ b~
define j = p
generate p p > z j
output zj
launch zj
set ickkw 1
set xqcut 20
set maxjetflavor 4
set use_syst True
set systematics_program none
set nevents 2000
done
```

`P1_gq_zq` then holds `g d > z d`, `g s > z s` and `g b > z b` in one
subprocess, with `d` first. At `maxjetflavor = 4` the `b` is a parton but not a
jet.

For each `g b > z b` event, read the gluon beam's `<pdfrwt>` entry in the
`<mgrwt>` block. As a control, generate `g b > z b` and `g b~ > z b~` alone
with the same cards. `repro.sh` (attached) runs both and tallies the codes.

### Expected vs observed

- **Expected:** the `g b` events behave as in the control, with no PDF ratio on
  the gluon line (`<pdfrwt>` codes `21` only) and one α_s ratio.
- **Observed:** most `g b` events take the PDF ratio of a `d` or `s` line, with
  codes `21,±1` or `21,±3`.

| version | `g b` events with a d/s PDF ratio | control (`g b` alone) |
|---|---|---|
| 3.5.17 | 97 of 103 | 0 of 2000 |
| 3.8.1 | 87 of 97 | 0 of 2000 |

The `g u`, `g c`, `g d` and `g s` events all read their own flavour.

### Cause

`Template/LO/SubProcesses/reweight.f`, in `rewgt` (3.8.1 lines 1537–1538;
3.5.17 lines 1504–1505):

```fortran
c        Set jet identities according to chosen subprocess
         if(isjet(idup(i,ipsel,iproc)))
     $        ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)
```

`ipdgcl` is indexed by (leg, graph, subprocess), not by flavour combination.
`initcluster` fills it from the first combination (`cluster.f:373`). After
that, a non-jet leg's code changes only when some event labels that leg a jet.

### Possible fix

I don't have a one-line fix.

- Dropping the `isjet` guard, so every leg takes the drawn combination's code,
  makes the survey stop with `Error: Failed despite same graph`. The per-config
  jet memo (`njetstore`/`iqjetstore`) assumes a config always has the same
  number of jets, which no longer holds once the `b` is correctly a non-jet.
- Two options that would avoid the problem:
  - do not group flavour combinations whose final-state jet-ness differs at
    the run's `maxjetflavor`;
  - key the jet memo by combination.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
