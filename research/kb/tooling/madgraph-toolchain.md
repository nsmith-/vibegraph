---
type: Procedure
title: "Running MadGraph here: the pixi env and the pinned submodule"
description: "The madgraph pixi env supplies the toolchain (packaged 3.5.7, never run as the generator); mg5_pinned.sh runs the pinned 3.7.1 submodule; LHAPDF paths and the C++ runtime link fix."
status: draft
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-09}]
tags: [madgraph, pixi, lhapdf, toolchain]
generated: {by: claude-code/claude-opus-5-5, at: 2026-10-09}
sources:
  - {id: n05, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/05-madgraph-setup.md#L14-L163", title: "Note 05: MadGraph setup via pixi/conda (package, env, caveats)"}
  - {id: n18-h1, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L479-L498", title: "Note 18 §5 H1: the LHAPDF oracle build"}
  - {id: n18-ldflags, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L856-L862", title: "Note 18 §5 H7: MG–LHAPDF link workaround"}
  - {id: n18-outcome, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/18-hadronic-xsec-design.md#L1003-L1009", title: "Note 18 Outcome: the LDFLAGS finding"}
  - {id: n27-b4, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/27-v3-backlog-plan.md#L482-L560", title: "Note 27 §B4: the 3.7.1 mechanism"}
  - {id: n41-m0, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/research/notes/41-mlm-feature-sprint-plan.md#L432-L440", title: "Note 41 §M0: libstdc++ on Linux"}
  - {id: mg5-pinned, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/mg5_pinned.sh", title: "validation/madgraph/mg5_pinned.sh"}
  - {id: build-sh, resource: "https://github.com/nsmith-/vibegraph/blob/787070e46f8b4d247ad020079ba9fcf9a5b37cd8/validation/madgraph/build.sh#L160-L185", title: "validation/madgraph/build.sh, the LDFLAGS block"}
  - {id: mg-repo, resource: "https://github.com/mg5amcnlo/mg5amcnlo/tree/b7687064", title: "mg5amcnlo at the pinned commit (3.7.1)"}
---
# Running MadGraph here: the pixi env and the pinned submodule

Every MadGraph reference run is made by the **pinned submodule**,
`research/refs/mg5amcnlo` at `b7687064` (MadGraph 3.7.1, 2026-04-29), run
through `validation/madgraph/mg5_pinned.sh`. The one exception is six runs in
the bundle that measure a 3.5.7 defect (`ee_to_mumu_tata_qcd0`'s window,
anti-window and three control runs, and `var_sde1`), kept at the version they
measure (`validation/manifest.toml`, the `[refdata]` comment). The `madgraph` pixi environment
supplies everything around it. Why 3.7.1 and not the packaged 3.5.x is
[validation/madgraph-oracle-pinning](../validation/madgraph-oracle-pinning.md);
the codebase itself is [MadGraph5_aMC@NLO](../references/codebases/madgraph5-amcnlo.md).
Per-process facts live in `validation/manifest.toml` and
`validation/madgraph/README.md`; which task to run after which change is the
`extended-validation` skill. This concept covers the environment.

## The pixi environment

`pixi.toml`'s `madgraph` feature pins `python = "3.11.*"`,
`mg5amcnlo = "==3.5.7"` from conda-forge and `pylhe`, and carries the
reference-generation tasks (`build-diagrams`, `generate-amplitude`,
`generate-references`, the `validate-*` gates that need generated data; 49
tasks in all at the time of writing; read `pixi.toml` for the list rather
than any copy of it).
Run everything with `pixi run -e madgraph <task>`; `pixi install -e madgraph`
creates it.[^n05]

The packaged `mg5amcnlo` 3.5.7 is not the generator for any reference. Two
generators still reach it, as `mg5_aMC` on PATH:
`gen_pta_windows.sh` runs a deliberate 3.5.7-versus-3.7.1 comparison (it
selects by version and checks `MGMEVersion.txt`), and `gen_higgs_window.sh`
calls `mg5_aMC` unconditionally; its committed
`higgs_window_reference.json` records `"mg_version": "3.5.7"` and is a study of
the 3.5.x resonance defect, not a 3.7.1 reference. Otherwise the package is
there for what it pulls in: gfortran (`fortran-compiler`), `cxx-compiler`,
`make`, LHAPDF (6.5.6, with `lhapdf-config`), Python with `six` and `numpy`,
and the rest of its conda dependencies. That is everything the submodule needs
to generate (Python 3 with `six`) and that a generated directory needs to build
and run `madevent` (gfortran, LHAPDF).[^n27-b4] The conda package resolves
natively on `osx-arm64`; the bank host is an Apple M3 Max, and generation has
also run on Linux containers.

The submodule is a plain source checkout, and it runs directly: generation
needs only Python and `six`. (An early belief that a git checkout cannot run
and only the release package works is wrong.) Populate it with
`git submodule update --init --depth=1` or `pixi run init-sm-submodule`;
`research/refs/README.md` has the details. Prefer a real checkout to a copied
tree: a copy without `.git` makes MadGraph print a red "UNKNOWN DEVELOPMENT
VERSION" warning that lands in every banner it writes, which `build.sh`'s
`strip_ansi_escapes` has to remove before an LHE banner parses as XML.

## `mg5_pinned.sh`

`bash validation/madgraph/mg5_pinned.sh <script.mg5>` runs
`python $MG5_ROOT/bin/mg5_aMC` on the script, where `MG5_ROOT` is the
submodule, or `$VG_MG5_ROOT` for a patched copy (how a generator applies a
patch from `validation/madgraph/patches/` without touching the tree the
source-level gates read). It handles three environment details so callers do
not repeat them:

- **Scratch directory.** MadGraph drops a `py.py` into its working directory,
  so it runs in a temporary one and the repository and the submodule stay
  clean. Paths inside the script must therefore be absolute; `build.sh`
  rewrites each script's `output` and `import model validation/ufo/…` lines to
  work-area paths before handing it over.
- **LHAPDF data path.** `LHAPDF_DATA_PATH` is `validation/pdf` first, then
  `lhapdf-config --datadir`, so a `pdlabel = lhapdf` run with `lhaid 247000`
  reads the set this repository pins rather than one MadGraph downloads; the
  installed directory stays on the path for `lhapdf.conf` and the set index.
  `gen_mlm_references.sh` sets the same path.
- **No browser, no notifications.** The pinned checkout has no site
  configuration, so `automatic_html_opening` and `notification_center` default
  to on, and a batch generation opens a browser tab and posts a desktop
  notification per process directory. The wrapper prepends
  `set automatic_html_opening False --no_save` and
  `set notification_center False --no_save`. `--no_save` keeps them out of the
  submodule **and** out of the generated `Cards/me5_configuration.txt`, so a
  caller that later runs `bin/generate_events` must silence that file itself
  (`silence_madgraph_ui` in `gen_hadronic_sigma.sh`).

## The C++ runtime link fix

A `pdlabel = lhapdf` run links LHAPDF's C++ library into `madevent`
(`libpdf.a`). MadGraph's `Source/make_opts` adds the C++ runtime only as
`LDFLAGS=$(STDLIB) $(MACFLAG)` inside an `ifeq ($(origin LDFLAGS), undefined)`
guard (`Template/LO/Source/.make_opts:71-73` at the pin; `STDLIB` is written at
output time, `-lc++` where the compiler uses libc++ and `-lstdc++` otherwise,
`export_v4.py:2428-2434`), and the conda
activation exports its own `LDFLAGS`, so the guard sees it set and drops the
runtime. The link then fails with `__cxa_throw` and `__gxx_personality_v0`
unresolved.[^n18-ldflags][^n18-outcome] The fix is to append the platform's C++ runtime to
`LDFLAGS` for any command that builds `madevent`:

| platform | runtime | where applied |
|---|---|---|
| macOS | `-lc++` (libc++) | `build.sh`, `madevent_seeds.sh`, `time_stages.py`, and hard-coded in `gen_hadronic_sigma.sh`, `gen_higgs_window.sh`, `gen_pta_windows.sh` |
| Linux | `-lstdc++` (libstdc++) | `madevent_seeds.sh` (`mes_ldflags`) and `time_stages.py`; `build.sh` appends nothing on Linux |

Naming the wrong runtime fails the link instead of fixing it: `-lc++` is
generally absent on Linux.[^n41-m0] The three scripts that hard-code `-lc++`
are macOS-only as written. When adding a generator, take the platform switch
from `madevent_seeds.sh`, and read the surrounding control flow in MadGraph's
make files rather than the single cited line.

## The LHAPDF oracle build

The PDF-grid oracle (`validation/pdf/gen_oracle.cpp`, task
`generate-pdf-oracle`) is a standalone C++17 program against the LHAPDF C++
API, built with the system `c++` and the environment's LHAPDF:

```
c++ -std=c++17 -O2 gen_oracle.cpp $(lhapdf-config --cflags --ldflags) -lLHAPDF \
    -Wl,-rpath,"$(lhapdf-config --libdir)" -o gen_oracle
```

The embedded rpath makes the binary run without `DYLD_LIBRARY_PATH`. It uses
MadGraph's own LHAPDF build, so there is no separate version pin to drift; it
writes JSON with `%.17g`.[^n18-h1] What it checks is
[the LHAPDF oracle](../validation/lhapdf-oracle.md).

## Other caveats

- **Output directories are not overwritten.** MadGraph refuses to write into an
  existing process directory, and `build.sh` skips any script whose output
  directory exists. Regeneration and its traps (a missing directory is rebuilt
  silently) are [reference generation](madgraph-reference-generation.md).
- **LO only.** The package carries NLO libraries (OneLoop, IREGI, Collier);
  nothing here uses them. Scripts use `generate …` without `[QCD]`.
- **3.5.7 and 3.7.1 write different Fortran.** 3.7.1 stores the colour matrix as
  packed upper-triangle integers over a common `DENOM` where 3.5.x wrote a
  square real array; any tool that reads generated Fortran must accept both
  ([colour oracles](../validation/colour-oracles.md)). Known defects of the
  3.5.x line are [madgraph-defects](../validation/madgraph-defects.md).
- **Checking what ran.** The banner of every run records the version
  (`#*  VERSION 3.7.1  2026-04-29  *`); check it rather than assuming which
  `mg5_aMC` ran.

[^n05]: Note 05, the conda package, its dependencies and the caveats list.
[^n18-h1]: Note 18 §5, H1 decision record (oracle backend).
[^n18-ldflags]: Note 18 §5, H7, "MG–LHAPDF link workaround".
[^n18-outcome]: Note 18, Outcome, the same `LDFLAGS` finding.
[^n27-b4]: Note 27 §B4, "The 3.7.1 mechanism".
[^n41-m0]: Note 41 §M0, the libstdc++ link on Linux.
