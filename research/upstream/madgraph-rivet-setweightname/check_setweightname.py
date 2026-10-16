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
