# Card 048: XtoYYprimeTo4Q_Par-MX-3500GeV-MY-600GeV-MYprime-170GeV

Heavy vector triplet, `p p > vz, (vz > h z)`. The gridpack repurposes h and Z
as the Y′ and Y: its customize card sets MH = 170, MZ = 600, MVz = 3500 and all
four widths to 1 MeV.

Source tarball (CVMFS):
`/cvmfs/cms-griddata.cern.ch/phys_generator/gridpacks_tarball/pp/13p6TeV/madgraph/XtoYYprimeTo4Q/XtoYYprimeTo4Q_Par-MX-3500GeV-MY-600GeV-MYprime-170GeV_el8_amd64_gcc10_CMSSW_12_4_8_tarball.tar.xz`

| file | from the tarball |
|---|---|
| `proc_card.dat`, `customizecards.dat`, `extramodels.dat` | `InputCards/` |
| `run_card.dat`, `param_card.dat`, `proc_card_mg5.dat` | `process/madevent/Cards/` |
| `gridpack_generation.log` | top level |
| `ufomodel/` | `process/madevent/bin/internal/ufomodel/`, as `gpdb run` stages it under `--ufo-dir` (symlinks dropped) |
| `vg-output/` | the CLI's output for `gpdb run` campaigns `sprintg-048-s<seed>-<A,B>`, final attempt; `attempt1-refusal.txt` is the first attempt's refusal |

The run card sets the hidden `etaonium = -1`, which the CLI refuses. Every run
here therefore carries `runpatch:etaonium=0.6`: the second attempt runs this run
card with `etaonium` reset to 0.6. A runs without `--param-card` (the model's
defaults); B runs with `--param-card param_card.dat`.

The tarball ships no events, and `runcmsgrid.sh` was not run.
MadGraph's survey line in the log is `1.811e+05 +- 778.8 pb`.
