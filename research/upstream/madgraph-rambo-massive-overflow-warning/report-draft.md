---
severity: trivial
rank: 8  # of 9; 1 = most severe
severity_reason: "A warning in the Python RAMBO used by the check commands can never print; the neighbouring massless warning already covers the same events, so nothing is lost in practice."
---

# Draft: GitHub issue for mg5amcnlo/mg5amcnlo

For the user to review before posting. The full evidence is in
[`README.md`](README.md).

---

**Title:** `rambo.py`: the massive-momenta overflow warning tests `iwarn[4] > 5` and can never print

**Affected versions:** 3.5.17 (LTS) yes, and 3.8.1 yes. Both were tested at
code level.

### Summary

`RAMBO` in `madgraph/various/rambo.py` prints at most five warnings of each
kind when the log weight may under- or overflow. Three of the four tests read
`iwarn[k] < 5`. The massive-momenta overflow test reads `iwarn[4] > 5`. Since
`iwarn` is a fresh list of zeros on every call, that warning never prints.

The practical impact is small. The mass correction only lowers the weight, so
the massless-weight overflow warning in the same call already fires.

A second, cosmetic slip sits nearby. The massless warnings format the weight
with `%f20.9`, which prints e.g. `EXP(191.10138020.9)`; `%20.9f` was meant.

### How to reproduce

No process directory is needed. The script below calls
`RAMBO(10, 1e6, masses)` with massless and with 1 GeV particles; the log weight
is then about 191, above the 174 threshold. It does the same with an in-memory
copy of `RAMBO` that carries the fix.

<details>
<summary><code>check_rambo_warning.py</code></summary>

```python
"""RAMBO's overflow warning for massive momenta (madgraph/various/rambo.py).

The massive branch warns with
    if(wt > 174  and iwarn[4] > 5):
while every other warning in the function tests `iwarn[k] < 5`. `iwarn` is a
fresh FortranList(5) of zeros on every call, so `iwarn[4] > 5` is never true
and this warning cannot print.

The script calls RAMBO at a centre-of-mass energy high enough for the log
weight to pass 174, once with massless and once with massive final-state
particles, and prints which warnings appeared. It then calls a copy of RAMBO
with `> 5` replaced by `< 5` to show the massive warning that should appear.

Usage: python check_rambo_warning.py <MadGraph root>
Needs only the MadGraph Python tree. Exits 1 if the massive warning is missing.
"""

import contextlib
import inspect
import io
import sys

sys.path.insert(0, sys.argv[1])

import madgraph.various.rambo as rambo  # noqa: E402

N, ET = 10, 1.0e6


def masses(m):
    xm = rambo.FortranList(N)
    for i in range(1, N + 1):
        xm[i] = m
    return xm


def run(func, m):
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        _, wt = func(N, ET, masses(m))
    lines = [line.strip() for line in out.getvalue().splitlines() if "OVERFLOW" in line]
    return wt, lines


source = inspect.getsource(rambo.RAMBO)
assert "iwarn[4] > 5" in source, "the massive overflow test is not in this version"
namespace = dict(vars(rambo))
exec(source.replace("iwarn[4] > 5", "iwarn[4] < 5"), namespace)
fixed = namespace["RAMBO"]

missing = False
for label, func, m in (("massless", rambo.RAMBO, 0.0),
                       ("massive", rambo.RAMBO, 1.0),
                       ("massive, fixed", fixed, 1.0)):
    wt, lines = run(func, m)
    print("%-15s log weight %.1f  overflow warnings printed: %d" % (label, wt, len(lines)))
    for line in lines:
        print("    " + line)
    if label == "massive":
        missing = len(lines) < 2
print("expected: the massive run prints the massive-weight warning after the massless-weight one (2 lines), as the fixed copy does")
sys.exit(1 if missing else 0)
```

</details>

### Expected vs observed

The massive call should print the massive-weight warning.

```
massless        log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive         log weight 191.1  overflow warnings printed: 1
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
massive, fixed  log weight 191.1  overflow warnings printed: 2
    RAMBO WARNS: WEIGHT = EXP(191.10138020.9) MAY  OVERFLOW
    RAMBO WARNS: WEIGHT = EXP(191.10137998648733) MAY OVERFLOW
```

That is the 3.8.1 output; 3.5.17 is the same.

### Cause

`madgraph/various/rambo.py`, function `RAMBO` (3.8.1 line 218; 3.5.17 line
220):

```python
    if(wt > 174  and iwarn[4] > 5):
```

### Suggested fix

```diff
-    if(wt > 174  and iwarn[4] > 5):
+    if(wt > 174  and iwarn[4] < 5):
```

Optionally also change `%f20.9` to `%20.9f` in the two massless warnings
(3.8.1 lines 162 and 165).

---
Found with an AI assistant while validating an independent LO generator
against MadGraph; reproduced on both versions before reporting.

Assisted-by: claude-code:claude-opus-5-5
