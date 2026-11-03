---
type: Session Report
title: "How MadEvent covers both halves of H → ZZ* in bbx_to_ccx_emmm_qcd0"
description: "Read-only study of MadGraph 3.7.1 and this row's MadEvent output: one integration channel per on/off-shell assignment of the Z pair, split by a hard 5Γ cut, so our one-sided H → ZZ* coverage was a deviation."
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-29}
---
# How MadEvent covers the two halves of H → ZZ* in `bbx_to_ccx_emmm_qcd0`

Question: does MadEvent (MG5_aMC 3.7.1) cover only one side of the H → Z Z* peak, as our multichannel sampler does? If not, what mechanism covers the other side?

**Short answer.** MadEvent covers both halves, each with its own dedicated integration channel. The mechanism is **BW-conflict splitting** (the `symmetry` executable).
- For every config whose resonances cannot all be on shell at once, MadEvent creates separate integration jobs (`G<config>.<code>`), one per on-/off-shell assignment of the conflicting resonances.
- Each job has a hard |m−M| < 5Γ / ≥ 5Γ cut (`cut_bw`).
- A required resonance gets a Breit–Wigner map. An excluded one gets a log-spaced (1/s) pre-shaped VEGAS grid, not a BW tail.

For H → Z Z* this yields three sub-channels per Higgs config with H required on shell: "Z_a on / Z_b off", "Z_a off / Z_b on", and "both off". "Both on" is removed as kinematically impossible. **Our one-sided coverage is a deviation from MadEvent, not shared with it.**

Legend: **[src]** = read from source or the process output, with location. **[inf]** = inferred.

Paths used below:
- `OUT` = `/home/user/wt/alpha-share-tails/validation/madgraph/output/bbx_to_ccx_emmm_qcd0`
- `P1` = `OUT/SubProcesses/P1_bbx_qqllll`
- `MG` = `/home/user/vibegraph/research/refs/mg5amcnlo` (VERSION 3.7.1, commit b7687064b). The run banner also says 3.7.1.

---

## 0. What the process output does and does not contain

- **[src]** `P1` contains only `matrix1_orig.f`, `leshouche.inc`, 417 `G*/run_01_log.txt` files and `results.dat`.
  - `configs.inc`, `props.inc`, `coloramps.inc`, `symfact.dat`/`symfact_orig.dat`, `symswap.inc` and `symperms.inc` are **absent**.
  - They are also absent from every other copy found: the main tree, all `/home/user/wt/*` worktrees and `seeded_sigma/`.
  - So the iforest and propagator tables could **not** be read directly. They are reconstructed below from three sources:
    1. the HELAS call graph in `matrix1_orig.f`;
    2. the per-channel `run_01_log.txt` diagnostics, which print every BW slot's mass, required/excluded status and grid setting;
    3. the MG templates that print those diagnostics.
- **[src]** Run card `OUT/Cards/run_card.dat`:
  - `sde_strategy = 2` (line 80), `bwcutoff = 15.0`, `nhel = 0`;
  - `ptj = 20`, `ptl = 10`, `drll = 0.4`, `mmll = 0`, `mmjj = 0`;
  - fixed-energy e e-type beams at 500 GeV, no PDF. The log says "Running at e e machine @ 500 GeV", "PDF set = none".
- **[src]** `OUT/build.log:129`: "Process has 615 diagrams". l.140: "Finding symmetric diagrams for subprocess group bbx_qqllll". l.375: "need to improve 13 channels". Final result: σ = 1.682e-06 ± 4.9e-09 pb.
- **[src]** Why `sde_strategy = 2`: `MG/madgraph/various/banner.py:4994-5014` picks 2 automatically when the process has a single colour flow and is not pure-lepton/proton-initial with QCD > 0. **[src]** `matrix1_orig.f:575` has `NCOLOR=1`.

## 1. The Higgs-diagram channels and their iforest

### 1a. Which diagrams

**[src]** Wavefunction identities in `P1/matrix1_orig.f`. External order: 1 b, 2 b~, 3 c, 4 c~, 5 e+, 6 e−, 7 μ+, 8 μ−.

| Wavefunction | Meaning | Line |
|---|---|---|
| W27 = `FFV2_5_3(W4,W3)` | Z → c c̄ | l.737-738 |
| W32 = `FFV2_3_3(W1,W2)` | Z from b b̄ (the ŝ line) | l.775 |
| W48 = `FFV2_4_3(W5,W6)` | Z → e⁺e⁻ | l.872 |
| W42 = `FFV2_4_3(W7,W8)` | Z → μ⁺μ⁻ | l.979 |

**[src]** Higgs-strahlung graphs, l.981-988:

| Diagram | Structure |
|---|---|
| 121 | Z*(bb̄) → Z(cc̄) H, H → Z(ee) Z(μμ) |
| 122 | Z*(bb̄) → Z(ee) H, H → Z(cc̄) Z(μμ) |
| 123 | Z*(bb̄) → Z(μμ) H, H → Z(cc̄) Z(ee) |

**[src]** Further graphs where an H → ZZ → 4f subgraph attaches to the b line through the b Yukawa (`FFS4_0`, one t-channel):
- 448 and 453 use H = `VVS1_3(W48,W42)` (H → ee μμ), l.1704.
- 464 and 467 use H = `VVS1_3(W27,W42)` (H → cc̄ μμ), l.1737.
- 474 and 475 use H = `VVS1_3(W27,W48)` (H → cc̄ ee), l.1758.
- 478 uses the l.1704 H; 481 uses the l.1737 H; 484 uses the l.1758 H.

**[src]** The b b̄ → H s-channel graphs (H from `FFS4_3(W1,W2)`, l.912) carry H at √ŝ = 500 GeV. They are not resonant at 125 GeV.

**[src]** Every diagram is its own config: `matrix1_orig.f` has 615 single-diagram lines of the form `AMP2(n)=AMP2(n)+AMP(n)*DCONJG(AMP(n))`, written by `export_v4.py:get_amp2_lines` (l.1387-1430).

### 1b. Which configs are integrated: the 615 → 149 channel reduction

- **[src]** G directories exist for 149 distinct configs. Among the Higgs graphs, **121 and 122 have G dirs; 123 does not**. 448, 453, 464, 467, 478 and 481 have G dirs; 474, 475 and 484 do not.
- **[src]** Mechanism: `MG/madgraph/various/diagram_symmetry.py:find_symmetry_subproc_group` (l.374-436).
  - Configs whose `IdentifySGConfigTag` is equal are merged: `symmetry[inum] = -first` (l.430-432).
  - The tag identifies external legs only by `(state, spin, colour, |charge| if colourless, mass, width)` (l.341-343).
  - It identifies propagators by `(colour, mass, width, QCD order, onshell)` (l.371-372).
  - So e⁺, e⁻, μ⁺, μ⁻ are interchangeable, but the coloured c, c̄ are not.
- **[inf]** 123 (H → Z_cc Z_ee, Z_μμ spectator) is the e↔μ image of 122 (H → Z_cc Z_μμ, Z_ee spectator), so 123 → −122. Likewise 474 → 464 and 475 → 467. 121 (H → ee μμ) maps onto itself under e↔μ.
- **[src]** The merged config is still integrated, inside the surviving channel:
  - `super_auto_dsig_group_v4.inc:488-504` reads `symfact.dat` into `SYMCONF`, collecting every config whose factor equals `-MAPCONFIG(ICONFIG)`.
  - `DSIG` loops over `J=1,SYMCONF(0)` (l.575-605) and selects one with weight `SUMPROB/SELPROC` (l.668-695).
  - `DSIGPROC` sets `ICONFIG=SYMCONF(ICONF)` and permutes the momenta with `SWITCHMOM(PP,P1,PERMS(1,MAPCONFIG(ICONFIG)),…)` (l.799-805).
  - So a point generated with config 122's map is also evaluated as config 123 with e↔μ relabelled.
- **[src]** Evidence in the logs:
  - Every `G122.*` log prints **two** rows under "Relative summed weights". In `G122.205` they are 0.4922 and 0.5078, i.e. configs 122 and 123.
  - `G121.*` and `G448.*` print one row; `G464.016` prints two.
- **[inf]** Across all G dirs the row counts give 5×1 + 32×2 + 88×4 + 24×8 = 613 configs, plus about 36 empty or no-result logs. This is consistent with every one of the 615 configs being covered by some channel; none is dropped.

### 1c. iforest / propagator structure of the H configs

The `G` logs print `Setting BW <iprop> <nbw> <mass>` and `Requiring/Excluding BW <iprop> <nbw>`. The format is in `MG/Template/LO/SubProcesses/myamp.f:88-93` and `:439, :446`.

**Config 121** — **[src]** logs `G121.*`:
- propagator −3 has mass 125.0 (the H);
- −1, −2 and −4 have mass 91.188;
- −5 is the ŝ propagator, printed as "Setting PDF BW 15 5".

**[inf]** Reconstruction: the iforest lists daughters before parents, so H(−3) = (−1, −2).
- −1 and −2 are Z→ee and Z→μμ. Which is which is undetermined: both off-shell grids have xo = 0.512e-4, a lepton pair.
- −4 = Z(cc̄) = (3,4).
- −5 = (−3, −4), i.e. ŝ = Z*(bb̄).
- No t-channel ("T-channel found: 0").

**Config 122** — same shape. **[src]** The off-shell grid printed for an excluded slot identifies the pair:
- `G122.203` excludes −1 with "Setting grid 1 0.51200E-04", a lepton pair: m_min² = pT1·pT2·0.8·ΔR² = 10·10·0.8·0.16 = 12.8 GeV², and 12.8/500² = 5.12e-5.
- `G122.205` excludes −2 with "Setting grid 2 0.20480E-03", a jet pair with pTj = 20 (`myamp.f:388-393`).
- **[inf]** So −1 = Z→μμ, −2 = Z→cc̄, −3 = H(−1,−2), −4 = Z→ee, −5 = ŝ.

**Yukawa configs 448, 453, 464, 467, 478 and 481** — **[src]** "T-channel found: 1"; −3 is the H; −1 and −2 are the Z's. In 464, −2 is the cc̄ pair: its excluded grid is 0.2048e-3 in `G464.016`.

**Order in which `genps.f` generates the invariants** — **[src]** `MG/Template/LO/SubProcesses/genps.f:one_tree`, l.821-865:
- The s-channel invariants are drawn **bottom-up in iforest order**, `ibranch = -1, -2, …, -ns_channel`.
- Each draw uses `smin = (m(d1)+m(d2))**2` and `smax = min((sqrt(s(-nbranch)) - totmass + sqrt(smin))**2, stot)`.
- `totmass` is updated after each draw (l.855-856).
- For config 121/122 the order is therefore Z(−1), then Z(−2), then H(−3) with lower limit (m₋₁+m₋₂)², then Z(−4).
- The ŝ propagator −5 is fixed here: there is no PDF, and the branch at l.867-874 handles `nt_channel=0`.
- **The Higgs is drawn *after* both of its Z daughters.** No Z is bounded by m_H − m(other Z); both are bounded only by √ŝ minus the other final-state masses.

## 2. The two Z's: separate channels for each assignment?

**Yes.** This is the central finding.

### 2a. Mechanism (source)

**Step 1 — conflict detection.** **[src]** `MG/madgraph/iolibs/template_files/madevent_symmetry.f`, `BW_Conflict` (l.286-432):
- It walks up the tree and gives each resonant daughter its on-shell mass, `xmass(-i) = max(xmass, M + 3Γ)` (l.381).
- A propagator is flagged "conflicting" when the sum of its daughters' masses exceeds its own pole mass (l.372-378). For H: 2×(91.188 + 3×2.44) ≈ 197 > 125, so H is conflicting.
- All daughters of a conflicting BW are then marked conflicting too (l.389-407), so both Z's become conflicting.

**Step 2 — enumeration.** **[src]** `write_bash` (l.197-266):
- Every conflicting BW gets an `iarray` slot with value 1 = "required on shell" or 2 = "excluded/off shell" (base 3; `bw_increment_array`, l.554-595).
- Every combination becomes its own integration channel, named `G<config>.<code>` (the "dconfig").
- `failConfig` (l.434-552) drops combinations that cannot be realised.
  - With `iarray=1` a BW contributes at least M − 5Γ, and the combination fails if the daughter sum exceeds M + 5Γ (l.532-538).
  - So "Z on + Z on + H on" fails: 2×(91.188 − 12.2) = 158 > 125.03.

**Step 3 — per sub-channel, a hard cut.** **[src]** `myamp.f:cut_bw`, l.186-200:
- `onshell = |m − M| < 5·Γ` (l.192-193). The bwcutoff value is used here only for forced BWs (`gForceBW`), l.188-190.
- The point is cut if a required BW is off shell or an excluded BW is on shell.
- So the sub-channels **partition** phase space exactly, and their sum is the config's full integral.

**Step 4 — per sub-channel, the map.** **[src]** `myamp.f:set_peaks`:
- A BW with `lbw ≤ 1` (required, or not conflicting) gets the Breit–Wigner map: `spole/swidth`, "Setting BW" (l.429-449). It is used in `dsample.f:1393-1396` through `transpole`, an arctan map over the full [0, 1]·stot range (`transpole.f:41-48`).
- A required BW also has its lower limit raised to M − 5Γ (l.405-407).
- An **excluded** BW (`lbw=2`) gets **no** BW map. It gets `setgrid(-i, xo, a, 1)` instead (l.462-485). This is a VEGAS grid preset to a 1/x (log-spaced) shape: 90% of bins between xo and 1 (`dsample.f:938-998`), with xo set from the pT·ΔR cuts.

### 2b. What the process actually ran

**[src]** From `P1/G121.*` and `G122.*/run_01_log.txt`. The `BW Setting` column order is (−1, −2, −3, −4, −5); 1 = required, 2 = excluded.

| Channel | BW Setting | Meaning | σ (pb) ± err | Iterations |
|---|---|---|---|---|
| G121.205 | 1 2 1 1 2 | Z₋₁(ℓℓ) on, Z₋₂(ℓℓ) off, H on, Z_cc on | 5.737e-10 ± 0.088e-10 | 3 |
| G121.203 | 2 1 1 1 2 | Z₋₁ off, Z₋₂ on, H on, Z_cc on | 5.805e-10 ± 0.088e-10 | 3 |
| G121.206 | 2 2 1 1 2 | both Z off, H on | 3.889e-10 | 3 |
| G121.211 | 1 1 2 1 2 | both Z on, **H off** | 1.912e-09 | 3 |
| G122.203 | 2 1 1 1 2 | **Z_μμ off, Z_cc on**, H on | 1.007e-09 ± 0.015e-09 | 3 |
| G122.205 | 1 2 1 1 2 | **Z_μμ on, Z_cc off**, H on | 4.912e-10 ± 0.071e-10 | 3 (χ²/dof 1.03) |
| G122.206 | 2 2 1 1 2 | both off, H on | 5.698e-10 | 3 |

- **[src]** Full list: G121 and G122 have 18 sub-channels each: .134 .158 .160 .161 .203 .205 .206 .211 .212 .214 .215 .230 .232 .233 .238 .239 .241 .242. **No (1,1,1,·,·) combination exists**, as `failConfig` requires.
- **[src]** The `.134/.158/.160/.161` jobs require the ŝ Z (−5) on shell. They report "No points passed cuts!", which is impossible at fixed √s = 500. They are harmless, since `failConfig` only checks stot ≥ mtot².
- **[src]** Log excerpts:
  - `G122.205`: "Requiring BW -1 1", "Excluding BW -2 2", "Setting BW -1 1 91.188", "Setting grid 2 0.20480E-03 1", "Setting BW -3 3 125.0".
  - In words: Z→μμ is BW-mapped and required within 5Γ_Z; Z→cc̄ is off shell (|m − 91.188| ≥ 12.2 GeV) and sampled on a log grid from about 7.2 GeV upward; H is BW-mapped and required within 5Γ_H ≈ 0.032 GeV.
- **[inf]** The session's problem points (m_μμ = 85.2, m_cc̄ = 20.9, m_4f = 124.99) satisfy |85.2 − 91.188| = 6.0 < 12.2, |20.9 − 91.188| > 12.2 and |124.99 − 125| = 0.01 < 0.032. They lie **inside the support of G122.205**, whose map was built for exactly this region: BW on μμ, log grid on cc̄, BW on H.
- **[inf]** The H → cc̄ ee analogue (config 123) is covered by the same channel through the e↔μ permutation (§1b).
- **[src]** The same split exists for the Yukawa-attached H configs:
  - 464: .014 = (Z₋₁ off, Z₋₂ on, H on) 1.92e-10; .016 = (Z₋₁ on, Z₋₂(cc̄) off, H on) 7.6e-11; .017 = both off; .022 = both on, H off.
  - 448: .014 / .016 = 1.03e-10 / 1.03e-10.
  - 478: .014 / .016 = 5.20e-9 / 5.55e-9.

### 2c. How the draw order and kinematic limit treat the off-shell Z

- **[src]** Within one sub-channel there are two cases, set by `lbw` and not by draw order:
  - With both Z slots required (`.211`-type), both are BW-mapped and drawn −1 then −2. Each smax is limited only by √ŝ − Σ(other masses) (genps.f:822-823), never by m_H.
  - In the H-on sub-channels exactly one Z is BW-mapped and the other has a 1/x grid.
- **[inf]** MG does **not** bound the second Z by m_H − m(first Z). That bound is enforced afterwards:
  - H's `smin = (m₋₁+m₋₂)²` (genps.f:822) rejects points (`jac=-5`, l.847-849) when the drawn Z's are too heavy for the H window;
  - `cut_bw` rejects points outside the H window.
  - VEGAS then adapts the grids away from the wasted region.
- **[inf]** Which Z sits at −1 vs −2 (and so is drawn first) is a fixed iforest property. It does not matter for coverage, because the on/off assignment is enumerated over both slots.
- **[src]** `transpole` is applied inside `sample_get_x`, after the [xmin, xmax] range has been translated into grid-bin limits (`dsample.f:1316-1330, 1393-1396`). A BW whose range has been cut by an earlier draw is therefore sampled only in the allowed sub-range, with the matching Jacobian.
- **[src]** `bwcutoff = 15` is not used for this split. It applies only to forced/decay-chain BWs (`gForceBW=1`, myamp.f:188-190, 402-404; symmetry.f:519-531) and to the `OnBW` flag written to the LHE (myamp.f:136-139). The split uses a hard-coded 5Γ.

## 3. Channel weighting (SDE)

- **[src]** Run card: `sde_strategy = 2`. Definition, `banner.py:4458`: "1 means full single diagram enhanced (hep-ph/0208156), 2 use the product of the denominator".
- **[src]** Implementation, `matrix_madevent_group_v4.inc:213-228`:
  - `XTOT = Σ_I AMP2(J)` over **all** configs `I=1..LMAXCONFIGS` with `CONFSUB≠0`, and `ANS = ANS·AMP2(channel)/XTOT`.
  - For `sde_strat=1`, `AMP2(J) = |M_J|² · GET_CHANNEL_CUT`.
  - For `sde_strat=2`, `AMP2(J) = GET_CHANNEL_CUT(P,I)`.
- **[src]** `GET_CHANNEL_CUT` (`genps.f:1817-1958`) with `sde_strat=2` is the product over the config's propagators:
  - 1/((s − M²)² + M²Γ²) for each s-channel propagator (l.1944-1952);
  - 1/(t − M² + stot·1e-10)² for each t-channel propagator (l.1930-1935);
  - `tmin_for_channel = -1` here (`build.log:303`), so the t-damping is inactive.
- **[src]** With `sde_strat=1` the `|M_i|²` lines are filled (`matrix_madevent_group_v4.inc:376-378`); with `sde_strat=2` they are not, so the channel weight here does not use any amplitude.
- **[src]** The `cut_bw` veto for `gForceBW=2` applies only for `sde_strat=1` (myamp.f:142-145). It is irrelevant here.
- **[inf]** At the problem point the weight factors of every H-containing config are enhanced by the H BW, a factor (MΓ)⁻² ~ 1/0.64 GeV⁻⁴ near the pole versus ~1/(Δs)² elsewhere. So the point is assigned overwhelmingly to the H configs, and among them to 122 (incl. 123) and the 464/467-type graphs, whose cc̄–μμ pairing matches.
- **[inf]** The sub-channel split works under any SDE weighting. `w_i(p)` is a property of the config, and its sub-channels partition the config's support. The cc̄-light configuration therefore gets `f·w_122/Σw` integrated by a job (G122.205) whose sampling density is shaped for that region. The weight is not left on a BW tail.
- **[inf]** For our sampler this difference matters:
  - MadEvent integrates each channel separately (`∫ f·w_i/g_i dg_i`, w_i = SDE fraction). A region gets a large weight only if the channel that owns it via w_i has a map that misses it.
  - Our one-sample balance heuristic f/Σα_k g_k produces large weights wherever **all** channel densities are small. Both schemes need some channel whose density covers the region, and MadEvent has one.
- **[src]** `symfact` does not drop channels. It merges lepton-relabelled copies, which are still integrated through `SYMCONF`/`PERMS` (§1b). No Higgs channel is lost: 121, 122 (+123), 448, 453, 464 (+474), 467 (+475), 478 and 481 (+484, inferred by the same e↔μ rule) all have G dirs or a merge partner.

## 4. Budget

- **[src]** Survey: every sub-channel job prints "Number of events and iterations 1000 5 3" and "Desired fractional accuracy 0.1". So each G sub-channel gets 1000 points per iteration, at least 3 and at most 5 iterations, stopping early at 10% accuracy.
  - Defaults: `madevent_interface.py:2053-2055` (`points` 1000, `accuracy` 0.1); minimum iterations, `gen_ximprove.py:72` (`min_iterations = 3`).
  - This minimum is per sub-channel and independent of its σ, so every on/off-shell half gets at least 3000 points.
  - **[inf]** It may be scaled in some modes; `madevent_interface.py:3579-3584` scales points by `(rate+2)`.
- **[src]** Two jobs per survey job (`export_v4.py:write_run_config_file`, `job_per_chan = 2`). `build.log`: 214 survey jobs for 417 G dirs.
- **[src]** Refine (`gen_ximprove.py:find_job_for_event`, l.1112-1136):
  - Channels are refined when `goal_lum / channel_luminosity ≥ 1.1` or their error is too large.
  - Channels with `axsec == 0` are skipped.
  - Other channels are **not dropped**; their survey events and weights stay in the combination.
  - Here only 13 channels were refined (`build.log:371-388`): G463, G476, G479, G480 and some `.004` sub-channels. **None** of the H sub-channels were refined; they were already converged in the survey at 1.2–1.7% relative error.
- **[src]** `refine_evt_by_job` / `min_event_in_iter = 1000` (`gen_ximprove.py:990-996`) set the minimum per refine job.
- **[src]** The Z → 4f-type channels with tiny σ were kept and integrated: G1 = 1.5e-14, G2 = 7.4e-13 pb, with 3–5 iterations. They are not starved below the survey floor.
- **[src]** Grid card `ngran = -1`.
- **[src]** Unweighting: `build.log` reports "efficiency 11 %, truncation 0.029 %".

## 5. Conclusion

- **[src + inf]** MadEvent covers **both** halves of the H → ZZ* peak, through dedicated channels for each:
  - **121** (H → ee μμ): G121.203 and G121.205. Their equal σ (5.81e-10 vs 5.74e-10) is the e/μ symmetry. **[inf]**
  - **122/123** (H → cc̄ ℓℓ): G122.203 (cc̄ on, ℓℓ off) and G122.205 (ℓℓ on, cc̄ off).
  - Plus a "both off-shell, H on" job (`.206`) and the H-off-shell jobs (`.211` etc.).
- **Mechanism.** In order: `BW_Conflict` (madevent_symmetry.f:367-407), on/off enumeration with `failConfig` (l.197-266, 434-552), the per-job hard 5Γ cut (`cut_bw`, myamp.f:186-200), BW mapping only for required resonances, and a log-grid map for excluded ones (`set_peaks`, myamp.f:399-490).
- **[inf]** Our "one side only" coverage is a **deviation** from MadEvent.
  - Our H channels fix a draw order: H, then a fixed pair on the Z pole, then the other pair bounded by m_H − m₁.
  - MadEvent draws daughters first and enumerates which daughter is on shell.
  - The direct analogue for us is to split each H channel into "pair A on-pole / pair B log-or-1/s" and "pair B on-pole / pair A log-or-1/s", rather than relying on BW tails.
  - MadEvent's split is a hard partition with a cut at |m − M_Z| = 5Γ_Z ≈ 12.2 GeV. With our multichannel weights, a symmetric pair of channels sharing α (no cut) is the smooth equivalent. **[inf]**
- **Differences to keep in mind when comparing:**
  - MadEvent uses SDE with propagator products (`sde_strategy=2`), not f/Σα g. Each sub-channel is integrated and refined independently.
  - Lepton-relabelled configs are merged (123 into 122) through random permutation in `DSIG`, rather than appearing as separate channels.

### Could not be determined from the sources

1. **Exact `configs.inc`, `props.inc`, `symfact.dat` and `symperms.inc` contents.** They are absent from every copy of this output.
   - The iforest labelling (which of −1 and −2 is ee vs μμ in config 121), the exact `sprop` IDs and the exact 123 → −122 entry are inferred from the logs (two SYMCONF rows in G122; grid xo values) and from the tag rules. They were not read.
   - Whether 474/475/484 map onto 464/467/481 is likewise inferred: no G dir exists for them, and the e↔μ tag rule predicts it.
2. **Whether the coloured c, c̄ legs' tag also matches across c ↔ c̄ swaps.** Not checked in `model.get_particle` colour sign; it does not affect the H conclusion.
3. **Per-event weight distributions inside G122.205.** The logs give only "Events wgts > 1: 97" out of 3649 found and the overflow fraction; no max-weight histogram survives. So I cannot quantify how MadEvent's weights in the cc̄-light region compare with ours, beyond the channel converging at 1.4% with χ²/dof ≈ 1.
4. **Why G122.203 is twice G122.205 (1.0e-9 vs 0.49e-9)** when |M_122|² is symmetric under the swap. The likely causes are a smaller SDE share for 122 where cc̄ is light (γ* → cc̄ configs have 1/s² denominators that compete) and the harder jet cuts on a light cc̄ pair (ptj = 20 vs ptl = 10). **[inf]** I did not verify this.
5. **The `madevent_interface` survey-point scaling** for this run: whether `rate` modified `points`. The logs show 1000 points per iteration, which is the effective value.
