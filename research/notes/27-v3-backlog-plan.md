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
