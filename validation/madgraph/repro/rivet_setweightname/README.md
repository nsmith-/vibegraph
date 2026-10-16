# MadGraph: `RivetCard.setWeightName` raises on every matched run

A code-level reproducer. It needs only a stock MadGraph5_aMC@NLO Python tree:
no Rivet, Pythia8, Fortran compiler or process directory.

## The defect

`madgraph/various/banner.py`, `RivetCard.setWeightName`:

| version | `def setWeightName` | failing line |
|---|---|---|
| 3.7.1 | `:1696` | `:1706` |
| 3.8.1 | `:1821` | `:1831` |

```python
        if self['weight_name'] == "default":
            if runcard['ickkw'] == 0:
                self['weight_name'] = "None"
            else:
                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
```

`str` has no attribute `str`, so the `ickkw ≠ 0` branch raises `AttributeError`.
- The caller is `do_rivet` in `madgraph/interface/common_run_interface.py`
  (`:3178` in 3.8.1). It calls this whenever `weight_name` is left at
  `"default"`, which is its default.
- So a Rivet (or Contur) analysis of any MLM-matched run stops before Rivet is
  launched.
- An unmatched run takes the `ickkw = 0` branch and is unaffected.

The fix is `.format(…)` in place of `.str(…)`.

## The test

`check_setweightname.py` calls `setWeightName` with:
- a default LO run card at `ickkw = 0` and at `ickkw = 1`;
- a `PY8Card` with `JetMatching:qCut = 30`;
- a default `RivetCard`.

It prints the expected and observed `weight_name`, and exits 1 if any case
differs.

```
python -I check_setweightname.py /path/to/MG5_aMC
```

No end-to-end run was made, since it needs Rivet and Pythia8 installed and
configured (`rivet_path`). The check calls the same method with the same
argument types the interface passes.

## Results

The runs used the Python 3.11 from `pixi run -e madgraph`.
- **3.7.1** is tag `v3.7.1`, commit `b7687064`.
- **3.8.1** is tag `v3.8.1`, commit `a5b17913`, the latest release on 2026-10-16.

```
== 3.7.1
ickkw = 0  expected weight_name = 'None'  observed: 'None'  [ok]
ickkw = 1  expected weight_name = 'Weight_MERGING=30.0'  observed: AttributeError: 'str' object has no attribute 'str'  [DEFECT]
exit 1
== 3.8.1
ickkw = 0  expected weight_name = 'None'  observed: 'None'  [ok]
ickkw = 1  expected weight_name = 'Weight_MERGING=30.0'  observed: AttributeError: 'str' object has no attribute 'str'  [DEFECT]
exit 1
```
