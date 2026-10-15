---
type: Session Report
title: "HY1 report: hygiene pass on the permuted-first-call census script"
description: "Review of validation/madgraph/census_permuted_first_call.py: its perms mode could pass vacuously in four ways, the docstring omitted the sigma-can-move condition, and its run-card parser duplicated extract_sigma's. All fixed; runs output is byte-identical."
status: draft
tags: [h1, hygiene, report]
generated: {by: claude-code/claude-sonnet-5-5, at: 2026-10-15}
verified: [{by: claude-code/claude-opus-5-5, at: 2026-10-15}]
---
Commit `b6d546d` on `sigma/h1-ickkw0-census`. Not pushed. No MadGraph run.

## Findings (written before the edits)

| id | pt | site | evidence | conf. | disposition |
|---|---|---|---|---|---|
| F1 | 2 | `perms`, `_symperms` | A `symperms.inc` with no `DATA PERMS` lines gives an empty map, so every config reads `none`: fixture `noperm` printed `none`, rc 0. | checked | fixed |
| F2 | 2 | `perms`, config loop | A `configs.inc` entry whose `mapconfig` is absent from `symperms.inc` was skipped by `if p is not None`: fixture `dangling` printed the other config and rc 0. | checked | fixed |
| F3 | 2 | `perms` | A directory with no `P*` (or a nonexistent path) printed only the header, rc 0; a `P*` without `symperms.inc` printed a soft row, rc 0. | checked | fixed |
| F4 | 2 | `_mapconfig` | Empty `mapconfig` (regex miss) gave nconfigs 0 and `none`. | checked | fixed |
| F5 | 2 | `runs` | No banner under `--output` printed a header, rc 0; a banner without `<MGRunCard>` was read as defaults (`dyn=-1`, scales unfixed, so `clusters=True`). | checked | fixed |
| F6 | 1 | module docstring | It stated the clustering criterion but not that sigma moves only through PDFs, an alpha_s coupling or a rejection (the brief's second item). | checked | fixed |
| F7 | 1/4 | `_card_values` | Duplicates `extract_sigma.read_run_card_scalars` (same regex job, 3875 parameters across the present run cards). | checked | fixed |
| F8 | 1 | `runs` | `l` as a variable name (E741). | checked | fixed |
| F9 | 3 | all | Nothing `pub`-like; `_symfact` silently returns `{}` when neither `symfact.dat` nor `symfact_orig.dat` exists. | checked | leave: the column is informational, and `symperms`/`configs` carry the gate |
| F10 | 1 | `runs`, `grouped` | The `ninit == 1 and several P dirs` heuristic is a proxy for "1 → N is not grouped"; the 83/6 table matches V1's. | suspected, unsettled without MG | leave |

Backlog check: no filed item names `census_permuted_first_call.py` or
`extract_sigma.py` paths. Process-directory walking: `extract_configs.py`,
`extract_diagrams.py` and `gen_amplitude*.py` each take one `P*` of one named row
and read different files, so there is nothing to reuse for the `P*` walk.
Banner parsing: no other script reads `<MGRunCard>` (`gen_kt_cluster_dumps.py`
only normalises banner lines), so the only reusable helper was the run-card one.

## Fixed

- F1 to F5: the script now `sys.exit`s with the path and the missing item.
  Before/after on synthetic fixtures (`output/` carries no `symperms.inc`; see
  brief corrections):

  | fixture | old | new |
  |---|---|---|
  | `good` (3 configs, one swapped) | `2:2:(1,2,3,5,4)` | same, rc 0 |
  | `noperm` | `none`, rc 0 | `no DATA PERMS lines`, rc 1 |
  | `dangling` | one config listed, rc 0 | `maps config 4 to 9, absent from symperms.inc`, rc 1 |
  | `nosym` | soft row, rc 0 | `no symperms.inc; generate the directory with output first`, rc 1 |
  | `empty` | header, rc 0 | `no SubProcesses/P* directory`, rc 1 |
  | nonexistent path | header, rc 0 | same message, rc 1 |

- F6: docstring now gives the sigma-can-move condition and says `runs` does not
  read the alpha_s dependence (judge from `lpp` and the model).
- F7: `extract_sigma.py` gains `parse_run_card_scalars(text)`;
  `read_run_card_scalars` calls it. The census imports it (`sys.path` insert of
  its own directory, as `-I` drops it).

## Mutations

The script has no test harness in the repo (no Python tests, no Python lint in
`pixi.toml` or CI). The fixtures above are the mutation set: each is the
one-line defect the corresponding check must catch; the old script passes all
four and the new one fails all four. Not reachable: a `symperms.inc` whose
PERMS lines parse but are wrong (the oracle is MG's file itself).

## Identity oracles

- `runs`: stdout byte-identical before and after (`cmp`), 90 lines, 83 clusters
  and 6 do not, matching V1's census.
- Parser: old `read_run_card_scalars` body versus the new one over every
  `output/*/Cards/run_card.dat`, equal dicts (3875 parameters).
- `python3 -I extract_sigma.py` still exits 0.

## Gate

- `python3 -I validation/madgraph/census_permuted_first_call.py runs`: rc 0,
  byte-identical to before.
- `perms`: **cannot run on the present refdata**: no `output/*` directory
  holds `symperms.inc` or `configs.inc` (the bundle drops them; V1 deleted its
  regenerated directories). On `output/pp_to_jj` the old script printed five
  "no symperms.inc" rows and exited 0; the new one exits 1. V1's perms table is
  not reproducible without regenerating with MG, which this session did not do.
- Python lint: none configured. Ad hoc `ruff` leaves `EXE001` (file not
  executable; the other scripts are the same), a `noqa` the default rules do
  not need, and C408 `dict(...)`.
- `pixi run --frozen kb-lint`: 0 errors, 2 warnings (pre-existing backlog).
- Rust gates not run: no Rust file changed.

## Method

Reading the script against the brief, then fixtures for each silent path
(about 70%); grep across `validation/madgraph` for duplicates (30%). The
"reuse banner and P* helpers" lead found one helper only.

## Brief corrections

- The gate says `perms` runs on the present refdata. It cannot.
- A stray empty-`$TMPDIR` redirect created `/hy1_runs.tsv` at the filesystem
  root; the sandbox refused my `rm`. It is mine and harmless, for the user to
  delete.

## Found

- `validation/madgraph/census_permuted_first_call.py` has no regression
  fixture. If the census is rerun on new rows, a committed three-file synthetic
  directory plus a one-line check in CI would keep the loud failures loud.
