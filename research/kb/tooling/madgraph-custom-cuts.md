---
type: Procedure
title: Custom MadGraph cuts via dummy_cuts
description: "Expressing a cut no run-card parameter can (an m(ττ) or pT(γ) window) by patching SubProcesses/dummy_fct.f, which passcuts calls last and which leaves phase-space generation untouched."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [madgraph, cuts, references, windows]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n27-b1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L84-L101", title: "Note 27 §B1: the window on MadGraph's side"}
  - {id: gen-higgs-window, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/gen_higgs_window.sh", title: "validation/madgraph/gen_higgs_window.sh"}
  - {id: gen-pta-windows, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/gen_pta_windows.sh", title: "validation/madgraph/gen_pta_windows.sh"}
  - {id: mg-dummy-fct, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/dummy_fct.f#L1-L40", title: "MadGraph dummy_fct.f, dummy_cuts"}
  - {id: mg-cuts, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cuts.f#L1223-L1229", title: "MadGraph cuts.f, passcuts calls dummy_cuts"}
---
# Custom MadGraph cuts via dummy_cuts

## When a run-card cut cannot express it

Some cuts a reference needs have no run-card parameter, or the nearest one
does something else. The worked case: a window on m(τ⁺τ⁻) in
`e+ e- > mu+ mu- ta+ ta-`.[^n27-b1]

- `mmll`/`mmllmax` apply to **every** same-flavour opposite-sign lepton pair;
  `setcuts.f` sets `s_min(j,i)=mmll*dabs(mmll)` guarded only by
  `abs(idup(i))==abs(idup(j))` and opposite sign. The window would bite the
  μ⁺μ⁻ pair too and measure a different thing.
- The per-PDG `mxx_min_pdg` cut is refused for leptons by `banner.py`: "Can
  not use PDG related cut for light quark/b quark/lepton/gluon/photon", with
  pdg 15 among them.

## The mechanism

Every generated LO process directory has `SubProcesses/dummy_fct.f`, whose
`logical function dummy_cuts(P)` is a user hook that returns `.true.` by
default:

```fortran
      logical FUNCTION dummy_cuts(P)
C            ALL MOMENTA ARE IN THE REST FRAME!!
      ...
      REAL*8 P(0:3,nexternal)
      ...
      dummy_cuts=.true.

      return
      end
```

`passcuts` in `cuts.f` calls it **after every other cut** and rejects the
point if it returns false:

```fortran
c   call the dummy_cuts function to check plugin/user defined cuts
      if(.not.dummy_cuts(P))then
         passcuts=.false.
         return
      endif
```

It touches nothing else: MadEvent's phase-space generation is unchanged, so a
windowed run integrates the same integrand an unwindowed run integrates,
restricted to the window. That is what makes window/complement/unwindowed
runs comparable and lets them close against each other
([windowed partition closure](../validation/windowed-partition-closure.md)).

## Procedure

As done in `validation/madgraph/gen_higgs_window.sh` (m(ττ)) and
`gen_pta_windows.sh` (pT(γ), m(μμ)):

1. Generate the process directory as usual, then **assert the leg order**
   against the generated `leshouche.inc` rather than assuming it, e.g.
   `DATA (IDUP(I,1,1),I=1,6)/-11,11,-13,13,-15,15/` so legs 5 and 6 are the τ
   pair. The index into `P(0:3,i)` is the external leg number. When the
   window run must integrate the banked run's channel decomposition, also
   diff the generated `configs.inc` against the bank's, as
   `gen_pta_windows.sh` does (and exits on a difference for its 3.7.1 leg).
2. **Patch the body by exact match.** Require the stock line
   `      dummy_cuts=.true.` to occur exactly once, then replace it with
   declarations, `dummy_cuts=.true.`, and the test. For the script's default
   window `[124.9, 125.1]` GeV (`MLO`/`MHI`) on legs 5,6:

   ```fortran
         double precision mtt2, mlo, mhi
         parameter (mlo = 124.9d0, mhi = 125.1d0)
         dummy_cuts=.true.
   c     window on m(tau+ tau-): externals 5 and 6 (idup -15, 15)
         mtt2 = (p(0,5)+p(0,6))**2 - (p(1,5)+p(1,6))**2
        &     - (p(2,5)+p(2,6))**2 - (p(3,5)+p(3,6))**2
         if (mtt2.lt.mlo*mlo .or. mtt2.gt.mhi*mhi) dummy_cuts=.false.
   ```

   The complement inverts the test
   (`if (mtt2.ge.mlo*mlo .and. mtt2.le.mhi*mhi) dummy_cuts=.false.`), so the
   window and its complement partition the space exactly. Echo the patched
   lines into the log (`grep -n mtt2 …dummy_fct.f`) so the run records what it
   applied.
3. Use the banked run's own `Cards/run_card.dat` and `param_card.dat`
   verbatim, changing only `nevents` and `iseed` (and, for
   `gen_pta_windows.sh`'s refocused windows, the photon pT cuts), so each
   windowed run is comparable to the banked cross section and to the others.
4. Run `bin/generate_events` with the C++ runtime on `LDFLAGS`
   ([the toolchain concept](madgraph-toolchain.md)) and read σ from
   `SubProcesses/results.dat`.

## Frame

The momenta reaching `dummy_cuts` are in the partonic rest frame (the
function's own header says so). With fixed equal-energy beams that is the lab
frame. With hadronic beams it is not: invariant masses and transverse momenta
need no boost, but a rapidity or any other non-invariant observable would have
to be boosted to the lab first, as `cuts.f`'s own rapidity cuts are: they
call `rap()` (`Source/kin_functions.f:95`), which adds the stored `cm_rap`.

## Caveats

- **Leakage.** In one MadGraph-only study, 27 of 500 000 MadEvent events lay
  outside a window its `dummy_cuts` should have enforced, and the cause is
  unexplained. It matters before any comparison reads a windowed MadEvent
  sample event by event: [madevent-dummy-cuts-window-leakage](../backlog/validation/madevent-dummy-cuts-window-leakage.md).
  Cross sections of windowed runs are not known to be affected.
- A patched directory is not a stock reference directory. The window
  generators generate their own process directories and copy only the banked
  run's `run_card.dat` and `param_card.dat` into them; they never patch a
  banked directory in place.
- `gen_higgs_window.sh` generates with `mg5_aMC` from PATH, the packaged
  3.5.7, and its committed `higgs_window_reference.json` says so; a new window
  study should go through `mg5_pinned.sh` as `gen_pta_windows.sh` does for its
  3.7.1 leg ([the toolchain concept](madgraph-toolchain.md)).

[^n27-b1]: Note 27 §B1 outcome, "The window on MadGraph's side".
