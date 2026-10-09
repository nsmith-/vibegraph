---
type: Algorithm
title: "MadGraph kT clustering: measures, admissible merges and the merge step"
description: "MadGraph 3.7.1 cluster.f for dynamical_scale_choice = -1: run-card constants, djb/dj/BW measures, the merge graph from configs.inc, visit order and strict-< tie-break, the merge step and the core."
status: draft
tags: [kt-clustering, madgraph, scales, cluster-f, spec]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n28-k1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L329-L754", title: "Note 28 §K1–K1.4 (entry, constants, measures, merge graph, tie-break, merge step)"}
  - {id: n28-k35, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2171-L2198", title: "Note 28 §K3.5 (single-leg complement PDG)"}
  - {id: n28-k37, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2220-L2244", title: "Note 28 §K3.7 (confirmed against the bank)"}
  - {id: n28-k45, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2539-L2561", title: "Note 28 §K4.5 (the single-leg complement explained)"}
  - {id: mg-cluster, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f", title: "MadGraph 3.7.1 cluster.f"}
  - {id: mg-export, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L2193-L2197", title: "MadGraph 3.7.1 export_v4.py (configs.inc writer, minimal-arity filter)"}
---
# MadGraph kT clustering: measures, admissible merges and the merge step

The specification vibegraph implements for `dynamical_scale_choice = -1`:
MadGraph 3.7.1's `cluster.f`, read at the pinned tree `b7687064` (line numbers
are that tree's; 3.5.7 numbers differ, and it behaves differently in the
`cut_bw` refresh below and the `scalefact` placement in
[setclscales](setclscales.md), which turns the cluster sequence into `μR` and
per-beam `μF`). The Rust transcription is
[kt-clustering-engine](kt-clustering-engine.md); the per-event dump that pins
every step is [kt-cluster-dump-oracle](../validation/kt-cluster-dump-oracle.md);
the MadGraph tree as a whole is
[madgraph5-amcnlo](../references/codebases/madgraph5-amcnlo.md).[^n28-k1]

## Entry and run-card constants

Under `-1` `set_ren_scale`/`set_fac_scale` return zero
([`setscales.f:46-49`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/setscales.f#L46-L49),
`:133-137`) and `setclscales` fills `scale` and `q2fact(1:2)` from the
clustering, driven per event by `update_scale_coupling_vec`
(`reweight.f:1890-1913`); the map is built once per process directory at the
first `passcuts` (`cuts.f:209-210` → `initcluster.f:48`, `filmap`).

| constant | default (`banner.py`) | read at |
|---|---|---|
| `dynamical_scale_choice` | `-1` (`:4266`) | `setscales.f:46,133` |
| `scalefact` | `1.0` (`:4283`) | `setscales.f:93`, `reweight.f` |
| `ickkw` | `0` (`:4284`) | `reweight.f:643,1103,1114,1195`; `cluster.f:621,880` |
| `ktscheme` | `1` (`:4286`) | `cluster.f:610,621,869,880` |
| `chcluster` | `False` (`:4288`) | `cluster.f:468-470`; forced true by `reweight.f:664` |
| `maxjetflavor` | `4` (`:4424`) | `isjet`, `kin_functions.f:289-294` |
| `bwcutoff` | `15.0` (`:4305`) | `myamp.f` on-shell test |
| `d` (hidden) | `1.0` (`:4441`) | `common/to_dj/D`, `kin_functions.f:297`; never assigned in LO `setrun.f` |

`D` enters squared in the denominator of the final-state measure, so a zero
would make every non-resonant final-state merge `+Infinity`. `clusinfo`
(`unwgt.f:838`) is gated on `ickkw ≠ 0`, so no unmatched LHE file carries a
`<clustering>` tag: instrumentation is the only route to the merge sequence.

## The four measures (all squared, GeV²)

**`DJB(p)`, one leg against the beams**
([`kin_functions.f:421-429`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/Source/kin_functions.f#L421-L429)):
`djb = max(p(0),0)**2` if `lpp(1) = lpp(2) = 0`, else
`djb = (p(0)-p(3))*(p(0)+p(3))`. The switch is on **both** beams being
PDF-less, not per beam and not on energy: `m² + p_T²` with any PDF, `E²` with none.

**`DJ(p1,p2)`, two final-state legs** (`kin_functions.f:230-311`): at
`lpp = (0,0)` the Durham form `2·min(E1²,E2²)(1−cos θ12)` (`:271`; a zero
3-momentum warns and returns 0). Otherwise, the massless–massive case returns
`DJB(massless leg)·(1+1e-6)` (`:291`, `:294`) when one leg has `m² < 1` and the
other `m² ≥ 3` with `maxjetflavor > 4` or `m² ≥ 1` with `maxjetflavor > 3`;
else `dj = max(m1², m2²) + min(pT1², pT2²)·2(cosh Δη − cos Δφ)/D²`
(`:296-297`, `cos Δφ` written as `(px1·px2 + py1·py2)/√(pT1²·pT2²)`).
Slot 4 of `pcl` is the mass squared, set from `dot(p,p)`
(`cluster.f:586`), and `dot` clamps to exactly zero below `1e-6` absolute and
relative (`kin_functions.f:593-605`), which is what makes the `p1(4) < 1`
massless test reliable. For a 2 → 2 the legs are back to back in the
transverse plane, so the factor is `2(cosh Δη + 1) ≥ 4`.

**Beam–leg pairs do not use `dj`**: `cluster.f:626` sets `pt2ij = djb` of the
final-state line; the beam enters only through the tie-break. `zclus` never
reaches a scale at `ickkw = 0`; `PYDJ`/`PYJB` (`ktscheme = 2`, `ickkw = 2`) are
reachable from no supported card.

**Breit–Wigner override.** A final-state pair whose mask is tagged on-shell
(`isbw`) is measured by its invariant mass squared, `SumDot(pcl_i, pcl_j, 1d0)`
([`cluster.f:604-605`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L604-L605)).
`checkbw` (`:386-434`) tags only **`this_config`'s** lines, refreshing `OnBW`
with `cut_bw(p)` (`:419-423`; new in 3.7.1), so tagging is a property of the
channel; `isbw` is a common block that keeps stale flags between events
([cluster-scale-channel-dependence](cluster-scale-channel-dependence.md)).

## Which merges are admissible

The clustering is graph-guided: a pair may merge only if a surviving diagram
has a propagator whose subtree is exactly that pair's legs.

1. **Diagrams of minimal vertex arity only.** `export_v4.py` computes
   `minvert` and skips any configuration with a larger vertex
   ([`:2193-2197`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L2193-L2197),
   "Only 3-vertices allowed in configs.inc"). Four-point contact diagrams leave
   the merge graph: `g g → g g` has three configurations, not four. vibegraph's
   enumeration keeps the contact diagram, so the derivation must drop it.
2. **`configs.inc`**: `iforest` daughters, `sprop` (s-channel PDG per
   subprocess), `tprid` (`|pdg|` of a t-channel line, `0` on an s-channel one)
   (`export_v4.py:2249-2267`);
   the QCD order per configuration goes to `config_nqcd.inc`.
3. **`filmap`** (`cluster.f:325-383`) skips every configuration whose `nqcd`
   differs from `nqcd(this_config)`
   ([`:359-366`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L359-L366)):
   a no-op when all diagrams share one order, live on mixed QCD/QED processes.
4. **`filgrp`** (`cluster.f:191-322`) registers each internal line under its
   leg mask and under the **complement** `2^nexternal − 1 − mask`
   ([`:262`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L260-L262)),
   so a t-channel line between beam 1 and leg 3 is found as `{1,3}`. Its PDG is
   `sprop` if nonzero, else `tprid`, else at the last level beam 2's own PDG.
   `filprp` appends configs in ascending order, which `findmt`'s sorted-list
   intersection relies on; `resmap` marks lines with `prwidth > 0`.
5. **`findmt`** (`cluster.f:436-515`) keeps, on the first merge, the graphs
   of the pair's mask filtered by `chcluster` (only `iconfig`) and by every
   tagged BW (`resmap` must hold for each); later merges intersect. A pair with
   no surviving graph keeps the sentinel `1e37` and is never a candidate.

The two beams never combine (the loop runs only for `i > 2`, `cluster.f:588`).
If no pair is admissible, `cluster` returns false (`cluster.f:672-675`) and
`setclscales` writes `Clustering failed` to the run's `error` file and `stop`s
(`reweight.f:667-677`); only in `init_mode` (the helicity-filtering pass) does
it return false instead, and the caller zero-weights the point
(`:1907-1908`). vibegraph's `ClusterFailure::NoAdmissiblePair` likewise stays
an error: `EventScaleSource::point_scales` turns only the factorisation floor
and `JetCut` into a veto.

**The single-leg complement.** Only the line closing a channel on the beams
has a single leg (beam 2) as complement, and the t-channel branch of the
`configs.inc` writer
([`export_v4.py:2264-2266`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/madgraph/iolibs/export_v4.py#L2264-L2266))
gives it `tprid = |leg 2's id|`, so the complement write is beam 2's own code up
to sign; `isqcd`, `isjet` and `is_octet` read `|pdg|`. The live `ipdgcl` keeps
each single-leg mask's subprocess flavour (the literal reading disagreed on 7837
of 10 000 events of the `pp_to_llj` dump as first banked; not re-counted on the
re-banked dump); vibegraph reproduces the live array, and both give
identical scales (`b b̄ → b b̄` at `maxjetflavor = 4` included). Pinned by
`only_the_closing_line_can_write_a_single_leg_entry`,
`a_single_leg_keeps_its_flavour`, `overwriting_a_single_leg_entry_moves_no_scale`
(`coupling/cluster/graph.rs`).[^n28-k35]

## Winner selection and the tie-break

Candidates are visited `i = 3 … nexternal`, `j = 1 … i−1`:
`(3,1), (3,2), (4,1), (4,2), (4,3), (5,1), …`, in the first pass
(`cluster.f:579-649`) and in every recompute pass over the shrunken map
(`:838-906`). A beam–leg candidate is multiplied by `(1d0+1d-6)` when
`sign(1d0,pcl(3,idi)).ne.sign(1d0,pcl(3,idj))`, "prefer clustering when outgoing
in direction of incoming"
([`cluster.f:629-631`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L629-L631)).
`sign(1d0, −0.0)` is `−1`, so signed zero is observable. Only beam–leg
candidates are inflated; when all admissible ones are crossed, the minimum
itself carries the factor.
The comparison is strict `<` against `1.0d37`
([`:641-645`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L641-L645)),
so an exact tie goes to the earlier-visited pair and a measure `≥ 1e37`
(including `+Infinity`) never wins.

Worked case, `u ū → u ū` at `√ŝ = 500`: flavour admits only `{1,3}` and
`{3,4}`; with leg 3 backward both beam–leg candidates are crossed, `(3,1)` wins
at `62500·(1+1e-6)`, and the walk gives `μR = μF = 250.000125` (`SCALUP` prints
`2.5000012E+02`; derivation in [madgraph-scale-choice](madgraph-scale-choice.md)).
The engine reproduces all 32 inflated candidates over 16 dumped events, pinned by
`a_wholly_crossed_event_carries_the_tie_break_into_the_scale`, its negative
control `colourless_beams_keep_the_tie_break_out_of_the_scale`, and
`an_exact_tie_goes_to_the_pair_visited_first` (`setclscales.rs`).[^n28-k37]

## The merge step and the core

Per merge `n` (`cluster.f:677-907`): `imocl(n)` is the mask sum, `pt2ijcl(n)`
the winning measure, and the graph list is intersected again (failure is fatal):

| | initial state (`iwin < 3`) | final state |
|---|---|---|
| mother momentum | `pcl(ida1) − pcl(ida2)`, spacelike (`:721`) | sum (`:756`) |
| `mt2ij(n)` | `djb` of the emitted final-state leg (`:712`) | 0 |
| mother mass² | `max(m₁², m₂²)` (`:730-733`) | same; `pt2ijcl(n)` if `isbw` (`:762-767`) |

The mass condition `A .or. B .and. .not.(A .and. B)` (`:731-733`) is `A .or. B`
by Fortran precedence: the "exactly one massive" exclusion is dead code.

**Boost and rotation** (`:736-752`). An initial-state merge with
`pcmsp·pcmsp > 100 GeV²` and `nleft > 4` (before the decrement) boosts every
surviving line into the rest frame of the new spacelike line plus the spectator
beam and rotates the new line onto `+z`; `djb` and `dj` are not invariant, so
every later measure, the core scale included, is taken there. At
`nexternal = 4` the guard never holds, so every 2 → 2 scale is frame-free.

**Termination.** When three lines remain (`:777`), the routine writes a
synthetic vertex `nc = nexternal − 2`: `idacl(nc,·)` are the two beam lines,
`imocl(nc)` the leftover blob, `pt2ijcl(nc) = djb(blob)` (`:799`). If the last
real merge was final-state, `mt2last = sqrt(djb(d1)·djb(d2))` (`:781`) and a
fired boost is undone (`:786-792`); after an initial-state last merge the core
scale stays in the boosted frame. "The 2 → 2 core" is therefore `nc − 1` and
`nc` read together; the stored `nc` is a 2 → 1. Finally, if `this_config` is
among the surviving graphs the list collapses to it
([`:809-817`](https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/cluster.f#L809-L817));
otherwise `igraphs(1)` is the lowest-numbered survivor. Every internal PDG the
walk reads comes from `ipdgcl(·, igraphs(1), iproc)`, so the PDG assignment is
channel-dependent. A 2 → 1 (`nexternal = 3`) short-circuits at `:651-668`.

Banked LHE fields cannot see a wrong tie-break or line PDG that leaves the
printed scale unchanged; only the dump can, and runs without one (the four
`2 → 3` partonic `llj` rows, `pp_to_jj`) are pinned by the printed scale alone
([pp-to-jj-tie-break-no-cluster-dump](../backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md)).

[^n28-k1]: Note 28 §K1–K1.4, read at `b7687064`.
[^n28-k35]: Note 28 §K3.5 (the observation from the dump) and §K4.5 (the explanation).
[^n28-k37]: Note 28 §K3.7.
