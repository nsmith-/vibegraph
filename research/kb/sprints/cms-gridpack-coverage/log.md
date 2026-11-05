# cms-gridpack-coverage log

## 2026-11-04
* **Creation**: Sprint folder created by the sprint-H manager session. Items 3 and 4 filed as `xpt-nonpositive-refused` and `pt-min-pdg-refused`.
* Reference data in the container: refdata-13, the PDF sets (`validation/pdf/fetch.sh`), and the `mg5amcnlo` submodule at b7687064.
* Plan posted (sprint.md). Questions to the user on items 2 and 5.
* The user answered: D1 (item 2) merge rounding-level pairs by raising GROUP_REL_TOL to about 1e-9 and split physical ones; D2 (item 5) decide after MadGraph is measured ([D1](decisions/D1-near-degenerate-merge-rounding.md)).
* Worktrees `wt-{cuts,massive-incoming,decay-duplicate,g0}` were created from `sprinth-plan` (9ba3d96) with refdata-13, NNPDF23 and NNPDF31, and the submodule's content. pixi is not installed here, so the gates run as plain cargo and `validation/validate.sh`.
* Dispatched: H-C1 (feature-dev, items 3 and 4), H-M1 (feature-dev, item 6), H-D1 (validation-dev, item 5, measurement only per D2) and H-G0 (validation-dev, the colour-basis diagnosis, read-only). H-N1 (item 2) waits for `colour-split-groups`, because both edit the degenerate-groups check in `derive_flavor_groups`.
* The user: MadGraph runs that need no banking happen in this container, not on the M3. The pinned submodule's `bin/mg5_aMC` runs here (tested with an `output` of `p p > z j j, z > j j`), and gfortran is installed. LHAPDF is not, so MadGraph uses `nn23lo1`. All four sessions were told. Only references committed to the banked bundle stay M3 requests.
* The manager relayed `colour-split-groups` at 729e88a (the CERN agent, on main 58ab186; proton.rs only). The CERN host verified the `proton::` lib tests (47 pass) and the commit metadata; the rest was asserted. First review: the join and separation-skip logic is sound, and the evidence is missing (banked layer, MadGraph σ, per-subprocess colour labels, kb docs). H-G1 (validation-dev) was dispatched to add them as commits on top of the branch. Replies to the manager session fail with a trusted-device error, so they go through the user.
* H-G0 checked (report 00bf78d on local `sprinth-g0`, copied here). The manager spot-checked the probe log: π = [7,3,5,1,8,10,9,11,6,2,4,0] for `u u > g g u u` / conjugate, CF equal under π; Singlet reproduced in the SM as 8 against 192 diagrams with "no counterpart".
  - The QCD conjugate pairs differ only by flow order. The raw-order CF check refuses them before π is computed.
  - Singlet is a dead-coupling diagram-set difference (ymb = 0 at the card), not colour.
  - MadGraph keeps conjugate pairs as separate matrix elements in one P-dir and maps no flows.
  - H-G1's scope was extended: compare CF under the fingerprint π, so the QCD pairs form one group; the split stays for genuinely incompatible pairs.
  - Found, to file: diagrams whose couplings the param card sets to exactly zero are kept (the Singlet cause; pruning departs from MadGraph); the item's MadGraph claim and its line numbers are stale.
* The vibegraph manager queued two high-priority items, approved by the user, to start after the colour-split PR. They are on branch `claude/fervent-mayer-2hv310` (b1ba8ab) until its kb PR merges: `absent-run-card-maxjetflavor-ignores-beam-flavours` (first: a silent difference from MadGraph) and `five-flavour-ppjjj-refused-cut-indicator-differs`. Both were added to the plan as items 7 and 8.
* H-D1 checked (f7ed6b0 on `sprinth-decay-duplicate`; report copied here). The manager spot-checked the census line (`30 30 True`, 226 subprocesses, 30 covered twice).
  - MadGraph adds the two assignments incoherently.
  - vibegraph's incoherent σ matches MadEvent (1.0σ gg, 0.12σ e+e−, 5 seeds).
  - Interference ≤ 1e-3.
  - Found: every hadronic decay chain with jets in the decay stops at `GroupCutsDiffer`; card 024's refusal is not reproduced with its stated form.
* The user answered D3 (item 5 matches MadGraph) and D4 (per-group cut indicators come into scope) ([D3](decisions/D3-decay-duplicates-match-madgraph.md)).
* H-D2 checked (506e455 on `sprinth-decay-duplicate`, draft PR #93). The manager re-ran `a_subprocess_from_two_decay_assignments_is_two_subprocesses` and `two_assignments_on_two_lines_are_refused`: both pass.
  - Taken from the report: the census matches MadGraph's 6/6/30 two-assignment contents; σ is +0.30% against local MadEvent at 5 seeds, the same as the +0.32% nn23lo1/LHAPDF calibration offset; the workspace tests pass (1807).
  - The item stays open until the literal cards 024, 029 and 068 are measured (CERN-host extraction), and until D4, because every such hadronic card stops at `GroupCutsDiffer`.
  - Found, to file: cross-line duplicates are refused where MadGraph sums them; decay-chain strings drop `/ X` in messages; local MadEvent rows carry a +0.32% PDF offset (nn23lo1).
* H-C1 done (cbfc8df, 604959c on `sprinth-cuts`). It deletes `xpt-nonpositive-refused` and `pt-min-pdg-refused`.
  - Taken from the report:
    - `validate.sh` exit 0 (213 ✅ / 5 ⚠️, measured cells = declared); workspace 1814 passed;
    - 8 of 8 mutations caught;
    - per-event Z pT ≥ 100 on vibegraph and MadGraph events;
    - σ of `p p > z j` at `{23: 100}`: vibegraph 604.136 ± 0.112 (20 seeds) against local MadGraph 603.55 ± 0.354 (`hard_survey = 2`, 5 seeds), +0.10%; the cut/no-cut ratio agrees to 0.07%.
  - Two MadGraph defects: the light-code guard at `banner.py:4722` never fires; `setcuts.f:681–683` indexes legs with the list index, in hash-seed-dependent order. The cards both reach are refused, and the manager catalogued both (f111d23).
  - Found, to file: `mxx_only_part_antipart` is wrongly `IgnoredBenign`; upstream report drafts for the two defects.
* H-M1 done (2886fb5, 8db1578 on `sprinth-massive-incoming`). The manager re-ran its 12 massive-parton lib tests, including `massive_incoming_helicity_amplitudes_match_madgraph_standalone`: all pass.
  - Taken from the report:
    - σ against local MadGraph at 5 seeds: b at 4.7 +1.1e-3, c at 1.55 −2.3e-4;
    - the mass effect σ(4.7)/σ(0) = 1.05268 ± 0.00069 against 1.05255 ± 0.00143;
    - massless runs bit for bit;
    - `validate.sh` exit 0.
  - The item stays open until a survey card's σ is measured.
  - Found 1: a silent ×2 on main for explicit single-ordering cards (`has_mirror`). The user chose to fix it in sprint H, so it was filed as `explicit-initial-state-ordering-counted-twice` (high).
* Base correction: `sprinth-plan` was cut from a stale local `main` (86b8e5c, before #86, #87 and #91), so every work branch lacked them; PR #92 conflicted. The manager merged origin/main into `sprinth-plan` (40244b4), then `sprinth-plan` into `sprinth-cuts`, `sprinth-decay-duplicate` and `sprinth-massive-incoming` (merge commits; only generated index files conflicted). H-R1 was told to do the same. The merged branches are re-gated before they are pushed. Lesson: create worktrees from `origin/main` after a fetch, and verify against `origin/main`, never the local ref.
* Re-gated after the merge, each branch in its own target dir: `sprinth-cuts` 8ab4ec8 (1820 passed), `sprinth-decay-duplicate` 02acf65 (1813 passed), `sprinth-massive-incoming` 45555f7 (1815 passed); fmt and clippy clean on all three, all pushed. A first attempt with one shared `CARGO_TARGET_DIR` reused the first branch's test binaries for the second branch and was discarded. The banked layer has not been re-run on the merged branches.
* `validate.sh` re-run by the manager on the merged `sprinth-cuts` (8ab4ec8): exit 0, 213 ✅ / 5 ⚠️, "the measured cells are the declared cells". The untracked `.cargo/config.toml` (release-debug `debug = 0`) it used was removed afterwards.
* #92 merged (fa10e2d). The sprint log continues on `sprinth-kb` (the two later plan-branch log commits were cherry-picked onto it). origin/main was merged into `sprinth-cuts`, `sprinth-decay-duplicate`, `sprinth-massive-incoming` and `sprinth-mirror`; only kb files changed, so their gates stand.
* H-R1 done (201ad6a on `sprinth-mirror`, PR #96). The manager re-ran its 4 mirror tests: all pass.
  - Taken from the report: `u u~ > e+ e-` 778.48 against MadGraph's 778.11 (it was 1556.77); both/single ratio 1.99975 ± 0.00079 against 2.00010 ± 0.00087; `p p > e+ e-` unchanged on all seeds; `validate.sh` exit 0.
  - The item stays open for the massive rows after #95.
  - Found, to file: MadGraph's mirror key ignores polarization; H-M1's σ cards need their symmetric forms after this fix.
* The CERN host posted the survey cards on branch `sprinth-gridpack-cards` (08797dc8): 11 decay-chain cards for #93, and 2 massive-parton cards for #95 (ZprimeToTauTau_M130, b at 4.7 with every quark massive; LL_HAHM_MZd_14, c at 1.42). They were extracted into the scratchpad. Card 024 actually ran `p p > w+ j j $ t t~ h, w+ > j j` on sm-ckm_no_b_mass.
* Dispatched: H-M2 (validation-dev) validates #95 on the two cards, including whether LL_HAHM's `etaonium = -1` / `ptonium = 0` are inert in MadGraph; H-D3 (validation-dev) runs the MadGraph census on the 11 decay-chain cards and adds census rows.
* The manager relayed: the user approved #94 and #96; #88 (sprint G refusals) merged as d571a5d6; the manager merged main into `sprinth-mirror` and `sprinth-cuts` with update-branch and will merge #96, then #94, on green CI. Neither branch gets a push from here meanwhile. H-M2, H-D3 and H-G1 were told to merge origin/main before their final gates.
* #96 merged (56bcf347) and #94 merged (cbfaec61); #98 put items 7 and 8 on main. #96's item stays open for the massive rows after #95. The user approved building LHAPDF locally, so that local MadGraph runs with the same PDF set as vibegraph (`pdlabel = lhapdf`, 247000). LHAPDF 6.5.4 (hepforge tarball, sha256 2443a4b3…) is being built into /home/user/lhapdf. This follows H-G1's 1.4–2.1% and H-M2's ×1.0049 (αs) nn23lo1 offsets.
* LHAPDF 6.5.4 built (`/home/user/lhapdf`, `--disable-python`), with 247000 and 315200 linked from `validation/pdf`. The main checkout's MadGraph config sets `lhapdf`. Checked by the manager: a C++ probe gives αs(91.188) = 0.1300027109 for 247000, the value vibegraph writes. A MadGraph `p p > e+ e-` launch with `pdlabel = lhapdf`, `lhaid = 247000` gives 667.1 ± 3.3 pb, with AQCDUP 0.12997 at 91.3 GeV. H-M2 was resumed and H-G1 told to redo their MadGraph σ on it.
* The container restarted and stopped H-D3, H-G1 and H-M2. Their worktrees, commits and scratch survived: unpushed merges and commits on `colour-split-groups`, `sprinth-decay-duplicate` and `sprinth-massive-incoming`, and the onium change uncommitted on the last. The manager removed the merged worktrees (`wt-cuts`, `wt-mirror`, `wt-g0`, `wt-plan`) to free disk, then resumed H-G1 and H-D3 from their on-disk state. H-M2 resumes when one finishes, to keep two `validate.sh` runs at most on 4 cores.
* H-D3 done (5f1d071f, 1ecf7d27, 02bea373 on `sprinth-decay-duplicate`). The manager re-ran the census gate on 02bea373: 27 of 30 cards matched, 107 contents reached twice, passes.
  - Taken from the report: workspace 1907 passed; `validate.sh` exit 0.
  - 7 of the 11 survey cards reached the cross-line refusal, so the session extended D3 to two lines whose decay assignments differ (MadGraph sums them).
  - The item's file is deleted. PR #93 is marked ready.
  - The cards now stop at `GroupCutsDiffer` (4), the compile timeout (full Zto2Q), a colour split (TpTp) and massive incoming u (VLLD/VLLVLL ×4).
  - Merge b78d280d lacks the trailer.
* H-M2 resumed from its on-disk state.
* The user approved #93; the manager merges it on green CI, coordinating with #90, and files its Found items. No pushes to `sprinth-decay-duplicate` meanwhile. The squash makes the missing b78d280d trailer moot.
* #93 Found triage (the user, via the manager): this sprint handles 1 (the stale `proc-card-grammar.md:159`, on `sprinth-kb`) and 4 (the TTToHcToWA proc card, a defect candidate, in the close-out catalogue). The manager files 2 (`define` optimize_order), 3 (the b̄ core leg order) and 5 (BSM census models). Nothing goes onto #93.
* H-G1 done (2e2dd355 on `colour-split-groups`, pushed; gates on the merged tree green, `validate.sh` exit 0). With LHAPDF 247000 on MadGraph the σ offset persists: vibegraph is +1.5% (2→3) and +2.0% (HT 2→4), and the single-subprocess controls are too, so grouping is not the cause. Filed `light-quark-qcd-sigma-high-vs-madgraph` (high) for a diagnosis session.
* The colour-split PR is open as #100 (ready). The user chose to diagnose the QCD σ offset now: H-Q1 (validation-dev) dispatched on `sprinth-qcd-sigma` from origin/main cbfaec61, working from per-point |M|² up to dσ at fixed points.
* H-M2 interim (uncommitted; runs still going). All against LHAPDF-247000 MadGraph:
  - LL_HAHM: 5 seeds per side, +3.3e-4, pull +0.66 (blind to the c mass).
  - ZprimeToTauTau: MadGraph 5 seeds at 10.6159; vibegraph 2 of 5 seeds; the @0 parts within +0.3%.
  - The five massive single-ordering rows for #96's item all pass, at +7e-4 to +1.1e-3 coherently (Found 2, b-initiated). H-Q1 was told, for context.
  - Correction: H-R1's massless rows were against nn23lo1 MadGraph, so they are being rerun on LHAPDF.
* H-Q1 done (04908a9c on `sprinth-qcd-sigma`; report copied here; no code change). Verdict: no vibegraph bug.
  - Per point, |M|² (≤9.3e-16 against standalone), the PDF product, αs, cuts (MadEvent's own `PASSCUTS` passes all vibegraph events), the symmetry factor and the mirror all agree with MadEvent's own routines. vibegraph's PDF reader agrees with LHAPDF 6.5.4 to 2.9e-16.
  - Uncut `u u~ > e+ e-` against a quadrature on MadEvent's PDG2PDF and SMATRIX1: vibegraph +2.8e-5 ± 8.9e-5. So H-M2's ~0.1% term is not in vibegraph's normalisation; the cut rows are untested (H-Q1 Found 3).
  - On a boost-invariant `etaj = -1` τ-fold oracle, vibegraph reads −0.088% ± 0.037% and MadEvent at `hard_survey = 2` −0.44%, its deficit at large √ŝ.
  - H-G1's +1.5% is ≈1.2% MadEvent's default survey (`madevent-default-survey-bias`) + 0.2% its iteration combination + the large-√ŝ under-sampling.
  - The manager deleted `light-quark-qcd-sigma-high-vs-madgraph` as MadGraph's side and catalogued the under-sampling in `madgraph-defects.md`.
  - Found, to file: vibegraph's overflowed iteration χ²/dof (~1e289) near threshold at fixed √ŝ = 60.5 GeV; fixed-beam 3-jet seeds scatter at χ²/dof 2.79; H-M2's cut rows need the quadrature with `PASSCUTS`.
* H-M2 done (d931b012 on `sprinth-massive-incoming`). The manager re-ran the onium and massive-parton lib tests on it: 12 passed.
  - Taken from the report, all against LHAPDF-247000 MadGraph at 5 seeds per side:
    - LL_HAHM +3.3e-4 (pull +0.66);
    - ZprimeToTauTau +1.2e-3 (pull +0.62), every part passing;
    - the 7 mirror rows +7e-4 to +1.1e-3, each passing;
    - `validate.sh` exit 0.
  - It deletes `massive-incoming-partons-refused` and `explicit-initial-state-ordering-counted-twice`, and lifts `ptonium`/`etaonium` as inert.
  - H-M2's coherent +0.1% on the mirror rows (Found 2) is unresolved. H-Q1 showed vibegraph exact to +2.8e-5 ± 8.9e-5 on a different, uncut `u u~ > e+ e-` card (343 pb), so the offset is not in the PDF, |M|², flux or map; H-M2's cards (777.9 pb) carry cuts. The open check is H-Q1's quadrature with `PASSCUTS` on H-M2's cards (H-Q1 Found 3); to file.
* #93 and #90 merged to main. origin/main was merged into `sprinth-massive-incoming` and is being re-gated.
* #95 re-gated on main 1c4e628c (368173a: fmt, clippy clean; 1928 passed) and marked ready. H-J1 (feature-dev) dispatched for item 7, `absent-run-card-maxjetflavor-ignores-beam-flavours`, on `sprinth-maxjetflavor` from origin/main 1c4e628c. It touches the run-card default only, not grouping, so it does not wait for #100.
* H-J1 done (accffd19, plus the manager's 3d8ad2dc retargeting item 8; PR #101). The manager re-ran `config::`: 9 passed. Taken from the report: σ of `p p > j j` under sm-no_b_mass with no run card, −2.7e-4 against LHAPDF MadGraph (`hard_survey = 2`, 5 seeds per side); `validate.sh` exit 0. `asrwgtflavor` was deliberately not changed, because MadGraph never writes it; raised to the user in #101. Filed from its Found: `absent-run-card-lepton-beams-panic`, `absent-run-card-mixed-multiplicity-unmatched`.
* The user approved #100 and #95. Main 9db27321 includes #101. The manager merged main into `colour-split-groups` (39ac99b1; only the generated sessions index conflicted) and re-gated it: fmt and clippy clean, 1928 passed, both colour-split tests ok. Pushed; no further pushes to it. #95 follows after #100 merges.
* #100 merged (c8d4253c). The manager merged main into `sprinth-massive-incoming` (1fc5ca25). The proton.rs module doc conflicted: #100's grouping text was kept with #95's `split_by_external_masses`. The first re-gate failed to compile: #100's split test called `probe_momenta` with the old signature, a semantic conflict the text merge could not see. It was fixed in 3ea7ce3e (massless beams). Re-gated: fmt and clippy clean, 1952 passed, the colour-split, massive-parton and onium tests ok. Pushed; no further pushes.
* #95 merged (20aca9fe). Every PR the user approved is in: #93, #94, #96, #100, #95, #101.
  - The massive-quark rows for #96's item were measured by H-M2 on the #95 branch, which contains #96, and that item's file left main with #95.
  - H-N1 (validation-dev) dispatched for item 2 per D1, on `sprinth-near-degenerate` from main 20aca9fe.
  - H-K1 (per-group cut indicators, D4, with item 8) is held. Sprint G's open #89 rewrites `ProtonIntegrand` (+520 lines in proton.rs), right where `GroupCutsDiffer` is raised, so H-K1 starts after #89 merges.
  - The merged worktrees were removed to free disk.
* The manager confirmed H-D3 Found 7 with local MadGraph: the TTToHcToWA as-run `proc_card_mg5.dat` fails in MadGraph 3.7.1 too (`No particle t~> in model` at @1; @2–@5 not executed), because its own history writer wraps lines mid-token. Catalogued in `madgraph-defects.md` (record-only, no weight). `proc-card-grammar.md`'s duplicates paragraph now describes main's cross-line rule (#93 Found 1).
* H-N1 done (bcf2c9f1).
  - `GROUP_REL_TOL` is now 1e-9: the worst rounding pair is 1.477e-10, the closest physical pair 1.18e-7. `DegenerateGroups` is removed.
  - The 9.08e-7 γ+jets split is not CP violation: CP holds at reflected momenta to 3e-16, the split is the ε-tensor part with the W width's absorptive phase, and it vanishes at Γ_W = 0, not at η = 0. D1's outcome stands; its premise is corrected.
  - The manager updated `flavour-groups.md` steps 4–5 (12fb74a7), merged main with #89 (736d77b4), and re-gated: 1965 passed. PR #103 is ready.
  - The item stays open: 3.88e-8 is unattributed, and the CERN rerun will record it.
  - σ of `p p > a j j` against MadGraph at `hard_survey = 2`: +0.18%.
* #89 merged (d56871e9). H-K1 (feature-dev) dispatched for D4 and item 8 on `sprinth-group-cuts` from d56871e9.
