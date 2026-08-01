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

**Upstream provenance (post-session archaeology).** The fix is mg5amcnlo commit
`286feb8e606a4e55951f6ea10ea0e3d145213b13` (Olivier Mattelaer, 2025-01-27,
*"change sde_strategy2 to avoid negative weights"*). It changes both
`get_channel_cut` branches: the spacelike one
`/((t-Mass)*(t+Mass)+stot*1d-10)**2` → `/(t-Mass**2+stot*1d-10)**2`, and the
resonant one from `tmp = (t-Mass)*(t+Mass)` with weight
`(tmp²−(MΓ)²)/(tmp²+(MΓ)²)²` to `tmp = (t-Mass**2)` with a plain Breit–Wigner
`1/(tmp²+(MΓ)²)`. The commit title names the second defect the expression fix
alone would have exposed: with the corrected `tmp`, the old numerator
`tmp²−(MΓ)²` is *negative* within one width of the pole, so the functional form
had to change too. First released in **3.6.2** (absent from 3.6.0/3.6.1,
present in all 3.7.x); **never backported to the 3.5.x LTS line** — 3.5.16, the
latest, still carries `(t-Mass)*(t+Mass)` — so no 3.5.x re-run can be a valid
narrow-resonance reference at `sde_strategy = 2`.

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

#### B2 outcome — ✅ closed, branch `v3b-b2`

**What MadGraph actually derives.** `setcuts.f:527-707` builds `smin` letter
class by letter class. Each class accumulates `smin_p = Σᵢ max(e_X, pt_X, …)`
over its legs and `smin_m = −Σᵢ mᵢ² + n(n−1)/2 · mm_XX²` over its pairs, takes
`max(smin_p², smin_m, class extras)`, and *adds* the classes; line 707 then
raises the total to `max(smin, (Σ pmass(i))², dsqrt_shat²)`. `genps.f:274`
passes `smin/stot` to `sample_get_x` as `τ_min`, so this is the same quantity
`Cuts::shat_min` feeds the `(τ, y)` map. For the banked `pp_to_bb_fixed` card
(`ptb = 20`, `eb = mmbb = dsqrt_shat = 0`, `mb = 4.7`, `maxjetflavor = 4` so
both b quarks are the `b` letter) only the b class fires:
`max(40², −2·4.7², 0) = 1600`, then `max(1600, (2·4.7)², 0) = 1600 GeV²`.

**What was implemented.** Not the per-class sum, which is a heuristic, but the
two bounds behind it, taken at once over the whole final state. In the partonic
centre of mass `√ŝ = Σᵢ Eᵢ`; a boost along the beam leaves each leg's transverse
momentum alone, so the lab-frame `pT` a cut holds a leg above also bounds that
leg's energy in that frame, and `Eᵢ ≥ max(mᵢ, pTᵢ)` gives

```
√ŝ  ≥  Σᵢ pTᵢ^min        and        √ŝ  ≥  Σᵢ mᵢ
```

for **any** number of outgoing legs — no back-to-back step, no two-body
assumption. `shat_min` is the max of those two, `dsqrt_shat²`, and the existing
`mmll²` term. Summing the transverse threshold over all classes at once equals
MadGraph's value when one class is cut and is tighter when several are, and it
is sound by the derivation, so it can only ever be at or above MadGraph's floor
while still never exceeding a surviving point's own `ŝ`.

The old narrow branch — "exactly two final legs, both leptons, `(2·ptl)²`" —
is subsumed exactly: two leptons give `Σ pTᵢ^min = 2·ptl`.

**Nothing else moved, by arithmetic and not by hope.** dy13 default and window:
`Σ pT = 2·10 = 20` reproduces the old `(2·ptl)² = 400`, and `mmll² = 3600`
still dominates the window card. llj: the new `(2·10 + 20)² = 1600` sits under
its `mmll² = 2500`, so `τ_min` is unchanged and the row is bit-identical. Both
were then re-run end to end (below).

**Measurements.**

| | |
|---|---|
| `shat_min` for `pp_to_bb_fixed` | **1600 GeV²**, asserted equal to the `setcuts.f` value, `ln(1/τ_min) = 11.568` |
| σ, three seeds at 300k × 10 | **2 145 255 ± 961 pb** vs MG **2 145 500 ± 3 414 pb** — rel **−0.011%**, pull **−0.07**, χ²/dof **0.51** |
| budget ladder (3-seed mean rel) | −0.07% @75k, +0.04% @150k, −0.01% @300k, −0.03% @600k, −0.00% @1.2M — flat, so the sweep measures agreement and not convergence |
| seed scatter over the ladder | χ²/dof 0.48, 1.67, 0.51, 0.96, 0.91 |
| samples, three seeds × 20 000 events | kinematics min KS p **9.7e-3**; `SPINUP` χ² p **0.57–0.78**; flavour χ² p **0.31–0.46**; **`ICOLUP` χ² 23–31 / 5 dof, p 1.0e-5–3.0e-4** |

**One new finding, filed not chased** (scope control, §0): the `ICOLUP` column.
The excess is entirely in the two sub-percent flows — MadGraph writes `0.07%`
and `0.08%` of its events there against our `0.23%` and `0.25%` — while the two
dominant flows agree to about a percent of themselves. Same shape and same
direction as `uux_to_uux`: MadEvent concentrates on the flows the integration
channel's own diagram admits, we spread `∝ JAMP2`. **B3's D1 change should move
this row**, so it is worth re-measuring inside B3 rather than diagnosing on its
own. It is not an integration defect — σ agrees at −0.01% and the ŝ floor cannot
reach the colour draw.

Cells: `integrals` ⛔ → **banked/gate**, `samples` ⛔ → **banked/info**.

**For B5.** The `pp_to_bb_fixed` `samples` cell is a candidate to flip to `gate`
after B3 lands; nothing else here needs re-cutting the bundle.

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

- **First: the oracle toolchain moves to MadGraph 3.7.1** (decision D3). The
  banking runs must come from the pinned submodule's 3.7.1 — B1 proved 3.5.7's
  `sde_strategy = 2` channel weight defective at narrow poles, and while the Z
  at Γ/m = 2.7% is far from that regime, banking new references on a
  known-defective version would build the next `refdata` on sand. Verify
  `VERSION 3.7.x` in the produced banner, and record the mechanism (submodule
  `bin/mg5_aMC` needs only plain Python + `six` to generate; `madevent` builds
  with the pixi env's gfortran) so B5 reuses it for the full re-bank.
- Oracle-layer MadGraph runs banking **events** for the two committed dy13
  cards — the committed cards with the fetched `lhaid 247000` set, *not* the
  existing banked run's MG-internal `nn23lo1`-at-dynamical-scale configuration,
  which is exactly what made that run's events unusable as a reference. They
  join the bundle at B5.
- After B1 and B3 land: measure `pp_to_ll`'s `samples` cell with the standard
  3-seed KS/χ² protocol (and resolve whether `pp_to_ll_qcd0` gets its own cell
  or a covered-by, matching how its other cells point).
- Extend the `samples` binning that currently runs only on
  `ee_to_mumu_tata_qcd0` to bin `dσ/dm_ll` on `pp_to_ll` down to the card's
  threshold — the general path's low-mass Drell-Yan spectrum against
  MadGraph's own events, as a live gate rather than a dead table.

Gate: two `uncovered` cells measured; a Drell-Yan `dσ/dm_ll` spectrum
regenerates from a committed gate again.

#### B4 outcome (2026-08-01) — ✅ closed, branch `v3b-b4`

Both cells filled, the spectrum gate is live, and the session found one defect —
in this crate, not in MadGraph.

**The 3.7.1 mechanism (B5 reuses this verbatim).** `validation/madgraph/mg5_pinned.sh`
runs the pinned submodule's generator in place of whatever `mg5_aMC` is on PATH;
`gen_hadronic_sigma.sh` calls it, and `build.sh` still does not (the rest of the
work area is B5's to re-bank). Everything runs under `pixi run -e madgraph`,
which supplies the Python 3.11 + `six` the submodule needs to *generate* and the
gfortran + LHAPDF that madevent needs to *build and run*; the packaged
mg5amcnlo 3.5.7 in that environment is never invoked. Four details the wrapper
handles so a caller does not repeat them:

- MadGraph drops a scratch `py.py` into its working directory, so it runs in a
  temporary one and the repository stays clean. The submodule itself is untouched
  — `git status` in it is empty after a generation, and no
  `input/mg5_configuration.txt` is created.
- `LHAPDF_DATA_PATH` puts `validation/pdf/` ahead of the installed data
  directory, so `lhaid 247000` resolves to the set this repository pins rather
  than one MadGraph downloads. The installed directory stays on the path for
  `lhapdf.conf` and the set index.
- `set automatic_html_opening False --no_save` and `set notification_center
  False --no_save` are prepended to every script. The pinned checkout carries no
  site configuration, so both default to **on**, and a batch generation opens a
  browser tab and posts a desktop notification per process directory. The
  packaged 3.5.7 had them off in its own
  `.pixi/envs/madgraph/MG5_aMC/input/mg5_configuration.txt`, which is why no
  existing script needed this.
- Those two `set` lines cover the *generation* only. `--no_save` keeps them out
  of the submodule, and — verified, not assumed — also out of the generated
  `Cards/me5_configuration.txt`, which still ships them commented out at their
  `True` defaults. So the later `generate_events` is a second place to silence:
  `silence_madgraph_ui` in `gen_hadronic_sigma.sh` rewrites that file before
  every run, which also fixes a process directory generated before the wrapper
  existed.

Verified in the produced banner: `#*  VERSION 3.7.1  2026-04-29  *` in both
`Events/*/run_*_tag_1_banner.txt`, against `VERSION 3.5.7 2024-11-29` in every
banked run of `refdata-2`.

**Sanity anchor — the σ that 3.7.1 produces agrees with the 3.5.7 reference**,
which is what B1 §"why no other row is affected" predicted for a Z at
`Γ/m = 2.7%`:

| card | 3.5.7 (committed) | 3.7.1 (this session) | pull |
|---|---|---|---|
| `dy13_default` | `933.11 ± 0.447` pb | `933.23 ± 0.480` pb | `+0.18σ` |
| `dy13_mmll_60_120` | `644.42 ± 0.315` pb | `644.33 ± 0.283` pb | `−0.21σ` |

No finding. `hadronic_sigma_reference.json` was **restored to the committed
3.5.7 numbers** — re-banking the reference is B5's step under D3, and the σ gate
and the samples gate agree either way.

**Two run-card changes, both forced and neither physical.**

`False = use_syst` is now in both `dy13` cards. MadGraph's systematics pass
appends one `<wgt>` per scale and PDF-error variation to every record, and with
`--pdf=errorset` on NNPDF23 that is a few hundred; at the cards' 200000 events
the unweighted file is **4.07 GB of text, 193 MB gzipped**, which no gate can
read into memory and no bundle can carry. With the pass off the same run writes
**20 MB gzipped**. It changes nothing else: σ is bit-identical
(`0.93323E+03 ± 0.47991E+00` before and after), and the first event's momenta,
colours, helicities and `XWGTUP` are identical to the last digit — the
systematics pass only rewrites the file through `lhe_parser`, which is also why
the formatting differs between the two.

**The defect: a sample's cross section was read without looking at `IDWTUP`.**
The first `dσ/dm_ll` run reported MadGraph's spectrum a factor `2.0e5` below
ours, uniformly across every bin — a constant ratio, so a normalisation and not
a shape. `EventSample::from_lhe` took σ as the **mean** of `XWGTUP`, correct
under `IDWTUP = -4`; these files carry `-3`, where the **sum** is σ, and
`2.0e5 ≈ 200000` is the event count.

Which of the two MadGraph writes is a property of the **run card**, not of the
version. `RunCardLO`'s `event_norm` is declared

```python
self.add_param("event_norm", "average", allowed=['sum','average','unity'],
               include=False, sys_default='sum', hidden=True)
```

(`madgraph/various/banner.py:4298`), and `sys_default` is documented at
`banner.py:2846` as "default used if the parameter is not in the card". MadGraph's
own full cards name `event_norm` — `pp_to_llj_fixed/Cards/run_card.dat:195` says
`average = event_norm` — so every banked run in `refdata-2` is `-4`. The `dy13`
cards are hand-written and minimal, never mention it, and therefore get `sum`.
`lhe_parser.py:517-526` turns that into strategy `3` (signed to `-3`), and
`madevent_interface.py:3885` is where the run card's value is handed over.

So the blind spot was structural: the only files the reader had ever seen were
the ones its assumption was true for, and every existing `samples` cell is blind
to it — KS is a statement about two cumulative distributions and χ² about
category frequencies, both invariant under rescaling one sample's weights. The
`dσ/dm_ll` gate is the first comparison in the category that is not, and it found
it on its first run. `from_lhe` now dispatches on the field, takes `XSECUP` under
`+3`, and **panics on an `IDWTUP` it does not know** rather than guessing; three
unit tests in `validation/samples.rs` pin all three branches, and
`WeightStrategy` gained a named `-3`. No existing row moved: all of them are
`-4`.

**The second finding: 3.7.1 changed how it writes the colour matrix.** This one
is scope the session did not ask for, taken because the alternative was a red
branch — the `color_cf_oracle` gate sweeps *every* `matrix1_orig.f` under the
work area, so the two new process directories joined it and failed with
"CF matrix has unfilled entries". 3.5.x emits a square array of reals,

```fortran
REAL*8 CF(NCOLOR,NCOLOR)
DATA (CF(I,  1),I=  1,  6) /3.166666666666667D+00, -3.333333333333333D-01, .../
```

and 3.7.1 emits integers over one common denominator, storing only the upper
triangle:

```fortran
INTEGER CF(NCOLOR*(NCOLOR+1)/2)
DATA DENOM/6/
DATA (CF(I),I=  1,  6) /19,-4,-4,-4,-4,8/
```

Its contraction runs `DO I = 1, NCOLOR; DO J = I, NCOLOR` with a single running
index, so each unordered pair is visited once; `MATRIX1` is `REAL*8` and takes
the real part, and `Re[c·a·conj(b)] = Re[c·b·conj(a)]` for real `c`, so an
off-diagonal entry must carry **twice** the symmetric matrix's value:

```text
CF(I,I) = packed / DENOM        CF(I,J) = CF(J,I) = packed / (2 DENOM)
```

Confirmed element for element against the square form on three processes
regenerated with the pinned MadGraph — `u u~ > u u~` (`NCOLOR = 2`,
`DENOM = 1`, packed `9,6,9` against `[[9,3],[3,9]]`), `g g > t t~`
(`NCOLOR = 2`, `DENOM = 3`, `16,-4,16` against `[[16/3,-2/3],[-2/3,16/3]]`) and
`g g > g g` (`NCOLOR = 6`, `DENOM = 6`, 21 packed integers against all 36 square
entries). The parser now takes either form. The live sweep only reaches the
packed one at `NCOLOR = 1`, where the factor of two is never exercised, so a
trial carrying both forms of `g g > t t~` verbatim pins it.

**This is the shape of every 3.7.1 re-bank.** Nothing about it is specific to
Drell-Yan: when B5 re-banks the rest of the work area, every `matrix1_orig.f` in
the bundle changes to the packed form at once, and any other gate that reads
generated Fortran should be checked the same way before the re-cut rather than
after — `gen_amplitude*.py`'s f2py path and `dy_integrand_oracle` both compile
these files.

**What is measured now.** `pp_to_ll` `samples` is `banked`/`gate`, one cell per
card, 200000 MadGraph events against 3 × 20000 of ours at the σ gate's own
`120000 × 12` budget:

| card | min KS p | min χ² p | σ (MG) |
|---|---|---|---|
| `default` | `2.6e-2` (`phi(l+)/pi`) | `2.9e-3` (`SPINUP`) | `933.230` pb |
| `mmll_60_120` | `2.2e-2` (`y(l-)`) | `6.9e-2` (`ICOLUP`) | `644.334` pb |

The `2.9e-3` is one seed of three (`0.49` and `0.72` on the others), so noise at
this trial count, not a structure. `pt(ll)` is a constant of the process at this
order and is named rather than compared. `pp_to_ll_qcd0`'s `samples` becomes
`covered-by = ["pp_to_ll"]`, matching where its `integrals` cell already points:
the two rows enumerate the same diagrams, so the order constraint changes what
the selection is *asked* for and not what it returns, and that is the `diagrams`
cell's business.

**The spectrum.** `dσ/dm_ll` in absolute picobarns on both cards, shared
machinery (`validation::samples::Spectrum`) with the `ee_to_mumu_tata_qcd0`
binning, which was migrated onto it. Every judged bin — a bin carrying at least
`1e-4` of MadGraph's sample — is within **2.3** combined errors on both cards,
against a `4σ` threshold set from the ~50-bin trial count. The Z bins agree to
`0.1–0.8%`, the `20–60 GeV` photon-pole region to `0.9–5.8%` at `0.3–2.2σ`.

The lower edge is `20 GeV` and that is exact, not approximate: at this order the
pair recoils against nothing, so both leptons carry the same `pt` and
`m_ll = 2 pt / sin θ ≥ 2 ptl`. `drll` never binds under it (`Δφ = π`). The gate
requires **zero** weight below that edge on both sides, which is a check that the
cut is in the right place and not only that the spectrum above it is right.

**Left for B5, with numbers.**

1. The two runs are in the local work area only. `pp_to_ll` carries
   `bundled = false`, and the branch is red on a fetching checkout — `banked_sample`
   fails naming the missing file — until `refdata-3` picks them up. Both runs are
   `Events/run_<name>/`, not `Events/run_01/`: `generate_events -f run_default`
   names the directory after the run tag. The gate's `Row.events` field says which.
2. Bundle cost: `20 MB` and `21 MB` gzipped, `~120 MB` of text each, against a
   `65 MB` `refdata-2`. If that is too much, `nevents` is the dial — but it also
   sets MadEvent's integration accuracy, so lowering it weakens the σ gate's
   reference, and the cards are shared verbatim with this crate.
3. When re-banking the rest with 3.7.1, **check each run's `IDWTUP`**. Every
   `refdata-2` run is `-4` because MadGraph's own full run cards name
   `event_norm`; any re-run driven from a hand-written card will be `-3`. The
   reader now handles both, and `from_lhe`'s panic branch is the backstop, but a
   re-bank that silently flips a row's convention is worth seeing rather than
   absorbing.
4. `output/dy13_default/SubProcesses/P1_qq_ll/matrix1_optim.f` is now 3.7.1's.
   `generate_references.sh`'s `refs` stage regenerates `dy_integrand_oracle.json`
   from it whenever that file exists, so the next `generate-references` run will
   recompute the committed oracle against 3.7.1's matrix element. Not run here.
5. Before the re-cut, sweep the *other* readers of generated Fortran for the
   packed-`CF` change above — `build_amplitude.sh` + `gen_amplitude*.py` compile
   `matrix1_orig.f` through f2py, so they execute MadGraph's own contraction and
   should be unaffected, but that is a prediction and not a measurement until the
   re-bank runs. The colour oracle parses the file by hand and is the one that
   already broke.

### B5 — hygiene riders + `refdata-3` + close-out

One session, because the bundle re-cut must happen exactly once (each re-cut
costs a new archive and a new pin — note 25 §refdata-2).

- **The 3.7.1 re-bank** (decision D3): regenerate the banked reference runs
  with the submodule's 3.7.1 via the mechanism B4 records, then re-measure
  every banked gate against the regenerated references — the h→ττ row's
  `integrals` cell becomes gateable for the first time (its correct number now
  exists), and any other cell that moves is a finding, not a nuisance: 3.5.7
  and 3.7.1 must agree wherever B1's analysis says the defect cannot reach.
- **`refdata-3` re-cut**: the re-banked runs, B1's windowed run and B4's two
  DY event banks (plus anything B2/B3 banked). Same verification protocol as
  `refdata-2`: two assemblies byte-identical, all runs' decompressed event
  text sha256-stable through pack/unpack, clean `git archive` export runs the
  banked layer green from the bundle alone. Publish, flip the manifest pin.
- **`.gitignore` the stray `py.py`** mg5_aMC drops in the cwd (B1's note), or
  make the drivers run MG from a scratch directory.
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

#### B5 outcome (2026-08-01) — ⛔ **blocked**, branch `v3b-b5`

The re-bank landed and two cells were promoted, but `pixi run validate` is **red**
and the branch must not be merged: MadGraph 3.7.1 emits Les Houches events in a
second numeric dialect our writer does not produce, and
`validate_lhef::banked_files_round_trip_byte_for_byte` — an enforced gate over
all banked runs — fails on 20 of the 34 banked event files. The fix belongs in
`lhef::parse`/`lhef::write` and is out of this session's scope; see **the LHE
dialect blocker** below.

Everything else in the session is done, measured and committed.

**The re-bank.** Every run in `validation/madgraph/output/` is now MadGraph
3.7.1's — `#*  VERSION 3.7.1  2026-04-29  *` in 28 of 34 banners — through
`mg5_pinned.sh`, which `build.sh` now calls in place of the `mg5_aMC` on `PATH`.
The six exceptions are deliberate and are B1's *evidence*: the
`ee_to_mumu_tata_qcd0` `hwindow`/`hanti`/`control_2026080{1,2,3}` runs and
`var_sde1` measure a 3.5.7 defect, so they stay at the version they measure.
`IDWTUP` is `-4` on all 32 MadGraph-carded runs and `-3` on B4's two `dy13`
banks, which is what §B4's hand-off predicted from `event_norm`'s system default
— no row flipped convention silently. The packed-`CF` prediction held: every
`amplitudes`, `color_cf_oracle` and `color_flow_tags_oracle` row passes against
regenerated 3.7.1 Fortran, so the f2py path really does execute MadGraph's own
contraction and only the hand-written parser needed B4's fix.

**`e+ e- > mu+ mu- ta+ ta-` gates, both cells.** `integrals` reads
`1.367003e-3 ± 2.685e-6` pb against `1.372500e-3 ± 2.674e-6` pb — pull `−1.45`,
rel `−0.40%`, against a `rel_tol` of `0.02` taken from the resonant channel's own
`0.45%` seed spread rather than from the reference's error. `samples` reads
worst KS p `6.5e-2` (`cos(ta+)`) and worst χ² p `2.5e-1` (`SPINUP`) over three
seeds, against `3.6e-6` and χ² 210–233 on 15 dof from the 3.5.7 bank. B1
predicted ~0.4σ and the measurement is 1.45σ; the difference is that B1's
prediction was against MadGraph's *windowed* sum, and this is against a fresh
independent 3.7.1 run with its own Monte-Carlo error.

**The finding the re-bank was not looking for: 3.5.7 ran every `lpp = 0` process
at `αs(M_Z) = 0.130`.** The partonic references moved far more than the pole fix
could explain — `gg_to_gg` `168830 → 142770` pb, `gg_to_ttx` `15.953 → 13.513`,
`uux_to_uux` `33428 → 28269`, all `−15.4%`; the `QCD=2 QED=2` 2→3 rows `−8%`
each; every pure-QED row unmoved. That is `0.920ⁿ` in the power of `αs`, and the
chain closes on the cards:

| | 3.5.7 (`refdata-2`) | 3.7.1 (`refdata-3`) |
|---|---|---|
| `run_card.dat` `lpp1`/`lpp2` | `0`/`0` | `0`/`0` |
| `run_card.dat` `pdlabel` | `nn23lo1` | `nn23lo1` |
| `param_card.dat` `SMINPUTS 3` | **`1.300000e-01`** | **`1.180000e-01`** |
| banked `SCALUP` | `250.0` | `250.0` |
| banked `AQCDUP` | `0.1113305` | `0.1024649` |

3.5.7 applied the `nn23lo1` set's `αs(M_Z)` override to a run whose *beams carry
no PDF at all*; 3.7.1 leaves the model's own value. Nothing on this side moved:
every gate resolves `αs` from the run's own parameter card, so our σ tracked the
step exactly and the three QCD `integrals` cells stayed green at pull `+0.05`,
`+0.68`, `−1.49`. Recorded because it changes what a banked partonic σ *means* —
a number quoted from `refdata-2` is not comparable to one from `refdata-3`.

**The alphas gate's oracle was withdrawn upstream.** 3.7.1 moved `setclscales`
from `cuts.f` into a vectorised `reweight.f` and commented out its
`write(6,*) 'alpha_s for scale ',scale,' is ',…` diagnostic, so the 17-digit
per-scale `αs` value the run logs used to print no longer exists — a *format*
change, not a behaviour change: the `New value of alpha_s from PDF lhapdf` line
still prints at 17 digits and still matches the grid to `9.86e-9`.
`the_grid_alpha_s_reproduces_the_scale_its_run_log_prints` is therefore folded
into `banked_run_logs_pin_the_alpha_s_source_rule` rather than kept alive on a
line that is never there. Every assertion with a surviving oracle is kept: the
grid reading against the printed `αs(M_Z)`, and — the half that was not a
duplicate — that the grid and the parameter card are separated by more than half
a printed `AQCDUP` digit (`1.78` half-digits, measured), without which
`banked_events_reproduce_aqcdup`'s 20 000 grid-sourced events would agree with
either source and pin neither. What is genuinely lost: a wrong interpolation
*away* from `M_Z` is now bounded by the events' six-digit `AQCDUP` budget instead
of by seventeen digits. Both `pdlabel = lhapdf` runs fix `μR = 91.188 = M_Z`, so
nothing measured today separates the two.

**`ee_to_mumua` is the one row that got worse.** Our σ is unchanged
(`1.007660e-1` pb, same seed, same budget); MadGraph's moved
`1.00630e-1 ± 3.865e-4 → 9.980100e-2 ± 2.335e-4` pb. The `integrals` pull went
`+0.31 → +3.12` against a `PULL_LIMIT` of `3.5`, and roughly half of that growth
is the tighter reference error rather than the `−0.83%` shift. Its `samples` cell
followed: minimum KS p `2.14e-3 → 2.74e-4` against a `1e-4` floor, worst
observable `y(a) → pt(a)`. Both cells still gate and both are now the tightest in
their category. Filed with the measurement, not chased: the photon is
soft/collinear-regulated by the run card's cuts, which is the region MadGraph's
channel-weight change reallocates, and deciding which side owns the remaining 1%
wants the windowed comparison B1 used on the Higgs pole.

**The LHE dialect blocker.** MadGraph writes `unweighted_events.lhe` twice over:
`rw_events.f` emits it in Fortran, and `madevent`'s Python post-processing reads
it back through `lhe_parser.py` and writes it out again. Which of the two dialects
survives depends on how much of the file that read-back had to parse:

```python
# lhe_parser.py:2606, Event.__str__
try:
    scale_str = "%2d %6d %+13.7e %14.8e %14.8e %14.8e" % \
        (self.nexternal, self.ievent, self.wgt, self.scale, self.aqed, self.aqcd)
except:
    scale_str = "%s %s %+13.7e %s %s %s" % \
        (self.nexternal, self.ievent, self.wgt, self.scale, self.aqed, self.aqcd)
```

`EventFile`'s `parsing == "wgt_only"` mode constructs each `Event` with
`parse_momenta=False`, and `assign_scale_line(line, convert=False)` then keeps
`nexternal`, `ievent`, `scale`, `aqed` and `aqcd` as **strings** (`lhe_parser.py`
:2233–2251). `"%2d" % "4"` raises, the `except` fires, and every field except the
weight — which the unweighting rescaled and therefore had to convert — is written
back verbatim. The particle lines are never touched at all, so they keep
`rw_events.f`'s `5e19.11,f3.1,f4.1`:

| | dialect P (converted) | dialect F (pass-through) |
|---|---|---|
| info line | `· 4 · · · ·1 +1.4277480e+05 2.50000000e+02 …` | `4 1 +1.4277480e+05 0.2500000E+03 …` |
| momentum | `+0.0000000000e+00` | `0.00000000000E+00` |
| lifetime / spin | `0.0000e+00 -1.0000e+00` | `0. 1.` |

3.5.7 produced dialect P for every run. Under 3.7.1 it depends on whether the
systematics step forced a full parse, which tracks `use_syst`, which tracks the
beams: all 8 `lpp ≠ 0` runs are still dialect P, and all 20 `lpp = 0` runs
regenerated here are dialect F. (B1's six 3.5.7 evidence runs stay P.) It is a
deliberate fast path, not a defect, and it is not reachable from the run card.

Nothing physical moved — every gate that *parses* the file is green, because Rust
reads `0.2500000E+03` as readily as `2.50000000e+02`. What broke is the one gate
that asserts on the bytes: our reader parses both dialects and our writer emits
only P, so re-serialising a dialect-F run does not reproduce it.

Recommended fix, for a session that owns `lhef/`: make the round trip
byte-exact **by construction** rather than by matching a format, which is the
same move MadGraph makes. `lhef::parse` already sees each numeric field's source
text; carrying it on the record and having `lhef::write` re-emit it unless the
value changed removes the whole class of "we reformat what we did not alter" —
and it is what makes the gate mean *this file round-trips*, rather than *this
file happens to be in the dialect we emit*. A dialect enum switching two format
tables would also pass, and would be the weaker claim: it goes stale the next
time upstream adds a third spelling.

**Riders.** `compact_events.py` deleted with its `lhe-compact` environment (D2);
the `diagrams` gate moved to the hermetic layer (registration, all 26 manifest
tiers, 26 rows in 1.25 s on a bare clone); `init-sm-submodule` calls
`vg_ensure_submodule`; `Process::Display` carries coupling orders;
`/py.py` ignored; `cargo clippy --workspace --all-targets --all-features` clean.
The mirror-term bound is now a function of `ŝ` —
`0.076 ŝ/(ŝ + m_Z²)`, the shape of a `γ*/Z` core whose forward-backward asymmetry
is set by `ŝ/m_Z²`, fitted to the measured plateau and halved, sitting 1.58× to
4.86× under `probe_mirror_visibility_ladder` from 25 GeV to 4 TeV over three
streams and two sample sizes. That ladder also says *why* the flat `1e-3` was the
wrong shape: the bound has to be a percentile, because the two beam orderings
agree exactly wherever the configuration is symmetric, so the *minimum*
visibility falls by a decade going from 32 draws to 512 at every energy — it
measures the sample, not the physics.

### B6 — the per-diagram `AMP2_d` accumulator (decision D4)

The §B3.2 design as its own session, added post-B3 by the user. Scope:

1. **Evaluator**: `AMP2_d = Σ_hel |A_d|²` per diagram as additional roots on
   the *same* compiled program — tap the existing per-diagram amplitude
   wires, add `|·|²` + helicity-fold nodes, keep one DAG so CSE sharing is
   untouched and the event path reads scratch indices. Accumulate only over
   diagrams that would carry a MadGraph config (no four-point vertex, per
   `get_amp2_lines`); recheck the `prune_zero_helicities`/`folded_hel`
   contract on the new roots.
2. **Oracle first**: per-diagram `AMP2` values against MadGraph's own (the
   f2py wrappers expose the `AMP2` array the generated `matrix1.f`
   accumulates), before any selection change — the [[helas-debugging-lessons]]
   order.
3. **Selection**: draw the per-event configuration ∝ `AMP2_d(x)`, apply that
   config's `ICOLAMP` mask (B3's `select_flow_reached_by`), weights ∝ JAMP²
   inside the mask. DY must reduce to a no-op (single flow / all-true mask).
4. **Gate**: `uux_to_uux` `ICOLUP` χ² clears the floor across 3 seeds
   (99.96/0.04 reproduced) **and** `pp_to_bb_fixed`'s two sub-percent flows
   land at MadGraph's 0.07–0.08% (the per-config-weight test a merely-on mask
   cannot fake); `gg_to_ttx`/`gg_to_gg`/llj `samples` re-measured; Pythia
   consumption re-run (event bytes change); both colour cells info → gate on
   success.

#### B6 outcome (2026-08-01) — ✅ closed, branch `v3b-b6`

Both acceptance targets reproduced, and the two colour `samples` cells are `gate`.

**The evaluator.** A new `Op::Configs` root bundles the amplitude root with the
per-configuration diagram amplitudes: `(Configs <Flows|scalar> A_0 … A_{k-1})`,
where each `A_d` is the *same* lowered subtree the JAMPs are built from,
referenced again. One DAG, no arithmetic added, CSE untouched; the helicity
expansion carries the bundle through, `Program::build` records the amplitudes'
scratch indices alongside the JAMP ones, and `BoundAmplitude::eval_amp2` reads
them back squared and helicity-summed. Which diagrams get one is
`get_amp2_lines`' own rule — drop any diagram whose widest vertex exceeds the
narrowest diagram's widest — so `g g > g g`'s four-gluon contact diagram gets no
configuration and its three colour structures never mask a flow.

Deviation from the sketch: the `|·|²` and the helicity fold are **not** arena
nodes. The arena has no real-valued square or fold op, adding one would touch the
op↔s-expr bijection, the layout, the egraph schema and the op-coverage allowlist,
and it would buy nothing — `eval_jamp2` already forms exactly this shape in Rust
off the helicity-expanded root, once per accepted event. `eval_amp2` is its
per-diagram twin.

**The oracle, first and hermetic.** Every committed amplitude table now banks
`amp2_groups`, the `AMP()` indices each `AMP2()` accumulator of MadGraph's own
generated `matrix1.f` sums, and the three multi-flow tables bank `AMP()` itself
(built against the *banked* `matrix1_orig.f`; the probe reproduces every banked
`JAMP` to ≤4.1e-16, which is what makes the addition safe). `amplitude_oracle`
then checks, on all 19 rows:

- the configuration count and grouping are MadGraph's, in MadGraph's order —
  `g g > g g` gives `[[3],[4],[5]]`, i.e. `AMP(1..3)` (the contact diagram) carry
  none;
- each configuration amplitude is MadGraph's `AMP()` up to a per-diagram unit
  phase (worst residual 1.7e-13, worst `||k|-1|` 8.0e-14 across the suite);
- `eval_amp2` reproduces `Σ_hel |AMP^mg|²` per configuration: `uux_to_uux`
  1.5e-15, `gg_to_ttx` 2.1e-15, `gg_to_gg` 1.5e-15, `uux_to_epemg` 2.0e-14, and
  3.5e-13 worst over the suite (`ee_to_mumu_tata_qcd0`, 25 configurations).

The phase is fitted **per diagram**, not globally: MadGraph puts the
annihilation/exchange relative sign in the colour coefficient and we put it in
the diagram root, so a global fit would show a spurious residual. `|k| = 1` is
the half with teeth, and it is exactly the claim `AMP2` rests on.

Two findings fell out of the oracle:

1. **MadGraph's export merges some configurations, and it is not derivable from
   the diagram list.** The banked exports use `get_amp2_lines`' `config_map`
   branch, where diagrams the channel mapping calls one topology are summed
   *coherently* into one accumulator. In the whole banked set it fires once:
   `e+ e- > e+ e-` merges its two t-channel diagrams (photon and Z) into config
   3, so MadGraph has 3 configurations where we derive 4. Recorded as
   `KNOWN_CONFIG_MERGE` with the reason it cannot reach an event: the process is
   colourless, its single `ICOLAMP` row admits everything, and the configuration
   label is unobservable. The entry is two-way — if the grouping ever agrees, the
   gate fails on the stale exemption.
2. **Helicity pruning moves `AMP2`, and by a lot.** `|M|²` is bit-for-bit under
   pruning because the dropped combinations are ~1e-30 of the coherent sum; the
   *incoherent* per-diagram sum has no such protection. Measured:
   `gg_to_ttx` **39.5%**, `gg_to_gg` **3.2%**, every other row exactly 0. The
   dropped combinations are the ones that vanish by `J_z` conservation about the
   beam axis, whose individual diagram amplitudes do not vanish at all. The
   production path draws on the pruned evaluator, which is the analogue of
   MadEvent's own `GOODHEL`-filtered accumulation (`|T| > ANS·LIMHEL/NCOMB`,
   `LIMHEL = 1e-8` against our 1e-24), and the measured `ICOLUP` frequencies say
   it is the right one — `gg_to_ttx` sits at p 0.46 to 0.71. The gate measures the
   gap every run rather than assuming it away.

**Selection.** `AmplitudeEvaluator::select_color_flow(amp2, jamp2, [u0, u1])` is
`SELECT_COLOR`: configuration `∝ AMP2(d)`, then flow `∝ JAMP2(i)` inside that
configuration's `ICOLAMP` row, with B3's fallback when the mask carries no
probability. Both event paths (`FixedBeamIntegrand::select_event`,
`ProtonIntegrand::select_event`) now take one more uniform and call it. A
single-flow process reduces to a no-op by construction, asserted directly.

**Measured** (3 seeds each, 20k events, against MadGraph's banked samples):

| row | ICOLUP before | ICOLUP after |
|---|---|---|
| `uux_to_uux` | 90.4% vs 99.96%, χ² 1015 / 1 dof | **99.960% vs 99.960%**, χ² 0.0 / 0.7 / 0.3, p 1.00 / 0.39 / 0.58 |
| `pp_to_bb_fixed` | 0.23%, 0.25% vs 0.07%, 0.08%; χ² 23–31 / 5 | **0.060%, 0.070% vs 0.070%, 0.080%**; χ² 2.0 / 2.5 / 5.2, p 0.39–0.85 |
| `gg_to_ttx` | (gate) | χ² 0.6 / 0.1 / 0.5 on 1 dof, p 0.46–0.71 |
| `gg_to_gg` | p 0.003 | χ² 14.0–18.1 / 5, p 2.8e-3 to 1.6e-2 — unmoved |
| `pp_to_llj_fixed` | (gate) | χ² 3.3–8.3 / 5, p 0.14–0.65 |

`gg_to_gg` not moving is worth keeping in view: its three configurations reach
four of six flows each, so the mask is live, and the residual is in the two
double-share flows rather than in a suppressed one. It clears the 1e-4 floor and
stays a gate, but it is the row where a further colour-selection subtlety would
show up first.

Cells: `uux_to_uux` `samples` ⚠️ info → **gate**, `pp_to_bb_fixed` `samples`
⛔ info → **gate**.

**For B5.** The three multi-flow amplitude tables now carry `amps`, so a
`generate-amplitude-tables` re-run must have `mg_amp_probe_*` built for them —
`build_amplitude.sh` already does. Worth filing separately: those tables' banked
`AMP()` come from a probe built beside the banked matrix element rather than from
the same binary that produced their `JAMP()` (agreeing to 4.1e-16), which a full
re-bank makes moot; and the multi-flow colour coefficient matrix is still not
banked, so the per-diagram *contribution* fit (`c_i·AMP(i)`) still runs on
single-flow rows only.

### B7 — the source-text-preserving LHE round trip

B5's blocker as its own session, added post-B5 by the user. Scope: `lhef/`, the
round-trip gate, and the `git archive` export proof B5 could not run while the
gate was red. Nothing else — no re-cut bundle, no re-banked run.

#### B7 outcome (2026-08-01) — ✅ closed, branch `v3b-b7`

The blocker is gone: `banked_files_round_trip_byte_for_byte` is green on
**34/34** banked event files, `pixi run --skip-deps validate` is green end to
end, and the sprint's census reads **75 measured / 74 ✅ / 1 ⚠️** exactly as §B5
predicted.

**The falsifier came first and cleared.** §B5's diagnosis said the 20 failures
were *formatting only* — that every field in a pass-through file decodes to the
value our reader already reads, and only its spelling differs. If that were
wrong, a source-text-preserving writer would still not reproduce the bytes. It
reproduces all 34, including the two 200k-event Drell-Yan banks: 714 759 events
and 3 711 197 particle lines, byte for byte.

**What was built.** `lhef::parse` keeps each block's record lines as a
`BlockSource` — one owned string per `<init>` and per `<event>`, covering the
body from its start through the newline that ends the last record line —
alongside the values it decoded from them. `lhef::write` splits that text back
into lines and hands a line back **verbatim only after decoding it again and
finding it to spell the record it is being asked to write**. Three consequences,
and the third is the point:

- the reuse is *checked*, not flagged. There is no mutation tracking and no
  dialect enum; a caller who edits a field gets that line in this writer's
  layout and the rest of the block in the file's own spelling, which is
  MadGraph's own behaviour arrived at from the other direction;
- the source is dropped where it stops describing the record —
  `observables::canonical` reorders the legs, and a block whose record-line
  count no longer matches (a leg added or dropped) is discarded whole rather
  than matched up by guesswork;
- a record *built* rather than read carries no source at all, so nothing this
  crate generates changed. Both `generated_events_serialise_into_a_coherent_file`
  samples came out at exactly the pre-change byte counts (5 921 500 and
  7 241 598), which is the check that the fix cannot have leaked into the
  emitted-events path.

`LheInit` and `LheEvent` now compare on their values with a hand-written
`PartialEq`: the source says how one file spelled a record, not what the record
is, and a parsed block has to keep comparing equal to the same block built from
scratch or every by-value round-trip check in the crate would start failing on
files it reads correctly.

**The gate would have got quietly weaker, so it was made to say so.** Once the
writer hands a file its own text back, a pass-through run round-trips *whatever*
this crate's columns are — the round trip on those 20 files is no longer evidence
that our layout is MadGraph's. The gate therefore re-serialises every run a
second time with the source dropped, requires at least one to still reproduce
MadGraph's bytes, and prints the split: **14 of 34 in this writer's own layout,
20 in the pass-through dialect**. That is 8 `lpp ≠ 0` runs plus B1's 6 3.5.7
evidence runs, which is precisely the partition §B5 derived from `use_syst`,
measured rather than assumed. A corpus that lost its converted runs now fails
with a message saying what stopped being evidence.

The gate also stopped looking only at each process's `run_01` and now sweeps
every `Events/*/unweighted_events.lhe.gz`: the Higgs-window evidence runs, the
two Drell-Yan banks and `var_sde1` are files MadGraph wrote too, and there was
no reason for the format oracle to be blind to 8 of them. 26 runs → 34.

**Cost, measured on the identical 34-file corpus** (`release-debug`, `/usr/bin/
time -l`, two runs each): with the source carried and the second pass, **23.7 s**
and **853 MB** peak RSS; with `source: None` forced at parse and the mismatch
branch reduced to a counter, **22.3 s** and **724 MB**. Carrying a verified line
is *cheaper* than formatting thirteen fields — the whole second pass costs about
what the reformatting it replaces cost — and the extra memory is the owned line
text of the two 200k-event banks. Owned strings rather than spans into the file:
the measurement says the trade is right and spans would have put a lifetime on
`LheFile` and every consumer of it.

**The `refdata-3` export proof ran, and its first attempt failed on a defect in
the acceptance script rather than in the tree**: `git archive` emits an *empty
directory* for each submodule gitlink, so the script's
`cp -R …/mg5amcnlo "$EXPORT/research/refs/mg5amcnlo"` nested the copy one level
down and `sm_interned_blob` could not find the SM UFO source. With
`cp -R …/mg5amcnlo/. "$EXPORT/research/refs/mg5amcnlo/"` the export runs the
banked layer green from the local bundle alone — same sha256 `10892f05…` as the
manifest pin, no re-cut — and reproduces this session's numbers exactly: the same
34/34 with the same 14/20 dialect split, and the same
`75 measured (74 ✅, 1 ⚠️, 4 ⏳, 16 ⛔, 9 —)` census. Worth knowing for anyone
writing a "clean export" check against this repo.

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
   back before implementing.) **Outcome: the contingency fired** — the premise
   is falsified (§B3.2); superseded by D4.
2. **D2 — `compact_events.py`: delete.** Note 26 records the verdict's
   numbers, git history keeps the script.
3. **D3 — re-bank with MadGraph 3.7.1** (user, post-B1). The oracle layer
   moves to the pinned submodule's 3.7.1; B4 banks its Drell-Yan events with
   3.7.1 directly (verified in the banner), and B5 re-banks the remaining runs
   with the same toolchain before the `refdata-3` re-cut, re-measuring every
   banked gate against the regenerated references.
4. **D4 — `AMP2_d` becomes its own session, B6** (user, post-B3): the
   per-diagram helicity-summed `|AMP_d|²` accumulator of §B3.2, as additional
   roots on the existing program DAG (CSE preserved, values read from scratch
   indices), config drawn ∝ `AMP2_d` over configs (no four-point-contact
   diagrams), `ICOLAMP` mask applied, with its own MadGraph `AMP2` oracle.
   Acceptance: `uux_to_uux` 99.96/0.04 reproduced **and** `pp_to_bb_fixed`'s
   two sub-percent flows at 0.07–0.08% (the sharper test — a merely-on mask
   cannot fake per-config weights).
