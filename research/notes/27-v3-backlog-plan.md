# 27 — `v3-backlog` sprint plan (validation-3 findings burn-down)

**Status: ACTIVE — launched 2026-08-01; decisions D1/D2 resolved (§6).**

The backlog-tackling sprint `validation-3` deferred everything to. Its charter
is the inverse of note 25 §8's: that sprint exposed and recorded; this one
**diagnoses and fixes**, taking note 25 §10's recommended order verbatim. The
standing rule is unchanged: never a loosened tolerance — a cell goes green
because the disagreement is *resolved*, or it stays ⚠️ with a curated note
saying exactly what is unresolved and why.

Scope control still applies in one direction: a *new* finding exposed while
fixing an old one (e.g. `pp_to_bb_fixed`'s first-ever σ measurement missing the
banked value) is recorded as ⚠️ + backlog entry, not chased inside the session
that stumbled on it.

## 1. Inputs

- Note 25 §10 — findings register, final table, recommended order.
- `TODO.md` §Validation backlog — every item below is there with its evidence.
- Branch base: `main` (validation-3 merged; `refdata-2` published and pinned;
  CI `banked` job gating).

## 2. Sessions

Sized for `validation-dev` agents, one session each. Ordering constraint that
matters: **B1 lands before B4's measurements** (a Breit–Wigner-map fix would
move every resonant channel, and Drell-Yan's Z channel is one), and **B3 lands
before B4's measurements** (B3 may change the flow-selection rule the emitted
samples carry). B4's MadGraph banking runs are code-independent and can start
any time. B2 is independent of everything. Worktrees pre-created off `main`
with COW-cloned data, per the worktree-fragility rule.

### B1 — the `h → τ⁺τ⁻` pole bin (`higgs-pole-in-m-tautau`)

The only ⚠️ that is a candidate *defect*: 159% of `ee_to_mumu_tata_qcd0`'s
+2.2% σ offset sits in one 200 MeV bin at the Higgs pole, factor 3.16 at 22σ,
and 3.158 is within errors of π — the first thing to check in a Breit–Wigner
map normalisation (`∫ds/((s−m²)²+m²Γ²) = π/(mΓ)`).

1. **The decisive measurement first**: a dedicated MadGraph run of
   `e+ e- > mu+ mu- ta+ ta- QCD=0` with an `m(τ⁺τ⁻)` window around 125 GeV.
   The session must pin which run-card cut constrains *only* the τ pair on a
   four-lepton final state (the `mmll` family applies to SFOS pairs generally;
   verify against MG's cut code before trusting the card) — a window that also
   bites the μ pair is a different measurement. Run it oracle-layer into the
   work area; it joins the bundle at B5.
2. Measure the same windowed σ on our side (same cut through the cut layer if
   it expresses it, else a high-statistics binned sample) — now the resonance
   is measured directly on both sides and the mis-covering side is identified.
3. Diagnose that side. Ours first: the multichannel BW map's normalisation and
   Jacobian, then the VEGAS `1/σ²` combination on the resonant channel (the
   [[seed-sweep-over-fixed-seed-pull]] failure shape). MadGraph's side is not
   presumed innocent — but its `results.dat` and its events already agree with
   each other, so an MG defect must be found in its *integration*, not its
   sampling.
4. If the defect is ours: fix, then re-run **every** resonant-channel gate —
   the multichannel machinery is shared, so the Z-pole rows, `ee_to_zh`, the
   dy13 σ rows and their seed sweeps all re-measure, and the
   `ee_to_mumu_tata_qcd0` `integrals` cell (+7.65 pull) and `samples` cell
   should both flip. If the defect is MadGraph's: both cells stay ⚠️ with the
   windowed run banked as the standing evidence and the curated note rewritten
   to say which side is wrong and how it was measured.

Rider (assigned here by `TODO.md`): delete the falsified "localised at low
m_ll" string `validate_sigma.rs` still writes into the
`ee_to_mumu_tata_qcd0` row file.

Gate: the windowed σ is measured on both sides; the factor 3.16 has a recorded
root cause; no tolerance moved.

### B2 — `hadronic-shat-floor`

Smallest and most contained. The general hadronic path derives every `ŝ` lower
bound from leptons, so `p p > b b~` gets `shat_min = 0`, an infinite
`ln(1/τ_min)`, and an `x = NaN` PDF call.

- Derive the missing bounds in the cut layer: the back-to-back transverse-
  momentum bound the lepton branch already makes, applied to the `ptb`-cut
  b quarks (`m_bb ≥ 2·ptb`), plus `ŝ ≥ (Σ m_out)²` from the final-state
  masses. Mirror MadGraph's own floor derivation (`setcuts.f` territory)
  rather than inventing one — the bound must match what the banked run
  integrated over.
- Flip `bb_fixed_has_no_shat_floor_for_the_general_path` from measuring the
  failure to asserting the floor.
- Measure `pp_to_bb_fixed`'s `integrals` cell (seed-swept, per the σ
  protocol) and its `samples` cell (3-seed KS/χ² protocol) against the banked
  run. Manifest: both ⛔ → declared measured. A first-time miss is a ⚠️ +
  backlog entry (scope control, §0), with the floor kept.

Gate: two ⛔ cells become measured cells; the NaN path is dead by construction
for any massive or pT-cut final state.

### B3 — the `uux_to_uux` colour-selection rule

The only place validation-3 found where what we hand a shower differs from
what MadGraph hands one: kinematics and helicities agree, realised `ICOLUP`
frequencies do not (99.96% vs 90.4%, χ² 1015/1 dof, seed-stable). Our 90/10
*is* the banked JAMP² ratio, so the rules differ, not the numbers. Candidate:
MadEvent conditions selection on the integration channel's own diagram via
`ICOLAMP`.

1. **Read before deciding**: trace MadEvent's selection end to end in the
   submodule work area (`SELECT_COLOR`, `ICOLAMP`, the channel context it runs
   under) and write the algorithm into this note — the candidate explanation
   is a hypothesis until the code says so.
2. Implement per decision **D1** (recommended: match MadEvent — condition the
   flow draw on the sampled channel's `ICOLAMP`-admitted flows, weights still
   ∝ JAMP² within the admitted set).
3. Re-measure the `samples` category on every coloured row — the rule change
   moves `ICOLUP` frequencies on `gg_to_ttx`, `gg_to_gg` (already at χ² p
   0.003 — it may move either way; measure, don't assume) and the llj row —
   and re-run the Pythia consumption gate. The `leshouche.inc` flow dictionary
   is untouched; only the draw among its entries changes.

Gate: `uux_to_uux` `samples` ⚠️ → settled (✅ if D1 = match and the χ² clears
the floor across 3 seeds; a documented-convention ⚠️ with the MadEvent
algorithm written down if D1 = keep). No other samples cell regresses
unexplained.

#### B3.1 — MadEvent's colour selection, as read (2026-08-01)

Traced in the pinned `research/refs/mg5amcnlo` submodule. Line numbers are that
checkout's.

**The draw.** `SELECT_COLOR(RCOL, JAMP2, ICONFIG, IPROC, ICOL, IVEC)` —
`madgraph/iolibs/template_files/super_auto_dsig_group_v4.inc:1087` for the
subprocess-group export, byte-equivalent in the ungrouped one (an emitted copy
sits at
`tests/input_files/IOTestsComparison/IOExportV4IOTest/export_matrix_element_v4_madevent_nogroup/auto_dsig.f:655`).
Its body is:

1. `cconfig = iconfig`, replaced by the clustering's graph (`igraphs(1)` /
   `vec_igraph(ivec)`) **only when `ickkw > 0`**, i.e. under MLM matching. Every
   run card in `validation/madgraph/output` leaves `ickkw` at its `0` default, so
   `cconfig` is the integration configuration throughout.
2. `nc = int(jamp2(0))` flows. Accumulate `targetamp(i) = targetamp(i-1) +
   jamp2(i)` for each flow `i` with `icolamp(i, cconfig, iproc)` true, and
   `targetamp(i) = targetamp(i-1)` for the rest — a cumulative sum with the
   unadmitted flows masked to zero weight.
3. If `targetamp(nc) == 0` (no admitted flow carries weight at this point), drop
   the mask and re-accumulate over every flow. This is the `is_LC = .false.`
   branch.
4. `xtarget = rcol * targetamp(nc)`, then walk to the first `icol` with
   `targetamp(icol) >= xtarget`. Weights inside the admitted set stay `∝ JAMP2`.

`rcol` is one fresh `ranmar` draw per phase-space point
(`madgraph/iolibs/template_files/auto_dsig_v4.inc:142`), passed into `SMATRIX`,
which calls `SELECT_COLOR` after dividing by `IDEN`
(`matrix_madevent_v4.inc:188`, `matrix_madevent_group_v4.inc:241`). The result
travels to the event through `UNWGT(..., selected_col, ...)`.

So the candidate description is confirmed **for the rule**: MadEvent masks
`JAMP2` with the integration configuration's `ICOLAMP` row and keeps `∝ JAMP2`
within the mask.

**The table.** `ICOLAMP(iflow, iconfig, iproc)` is written by
`get_icolamp_lines` (`madgraph/iolibs/export_v4.py:1295`): `max_Nc` is the
largest `Nc` power over the whole colour basis, and flow `f` is admitted for the
diagram a config maps to exactly when that diagram contributes to `f` with
`Nc` power `max_Nc`. A colourless process gets `.true.` for its one flow at every
config (`export_v4.py:1308`), which is why the rule is a no-op on Drell-Yan
without a special case in the caller.

Configs are the diagrams with no four-point vertex: `get_amp2_lines`
(`export_v4.py:1390`) skips a diagram whose largest vertex-leg number exceeds the
minimum, so `g g > g g`'s four-gluon contact diagram gets no `AMP2`, no config and
no `ICOLAMP` column. In group mode the config list is shared across the group's
subprocesses (`write_coloramps_file`, `export_v4.py:6643`), which is what `iproc`
indexes.

**Where `ICONFIG` comes from.** `common/to_mconfigs/mapconfig, this_config`
(`Template/LO/SubProcesses/genps.f:646`), set once per run at
`genps.f:681-684`: MadEvent integrates **one configuration per `G<n>`
directory**, so every event a `G<n>` writes carries `ICONFIG = n`. There is no
per-event channel draw at all.

**What that means for the config's distribution.** Under multi-channel
(`matrix_madevent_v4.inc:174-185`) channel `j`'s integrand is multiplied by
`AMP2(j) / Σ_i AMP2(i)` — a partition of unity over configs formed from
*per-diagram squared amplitudes*, not from sampling densities. Summing the
channels' event distributions returns `f(x)`, so at a phase-space point `x`

    P(config = j | x) = AMP2_j(x) / Σ_i AMP2_i(x).

That is the distribution MadEvent's colour label is conditioned on, and it is
independent of how each channel samples.

**Confirmed against MadGraph's own files.** `coloramps.inc` regenerated from the
submodule (`generate <proc>; output madevent DIR`, plain Python plus `six`, no
Fortran build) reproduces our `LeadingColorFlows` table row for row and column
for column — `u u~ > u u~`, `g g > t t~`, `g g > g g`, so the diagram order and
the flow order match MadGraph's too. Pinned hermetically by
`vibegraph-lib/tests/color_cf.rs::leading_color_flows_match_madgraphs_coloramps`.

And confirmed against MadGraph's own numbers: `uux_to_uux`'s banked per-config
cross sections are `G1 = 18.49 pb` (the s-channel config, whose `ICOLAMP` row
admits only flow 2) and `G2 = 33400 pb` (the t-channel config, only flow 1), so
the rule predicts flow 1 on `33400/33418.5 = 99.945%` of events. MadGraph's
banked sample writes it on `9996/10000 = 99.96%`. The rule and the observed
frequencies agree.

#### B3.2 — why conditioning on *our* sampled channel does not match, and what does

The premise B3 was written on — that our multichannel's per-event sampled channel
is the analogue of `ICONFIG` — is **false**, and measurably so.

MadEvent's config label is an *amplitude* share, `AMP2_j(x)/Σ AMP2(x)`. Ours is a
*density* share: channel `j`'s term is `f(x)·α_j g_j(x)/g(x)`, so
`P(channel = j | x) = α_j g_j(x)/g(x)`. Both partition the same integral and give
the same σ; they label events completely differently.

Measured on `uux_to_uux` (30k × 5 through the production integrand, the σ gate's
own budget):

| | s-channel | t-channel |
|---|---|---|
| MadGraph per-config σ | 18.49 pb (0.055%) | 33400 pb (99.945%) |
| ours per-channel σ | 1.6553e4 pb (49.6%) | 1.6792e4 pb (50.4%) |

and the reason is sharper than "a different decomposition": for this process the
two per-diagram channel *maps are identical*. Over 2000 accepted points the worst
pairwise relative difference between the two `DiagramChannel` densities is
**0.000e0** — bit-identical — the α-adaptation stays frozen at `[0.5, 0.5]` with
equal variance shares to 15 digits, and the channel index carries exactly zero
information about which diagram produced the point. `g g > g g` is the same: all
four channel densities bit-identical, α frozen at `[0.25; 4]`, per-channel σ
25%/25%/25%/25%. Both processes have only massless propagators, so the
timelike/spine maps degenerate onto the flat one. `g g > t t~` is the exception
(the `173 GeV` top pole makes the t/u maps real): worst pairwise density
difference 0.84, α adapts to `[0.267, 0.364, 0.369]`, per-channel σ 26/37/37%.

Implementing the channel-conditioned rule and measuring it confirms the
consequence: `uux_to_uux`'s `ICOLUP` χ² goes from **1015 → 7268** on one degree
of freedom, our flow-1 share moving 90.4% → 51.0% against MadGraph's 99.96%,
because each degenerate channel deterministically forces its own diagram's flow
and the channels are drawn 50/50.

**What would match.** Draw the configuration per event from
`AMP2_d(x) / Σ_c AMP2_c(x)` — MadEvent's own conditional — and mask `JAMP2` with
that diagram's `ICOLAMP` row. This reproduces MadGraph's `ICOLUP` marginal by
construction and is *independent of our sampler*, which is the right property: the
colour label should not depend on our channel technology. It also handles the
`symfact` folding MadEvent applies to symmetric configs automatically, since the
`AMP2` share is defined before folding. It needs `AMP2_d`, the helicity-summed
squared modulus of each diagram's coherent amplitude, in the production evaluator
— a second folded root beside `Op::Flows`, the per-diagram counterpart of
`eval_jamp2`, with `AMP2` accumulated only over diagrams that would carry a config
(no four-point vertex, matching `get_amp2_lines`). That is new evaluator
machinery, out of B3's scope, and it wants its own oracle against MadGraph's
`AMP2`.

**The same signature elsewhere.** B2 independently found it on
`pp_to_bb_fixed`: the two sub-percent `ICOLUP` flows overproduced by about 3×,
χ² 23–31 on 5 dof. That is the same mechanism seen from the other end — a flow
that only a *subdominant* configuration reaches at leading colour, or that no
configuration reaches except through its `1/N` remainder, takes its full
`JAMP2` share from an unmasked draw and next to nothing from MadEvent's masked
one. It is milder than `uux_to_uux`'s (9.6% against 0.04%, a factor 240) for a
structural reason worth keeping: `g g > b b~`'s s-channel triple-gluon
configuration admits *both* flows, so on that subprocess the mask is inactive
and only the t/u configurations discriminate. The `AMP2_d` fix below addresses
both rows, and reproducing `pp_to_bb_fixed`'s 3× is a sharper acceptance check
on it than `uux_to_uux`'s near-deterministic 99.96% — a mask that is merely
"on" reproduces the latter, while only the right per-configuration weights
reproduce the former. Not measured here: that row is blocked on
`hadronic-shat-floor` in this worktree.

**What landed in B3** (branch `v3b-b3`): the reading above, and the verified half
of the rule — `LeadingColorFlows` (the `ICOLAMP` table off the colour basis,
pinned against MadGraph's generated `coloramps.inc`) and `select_flow_reached_by`
(the mask-plus-fallback draw, pinned on both branches including the `is_LC` one).
Neither is wired into the selection path: the flow draw is unchanged, still
`∝ JAMP2` over every flow, and `uux_to_uux`'s `samples` cell stays informational
with its note rewritten around the measurements above. The follow-up is the
`AMP2_d` accumulator plus the config draw.

### B4 — banking Drell-Yan events (`samples` for the dy13 cards)

Fills the last two `uncovered` cells a run can fill, and restores the
Drell-Yan low-mass spectrum measurement the deleted `dy_dsigma_dmll.md` table
stood in for.

- Oracle-layer MadGraph runs banking **events** for the two committed dy13
  cards — the committed cards with the fetched `lhaid 247000` set, *not* the
  existing banked run's MG-internal `nn23lo1`-at-dynamical-scale configuration,
  which is exactly what made that run's events unusable as a reference. These
  runs can start while B1–B3 are in flight; they join the bundle at B5.
- After B1 and B3 land: measure `pp_to_ll`'s `samples` cell with the standard
  3-seed KS/χ² protocol (and resolve whether `pp_to_ll_qcd0` gets its own cell
  or a covered-by, matching how its other cells point).
- Extend the `samples` binning that currently runs only on
  `ee_to_mumu_tata_qcd0` to bin `dσ/dm_ll` on `pp_to_ll` down to the card's
  threshold — the general path's low-mass Drell-Yan spectrum against
  MadGraph's own events, as a live gate rather than a dead table.

Gate: two `uncovered` cells measured; a Drell-Yan `dσ/dm_ll` spectrum
regenerates from a committed gate again.

### B5 — hygiene riders + `refdata-3` + close-out

One session, because the bundle re-cut must happen exactly once (each re-cut
costs a new archive and a new pin — note 25 §refdata-2).

- **`refdata-3` re-cut**: adds B1's windowed run and B4's two DY event banks
  (plus anything B2/B3 banked). Same verification protocol as `refdata-2`:
  two assemblies byte-identical, all runs' decompressed event text sha256-
  stable through pack/unpack, clean `git archive` export runs the banked layer
  green from the bundle alone. Publish, flip the manifest pin.
- **`validate_madgraph_diagrams` → hermetic**: it reads committed files only
  and runs in 1.5 s; move the registration, flip the manifest tiers back to
  `hermetic`, and the whole `diagrams` column runs on a bare clone.
- **`init-sm-submodule`** calls `vg_ensure_submodule` instead of a bare
  `git submodule update --init`, so a `git archive` export stops failing.
- **`Process::Display`** carries coupling-order constraints (`p p > b b~
  QCD=2` prints as its generate line), so the report's two `pp_to_bb*` rows
  stop reading as the same measurement.
- **`compact_events.py`** per decision **D2** (recommended: delete — note 26
  records the verdict's numbers, git history keeps the script, and a committed
  generator nothing exercises is the known bad shape).
- **Mirror-term bound as a function of ŝ**: measure how the weakest mirror
  term scales with `√ŝ` over the extended ladder and replace the flat `1e-3`
  visibility bound in
  `the_mirrored_beam_ordering_needs_the_reflected_matrix_element` with the
  measured function — the item's own "wanted", not a wider ladder and a
  smaller number.
- Close-out: report re-rendered and asserted with the new cells, `TODO.md` and
  the `validation/madgraph` README updated, this note's close-out section
  written. Checklist for the user: the weekly `schedule` trigger on
  `acceptance.yml` still waits on the first *binary* release tag, not on this
  sprint.

Gate: `pixi run validate` green end-to-end on the new manifest; bundle
round-trip verified; no hygiene item left half-moved.

## 3. What this sprint deliberately does not take

- **`kt-clustering`** and everything ⛔ behind it (6 scale rows, 4 llj partonic
  σ rows, 5 samples cells) — a feature sprint with its own 4-session sketch in
  `TODO.md`.
- **Multi-rung t-channel spine** (and the `uux_to_uux` −0.30% residual it
  explains) — feature backlog, note 21 hand-off design.
- **V7 per-flavour diagram union** — deferred coverage, note 19 §3/§V7 design
  preserved; `diagrams.json` stays counts-only for now.
- **VEGAS first-iteration bias** and the **`w_max` scan budget** — performance
  backlog; B1 may *implicate* the iteration combination on the resonant
  channel, in which case B1 fixes what it must for correctness and leaves the
  budget optimisation where it is.
- **2→6 σ rows** (⏳ oracle layer) — cost, not coverage; performance backlog.

## 4. Sequencing

```
B1 (pole)  ──────┐
B2 (ŝ floor) ────┼──→ B4 measurements ──→ B5 (hygiene + refdata-3 + close-out)
B3 (colour) ─────┘
B4 banking runs (MG wall time, code-independent) — start any time
```

B1/B2/B3 can run in parallel worktrees; B1 and B3 both gate B4's measurement
step (resonant-channel machinery and flow-selection rule respectively).

## 5. Agents

`validation-dev` (Opus) for B1–B4; B5 is Opus too — the bundle re-cut is the
one step where a cheap mistake costs a published pin. Sessions get the
worktree-fragility rule and the B1/B3-before-B4 constraint verbatim.

## 6. Decisions (user, 2026-08-01) — resolved

1. **D1 — colour-selection rule: match MadEvent** — condition the flow draw on
   the integration channel's `ICOLAMP`-admitted flows, weights still ∝ JAMP²
   within the admitted set. (Contingent on B3's reading confirming the
   candidate algorithm; if MadEvent does something else, the session reports
   back before implementing.)
2. **D2 — `compact_events.py`: delete.** Note 26 records the verdict's
   numbers, git history keeps the script.
