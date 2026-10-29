# systematics log

## 2026-10-28
* **Creation**: sprint folder created. User decisions Q1–Q9 recorded in the sprint concept.
* S-D1 (design only) dispatched on worktree `/home/user/wt/S`, branch `sprinte-systematics`, from main f5fb3e65.
* S-D1 checked (c960a5cb): design only; kb-lint 0 errors (rerun). The claim that the 12 banked runs share one `<initrwgt>` block was rechecked by the manager: 12/12 hash `eb55efbb97a2cabf`. Open choices O1–O12 and the brief's corrections (CMS passes `--start_id=1001`; MadGraph needs the run's own PDF in the `--pdf` list, so the M3 run uses copies with lhaid 325300) go to the user.
* 2026-10-28: the user approved the S-D1 design and took O1–O12 as recommended. Found F5 (the survey database is not banked): the user asked the manager to contact the CERN host session. The manager sent it three asks: the O9 query, a derived per-gridpack table on #71's branch, and the three VBF cards for #81.
* S-F1 dispatched. The M3 banking request was posted on #83, held until the CERN host confirms the PDF list.
* 2026-10-28: M3 banking passed on all four runs (PR #83 comment 6477954946). The real CMS header hash is `ba7e21bc…`; the design's predicted `2cbb8781…` came from a PyYAML stand-in for LHAPDF. The banked block is the reference, and S-V1 reconciles the description handling. Files are staged on the M3 for the next refdata cut.
* 2026-10-29: S-F1 checked (e1567121, f0883cd4). Manager re-ran `cli_systematics` (4 passed). Banked gates: 17.4M/17.4M weights exact, header identical from the three entry points, controls as designed, round trip 99.99% exact.
* S-F2 dispatched. It also takes S-F1's FA (`n_qcd` from the clustering graph under a dynamical scale) and FE (`<MGRunCard>` missing on unmatched samples, which Q6's purpose needs).
