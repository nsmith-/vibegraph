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
- **Attach:** `repro.sh` from this folder.

---

**Title:** MLM `rewgt`: a final-state parton that is not a jet keeps the PDG
code an earlier event left in `ipdgcl`, so its PDF ratio is taken for the
wrong flavour and depends on event history

**Version:** 3.8.1 and 3.7.1, `Template/LO/SubProcesses/reweight.f:1537-1538`.

**Summary.** `rewgt` labels the clustering tree with the drawn combination's
codes for jet legs only
(`if(isjet(idup(i,ipsel,iproc))) ipdgcl(j,igraphs(1),iproc)=idup(i,ipsel,iproc)`).

`ipdgcl` is kept per (leg, graph, IPROC), not per combination.
`initcluster` fills it from the IPROC's first combination (`cluster.f:373`).
So a non-jet parton leg carries the code of:
- the first combination; or
- the last event that labelled that leg a jet.

That code drives the flavour transmitted along the clustering, the density
in the PDF-ratio chain and `goodjet`. The next event's first `setclscales`
reads it too.

**Reproduce.** Run `p p > z j` with:
- `sm-no_b_mass`, with b in `p` and `j`;
- `ickkw = 1`, `xqcut = 20`, `maxjetflavor = 4`;
- `use_syst = T`, `systematics_program = none`.

One IPROC of `P1_gq_zq` then holds `g d > z d`, `g s > z s` and
`g b > z b`, and the b is not a jet. Read `<pdfrwt>` and `<asrwt>` on the
`g b > z b` events.

**Expected:** the same as when `g b > z b` is generated alone with the same
cards, where the b is the first combination and its label is right. That is
one α_s ratio and no PDF ratio on every event.

**Observed:** on 3.7.1, 93 of 97 `g b` events take the PDF ratio of a `d` or
`s` line (`<pdfrwt>` codes `21,±1` or `21,±3`), whichever an earlier event
left. On 3.8.1, 87 of 97 do. Every `g u/c/d/s` event reads its own flavour.

**Fix.** No one-line fix exists. Labelling every leg with the drawn code
makes the b a non-jet in `setclscales` as well, and the per-config jet memo
(`njetstore`/`iqjetstore`, `reweight.f:985-1018`) then aborts with
`Error: Failed despite same graph` (tried on 3.7.1).

A fix has to make two things consistent: the memo, or the grouping, with
combinations of different jet-ness within one IPROC. The simplest option is
not to group combinations whose final-state jet-ness differs at the run's
`maxjetflavor`. Another is to key the memo by combination.

Assisted-by: claude-code:claude-opus-5-5
