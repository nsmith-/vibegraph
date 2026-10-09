---
type: Caveat
title: Rows gated differently from the default, and why
description: "Why some rows report rather than assert a statistic (ee_to_mumua's pull, the 2->6 rows, toy-row AQCDUP, pp_to_jj tie-breaks, info cells); open residuals are linked backlog items."
status: draft
tags: [validation, gating, cross-section, samples, exceptions]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
sources:
  - {id: n21-prod, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/21-resonance-sampling-and-events-plan.md#L300-L511", title: "Note 21 addenda (sampler in production; grid per channel)"}
  - {id: n23-e2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/23-event-output-lhef-plan.md#L281-L343", title: "Note 23 E2 outcome (ee_to_mumua overweight tail)"}
  - {id: n32-s7, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L354-L597", title: "Note 32 waves 2 and §5.1 (the 2→6 rows turned on)"}
  - {id: n32-follow, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/32-perf-addendum-plan.md#L749-L955", title: "Note 32 §5.4 and §7 (heavy-tail falsifier; wide-split mechanisms)"}
  - {id: n34-floor, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/34-draw-followup-plan.md#L56-L250", title: "Note 34 §1.2 and wave 1 (the gate cascade; ee_to_mumua decisions)"}
  - {id: n36-b6, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L458-L524", title: "Note 36 B6 (scale columns; gg_to_gg_cg)"}
  - {id: n36-b8, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36-banked-open-ends-plan.md#L709-L749", title: "Note 36 §7.1 (B8: the fixed-beam scale record)"}
  - {id: n36a, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/36a-seed-headroom-census.md#L140-L190", title: "Note 36a §1c (exemption is load-bearing)"}
  - {id: n37, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/37-madevent-map-survey-and-soft-angle.md#L231-L249", title: "Note 37 §3.2 (ee_to_mumua under the soft-angle rule)"}
  - {id: n38-d, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L497-L881", title: "Note 38 D1, D3, S3 (decays, chains, on-shell veto)"}
  - {id: n38-z2, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/38-process-grammar-sprint-plan.md#L1528-L1686", title: "Note 38 §8.5 (seeded grammar gate)"}
  - {id: n40, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/40-per-group-dynamic-scales.md#L117-L144", title: "Note 40 §5 (σ rows after per-group scales)"}
  - {id: vsigma, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/vibegraph-lib/tests/validate_sigma.rs#L136-L218", title: "validate_sigma.rs PULL_LIMIT, PULL_REPORTED_NOT_ASSERTED, SCALE_FALLBACK_ROWS"}
  - {id: manifest, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/manifest.toml", title: "validation/manifest.toml cell notes"}
---

By default a row's cells gate: the [σ gate](sigma-gate.md) asserts pull and
`rel_tol`, the [samples gate](samples-gate.md) asserts every column against
`P_FLOOR`. This concept collects the rows that depart from that, and the
standing reason each does. It does not track open residuals: those are backlog
items, linked here. A row's current numbers are in its manifest cell note.

## The rules a departure must follow

These come from the gates' own doc comments and from `AGENTS.md`'s validation
section; the exceptions below are worked cases of them.

1. **A disagreement is recorded, never absorbed.** A row that disagrees drops
   to `info` (or `Plan::Info`) with the measurement and its tracking item in the
   note; no tolerance or floor is widened to make it pass.[^vsigma]
2. **A pull is reported, not asserted, only for a systematic of measured
   size**, with `rel_tol` still enforced. A row leaves that list only when its
   residual behaves like Monte Carlo (shrinks with budget, five-seed χ²/dof near
   1). See [the σ gate](sigma-gate.md#when-a-pull-is-reported-not-asserted).
3. **The manifest's mode and the test's mode must agree**; the collator fails
   otherwise ([validation report](validation-report.md)).
4. **A one-seed reading decides nothing.** Each departure below was decided on a
   five-seed (or larger) sweep, and several were triggered by a one-seed
   reading that a sweep then overturned.[^n34-floor] The two defects that
   held the resonant rows back from gating were found the same way: by sweeping
   seeds, not by a fixed-seed pull.[^n21-prod]

## Current departures

| row · cell | treatment | standing reason | tracked as |
|---|---|---|---|
| `ee_to_mumua` · integrals | gate on `rel_tol 0.03`; pull reported | a fixed +1.04% offset, not a spread | [ee-mumua radiative return](../backlog/validation/ee-mumua-radiative-return-sigma-high.md) |
| `ee_to_mumua` · samples | `info` | `pt(a)` measures MadGraph's own sample | same item |
| `uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0` · integrals | `long` / `info` (`Plan::Long`, no `rel_tol`) | heavy-tailed estimator; the sweep licenses no bound | [2→6 integrals not enforced](../backlog/validation/two-to-six-integrals-not-enforced.md) |
| same rows · samples | `long` / `gate` | cost: ~40 min of serial unweighting | [2→6 samples serial cost](../backlog/performance/two-to-six-samples-serial-unweighting-cost.md) |
| six toy rows · samples `AQCDUP` | reported, `SCALUP` gated | the model declares no `aS` | [MadGraph defects](madgraph-defects.md) |
| `pp_to_jj` · scale replay | 9 of 10 000 events admitted by signature | tie-break below printed precision | [jj tie-break, no cluster dump](../backlog/hygiene/pp-to-jj-tie-break-no-cluster-dump.md) |
| `gg_to_gg_cg` · integrals | `info` | converged −0.22% offset, unattributed | [gg_to_gg_cg σ offset](../backlog/validation/gg-to-gg-cg-sigma-offset.md) |
| `gg_to_gg`, `gg_to_gg_cg` · diagrams | `info` | four-gluon contact counted per colour structure by MadGraph | none (a convention) |
| `ee_to_wpwm_cw`, `ee_to_zh_smeft`, `wpwm_to_wpwmz_cw` · amplitudes | `info` | see [non-SM rows](non-sm-rows.md) | [wpwmz five-vector residual](../backlog/validation/wpwmz-cw-ow-five-vector-residual.md) and the non-SM concept |
| `pp_to_ttx_0j1j_mlm` · integrals | `long` / `info` | `@1` reads +1.28%, undiagnosed | [ttx MLM @1 high](../backlog/validation/ttx-mlm-at1-sigma-high.md); [MLM σ gate](mlm-sigma-gate.md) |
| `u u~ > w+ b w- b~ $ t t~` (on-shell veto) | reported | MadGraph's `FFV2P1D_1` defect, then a residual ~2% | [uux wbwb veto 2% off](../backlog/validation/uux-wbwb-onshell-veto-tt-2pct-off.md) |
| `e+ e- > z z, z > e+ e-` (decay chain, `INFORMATIONAL` in `cli_decay_chain.rs`) | reported | we keep both pairings and their interference; MadGraph takes one pairing ÷ 2 | none (a decision; [identical particles across decays](../process/identical-particles-across-decays.md)) |

Every manifest cell declared `mode = "info"` at the time of writing appears
in this table; list the current set with
`awk '/^key = /{k=$3} /mode = "info"/{print k, $1}' validation/manifest.toml`.

## Why each standing reason holds

**`ee_to_mumua`.** Five seeds at the gate budget put this side +1.04% above the
bank at χ²/dof 0.50, so the residual is a fixed offset; the cut-implied
timelike floors move σ by +0.006%, and the soft-angle split rule leaves it at
+0.74% to +1.05%.[^vsigma][^n37] Its pull is +4.11 at five seeds; the gate's own
seed once read +2.83 only because it sat 0.089% below the five-seed mean, which
is how the row passed a one-seed gate until a stream change re-rolled
it.[^n34-floor] Without the exemption the gate fails (worst pull over five
seeds 3.56 > 3.5), which is what keeps the exemption from being
vacuous.[^n36a] Where the offset lives was adjudicated by windowed
re-integration: MadGraph's `pt(a)` and `m(μμ)` partitions of its own run
disagree with each other at 16.7σ, and its `m(μμ)` partition recovers our
total; see [windowed partition closure](windowed-partition-closure.md). The
samples cell is `info` because the `pt(a)` KS flapped across the floor (worst
p 8.3e-6) on an unchanged shape, and MadGraph's banked sample puts 8.73% of σ
below `pt(a) = 20` GeV where its own windowed cross sections put 9.40%: the
column measures the reference's sample.[^manifest] The row also carries the
heaviest unweighting overweight tail of the early rows (an event at 8.4× its
channel's scanned maximum), the same photon-pole region.[^n23-e2]

**The 2→6 rows.** The evaluator is not the obstacle (the manifest records
5.8 µs per integration point at uniform α on `uux_to_ccx_emmm_qcd0`, since
rejected points no longer walk the channels). The channel floor is: 579/615
channels at `MIN_CHANNEL_NEVAL = 512` put about 300 000 evaluations under the
first iteration whatever budget is asked, and the acceptance-corrected floor
(capped at 4×) raises later iterations to about 1.2 million on
`uux_to_ccx_emmm_qcd0`. The flat map is no alternative: six outgoing legs put
the poles on a set of vanishing flat measure, and it returns σ 15–18 orders of
magnitude low (note 32 first measured eleven and fifteen) even though 46% of its
draws pass the cuts. Under the multichannel the five-seed means agree with the
0.30%-precision bank to about 1%, but single seeds swung +4.8% / −4.5% / +3.5%
at every rung of a 300k–1.2M ladder without shrinking: a heavy-tailed
estimator, so no `rel_tol` is licensed. The accepted-point floor since brought
the worst single seed under 1% at one budget, which does not yet show the
swings shrinking with budget.[^n32-s7][^manifest]
A fix must make the single-seed swings shrink with budget; reducing them at
fixed budget is a variance win, not a resolution.[^n32-follow] Their per-iteration
χ²/dof overflows (above 1e250) on wide splits and is passed through as "not a
statistic"; `--target-rel`'s stop no longer consumes it.[^n32-follow] The kT
replay enforces their scales event by event instead
([scale replay](scale-replay-gate.md)).

**Toy-row `AQCDUP`.** For the six toy rows in `UNDECLARED_ALPHA_S_RUNS`
(`ll_to_qqx_toy_*`, `p3r3_to_p3r3_toy_*`, `qqx_to_o8o8_toy_dcolor`), MadGraph's
`export_v4.py` injects `aS = 0.138` beside `G = 4.1643` while `setrun.f` runs
from `G` (`αs(M_Z) = G²/4π ≈ 1.38`): the two halves disagree by a factor of
ten. We build no coupling for a model with no strong interaction and report the
field with that reason; the test asserts the absence of a coupling matches
membership in the list, both ways.[^n36-b8] Every other fixed-beam row, including
α_s-free ones, records the scale its card's prescription produces, as MadGraph
does under `dynamical_scale_choice = -1`, and its `SCALUP`/`AQCDUP` gate; a
fixed-beam row that compiles no prescription fails the samples
gate.[^n36-b8][^manifest]

**`pp_to_jj` tie-breaks.** Nine `q q' → q q'` events in 10 000 have a single
integration channel and two allowed beam-leg pairs; MadGraph inflated the
winning candidate by `√(1 + 10⁻⁶)` and the replay did not. The gate admits an
event only with that exact signature and asserts the count for equality
(`TIE_BREAK_MISSES` in `validate_scales.rs`), so the class cannot become a
different one unnoticed.[^manifest] See [scale replay](scale-replay-gate.md).

**`gg_to_gg_cg`.** The only banked σ at one of `setscales.f`'s closed forms
(MadGraph chose `dynamical_scale_choice = 3` itself). Five seeds are mutually
consistent at χ²/dof 1.01 and sit at −0.22%, a converged offset 2.6× the
reference's error, with a ladder that settles rather than shrinks; `gg_to_gg`
under a card differing in that one field agrees, so the offset is localised to
the operator's coupling under the scale choice. Filed, not gated.[^n36-b6] It
and `gg_to_ttx_smlimit_qcd2` are `SCALE_FALLBACK_ROWS`: their pools are not
monomials in `G`, so each scale change re-evaluates the model (a named cost,
not a gating exception).[^vsigma]

**Seeded references.** Rows read against MadEvent seed sets under the
[seed policy](madevent-reference-seed-policy.md) gate on the mean pull, each
seed's pull and this side's χ²/dof band. A row whose reference is a single
MadEvent run is `covered-by` a seeded row rather than gated, because one run is
not a reference under that policy.[^n38-z2] Where MadEvent's own seeds scatter
far beyond their quotes (`ee_to_ee_nsz`: χ²/dof 20; the unrestricted Bhabha
control: 93), the reference's error is its spread and a pull of ~1.8 is this
side sitting with MadEvent's majority.[^n38-z2] This side's own low-budget tail
on that row is
[ee_ee_nsz low at low budget](../backlog/validation/ee-ee-nsz-sigma-low-at-low-budget.md).
Several decay and chain rows found MadEvent the side that converges with budget
(`h > e+ e- mu+ mu-`, the `cut_decays = T` chain row).[^n38-d]

## Not exceptions any more

- `ee_to_mumu_tata_qcd0` gates on both cells. Its old offset was MadGraph
  3.5.7's `sde_strategy = 2` channel-weight defect at the narrow Higgs pole; the
  3.7.1 references fixed it ([MadGraph defects](madgraph-defects.md)).
- The llj partonic rows `gu_to_epemu` and `gux_to_epemux` left the
  reported-pull list once each point's configuration was drawn from its own
  `AMP2`; their residual is Monte Carlo.[^vsigma]
- Per-group dynamic scales moved `pp_to_llj_dyn` and `pp_to_llj` onto
  MadGraph; no `rel_tol` or mode changed.[^n40]

[^n21-prod]: Note 21 addenda.
[^n23-e2]: Note 23 E2 outcome.
[^n32-s7]: Note 32 §5.1, S7.
[^n32-follow]: Note 32 §5.4 and §7.
[^n34-floor]: Note 34 §1.2, the gate cascade and its three decisions.
[^n36-b6]: Note 36 B6 item 4.
[^n36-b8]: Note 36 §7.1 (B8).
[^n36a]: Note 36a §1c.
[^n37]: Note 37 §3.2.
[^n38-d]: Note 38 D1, D3 and S3 outcomes.
[^n38-z2]: Note 38 §8.5.
[^n40]: Note 40 §5.
[^vsigma]: `vibegraph-lib/tests/validate_sigma.rs`, `PULL_REPORTED_NOT_ASSERTED`, `SCALE_FALLBACK_ROWS`, `Plan`.
[^manifest]: `validation/manifest.toml`, the named rows' cell notes.
