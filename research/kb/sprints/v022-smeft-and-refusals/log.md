# v022-smeft-and-refusals log

## 2026-11-17
* **Creation.** The vibegraph manager session started the sprint-K session on main 703f57f (#139).
* `send_message` to the vibegraph manager fails (`elevated_session_untrusted_device`), as it did in sprints I and J. Messages go through the user.
* **Container.** 4 cores, 15 GB RAM, 30 GB free disk. Provisioned here: pixi 0.84.1, the mg5amcnlo submodule, refdata-13, NNPDF23_lo_as_0130_qed and NNPDF31_lo_as_0130 (`VIBEGRAPH_FETCH_CONSENT=1`), and the `madgraph` pixi env (MadGraph 3.5.7). cvmfs is unreachable. The CMS model store and raw.githubusercontent.com are reachable.
* **Dispatch rule.** At most three agents at once, with one heavy job at a time gated by the sprint session. Each worktree has its own `CARGO_TARGET_DIR`; reference data is hard-linked in.
* **Batch A dispatched** off 703f57f: K-O1 (#141, feature-dev, Opus), K-P1 (#142, feature-dev, Opus) and K-B1 (#143, feature-dev, Sonnet).
* **Blocked on input from the manager:** PR 4 needs the g2HDM_UFO model or `mlm-panic-repro.tar.gz`; PR 10 needs the GJ-4Jets card. This container's 15 GB RAM is below the reported 23.9 GB growth.
* K-P1's MadGraph runs (2 cores) were allowed alongside K-O1's batch, which sat on a single-core gfortran compile of SMEFT_alpha's `couplings.f`. That is a deliberate exception to one heavy job at a time, on a 4-core box.
* K-P1 (claimed): main 703f57f and the branch are bit for bit on three ≥ 2-body cards (ee_mumu, dy13, llj_mlm: σ and grid sha256). `p p > z`, 20 seeds: mean 42.1154 nb, spread 0.0145, χ²/dof 0.93. Fixed-beam 2 → 1 stays a named refusal: MadGraph's lpp = 0 number reads as a δ(s − M²) coefficient, still to be confirmed by a run.
* K-B1's first `cargo test --workspace` hit ENOSPC: its target dir had reached 16 GB, built before the lean variables were set. That was a kill, not a result. The sprint session deleted the tree and re-ran on b3911a8: fmt and clippy clean, **2142 passed, 0 failed** (verified). It also fixed the dangling link in `run-card-resolution-not-reported` and regenerated the index.
* **CI's `banked` job runs `pixi run --skip-deps validate` on every push**, so local `validate.sh` runs duplicate it. Agents no longer run it locally; the banked gate is CI on the PR head.
* K-P1 done at 8a55fef. Verified by the sprint session from its files: `p p > z` pull −1.47 (MadGraph 42121.0 pb over 5 seeds, vibegraph 42115.35 pb over 20); the fixed-beam MadGraph `results.dat` is 0 at √s = 91.188 and 200 GeV; the main/branch outputs are identical. **Plan correction:** PR 7 closes on the hadronic card plus a named fixed-beam refusal, since MadGraph has no fixed-beam 2 → 1 σ.
* K-O1 (claimed): MadGraph's vertex-by-vertex order budget is reproduced, and diagram counts match on 22 toy and 10 SMEFT_alpha processes (pre-fix: 8 toy rows wrong). The 838/838 per-diagram amplitude match is at about 1e-10. The SMEFT_alpha σ for `e+ e- > z h h NP=1` is 116.931 ab (5 seeds, χ²/dof 0.94) against MadGraph's 117.07 ± 0.15, a pull of −0.91; the vev insertion carries about 20% of σ. GluGluH_HToUpsilon (`onia_sm`) is unreachable, so the substitution is named.
* **Batch B dispatched** off 703f57f: K-R1 (#144, feature-dev, Opus) and K-U1 (#145, feature-dev, Sonnet).
* K-O1 done at a30a682. Its item deletion left two dangling links (in `ufo-parameter-value-expressions-refused` and the 202a540 census); the sprint session repointed them in b3c4611. Verified on b3c4611: fmt, clippy, **2117 passed, 0 failed**. The σ was re-read from the five grids with `vibegraph inspect`: mean 116.931 ab against MadGraph's 117.07 ± 0.149. The agent's sweep summary had failed to parse the grids. MadGraph's diagram counts match the reported table.
* K-R1 done at 456aea4. **Plan correction:** MadGraph 3.7.1 refuses a restriction card that omits an external parameter (`model_reader.py` L178–L184), so `restriction-card-missing-ckm-entry-refused` closes as "refusal kept, reason recorded". A census of the store's 9739 U35 cards found 6173 with no CKMBLOCK and another model's numbering, which is a store-data issue for simshop. Two CI reds were the agent's: a stale index, and an integration test pinning the old refusal, rewritten.
* **Batch C dispatched** off 703f57f: K-S1 (#146, feature-dev, Opus) and K-X1 (#147, feature-dev, Opus).
* **Ready for merge** (CI all green including banked, the head contains main 703f57f, and the sprint session's own fmt/clippy/workspace run passed): #143 (PR 6) at b3911a8, #142 (PR 7) at 8a55fef, #141 (PR 1) at b3c4611.
* K-U1 done at 12e15b1. Seven of eight items close; `ufo-coupling-order-expressions-refused` stays open, because `dim6top_LO_UFO` is refused earlier at conditional `Parameter` definitions. #145's `Backlog:` line for it was dropped. **Brief correction:** MadGraph does not sum a vertex coupling list; it files each member by its coupling order (`import_ufo.py` L1818–1862). Verified on 12e15b1: fmt, clippy, **2144 passed, 0 failed**.
* K-S1 has its fix (generator chains between a 3 and a 3̄ slot) on a WIP commit; its oracles are running. **Card substitution:** SMS-StopStop is MLM-matched (0–2 jets), so σ is taken on its 0-jet part, with the CMS mass customisation and ickkw = 0.
* K-X1 (claimed): the outgoing-mass split keys on masses in slot order after content pairing, as MadGraph's `find_process_classes` does. **Brief corrections:** the brief's repro card cannot reproduce the refusal (it needs a massive lepton in different slots), and this item counts 2 campaign refusals, not 35. It also suspects that the fixed-beam/`generate` path may take every subprocess's outgoing masses from the first one; that is being checked now.
* **Ready for merge:** #144 (PR 2) at 456aea4 and #145 (PR 3) at 12e15b1. Each has CI green including banked and the sprint session's own run. #144 merges before #145; their code merges cleanly, and the only conflict is the generated `sessions/index.md`.
* K-S1 done at fb624bb. Verified: σ recomputed from the per-seed files (MadGraph 152.098 fb over 5 seeds with distinct iseeds, vibegraph 152.084 fb over 20, pull −0.12); fmt, clippy, **2117 passed, 0 failed**; banked green on the code commit 951a2f6, and its log shows the new MSSM tests running. Only the per-structure test against MadGraph's JAMP lines caught a reversed generator product.
* K-X1 done at 620904b. Verified: the ZPrimeToTT pull recomputed from the per-seed files (MadGraph 18.9186, vibegraph 18.9280 fb, 5 each, +0.50); the bitdiff files are identical; fmt, clippy, **2115 passed, 0 failed**; banked green on 42f3495. The fixed-beam mixed-mass suspicion was already refused by name.
* Filed from K-X1: `zprime-tt-quoted-error-below-seed-spread` and `fixed-beam-flavour-mixtures-refused`.
* **#141 merged** (54ded74). **The manager's faster merge rule (Nick, 2026-11-17):** a PR whose only conflict with main is a generated kb index merges once test, fmt and kb are green on the sync push, and `banked` is watched on main. A PR with a code conflict still waits for `banked`.
* Main synced into every open PR. Five had index-only conflicts: #142, #144, #146, #147 and #148. #143 had a code conflict in `diagrams/mod.rs`: the progress position against the gluon-free trials. Resolved so that trial enumerations report no progress. A first resolution sent the running diagram count backwards, and `cli_enumerate_progress` caught it. Merge tested at 2145 passed, 0 failed. #145 had a code conflict in `ufo/couplings.rs`: signed orders against configuration-flag expressions. Its test listing `-1 if …` as refused moved to the forms that load. Merge tested at 2147 passed, 0 failed.
* simshop's 31 refused restrictions rechecked on #144 (2fa8d82): the 11 missing-CKMlambda cards are still refused by name, as MadGraph refuses them; all 37 `9.999999e-1` probes load.
* **#149 merged** (c528465), adding PR 11. **PR 11 (#150), K-D1 done at 7cf2482.** The cause was `stitch_set` deduplicating by a slot-order-dependent key, which kept every diagram twice when identical decays sit on interchangeable slots. Neither of the item's readings held. Verified by the sprint session: the σ pulls recomputed from the per-seed files (M-15 +0.96, M-40 +1.94, distinct MadGraph iseeds); the bit-for-bit outputs identical on 4 decay-chain cards; fmt, clippy, **2120 passed, 0 failed**.
* Filed from K-D1: `wh-htoaa-sigma-reads-a-permille-high` and `decay-chain-census-dump-cannot-regenerate-in-one-run`.
* **PR 5 scope addition (Nick).** PR 5 adds `rank-jet-pt-cuts-unimplemented` and files `rank-lepton-pt-cuts-unimplemented`. Its gate is CMS ZH SMEFTsim_topU3l card 832 (SM), or 837 as the alternate, at `ptl1min` = 10, against MadGraph 3.7.1 references regenerated on the CERN host (0.07481 ± 0.00005 pb and 0.09970 ± 0.00006 pb). The cards and references are only on the CERN host, so they must be copied in before PR 5 dispatches.
* **#143 merged** (cce6811). **#151 merged** (67fd0b5): folder `index.md` files are no longer tracked. The manager held all re-syncs until it landed.

## 2026-11-17 (after #151)
* **#144 re-synced** to 37feab2: the conflict was only the generated `sessions/index.md`. Merge tested at 2157 passed, 0 failed. Merged (0949012).
* **#145 re-synced** to 1b246ae: the generated index, plus the hand-written refusal table in `cms-gridpack-refusals-202a540.md` (main's row 1, the branch's rows 2–3). Merge tested at 2182 passed, 0 failed. Merged (f65d1bb) on its existing green, after `git merge-tree` showed it merged cleanly onto 0949012.
* **#147 re-synced** to 579f7e9: the generated index, plus the refusal table again (main's row 7, the branch's row 8). Merge tested at 2188 passed, 0 failed; CI green including banked.
* **#150 re-synced** to 2d1ccbe with a code conflict in `diagrams/chain.rs`: main threads the progress `LinePosition` through `chain_sets`, and the branch threads the merge record. Both are kept. Merge tested at 2190 passed, 0 failed; the full CI re-ran green including banked.
* **#148** merged main cleanly.
## 2026-11-18
* **#147 merged** (15de3a8) and **#150 merged** (0340a89). Both were green on heads containing main, and they didn't conflict with each other. Main's banked run is green for #144 and for #145.

### State on main 0340a89

**Merged:**

| PR | Sprint PR | Merge commit |
|---|---|---|
| #141 | PR 1, negative orders | 54ded74 |
| #142 | PR 7, one-body phase space | 7b46311 |
| #143 | PR 6, batch safety | cce6811 |
| #144 | PR 2, restriction cards | 0949012 |
| #145 | PR 3, UFO value forms | f65d1bb |
| #146 | PR 8, seagull colour | 82ed311 |
| #147 | PR 9, mass split | 15de3a8 |
| #149 | adds PR 11 | c528465 |
| #150 | PR 11, repeated decay | 0340a89 |
| #151 | the merge-queue CI change and untracked folder indexes, merged mid-sprint | 67fd0b5 |

**Plan corrections recorded above:**
- #142: MadGraph has no fixed-beam 2 → 1 σ.
- #144: `restriction-card-missing-ckm-entry-refused` keeps its refusal, because MadGraph refuses those cards too.
- #145: `ufo-coupling-order-expressions-refused` stays open, and MadGraph files coupling-list members by order rather than summing them.
- #147: the brief's reproduction card was wrong, and the item covers 2 refusals, not 35.
- #150: neither of the item's readings was the cause.

**Scope change:** PR 5 now includes the rank cuts (`ptj1`–`ptj4` and `ptl1`–`ptl4`, min and max), as recorded in sprint.md.

**Remaining:**

| Sprint PR | Started | Blocked on |
|---|---|---|
| PR 4, MLM merge panic | no | the g2HDM_UFO model or `mlm-panic-repro.tar.gz` (Nick). It is not in the reachable CMS model store, and cvmfs is unreachable. |
| PR 5, run-card sentinels and rank cuts | no | PR 4 under the plan's ordering (PR 7 has merged). Also the ZH gridpacks 832/837 and the `run-*-ptl10` references copied from the CERN host. |
| PR 10, MLM memory growth | no | the GJ-4Jets card. This container has 15 GB RAM against the reported 23.9 GB growth, so the run needs a reduced card or the CERN host. |

After those three come a hygiene pass per merged feature PR, then the release candidate.
