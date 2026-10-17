# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `addmothers.f`: fallback to `igraphs(1)` compares it with a finished loop index (code quality, no runtime effect found)

**Affected versions:** 3.8.1 yes. 3.5.17 (LTS) no: there `addmothers` sets
`lconfig = igraphs(1)` directly. Both were checked by reading the source.

### Summary

The test that chooses the configuration for the event record compares
`igraphs(1)` with `i`. `i` is the index of a loop that has already finished, so
it always holds `nexternal+1`. The intended test is probably
`igraphs(1).ne.0`. I could not reach this branch at runtime, so this is a code
cleanup, not a wrong-output report.

### Code

`madgraph/iolibs/template_files/addmothers.f` (3.8.1):

```fortran
      do i=-nexternal+2,nexternal          ! line 100, last use of i
      ...
      lconfig = iconfig
      if (ickkw.gt.0) then
         ...
         lconfig = vec_igraph(ivec)
         if (lconfig.eq.0.and.igraphs(1).ne.i.and.ivec.eq.1) lconfig = igraphs(1)   ! line 115
      endif
```

### Why it does not show at runtime

The fallback needs `vec_igraph(ivec) = 0` on an event that is written with
`ickkw > 0`. `rewgt` (`reweight.f:1419-1471`) zeroes `vec_igraph(ivec)` and
then does one of two things:
- it returns weight 0, and the event is never written; or
- it sets `vec_igraph(ivec) = igraphs(1)` after a successful `setclscales`.

`cluster` only succeeds after it has stored a graph. So the branch is dead on
written events, and I never saw `vec_igraph = 0` on a written matched event in
3.7.1.

### Suggested change

```diff
-         if (lconfig.eq.0.and.igraphs(1).ne.i.and.ivec.eq.1) lconfig = igraphs(1)
+         if (lconfig.eq.0.and.igraphs(1).ne.0.and.ivec.eq.1) lconfig = igraphs(1)
```

Alternatively, remove the dead fallback.

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; checked against both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
