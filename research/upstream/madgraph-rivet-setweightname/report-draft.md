---
severity: low
rank: 5  # of 7; 1 = most severe
severity_reason: "Running Rivet on any matched sample crashes straight away; it fails loudly and setting weight_name by hand avoids it."
---

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

The script below does this for `ickkw = 0` and `ickkw = 1`.

<details>
<summary><code>check_setweightname.py</code></summary>

```python
"""RivetCard.setWeightName on a matched run (ickkw = 1) with the default
weight_name.

The method builds the merged-weight name for Rivet as
    "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'], 3))
and `str` has no attribute `str`, so every Rivet run on an MLM-matched sample
that leaves weight_name at "default" stops with AttributeError. The
ickkw = 0 branch never reaches that line.

Usage: python check_setweightname.py <MadGraph root>
Needs only the MadGraph Python tree: no Rivet, Pythia8 or process directory.
Prints the observed result of each case and the expected one; exits 1 if any
case differs from the expected.
"""

import sys

sys.path.insert(0, sys.argv[1])

import madgraph.various.banner as banner  # noqa: E402

QCUT = 30.0
EXPECTED = {0: "None", 1: "Weight_MERGING=%s" % round(QCUT, 3)}

failed = False
for ickkw in (0, 1):
    run_card = banner.RunCardLO()
    run_card["ickkw"] = ickkw
    py8_card = banner.PY8Card()
    py8_card["JetMatching:qCut"] = QCUT
    rivet_card = banner.RivetCard()
    try:
        rivet_card.setWeightName(runcard=run_card, py8card=py8_card)
        observed = repr(rivet_card["weight_name"])
    except Exception as err:  # the defect raises AttributeError
        observed = "%s: %s" % (type(err).__name__, err)
    ok = observed == repr(EXPECTED[ickkw])
    failed |= not ok
    print("ickkw = %d  expected weight_name = %r  observed: %s  [%s]"
          % (ickkw, EXPECTED[ickkw], observed, "ok" if ok else "DEFECT"))

sys.exit(1 if failed else 0)
```

</details>

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
