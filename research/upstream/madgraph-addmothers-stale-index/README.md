# MadEvent: `addmothers` compares `igraphs(1)` to a stale loop index

## What it shows

Line 115 of `addmothers.f` compares a graph number with the index of a loop
that has already finished, so the test does not mean what it reads as. It has
no effect at runtime: the branch it guards cannot be reached on a written
event.

## Status

**Present in 3.8.1; unreachable at runtime** (read). The file is
byte-identical in 3.7.1 (`b7687064`) and 3.8.1 (`a5b17913`).

**Absent from the 3.5.17 LTS release** (`4492b3e3`, read). There,
`addmothers.f:110-116` sets `lconfig = iconfig` and, under `ickkw > 0`,
`lconfig = igraphs(1)`. It has no `vec_igraph` and no fallback test; those came
with the vectorised code of the 3.6+ line. There is no
runtime reproducer, since no card reaches the line. The report draft is
[`report-draft.md`](report-draft.md), as a code-quality report.

## Where

`madgraph/iolibs/template_files/addmothers.f:115`:

```fortran
      do i=-nexternal+2,nexternal          ! :100
         icolalt(1,i)=0
         icolalt(2,i)=0
      enddo
      ...
      lconfig = iconfig
      if (ickkw.gt.0) then
         ...
         lconfig = vec_igraph(ivec)
         if (lconfig.eq.0.and.igraphs(1).ne.i.and.ivec.eq.1) lconfig = igraphs(1)   ! :115
      endif
```

## Root cause

- *Verified by reading:* `i` is not assigned between the loop at `:100` and
  `:115`, so it holds `nexternal+1`. The test compares `igraphs(1)`, a graph
  number, with `nexternal+1`.
- *Inferred:* the intended test is `igraphs(1).ne.0`. The fallback would then
  take the clustering's graph when the event carries none.

## Why there is no runtime reproducer

The fallback needs `vec_igraph(ivec) = 0` on an event being written with
`ickkw > 0`. By reading, both versions:
- `rewgt` (`reweight.f:1419-1471`) sets `vec_igraph(ivec) = 0`. It then
  either returns weight 0, so the event is never written, or sets
  `vec_igraph(ivec) = igraphs(1)` after a successful `setclscales`.
- `cluster` (`cluster.f:663-665`, `:810-830`) returns success only after
  storing at least one graph in `igraphs`.

Measured on 3.7.1: `vec_igraph` was never 0 on the written events of five
MLM-matched samples, in vibegraph's census of MadEvent's per-event
intermediates (`research/kb/validation/mlm-dump-oracle.md`). No 3.8.1 runtime
check was made.
