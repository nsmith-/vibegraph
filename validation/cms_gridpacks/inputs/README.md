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

The per-P-directory σ of that survey, summed from each channel's `pilotrun_results.dat`. These sums reproduce the totals above exactly:

| gridpack | `P0_qq_llqq` (e, μ) | `P0_qq_taptamqq` (τ) | sum |
|---|---|---|---|
| `DYto2L-2Jets-EWK_Bin-MLL-50` | 9.0052 ± 0.028 | 4.1146 ± 0.013 | 13.120 ± 0.031 |
| `VBFto2L_Bin-MLL-50` | 5.4057 ± 0.027 | 2.1686 ± 0.012 | 7.5743 ± 0.030 |
| `VBFto2L_Bin-MLL-4to50` | 87.271 ± 0.37 | 30.377 ± 0.22 | 117.65 ± 0.44 |

**A channel's `results.dat` is not the survey.** It is the gridpack's own test run (`GridRun_234567`, `gseed` 234567), a smaller integration whose sums differ:
- DY gives 9.0205 ± 0.066 and 4.0458 ± 0.046;
- VBF MLL-50 gives 5.3933 ± 0.036 and 2.1736 ± 0.027;
- VBF MLL-4to50 gives 88.498 ± 0.70 and 29.822 ± 0.90.

`mg_channels.csv` in each directory lists every channel's survey and test-run `results.dat` fields, and its survey χ²/dof from the channel log. `../mg_channels.py` writes it from an extracted tarball:
```
python3 -I validation/cms_gridpacks/mg_channels.py <extracted gridpack> --csv mg_channels.csv
```
