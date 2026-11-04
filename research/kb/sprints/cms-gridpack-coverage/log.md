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
