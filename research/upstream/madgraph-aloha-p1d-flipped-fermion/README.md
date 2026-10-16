# ALOHA: `$` never vetoes a fermion propagator from a vertex's second spinor slot

## What it shows

`$ X` on a process line multiplies each marked propagator by a step function
that vanishes on its Breit–Wigner window (ALOHA's `P1D` routines). In 3.7.1,
for a fermion built from a vertex's second spinor slot (`FFV2P1D_1`), the
momentum flip turns `P(-1,id)**2` into `-(p²)`. The step function is then 1
everywhere, so the window is never removed. On
`u u~ > w+ b w- b~ $ t t~` at √s = 500 GeV, MadEvent's σ is 37 % too high.

## Status

**Fixed in 3.8.1.** No report is needed, so this folder has no draft.
- **The fix:** mg5amcnlo PR #428 ("fix the dead $ veto on outgoing-fermion
  propagators"), merged 2026-09-29. Its commits are `a8561cf80`/`b496afe81`
  (2026-09-24) and follow-ups `822525b2b`, `4e9883b33`, `6ffb13e07`. In
  3.8.1, `aloha/create_aloha.py:263-280` leaves even powers of a flipped
  momentum unsigned.
- **Releases with the fix:** 3.8.1 (`a5b17913`), the first in the 3.8 line,
  and the LTS release 3.5.17. 3.8.0 does not have it. The 3.7.1 defect is
  `create_aloha.py:262-263` (the flip) with `:527-530` (the `1D` numerator).

## Check (ran 2026-10-16)

The same MG5 script was run with each version (clones of tags `v3.7.1` and
`v3.8.1`, Python 3.11 from vibegraph's `madgraph` pixi environment):

```
import model sm
generate u u~ > w+ b w- b~ $ t t~
output standalone uuwbwb_veto
```

Then `grep THETA_FUNCTIONR uuwbwb_veto/Source/DHELAS/FFV2P1D_[12].f`:

| | `FFV2P1D_1.f` (the flipped slot) | `FFV2P1D_2.f` |
|---|---|---|
| 3.7.1 | `THETA_FUNCTIONR((TMP2*(TMP2+FCT1+FCT0)+FCT1*FCT0),1D0,0D0)`, i.e. θ((p²+A)(p²+B)) = 1 | `THETA_FUNCTIONR((TMP3*(-1D0)*(-TMP3+FCT4+FCT3)+FCT4*FCT3)` |
| 3.8.1 | `THETA_FUNCTIONR((TMP2*(-1D0)*(-TMP2+FCT1+FCT0)+FCT1*FCT0)`, i.e. θ((p²−A)(p²−B)) | unchanged |

In 3.8.1, `FFV2P1D_1` has the same form as `FFV2P1D_2`. The cross-section
effect was not rerun on 3.8.1.

vibegraph's 3.7.1 references use the same one-line change, as
`validation/madgraph/patches/aloha-p1d-flipped-fermion.patch`. The original
finding is in note 07's appendix
(`research/kb/history/notes/07-mg5-code-quality.md`).
