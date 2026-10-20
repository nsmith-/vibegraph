# MadGraph: `RivetCard.setWeightName` raises on every matched run

A code-level reproducer. It needs only a stock MadGraph5_aMC@NLO Python tree:
no Rivet, Pythia8, Fortran compiler or process directory.

## What it shows

With the default `weight_name`, `RivetCard.setWeightName` raises
`AttributeError` for any run with `ickkw ≠ 0`. A Rivet (or Contur) analysis of
an MLM-matched sample therefore stops before Rivet is launched. Unmatched runs
are unaffected.

## Status

**Filed** as [mg5amcnlo/mg5amcnlo#457](https://github.com/mg5amcnlo/mg5amcnlo/issues/457) on 2026-10-19.

**Present in 3.8.1 and in the 3.5.17 LTS release** (ran, code level). It is
also present in 3.7.1. The report
draft is [`report-draft.md`](report-draft.md).

## Where

`madgraph/various/banner.py`, `RivetCard.setWeightName`:

| version | `def` | failing line | caller |
|---|---|---|---|
| 3.7.1 | `:1696` | `:1706` | |
| 3.5.17 | | `:1675` | |
| 3.8.1 | `:1821` | `:1831` | `madgraph/interface/common_run_interface.py:3178`, in `do_rivet` |

```python
        if self['weight_name'] == "default":
            if runcard['ickkw'] == 0:
                self['weight_name'] = "None"
            else:
                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
```

## Root cause

- *Verified* (the run below): `str` has no attribute `str`, so the
  `ickkw ≠ 0` branch raises `AttributeError: 'str' object has no attribute
  'str'`.
- *Inferred from source:*
  - `do_rivet` calls `setWeightName` whenever `weight_name` is `"default"`,
    which is its default, so every Rivet run on a matched sample hits it.
  - The intended call is `.format(…)`.
  - No end-to-end Rivet run was made: Rivet and Pythia8 are not installed
    here.

## The test

`check_setweightname.py` calls `setWeightName` with:
- a default `RunCardLO` at `ickkw = 0` and at `ickkw = 1`;
- a `PY8Card` with `JetMatching:qCut = 30`;
- a default `RivetCard`.

It prints the expected and observed `weight_name`, and exits 1 if any case
differs.

## Versions tested

| version | source | commit |
|---|---|---|
| 3.7.1 | `git clone --depth 1 --branch v3.7.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.7.1` | `b7687064` |
| 3.5.17 (latest LTS release, 2026-10-09) | `git clone --depth 1 --branch v3.5.17 https://github.com/mg5amcnlo/mg5amcnlo mg-3.5.17` | `4492b3e3` |
| 3.8.1 (latest release, 2026-10-09) | `git clone --depth 1 --branch v3.8.1 https://github.com/mg5amcnlo/mg5amcnlo mg-3.8.1` | `a5b17913` |

3.7.1 and 3.8.1 ran on 2026-10-16 and 3.5.17 on 2026-10-17, with Python 3.11 from vibegraph's `madgraph` pixi
environment. The commands, from this directory, with `$MG` the directory
holding the two clones:

```
pixi run -e madgraph python -I check_setweightname.py $MG/mg-3.7.1
pixi run -e madgraph python -I check_setweightname.py $MG/mg-3.5.17
pixi run -e madgraph python -I check_setweightname.py $MG/mg-3.8.1
```

## Expected against actual

**Expected:** `weight_name = 'None'` at `ickkw = 0` and
`'Weight_MERGING=30.0'` at `ickkw = 1`.

```
== 3.7.1
ickkw = 0  expected weight_name = 'None'  observed: 'None'  [ok]
ickkw = 1  expected weight_name = 'Weight_MERGING=30.0'  observed: AttributeError: 'str' object has no attribute 'str'  [DEFECT]
exit 1
== 3.5.17
ickkw = 0  expected weight_name = 'None'  observed: 'None'  [ok]
ickkw = 1  expected weight_name = 'Weight_MERGING=30.0'  observed: AttributeError: 'str' object has no attribute 'str'  [DEFECT]
exit 1
== 3.8.1
ickkw = 0  expected weight_name = 'None'  observed: 'None'  [ok]
ickkw = 1  expected weight_name = 'Weight_MERGING=30.0'  observed: AttributeError: 'str' object has no attribute 'str'  [DEFECT]
exit 1
```

**Actual:** on all three versions the `ickkw = 1` case raises `AttributeError`, and
the script exits 1.
