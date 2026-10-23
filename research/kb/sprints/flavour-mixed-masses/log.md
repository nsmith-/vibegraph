# flavour-mixed-masses log

## 2026-10-22
* **Creation**: Sprint folder created.
- 2026-10-22: the sprint opened off main at c0e5907. The user chose option 3: grouping changes only, with any needed map-code change recorded for the maps session.
- 2026-10-22: F1 checked (29692e6). The split is one integrand per set of outgoing masses, combined by MultiplicitySum, as MadGraph does with P directories (group_subprocs.py:482-490). No map code touched; the map-changes note says none is needed. The e+e- part matches MG dy13 (pull +1.68); a banking request for the e+tau card is posted.
- 2026-10-22: V1 checked (d72f73b). The banking row, the planned manifest row and the test are landed; the test prints NOT YET BANKED; validation-report tests pass. Banking request posted.
- 2026-10-23: the M3 banked dy13_etau_default (MadGraph 3.7.1, two P directories P1_qq_ll and P1_qq_taptam). V2 checked (aabda36): classification on the group label, a known-wrong control (e part read against the tau directory, pull +16.7) and the cells gated. The M3 ran the gate at ee6f18c: e part +1.41, tau part +0.58, total +1.43; the row is unplanned and bundled = false until a bundle carries the record. This is the closing evidence for `flavour-group-mixed-outgoing-masses-refused`: the survey's p p > l+ l- with l = e, ta passes grouping with sigma matching MadGraph. Item deleted.
