# Massive-incoming-parton cards (#95)

Two of the 14 gridpacks whose final attempt in campaign `main-23d3a76-200-pc`
(build `refdata-9-69-g23d3a766`) is refused with `subprocess {} has an incoming
leg of mass {} GeV`, chosen as the cheapest b-mass and c-mass cases. The file
layout is the one in `../pr93-decay-chain-duplicates/README.md`.

| gridpack | process | incoming masses (param card) | MG σ (pb) | MG cumulated time |
|---|---|---|---|---|
| ZprimeToTauTau_M130_xqcut20_b0p02_Mu0p0_Tau0p17_bs1 | `p p > zp (j j), zp > ta+ ta-`, 0–2 jets MLM, p includes b | MD 5.04e-3, MU 2.55e-3, MS 0.101, MC 1.27, MB 4.7; maxjetflavor 5 | 13.06 ± 0.028 | 706 s |
| LL_HAHM_MS_400_kappa_0p01_MZd_14p0_ctau_1000 | `p p > h > zp zp` | MC 1.42, MB 4.7, u/d/s massless; maxjetflavor 4 | 0.01106 ± 5.7e-5 | 1.3 s |

- **b-mass case:** no gridpack in the set has a b quark as the *reported* leg;
  the refusal names the first massive leg it meets. ZprimeToTauTau is the only
  family with b in the proton and MB = 4.7 (its refusal names `s b~ > ta+ ta-`
  with MS = 0.101). Its sibling `_M60_xqcut10` has the same cards at a lower
  mass and took 1246 s.
- **c-mass case:** the other c-mass family, `idm_dilepton_*` (MC = 1.2), is a
  2 → 4/6 process.
- **Patches on the final attempt:** ZprimeToTauTau ran the gridpack's bundled
  model (`model:gp_f649c998`). LL_HAHM ran the bundled model and had `ptonium`
  and `etaonium` reset to MadGraph's defaults (1 and 0.6); its run card sets
  values the CLI refuses. Those resets are vibegraph's, not the card's.
- **LL_HAHM model path:** `proc_card_mg5.dat` imports the model by an AFS path
  split over two lines; the model is `HAHM_MG5model_v3/HAHM_variablesw_v3_UFO`
  (InputCards proc card), shipped as `ufomodel/`. It has no restriction.
