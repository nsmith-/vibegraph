# Decay-chain duplicate-subprocess cards (#93)

The 11 gridpacks whose final attempt in campaign `main-23d3a76-200-pc`
(build `refdata-9-69-g23d3a766`) is refused with `subprocess '{}' is produced
by two process lines`. Every one was refused on attempt 1, with no patches.

Each directory holds files copied unmodified from the gridpack tarball:

| file | from the tarball |
|---|---|
| `<label>_proc_card.dat`, `_run_card.dat`, `_customizecards.dat`, `_extramodels.dat` | `InputCards/` |
| `run_card.dat`, `param_card.dat`, `proc_card_mg5.dat` | `process/madevent/Cards/` |
| `gridpack_generation*.log` | top level |
| `restrict_<name>.dat` | `mgbasedir/models/<model>/`, when the imported model has a restriction |
| `ufomodel/` | `process/madevent/bin/internal/ufomodel/`, for a model that is not built in (symlinks dropped, `*.pyc` skipped) |

`refusal.txt` is the CLI's full error text on the final attempt. `source.json`
records the tarball path, the imported model, the patches, MadGraph's reference
σ (the survey line of `gridpack_generation.log`) and the list of copied files.

| gridpack | model | MG σ (pb) |
|---|---|---|
| TTToHcToWA_AToMuMu_MHc160_MA125 | 2HDM_UFO-type1_5FS | 5.297e-4 |
| TTToHcToWA_AToMuMu_MHc160_MA15 | 2HDM_UFO-type1_5FS | 0.591 |
| TpTpTo2T2STo2Mu2B_MS30 | eVLQS012_v8_4FNS_UFO | none in the log |
| VLLD_ele_M600 | VLL | 0.1305 |
| VLLD_mu_M1150 | VLL | 0.18 |
| VLLVLLToZHTo2MuInv_MVLL-250_MH-95 | VLL | 3.057e-8 |
| VLLVLLToZHTo2MuInv_MVLL-500_MH-80 | VLL | 1.46e-7 |
| Wto2Q-3Jets_Bin-HT-400to800_madgraphMLM-pythia8 | sm-ckm_no_b_mass | 1777 |
| Wto2Q-3Jets_Bin-HT-800to1500_madgraphMLM-pythia8 | sm-ckm_no_b_mass | 148.9 |
| Zto2Q-4Jets_Bin-HT-100to400_madgraphMLM-pythia8 | sm-ckm_no_b_mass | 32870 |
| Zto2Q-4Jets_Bin-HT-800to1500_madgraphMLM-pythia8 | sm-ckm_no_b_mass | 155.5 |
