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

#### B1 outcome (2026-08-01) — **the defect is MadGraph's**

The suspected Breit–Wigner normalisation is not it: our map is exactly
measure-preserving, and the π was a coincidence. Both sides were asked for the
*same window* and they agree.

**The window on MadGraph's side.** No run-card cut can express it: `setcuts.f`
applies `mmll`/`mmllmax` to every same-flavour opposite-sign pair
(`s_min(j,i)=mmll*dabs(mmll)` guarded only by `abs(idup(i))==abs(idup(j))` and
opposite sign), so it bites the muons too, and `banner.py` refuses a per-PDG
`mxx_min_pdg` for any lepton ("Can not use PDG related cut for light
quark/b quark/lepton/gluon/photon", pdg 15 among them). The window therefore
goes into `dummy_cuts` (`SubProcesses/dummy_fct.f`), which `passcuts` calls after
every other cut and which leaves MadEvent's phase-space generation untouched —
so the windowed run integrates the same integrand MadGraph already integrated.
Legs 5,6 are the τ pair, asserted against the generated `leshouche.inc`
(`DATA (IDUP(I,1,1),I=1,6)/-11,11,-13,13,-15,15/`), not assumed.
Driver: `validation/madgraph/gen_higgs_window.sh`; result:
`validation/madgraph/higgs_window_reference.json` (committed — a few scalars).

**The measurement.** All with the banked run card (`ptl 10`, `etal 2.5`,
`drll 0.4`, `bwcutoff 15`, `sde_strategy 2`, `e+e-` at 250+250):

| | σ (pb) |
|---|---|
| MG, `m(ττ) ∈ [124.9, 125.1]` | `7.2077e-5 ± 2.94e-7` |
| MG, complement | `1.2965e-3 ± 3.43e-6` |
| **MG, sum** | **`1.36858e-3 ± 3.44e-6`** |
| MG, unwindowed, seeds 20260801/2/3 | `1.3380e-3`, `1.3421e-3`, `1.3322e-3` |
| MG, unwindowed, banked | `1.3373e-3 ± 2.8e-6` |
| ours, production (VEGAS + 25-channel + unweighting) | `1.367e-3 ± 3e-6` |

MadGraph's own partition of its own phase space **exceeds its own unwindowed
integral by 3.1e-5 pb, 7.2σ on its own quoted errors**, and the excess is the
pole. Our σ sits 0.35σ from MadGraph's sum. The unwindowed run is the wrong
number, and its quoted 0.2% error does not cover a 2.3% miss — the
[[seed-sweep-over-fixed-seed-pull]] shape, on MadGraph's side this time: three
fresh seeds agree with each other and with the banked run, all confidently wrong.

That the windowed σ *is* the resonance and not a continuum shoulder: 1898 of
MadGraph's 2000 windowed events lie within ±5Γ_h (31.9 MeV) of 125.000.

**The window on our side**, three maps, none sharing VEGAS or the unweighter:

| map | σ over the window (pb) |
|---|---|
| single Breit–Wigner channel of the one Higgs diagram | `7.2065e-5 ± 3.2e-8` |
| α-adapted 25-channel combiner, flat draws | `7.1948e-5 ± 3.1e-7` |
| flat RAMBO, 2e7 draws | `5.37e-5 ± 1.0e-5` (crude, 1.8σ) |

**Root cause, in MadGraph's integration.** MadGraph **3.5.7** produced every
banked run (the banked banner says `VERSION 3.5.7 2024-11-29`, and it is the
version in the `madgraph` pixi environment). Its `get_channel_cut` in
`genps.f` — the `sde_strategy = 2` multichannel weight, which is the run-card
default — computes a propagator's off-shellness as

```fortran
tmp = (t-Mass)*(t+Mass)                                  ! 3.5.7
get_channel_cut = get_channel_cut* (tmp**2 - tmp2**2)/(tmp**2 + tmp2**2)**2
```

where `t = dot(ptemp(0,-i), ptemp(0,-i))` is *already* `p²` and `Mass` is a mass.
`(t−M)(t+M) = t²−M²` is dimensionally inconsistent and never vanishes on the
pole. The pinned submodule, **3.7.1**, has

```fortran
tmp = (t-Mass**2)                                        ! 3.7.1
get_channel_cut = get_channel_cut/(tmp**2 + tmp2**2)
```

Re-evaluating both expressions over `configs.inc` + `props.inc` at MadGraph's own
on-pole event (`m(ττ) = 124.999999`) gives, for the one config carrying the Higgs
propagator (config 9, `SPROP = 25`):

- 3.5.7: **α₉ = 1.90e-3**, the eight t-channel continuum configs taking 99.8%
- 3.7.1: **α₉ = 0.9999998**

and the windowed run's realised per-channel split confirms the 3.5.7 number:
`G9.*` collect `1.42e-7` of `7.19e-5`, a share of **0.198%** against the
predicted 0.190%. So MadEvent hands 99.8% of the pole to channels whose maps have
no density at a 6.4 MeV structure inside a 500 GeV range; in a windowed run the
cut forces them onto it and they find it, in an unwindowed run they do not.

Confirming experiment: the same unwindowed run with `sde_strategy = 1` (the
amplitude-squared weighting, which does not use the broken expression) gives
**`1.3742e-3 ± 3.86e-6` pb** — 1.1σ from MadGraph's own windowed+complement sum
and 1.5σ from ours, against −7.4σ for the `sde_strategy = 2` default.

The changelog does not announce the fix; the code on both sides is the evidence.

**Why no other row is affected.** Only four banked processes carry a Higgs
propagator at all (`FK_MDL_WH` in `matrix1_orig.f`: this row,
`uux_to_ccx_emmm_qcd0`, `bbx_to_ccx_emmm_qcd0`, and — as an external leg, not a
propagator — `ee_to_tatah`/`ee_to_zh`), and of those only this one has its σ
measured. Every other resonance in the suite is a Z at Γ/m = 2.7%, wide enough
that neighbouring channels' maps cover it. That is why the rest of the σ column
is green.

**Disposition.** Both cells stay ⚠️ `info`, per the plan's MadGraph branch. The
curated notes in `validation/manifest.toml` are rewritten to say which side is
wrong and how it was measured; `validate_sigma`'s falsified "localised at low
m_ll" reason is replaced (the rider); and
`validate_samples::the_higgs_pole_window_is_measured_against_madgraph` makes the
windowed agreement a live measurement against the committed reference rather than
a claim in a note. No tolerance moved, and nothing on this side changed — there
was nothing to fix here.

**Filed for the user / B5, not done here:** the banked reference for this row
(and any future narrow-resonance row) is defective and would need re-banking with
a MadGraph that weights the resonant channel correctly — 3.7.1, or 3.5.7 with
`sde_strategy = 1`. Re-banking changes the pinned reference bundle and the
question of which MadGraph the oracle layer should run, so it is a decision, not
a session task. Until then the `integrals` cell cannot be gated: there is no
correct number to gate against.

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
