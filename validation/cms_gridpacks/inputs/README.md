# VBF/EWK Drell-Yan gridpack inputs

These are the cards of three CMS Run-3 gridpacks (RunIII2024Summer24, MadGraph 2.9.18, `CMSSW_12_4_8`), copied byte for byte from the tarballs under
`/cvmfs/cms-griddata.cern.ch/phys_generator/gridpacks_tarball/CommonBkg/RunIII2024Summer24/MadGraph5_aMCatNLO/{DY,VBF}/`.

| file | path in the tarball |
|---|---|
| `proc_card.dat` | `InputCards/<name>_madgraph-pythia8_proc_card.dat` |
| `customizecards.dat` | `InputCards/<name>_madgraph-pythia8_customizecards.dat` |
| `run_card.dat` | `process/madevent/Cards/run_card.dat` |
| `param_card.dat` | `process/madevent/Cards/param_card.dat` |
| `restrict_ckm_no_b_mass.dat` | `mgbasedir/models/loop_sm/restrict_ckm_no_b_mass.dat` |

**`run_card.dat` is the resolved card MadGraph ran with.** The `InputCards` run card is one shared template: it still contains `$DEFAULT_PDF_SETS`, and the customize card supplies its `mmjj` and `mmll`.

**`param_card.dat` is the card the reference σ was computed with.** It carries the customize card's settings, which differ from the restrict file's defaults:

| parameter | customize card | restrict file default |
|---|---|---|
| `aEWM1` | 128.825 | 132.507 |
| `Gf` | 1.1663787e-5 | 1.16639e-5 |
| `MZ` | 91.1535 | 91.188 |
| `MW` (derived) | 79.907 | |
| `WZ` | 2.4932 | 2.4414 |
| `MT`, `ymt` | 172.5 | 173.0 |
| `WT` | 1.311 | 1.4915 |

A run without these settings will not reproduce MadGraph's σ, even with full coverage.

| gridpack | `mmjj` | `mmll` | `mmllmax` | `ptj`, `ptl` | MadGraph σ (pb), survey |
|---|---|---|---|---|---|
| `DYto2L-2Jets-EWK_Bin-MLL-50` | 4 | 50 | none | 0, 0 | 13.12 ± 0.031 |
| `VBFto2L_Bin-MLL-50` | 100 | 50 | none | 0, 0 | 7.574 ± 0.030 |
| `VBFto2L_Bin-MLL-4to50` | 100 | 4 | 50 | 0, 0 | 117.6 ± 0.44 |

The σ values are the survey results from each tarball's `gridpack_generation.log`, the reference the CMS gridpack survey compares against. All three use PDF 325300 (`NNPDF31_nnlo_as_0118_mc_hessian_pdfas`), `dynamical_scale_choice = -1`, and `ell = e mu ta`. The τ pairs are in their own MadEvent P directory.

The per-P-directory σ, summed from each P directory's channel `results.dat`:

| gridpack | `P0_qq_llqq` (e, μ) | `P0_qq_taptamqq` (τ) |
|---|---|---|
| `DYto2L-2Jets-EWK_Bin-MLL-50` | 9.0205 ± 0.066 | 4.0458 ± 0.046 |
| `VBFto2L_Bin-MLL-50` | 5.3933 ± 0.036 | 2.1736 ± 0.027 |
| `VBFto2L_Bin-MLL-4to50` | 88.498 ± 0.7 | 29.822 ± 0.9 |
