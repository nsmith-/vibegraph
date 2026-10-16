# Draft: MadGraph bug report

This draft is for the user to review, edit and file, either on Launchpad
(<https://bugs.launchpad.net/mg5amcnlo>) or as an issue on
<https://github.com/mg5amcnlo/mg5amcnlo>, which MadGraph's README names as the
places for bug reports.
- **Policy.** MadGraph publishes no policy on AI-assisted reports; checked on
  2026-10-16 (see the index,
  `research/kb/validation/madgraph-upstream-report-drafts.md`).
- **Before filing.** Re-run the claims from [`README.md`](README.md) and be
  ready to answer questions on them.
- **Disclosure.** Keep or reword the `Assisted-by` line at the end; it
  discloses the tool use.

---

**Title:** `addmothers.f:115` compares `igraphs(1)` with the loop index `i`
after its loop has ended

**Version:** 3.8.1 and 3.7.1, `madgraph/iolibs/template_files/addmothers.f:115`
(byte-identical file).

**Summary.** The line reads
`if (lconfig.eq.0.and.igraphs(1).ne.i.and.ivec.eq.1) lconfig = igraphs(1)`.
Here `i` is the index of the `do i=-nexternal+2,nexternal` loop at `:100`, so
it always holds `nexternal+1`.

The test is meant to fall back to `igraphs(1)` when the event carries no
matched graph. As written, it compares a graph number with a leg count.
The likely intent is `igraphs(1).ne.0`.

**Reachability.** The branch needs `vec_igraph(ivec) = 0` on a written event
with `ickkw > 0`. By reading, this never happens:
- `rewgt` (`reweight.f:1419-1471`) zeroes `vec_igraph(ivec)`.
- It then either returns weight 0, so the event is not written, or sets it to
  `igraphs(1)` after a successful `setclscales`.
- `cluster` returns success only after storing at least one graph
  (`cluster.f:663-665`, `:810-830`).

Measured: `vec_igraph` was never 0 on the written events of five MLM samples
in 3.7.1, read from MadEvent's own per-event intermediates.

**Suggested change:** replace `igraphs(1).ne.i` with `igraphs(1).ne.0`, or
drop the dead fallback. This is a code-quality report, with no observable
effect today.

Assisted-by: claude-code:claude-opus-5-5
