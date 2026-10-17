# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `RivetCard.setWeightName` raises `AttributeError` for every matched run (`"…".str(…)` instead of `.format(…)`)

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested at
code level.

### Summary

For a run with `ickkw ≠ 0` and the default `weight_name`, building the Rivet
weight name raises `AttributeError: 'str' object has no attribute 'str'`.
`do_rivet` calls this method for every Rivet run that leaves `weight_name` at
its default, so Rivet (or Contur) cannot run on an MLM-matched sample.

### How to reproduce

No Rivet installation is needed:

```python
import sys; sys.path.insert(0, "/path/to/MG5_aMC")
import madgraph.various.banner as banner
run_card = banner.RunCardLO(); run_card["ickkw"] = 1
py8 = banner.PY8Card(); py8["JetMatching:qCut"] = 30.0
banner.RivetCard().setWeightName(runcard=run_card, py8card=py8)
```

`check_setweightname.py` (attached) does this for `ickkw = 0` and `ickkw = 1`.

### Expected vs observed

- **Expected:** `weight_name = 'Weight_MERGING=30.0'`.
- **Observed:** `AttributeError: 'str' object has no attribute 'str'`.
- The `ickkw = 0` branch works.

### Cause

`madgraph/various/banner.py`, `RivetCard.setWeightName` (3.8.1 line 1831;
3.5.17 line 1675):

```python
                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
```

### Suggested fix

```diff
-                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
+                self['weight_name'] = "Weight_MERGING={0}".format(round(py8card['JetMatching:qCut'],3))
```

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
