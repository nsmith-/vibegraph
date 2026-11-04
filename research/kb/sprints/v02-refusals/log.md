# v02-refusals log

## 2026-11-03
* **Creation**: sprint G opened by the vibegraph manager on the user's instruction; split into three PRs (refusals, diagnoses, CLI output). Worktrees `/home/user/wt/G1`, `G2`, `G3` off main 86b8e5c.
* **Decisions (user)**: G1 refuse spin 2; G2 refuse σ = 0 and too few nonzero points, no artifact; G3 port MadGraph's restriction rule and list it as a MadGraph-compatibility site; G4 all three input cards classified, param card first. For the VEGAS item, a runtime warning is the known-limitation fallback (recorded on v02-diagnoses).
* 2026-11-04: G-F1 checked (3f89868, a9e94a2). Manager re-ran `toy_models` (17 passed), confirmed no `todo!` or spin-code `panic!` remains on the four sites, and checked authorship and trailers. Spin 2 is refused at `AmplitudeEvaluator::compile` on any line; an out-of-range Lorentz index at rooting. Both items' `closes_when` hold; their files are deleted here. Found (to file at close-out): `summed_states` gives spin 3/2 an empty state set; the internal spin-3/2 refusal is untested; the #71 survey models are unswept for `C`.
