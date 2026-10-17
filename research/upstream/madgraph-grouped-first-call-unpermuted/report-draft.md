# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** Grouped output sets the scales (and applies the `xqcut` rejection) on the unpermuted phase-space point: σ about 4 % low on channels with symmetry permutations

**Affected versions:** 3.8.1 yes. @LTS@

### Summary

In grouped output (`group_subprocesses True`, the default), `DSIGPROC`
evaluates the matrix element on `P1`, the channel's symmetry permutation of the
sampled point `PP`. The first `setclscales` call, made through
`update_scale_coupling`, is handed the unpermuted `PP` instead. That call does
three things:
- it sets μR;
- it sets the PDF scales;
- with matching, it can reject the point through the `xqcut` test.

On channels whose permutation is not the identity, it clusters an event whose
momenta don't match the flavours and diagram being evaluated. σ comes out low
at default cards, and also with MLM matching.

### How to reproduce

```
import model sm
define q = u d
generate u q > z u q
output grouped
```

Launch with the default run card (`nevents = 5000` is enough), with several
`iseed` values, each in a fresh copy of the output. Then apply the one-line fix
below to the generated `SubProcesses/P1_qq_zqq/auto_dsig.f`, and run the same
seeds again. `P1_qq_zqq/symperms.inc` shows the `(1,2,3,5,4)` final-quark swap
on configs 2, 5, 6 and 8. `repro.sh` (attached) automates this, including a
`group_subprocesses False` comparison.

### Expected vs observed

The one-line fix should not change σ, since grouping is an implementation
detail. Five seeds per row on 3.8.1; errors are the spread over seeds:

| card | grouped (pb) | grouped + fix (pb) | change |
|---|---|---|---|
| default | 106.64 ± 0.20 | 110.55 ± 0.19 | +3.7 % |
| `ickkw = 1`, `xqcut = 40` | 55.61 ± 0.09 | 57.53 ± 0.09 | +3.4 % |

The deficit sits in the channels holding the permuted configs (`G4` and `G7`
at default cards).

**Comparing with `group_subprocesses False` in 3.8.1.** In 3.7.1, grouped + fix
agreed with the non-grouped output to 0.05 %, while stock grouped was 4 % low.
In 3.8.1 that comparison has a second, unrelated offset. Setting
`keep_grid_for_refine = False` in `gen_ximprove.py` (commit `ebbdc974`) raises
grouped σ by about 2 % in every channel, including unpermuted ones.
- With `keep_grid_for_refine = True` restored on 3.8.1, grouped + fix is
  108.65 ± 0.32 pb against non-grouped 108.60 ± 0.45 pb, and stock grouped is
  3.9 % low.
- I don't know which refine setting gives the right answer, so grouped + fix
  against stock grouped is the cleaner test.

### Cause

`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc`, `DSIGPROC`
(3.8.1):

```fortran
        CALL SWITCHMOM(PP,P1,PERMS(1,MAPCONFIG(ICONFIG)),JC,NEXTERNAL)   ! line 819
        ...                                                              ! mirror of P1, line 829
            call update_scale_coupling(pp, wgt)                          ! line 856
```

The matrix element and `REWGT`'s second `setclscales` call read `P1`; the
first call reads `PP`.

### Suggested fix

```diff
       IF (VECSIZE_MEMMAX.LE.1.and.imode.ne.5) THEN ! no-vector (NB not VECSIZE_USED!)
-            call update_scale_coupling(pp, wgt)
+            call update_scale_coupling(p1, wgt)
       endif
```

This covers the scalar path (`vector_size = 1`, the default). By reading, the
vectorised path has the same problem and needs a larger change.
`DSIG_VEC` calls `update_scale_coupling_vec(all_p, …)` (line 317) before
`DSIGPROC_VEC` builds and mirrors `ALL_P1`, so the call would have to move after
the mirror loop. I have not tested that path.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
