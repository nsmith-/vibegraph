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
