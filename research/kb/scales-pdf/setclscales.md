---
type: Algorithm
title: "setclscales: from the cluster sequence to μR and per-beam μF"
description: "reweight.f's walk (ipart, jfirst/jlast/jcentral), mt2last/jcentral overrides, the μR geometric mean, per-beam μF branches, where scalefact lands on 3.7.1, and what SCALUP holds."
status: draft
tags: [scales, kt-clustering, setclscales, madgraph, scalefact]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n28-k15, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L755-L839", title: "Note 28 §K1.5 (cluster sequence to jfirst/jlast/jcentral)"}
  - {id: n28-k16, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L840-L910", title: "Note 28 §K1.6 (μR, the geometric-mean prescription)"}
  - {id: n28-k17, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L911-L962", title: "Note 28 §K1.7 (per-beam μF)"}
  - {id: n28-k19, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1097-L1145", title: "Note 28 §K1.9 (where scalefact lands, against 3.7.1)"}
  - {id: n28-k111, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L1268-L1298", title: "Note 28 §K1.11 (findings for downstream sessions)"}
  - {id: n28-k37, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2220-L2244", title: "Note 28 §K3.7 (confirmed against the bank)"}
  - {id: n28-k42, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/28-kt-spine-feature-sprint-plan.md#L2452-L2478", title: "Note 28 §K4.2 (what replaced the closed forms)"}
  - {id: mg-reweight, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/reweight.f#L555-L1284", title: "MadGraph reweight.f setclscales (3.7.1)"}
  - {id: mg-unwgt, resource: "https://github.com/mg5amcnlo/mg5amcnlo/blob/b7687064b9a013317ca164aa1395bc9c0e39ae1e/Template/LO/SubProcesses/unwgt.f#L751-L758", title: "MadGraph unwgt.f (SCALUP)"}
  - {id: setcl-rs, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/src/coupling/cluster/setclscales.rs#L1-L40", title: "vibegraph setclscales.rs module documentation"}
  - {id: vs-scalefact, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_scales.rs#L322", title: "validate_scales.rs SCALEFACT_RUNS"}
---

# `setclscales`: from the cluster sequence to μR and per-beam μF

For `dynamical_scale_choice = -1` MadGraph clusters each event into a merge sequence
([scales-pdf/kt-clustering-algorithm](kt-clustering-algorithm.md)) and then
`setclscales` (`reweight.f:555-1284`, MadGraph 3.7.1) decides which vertices the scales
are read off.[^mg-reweight] vibegraph ports it in
`vibegraph-lib/src/coupling/cluster/setclscales.rs`;[^setcl-rs] the engine and its wiring are
[scales-pdf/kt-clustering-engine](kt-clustering-engine.md), and the choices other than
`-1` are [scales-pdf/madgraph-scale-choice](madgraph-scale-choice.md). This concept is
the unmatched walk; what changes under `ickkw = 1` is [scales-pdf/mlm-scales](mlm-scales.md).
Line numbers are `Template/LO/SubProcesses/reweight.f` at the pinned 3.7.1 tree.

## The walk: `jfirst`, `jlast`, `jcentral`

Every question the walk asks is answered by `ipdgcl(·, igraphs(1), iproc)`, the PDG
codes the merge graph puts on each line.[^n28-k15]

**Line provenance (`ipart`).** `:720-727` seeds `ipart(1, 2^(i-1)) = i` and calls
`ipartupdate` (`:224-441`) for `n = 1 … nexternal−3`, not the terminal vertex.
`ipart(1, mask)` is the beam number for a t-channel line and the hardest constituent for
an s-channel line; `ipart(2, ·)` is the softer partner of a `g → qq̄`, octet, sextet or
singlet splitting. `ipartupdate` also **rewrites** `ipdgcl(imo)` on jet lines, and
`stop 3`s on a colour structure it does not recognise: an honest signal that a process
is outside the algorithm.

**Beam-side state** (`:730-754`), per beam `i`:

```text
ibeam(i)      = 2^(i-1)        advanced to the mother at each initial-state merge
jfirst(i)     = 0              first initial-state splitting on side i
jlast(i)      = 0              last IS vertex at which side i was still a *parton* line
jcentral(i)   = 0              last IS vertex at which side i was still *QCD*
qcdline(i)    = isqcd(pdg(beam i))          |colour| > 1
partonline(i) = qcdline(i)
goodjet(beam i) = partonline(i)
goodjet(leg)  = isjet(pdg(leg))             legs 3..nexternal
```

`isjet(pdg)` is `|pdg| ≤ maxjetflavor` or a gluon. At the default `maxjetflavor = 4`,
**`b` and `t` are not jets**, which drives the `p p → b b̄` branch.[^n28-k111]

**Initial-state step** (`:766-830`), when some `idacl(n, i) = ibeam(j)`. At the terminal
vertex (`n = nexternal−2`) the line continuing past the vertex is the *other beam's*
line. Then:

```text
if partonline(j):  if jfirst(j)==0: jfirst(j)=n
                   jlast(j) = n
                   partonline(j) = goodjet(ida(3-i)) .and. isjet(pdg(imo))
elif jfirst(j)==0: jfirst(j)=n ; goodjet(imo)=.false.
else:              goodjet(imo)=.false.
if qcdline(j):     jcentral(j) = n ; qcdline(j) = isqcd(pdg(imo))
```

`jlast` and `jcentral` are assigned **before** the predicate that turns the flag off, so
both name the last vertex at which the line was still good, inclusive.

**Final-state step** (`:831-950`): `isjetvx`, jet tagging, `goodjet` propagation and
the `g → gg` `ipart` repair. It never moves `jfirst`/`jlast`/`jcentral`; it sets
`iqjets` (read by `xqcut` and `ptclus`) and `goodjet`. Afterwards `jfirst(j) ≤ 0` is
replaced by `jlast(j)` (`:970-971`).

**The `njetstore` jet memo** (`:985-1030`) makes the scale depend on the integration
channel as well as the momenta: the first event of a channel is clustered restricted to
`iconfig` and its jet count stored, and a later event whose unrestricted count differs is
re-clustered restricted (`stop 4` if that fails too). The port stores *this event's*
restricted count. That equals MadEvent's memo on every event, not only a channel's first,
because the restricted count was measured to be a single value per channel on every
channel of the five MLM reference rows. The other channel dependences
(`filmap`'s `nqcd(this_config)` filter, `checkbw`'s `this_config`) are live on the bank
too; see [scales-pdf/cluster-scale-channel-dependence](cluster-scale-channel-dependence.md).[^n28-k37]

## Rewrites before the formulas

1. **`mt2last`** (`:1034-1045`). Fires only when `mt2last > 4`, `nexternal > 3`,
   `jlast(1) = jlast(2) = nexternal−2`, and all three PDGs of the last *real* merge are
   `isqcd`. Then `mt2ij(nexternal−2) = mt2ij(nexternal−3) = mt2last`, the geometric mean of
   the two daughters' squared transverse masses, set only when the last real merge was
   final-state (`MT2LAST_FLOOR = 4.0` in `setclscales.rs`).[^n28-k16]
2. **Central vertex** (`:1048-1055`). If `jcentral(j) > 0` and `mt2ij(jcentral(j)) > 0`,
   then `pt2ijcl(jcentral(j)) = mt2ij(jcentral(j))`. Where a colour line ends on an
   initial-state vertex, that vertex takes the emitted leg's transverse mass instead of
   the merge measure.
3. **The `jfirst` floor** (`:1109-1112`), after the early return at `:1103-1106`
   declines: `pt2ijcl(jlast(j)) = max(pt2ijcl(jlast(j)), pt2ijcl(jfirst(j)))`. This is how
   a clustering tie-break inflation reaches the final scale: `u ū → u ū`'s 16 inflated
   events land at exactly `μR = μF = 250.000125` (`the_general_path_keeps_the_beam_crossing_population`).[^n28-k37]

## μR: the geometric mean

`:1150-1174`, guarded by `scale == 0`, so a fixed `μR` never enters, while a fixed `μR`
with a dynamic `μF` still runs everything above. The branch is the first that holds:

| condition | line | `μR` |
|---|---|---|
| `jlast(1) > 0` and `jlast(2) > 0` | `:1153-1154` | `(pt2(jlast₁)·pt2(jcentral₁)·pt2(jlast₂)·pt2(jcentral₂))^⅛` |
| `jlast(1) > 0` | `:1157` | `(pt2(jlast₁)·pt2(jcentral₁))^¼` |
| `jlast(2) > 0` | `:1160` | `(pt2(jlast₂)·pt2(jcentral₂))^¼` |
| `jcentral(1) > 0` and `jcentral(2) > 0` | `:1163` | `(pt2(jcentral₁)·pt2(jcentral₂))^¼` |
| `jcentral(1) > 0` | `:1165` | `√pt2(jcentral₁)` |
| `jcentral(2) > 0` | `:1167` | `√pt2(jcentral₂)` |
| otherwise | `:1169` | `√pt2(nexternal−2)` |

then `scale = scalefact·scale` (`:1171`) and `G = √(4π·αs(scale))`. Every row is the
geometric mean of the participating **linear** scales.[^n28-k16]

## Per-beam μF

There is one factorisation scale per beam, `q2fact(1)` and `q2fact(2)`, each with its own
`fixed_fac_scale` guard:[^n28-k17]

1. `:1121-1124`: `nexternal = 3` with two incoming legs; both beams take `pt2ijcl(nexternal−2)`.
2. `:1126-1137`, the main branch, entered when either `q2fact` is still zero:
   `q2fact(j) = √(pt2(jlast_j)·pt2(jcentral_j))`, so `μF_j = (pt2_jlast·pt2_jcentral)^¼`.
   If `jcentral(1) = jcentral(2) > 0` and neither beam is fixed, both take the larger
   ("a QCD line through the whole event, use a single scale"). That is why both beams of
   `p p → ℓℓ` carry one number.
3. `:1138-1147`: `scalefact²` and the `q2bck` back-up.
4. `:1180-1194`, after `μR`, the `jcentral = 0` fill-ins. With both zero and both `q2fact`
   still zero, each beam takes `scalefact²·pt2ijcl(nexternal−2)`; with one zero, that beam
   takes `scalefact²·pt2ijcl(jfirst)`. The sub-branches that only back-fill `pt2ijcl` are
   unreachable for a free beam under `-1`.
5. `:1206-1220`: a beam carrying a PDF whose dynamic `q2fact < 4 GeV²` rejects the event
   ([scales-pdf/factorisation-scale-floor](factorisation-scale-floor.md)).

`-1` with exactly one fixed factorisation scale is refused
(`ScaleError::MixedFixedFactorisationScales`): the guard at `:1138` reads
`.not.fixed_fac_scale1.or.fixed_fac_scale2`, which is false when beam 1 is fixed and
beam 2 dynamic, so beam 2 then misses its `scalefact²` and its `q2bck` back-up; no
reference run exercises either mixed case.

## Where `scalefact` lands on 3.7.1

On the pinned 3.7.1, `μR` and each `μF` carry **exactly one** power of `scalefact` in
every reachable branch:[^n28-k19]

| path | where it enters | `μR` | `μF(1)` | `μF(2)` |
|---|---|---|---|---|
| `fixed_ren_scale` | never | 0 | — | — |
| `fixed_fac_scaleN` | guarded | — | 0 | 0 |
| choices 1–5 | `setscales.f:93`, then squared into `q2fact` | 1 | 1 | 1 |
| `-1`, `μR` | `:1171` | 1 | — | — |
| `-1`, main `μF` branch | `:1139-1140` | — | 1 | 1 |
| `-1`, both `jcentral` zero | `:1188-1189` | — | 1 | 1 |
| `-1`, one `jcentral` zero | `:1192` / `:1194` | — | 1 | 1 |

MadGraph 3.5.7 built beam 2 in the both-zero branch from an already-scaled `q2fact(1)`
and so applied the factor twice; 3.7.1 builds it from `pt2ijcl(nexternal−2)`. vibegraph
follows 3.7.1: `scalefact_reaches_every_scale_exactly_once` (`coupling/scales.rs`)
asserts one power on that branch.[^n28-k42] The placement is pinned by reference data:
`pp_to_ll_scalefact2` is a banked MadGraph run at `scalefact = 2.0`, replayed by
`validate_scales` (`SCALEFACT_RUNS`).[^vs-scalefact] The module doc of
`validate_scales.rs` (lines 46-48) still says every banked run has `scalefact = 1` and
that MadGraph applies it twice in one place; both halves are stale
([backlog: validate-scales-module-doc-stale](../backlog/hygiene/validate-scales-module-doc-stale.md)).

## What the record carries

`SCALUP` is not `μR`. `unwgt.f:752` writes `√max(q2fact(1), q2fact(2))`, the larger
per-beam factorisation scale; `<rscale>` carries `μR` and `<pdfrwt beam="j">` carries
`√q2fact(j)`.[^mg-unwgt] The record conventions, including AQCDUP, are
[scales-pdf/record-scales](record-scales.md).

## The general path replaced the closed forms

The scale is computed by the general walk for every `-1` process; the input is a
`ClusterInput` (the process's `ChannelSet`, a `ColorTable`, the integration channel and
the subprocess) that `hadronic::compile_scale_source` builds from the diagrams, so no
integrand carries a table keyed by process name. Against the closed forms it superseded,
8 of 14 runs agreed to `1.1e-16` or better; on `pp_to_bb*` they differed by `1e-9` to
`1.5e-6`, because the first merge is initial-state and the leftover leg's measure is
taken in the boosted frame. There the general path is the one pinned bit-for-bit
against MadGraph's own intermediates, so the closed form was the approximation.[^n28-k42]
The per-event gate is [validation/kt-cluster-dump-oracle](../validation/kt-cluster-dump-oracle.md).

## Facts from the reading that a port must keep

- MadGraph's merge graph drops four-point vertices (`export_v4.py:2193-2197`); a merge
  map derived from this crate's diagrams must drop any diagram whose largest vertex
  arity exceeds the process minimum.
- `Template/LO` never assigns the clustering parameter `D`; it arrives from the hidden
  run-card parameter `d = 1.0`.
- The guard `A .or. B .and. .not.(A .and. B)` in the mass propagation
  (`cluster.f:731-733`, `kin_functions.f:456-457`) is dead code; the port implements
  `A .or. B`, confirmed on every `2 → 6` event.
- `clusinfo` is gated on `ickkw ≠ 0` (`unwgt.f:838`), so an unmatched banked LHE has no
  `<clustering>` tag, and instrumentation is the only route to the merge sequence.
- `ipartupdate`'s in-event rewrite of `ipdgcl` is visible: a beam line that emitted a
  gluon carries the mutated jet flavour.[^n28-k111][^n28-k37]

[^n28-k15]: Note 28 §K1.5, the walk and the jet memo, read off `reweight.f:708-1030`.
[^n28-k16]: Note 28 §K1.6, the two rewrites and the `μR` branches.
[^n28-k17]: Note 28 §K1.7, the per-beam `μF` branches and `SCALUP`.
[^n28-k19]: Note 28 §K1.9, `scalefact` placement corrected against 3.7.1.
[^n28-k111]: Note 28 §K1.11, findings for downstream sessions.
[^n28-k37]: Note 28 §K3.7, confirmed against the bank.
[^n28-k42]: Note 28 §K4.2, the general path against the closed forms, and the deletion of the 3.5.7 double factor.
[^mg-reweight]: MadGraph `reweight.f`, `setclscales`.
[^mg-unwgt]: MadGraph `unwgt.f`, the event line.
[^setcl-rs]: `vibegraph-lib/src/coupling/cluster/setclscales.rs` module documentation.
[^vs-scalefact]: `vibegraph-lib/tests/validate_scales.rs`, `SCALEFACT_RUNS = [("pp_to_ll_scalefact2", 2.0)]`.
