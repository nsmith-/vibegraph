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

## 2026-11-19

**Merged:**

| PR | Sprint PR | Merge commit |
|---|---|---|
| #153 | PR 4, MLM merge panic | 74596ba |
| #152 | PR 5, run-card sentinels, report, table and rank cuts | f78fb60 |

#152 merged over #153 without a re-sync: the two merge cleanly and each had
its own green CI, banked included; main's own run on f78fb60 tests the pair.

**PR 4 (#153).** The cause was not the bug report's reading (a decay b under
the five-flavour `p`). At the card ρ_tu = 0 and the CKM is Cabibbo-only, so
the u-line h± subprocesses vanish identically, and flavour grouping joined
them on 0 = 0 to live-topology representatives; `rewgt` then built
`u → h+ g`. A MadGraph 3.5.7 replay of the panic point puts the two
subprocesses in separate P-dirs and never reaches that merge. The fix skips
groups whose couplings vanish at the card and turns the remaining sampled-point
faults into named refusals. No M300 σ: one seed is ≈ 4.2 h here, and the
release-candidate CERN campaign covers all seven masses end to end (manager,
2026-11-19).

**PR 5 (#152).** Lepton rank cut: ZH832 at ptl1min = 50, +0.065% (pull 1.2)
against MadGraph 3.5.7; the ZH gate at ptl1min = 10 against the CERN host's
3.7.1, pulls 0.06 and −0.16. The DY1j jet-rank bins' +0.2–0.3% is the ℓℓj
family's existing offset (single-leg control). Brief correction: `ht2max`–
`ht4max` are off at < 0 or ≥ ebeam1 + ebeam2; the template's own off value
is −1.

**Manager decisions (2026-11-19):**
- `misset = −1`'s √ŝ floor is settled in this sprint: proved inert,
  implemented or refused (PR 12).
- The fixed-beam sampled-point scale panic is fixed in this sprint as a named
  refusal (PR 12).
- PR 10 runs under a hard memory cap well below 15 GB and closes on a bounded
  full card 244, on the CERN host if needed.

**Started:**

| Sprint PR | PR | Branch | Items |
|---|---|---|---|
| PR 10, MLM memory growth (K-G1) | #155 | `sprintk-mlm-memory` | `mlm-photon-four-jet-integrate-memory-grows-unbounded` |
| PR 12, named refusals (K-N1) | #156 | `sprintk-named-refusals` | `fixed-beam-sampled-point-scale-error-panics`, `misset-negative-sqrt-shat-floor-unapplied` |

PR 12's two items are filed in this record rather than on main; this record
deletes them when #156 merges.

**Filed from Found sections:**
- From K-M1:
  - `fixed-beam-sampled-point-scale-error-panics` (high);
  - `mlm-vanishing-flavour-groups-still-sampled`;
  - `flavour-grouping-joins-zero-matrix-elements`;
  - `sampled-point-fault-refused-only-at-end`;
  - `reweight-cannot-revive-vanishing-subprocess`;
  - `madevent-single-point-replay-driver`.
- From K-C1:
  - `misset-negative-sqrt-shat-floor-unapplied` (high);
  - `run-card-value-refusals-hand-listed`;
  - `run-card-dollar-lines-skipped-silently`.
- From K-C2:
  - `madgraph-class-flags-per-pdir`;
  - `resolve-mom-dead-in-release-builds`;
  - evidence added to `llj-family-reads-high-vs-seed-records` and `zprime-tt-quoted-error-below-seed-spread`.

### 2026-11-19 (later): PR 10 and PR 12 merged

| PR | Sprint PR | Merge commit |
|---|---|---|
| #155 | PR 10, MLM memory growth (K-G1) | c1a9531 |
| #156 | PR 12, named refusals (K-N1) | b3bc539 |

**PR 10.** `ChannelHistory::record` cloned each kept iteration's per-channel
share vector: 1068 channels × 24,297 shares × 16 B = 396 MiB per iteration
predicted, 399–420 MiB measured, with heaptrack naming the site. A fixed
~3.2 GB transient came on top from collecting every chunk's tiles before
reducing any. After the fix, the full card 244 peaks at 2140 MiB over 181
iterations, against 23.9 GB by iteration 45 on the CERN host. Card 244 has
σ = 0 at LO (pta 200 against htjmax 100) and ends in the "none passed the
cuts" refusal. Brief corrections: the growth scales with channels × shares
per kept iteration, and the other GJ-4Jets cards converge before it shows.
Local `validate` was declined; CI's banked run is the gate.

**PR 12.** The fixed-beam scale panic is now the named `SampledPointFault`
refusal. `misset < 0` can bind: it is refused by name exactly where `misset²`
is MadGraph's lepton smin term, and the term reproduces MadGraph's printed
smin on 3 banked runs. It is not implemented, because vibegraph's τ floor is
not MadGraph's class-by-class smin. Correction to K-C1: a default `mmll` does
not empty the region, because setcuts.f's loop counts only the first lepton
of a pair; `ptl` does. Consequence: campaign cards with `misset = -1` and no
lepton cut ≥ 1 GeV are refused by name. This record deleted
`fixed-beam-sampled-point-scale-error-panics` and
`misset-negative-sqrt-shat-floor-unapplied`.

**Filed from Found sections:**
- From K-G1:
  - `zero-weight-integrand-runs-to-point-cap`;
  - `configuration-shares-span-whole-layout`;
  - `gj4jets-ht40to100-madgraph-sigma-unrecorded`, filed and then closed the same day (below).
- From K-N1: `misset-floor-smin-unmeasured-in-madgraph`.
- Evidence added to `sampled-point-fault-refused-only-at-end`.
- Not filed:
  - K-G1's `resolve_mom` warning (already `resolve-mom-dead-in-release-builds`);
  - the worktree submodule git quirk (local tooling);
  - 0.13% degenerate points on card 244, which the run's own warning reports.

**Remaining:** a hygiene pass per merged feature PR, then the release-candidate head.

**Gridpack 244's MadGraph σ.** The CERN host read the cvmfs tarball (MG5_aMC 2.9.13):
- the banner gives "Integrated weight (pb) : 0";
- `results.dat` gives a total of 0 over 823 channels, after about 14 CPU-hours;
- 0 of the 823 per-channel `results.dat` are non-zero.

The integrated run card is pta 200, htjmin 40, htjmax 100, ptj 0, maxjetflavor 5. CMS's build filled in the template's 0 / 0 / −1. So MadGraph agrees σ = 0: the card is defective, and the htj reading is not at fault. `gj4jets-ht40to100-madgraph-sigma-unrecorded` is closed on that evidence. The memory growth on all-zero-weight runs is fixed by #155, and the wasted iterations are `zero-weight-integrand-runs-to-point-cap`.

## 2026-11-19 (later): hygiene passes

The twelve merged feature PRs were grouped into four hygiene passes by code
area, so related PRs are reviewed together and no two passes edit the same
files. Each pass committed its findings before any edit and ran one mutation
per test the PRs added. σ and grid sha256 had to stay bit for bit against main
b3bc5399.

| pass | PRs reviewed | agent | PR | head | gates (sprint session) |
|---|---|---|---|---|---|
| K-H1 model loading | #141, #144, #145 | hygiene-dev, Opus | #157 | d39396d5 | 2238 passed, 0 failed |
| K-H4 integration | #142, #155 | hygiene-dev, Sonnet | #158 | 67344677 | 2238 passed, 0 failed |
| K-H3 run card, CLI, batch | #143, #152, #156 | hygiene-dev, Sonnet | #159 | 0e903f8c | 2237 passed, 0 failed |
| K-H2 diagrams, colour, grouping | #146, #147, #150, #153 | hygiene-dev, Opus | — | — | starts after #157 merges (#157 edits `diagrams/chain.rs` and `diagrams/mod.rs`) |

For each of #157, #158 and #159:
- fmt and clippy `-D warnings` are clean and kb-lint has 0 errors, all re-run by the sprint session;
- CI is all-pass including banked, on a head that contains main;
- the three branches merge cleanly in every pair.

Findings that change behaviour:
- **#157 H1.** An unreadable or non-UTF-8 optional UFO file used to be read as
  absent. For SMEFTsim that silently replaced the model's own `sec`/`csc` with
  the generic complex definitions. The file is now refused by name.
- **#159 H8.** A `generate` run refused for a sampled-point fault used to leave
  a complete event file under its final name. That file is now removed with the
  refusal.
- **#158 F3.** The configuration shares took a combine rule as an argument
  instead of using the channel's own rule.

Mutations:
- **#157:** 47 rows. The merged tests let 3 mutations through, all findings
  that were then fixed.
- **#158:** 20 rows. The merged tests let 6 through. Now 18 are caught; the
  other 2 are one the code never reaches and one that doesn't change the result.
- **#159:** 72 rows, all caught. Two of the merged misset tests passed a
  skipped refusal; the replacement tests fail it.

Found, filed in this PR:
- From K-H4: `amplitude-probe-skips-two-to-one`. Evidence added to
  `resolve-mom-dead-in-release-builds`.
- From K-H1: `particle-charge-never-read`,
  `selector-signed-constraint-arms-untested` and
  `ufo-tempdir-copy-loop-repeated`. Evidence added to
  `weighted-order-computed-three-ways` (three sites left) and
  `diagram-count-test-helper-duplicated` (a fifth copy).
- From K-H3: `sampled-point-fault-slot-duplicated` and
  `event-generate-functions-long`. Evidence added to
  `run-card-value-refusals-hand-listed` and
  `sampled-point-fault-refused-only-at-end`: #159 removes the event file, but
  no card drives a sampled scale fault end to end.
