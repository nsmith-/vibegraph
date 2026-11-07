# Gridpacks with new failures in campaign `main-202a540-200-pc`

The 11 gridpacks in this directory reach a failure in campaign `main-202a540-200-pc` (build
`refdata-9-90-g202a540f`, with the survey parser's per-tag multiplicity fix on
top) that they did not reach in the baseline `main-23d3a76-200-pc`. Lifting an
earlier refusal exposed each failure. The file layout is the one described in
`../pr93-decay-chain-duplicates/README.md`. `outcome.txt` replaces
`refusal.txt`: its first line holds the final attempt's outcome, σ and gpdb
patches, and the rest is the error text.

Each directory carries the run and param cards in two forms: the ones the
gridpack used (`run_card.dat`, `param_card.dat`) and the CMS input cards
(`*_run_card.dat`, `*_customizecards.dat`).

| group | gridpack | process | outcome (seed 1) | MG σ (pb) |
|---|---|---|---|---|
| A massive light quark | TCP_m50_ht_400toInf | `p p > alp j (j), alp > ta+ ta-`, MLM | ok, σ/MG 3.89 | 4.079e-5 |
| A | TCP_m55_ht_400toInf | same, m_alp 55 | ok, σ/MG 4.21 | 4.364e-5 |
| A | VBF_HH_kl_1p00_cv_1p00_c2v_1p00 | `p p > h h j j $$ z w+ w- / a j QED=4` | ok, σ/MG 0.854 | 1.911e-3 |
| B CKM near-degenerate groups | GJ-2Jets-EWK_Bin-PTG-40_madgraph-pythia8 | `p p > a j j QCD=0 QED=3 $ t t~` | ok, σ/MG 0.588 | 164.6 |
| B | GJ-2Jets-EWK_madgraph-pythia8 | same, inclusive | ok, σ/MG 0.503 | 5894 |
| B | VBFtoG_Bin-PTG-100to200_madgraph-pythia8 | `p p > a j j QCD=0 QED=3` | ok, σ/MG 0.745 | 7.57 |
| B | VBFtoG_Bin-PTG-200_madgraph-pythia8 | same | ok, σ/MG 0.738 | 1.08 |
| C panic | DarkPhoton-monojet_…_MZd-40_… | HAHM monojet | panic, RAMBO needs ≥ 2 final momenta | 0.2167 |
| C | DarkPhoton-monojet_…_MZd-86_… | same | panic, same | 4.431 |
| C | idm_dilepton_mH80_mA160 | InertDoublet, 2 → 4/6 | panic, no cluster scale | 0.009 |
| D scale floor | QCD-4Jets_HT-100to200_madgraphMLM-pythia8 | 4-jet MLM, HT 100–200 | refused, every μF below 2 GeV | 1.643e8 |

For A and B, the offsets hold on every one of seeds 1–5 (campaigns
`main-202a540-outl-s1..s5`):

| gridpack | mean σ/MG over seeds 1–5 |
|---|---|
| TCP_m50 | 4.19 ± 0.16 |
| TCP_m55 | 4.22 ± 0.05 |
| VBF_HH | 0.840 ± 0.012 |
| GJ-2Jets-EWK_Bin-PTG-40 | 0.595 ± 0.026 |
| GJ-2Jets-EWK inclusive | 0.443 ± 0.039 |
| VBFtoG_Bin-PTG-100to200 | 0.686 ± 0.022 |
| VBFtoG_Bin-PTG-200 | 0.774 ± 0.032 |

- **DarkPhoton attempt 1** is refused because the param card sets mass 1000022,
  which the model lacks. The panic comes on attempt 2, which runs without the
  param card.
- **idm_dilepton's `gridpack_generation.log`** is cut to its first and last 400
  lines. The full log is 34 MB.
