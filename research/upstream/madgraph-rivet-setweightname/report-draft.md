# Draft: MadGraph bug report

This draft is for the user to review, edit and file, either on Launchpad
(<https://bugs.launchpad.net/mg5amcnlo>) or as an issue on
<https://github.com/mg5amcnlo/mg5amcnlo>, which MadGraph's README names as the
places for bug reports.
- **Policy.** MadGraph publishes no policy on AI-assisted reports; checked on
  2026-10-16 (see the index,
  `research/kb/validation/madgraph-upstream-report-drafts.md`).
- **Before filing.** Re-run the claims from [`README.md`](README.md) and be
  ready to answer questions on them.
- **Disclosure.** Keep or reword the `Assisted-by` line at the end; it
  discloses the tool use.
- **Attach:** `check_setweightname.py` from this folder.

---

**Title:** `RivetCard.setWeightName`: `"Weight_MERGING={0}".str(…)` raises
`AttributeError` for every `ickkw ≠ 0` run, so Rivet cannot run on an
MLM-matched sample with the default `weight_name`

**Version:** 3.8.1 `madgraph/various/banner.py:1831` (3.7.1 `:1706`).

**Summary.** For `ickkw ≠ 0` with `weight_name = "default"`, the method builds
the merged-weight name with `.str(round(py8card['JetMatching:qCut'],3))`, and
`str` has no attribute `str`. `do_rivet`
(`common_run_interface.py:3178`) calls it for every Rivet run that leaves
`weight_name` at its default.

**Reproduce** (no Rivet needed):
`python check_setweightname.py <MG5 dir>`. It calls the method with a
default `RunCardLO` at `ickkw = 1`, a `PY8Card` with
`JetMatching:qCut = 30`, and a default `RivetCard`.

**Expected:** `weight_name = 'Weight_MERGING=30.0'`.

**Observed:** `AttributeError: 'str' object has no attribute 'str'`, on 3.7.1
and on 3.8.1. An end-to-end Rivet run was not made, since Rivet is not
installed here.

**Fix:**

```diff
-                self['weight_name'] = "Weight_MERGING={0}".str(round(py8card['JetMatching:qCut'],3))
+                self['weight_name'] = "Weight_MERGING={0}".format(round(py8card['JetMatching:qCut'],3))
```

Assisted-by: claude-code:claude-opus-5-5
