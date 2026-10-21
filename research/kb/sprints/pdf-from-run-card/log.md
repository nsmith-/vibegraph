# pdf-from-run-card log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20 (CVMFS layout, measured on the CERN host and relayed by the planner)
* `/cvmfs/sft.cern.ch/lcg/external/lhapdfsets/current` is a real directory (3202 entries). Each set appears twice, unpacked as `<Set>/` and as `<Set>.tar.gz`. It also holds `pdfsets.index` (47618 B, lines `<lhaid> <name> <version>`), `@README`, `safe-resync.sh`, and the directories `@DEPRECATED` and `@UNVALIDATED`, which are not sets and are skipped.
* `lhapdf.conf` is not in this directory; per-set `.info` keys override it, so the tier is a pure data path.
* `NNPDF31_nnlo_as_0118_mc_hessian_pdfas` (325300): 103 members of about 1.25 MB, tarball 44.9 MB, under `HttpFetch`'s 256 MiB cap. Its `.info` has `SetIndex: 325300 ` **with a trailing space**: the parser trims, and a test pins it. sha256 `.info` 63efe51f3207bffdfcdde3703dca712f760ff2611623c4796c3918ba1354706b, `_0000.dat` ccbf6996fbaff05dfcf5e740e3649dc0873abaffb586ab5991037957248bc86f.
* `NNPDF23_lo_as_0130_qed` (247000): 101 members, tarball 27.6 MB; `.info` f961a10504b4cf070bc0d588006a599954b1b856f3c104ba6e9085f5f81ee4be, `_0000.dat` 5ac9499dcf7f6fe0a363f89f9320782f412eb91732725f7c435f0e2c119e2bf0, identical to the pinned copy, so a CVMFS hit for the default set reproduces today's digests.
* Index entries: 247000 NNPDF23_lo_as_0130_qed, 306000 NNPDF31_nnlo_hessian_pdfas, 325300 NNPDF31_nnlo_as_0118_mc_hessian_pdfas.

## 2026-10-21 (user decision on D1, relayed by the planner)
* **Principle (user):** information a workflow step necessarily introduces is encoded in that step's artifact and is not requested again by later steps; a later step that re-took it could only compare it, so it does not take it.
* **Consequences:** (1) the param card feeds integration only: integrate records the moved parameters; `generate` takes no `--param-card`. (2) `generate` is gridpack style: the artifact plus generate-time inputs (`--nevents`, `--seed` replacing `iseed`, output and weight-strategy options, the reweight card); no proc card, `--run-card`, `--param-card` or PDF option. (3) The run card splits by data dependency into integrate-time settings (in the integrate artifact) and generate-time settings (nevents, iseed, event_norm, use_syst/systematics_*/sys_*, anything affecting only event writing) in a separate structure; the library's generate API takes the artifact plus a generate-settings value and never takes or diffs a run card; the MadGraph-style CLI builds both from the one card at integrate and records the generate part for replay. (4) Model and PDF files are still read at generate as assets checked against recorded digests. (5) `card_mismatches`/`refuse_on_mismatch` go.
* **D1 revision (D1b)** re-opened on this; the 12 open choices are re-stated after it. If H no longer fits one PR, D1b proposes the split.
* **D1b checked** (c501b2e): design only (no code in the diff); kb-lint 0 errors; the generate-time rows spot-checked against `FIELD_CLASSES`. New concept `pipeline/integrate-generate-split.md`; ten choices N1–N10 and a split H1 (gridpack generate) / H2 (PDF from the run card) / param-card PR put to the user.

## 2026-10-21 (user answers on the original choices, relayed by the planner)
* **C1:** per-file SHA-256 of the files actually read. PDF systematics read more members; they are introduced at generate, so their checksums go in what generate writes (LHE header / generate record), not the integrate artifact.
* **C2:** moved externals only. **C3:** refuse old hadronic artifacts.
* **C4:** refuse every built-in pdlabel, nn23lo1 included (it is not MadGraph's own grid; LHAPDF is always required). Our default proton run card says `pdlabel = lhapdf`, `lhaid = 247000`, documented as a departure from MadGraph's default card; a card saying nn23lo1 is refused with a message to set `pdlabel = lhapdf` and an `lhaid`. The banked references were made with MadGraph's internal nn23lo1 grid and the gates already compare LHAPDF 247000 against them, so no gate's comparison changes.
* **C7:** retire `VIBEGRAPH_PDF_DIR`; use `LHAPDF_DATA_PATH` as a colon-separated list, as LHAPDF does; the legacy `LHAPATH` is the design's call.
* **C8:** remove the `validation/pdf` fallback. **C9:** refuse on a checksum mismatch.
* C5, C6, C10, C11, C12: pending; the design's recommendations are the working assumption.
* **C5, C6, C10, C11, C12** (user, 2026-10-21): the recommendations taken: pins carry set_index/num_members; fall through tiers when the lhaid is not covered; the prompt says size and checksum are unknown (no HEAD); always the new version (moot under the split); download tested at library level, consent prompt separately. All twelve original choices are settled; N1–N10 of the split design remain open.
* **Correction (D1c, checked by the manager against `manifest.toml:149-153`):** the C4 entry above misstates the references. The banked proton σ and sample rows were re-carded at cut 5 to `pdlabel = lhapdf`, `lhaid = 247000` on the MadGraph side too, so both sides already use LHAPDF 247000; the conclusion (no gate comparison changes) stands. The one MadGraph proton comparison on internal `nn23lo1` is the reported, ungated `uu_tt` on-shell-veto row, which needs re-carding and rebanking. **D1c checked** (bcfbc1e): design only, kb-lint 0 errors.
- 2026-10-21: the user decided N1–N12. N4 differs from the recommendation: iseed feeds the integrate seed (after --seed); generate requires --seed, with no default. N3 approved; the bookkeeping rows are explained in the design §5.
