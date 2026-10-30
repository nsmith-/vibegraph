# param-card log

## 2026-10-28
* **Creation**: sprint folder created. User decisions P1 (refuse Auto widths), P2 (dependent parameters ignored with a warning), P3 (write `<slha>`), P4 (refuse a move that splits a restriction-merged pair).
* P-F1 dispatched on worktree `/home/user/wt/P`, branch `sprinte-param-card`, from main f5fb3e65.
* 2026-10-28: P-F1 checked (02423b2b, 5619d54f). Manager re-ran fmt and `cli_param_card` (7 passed). No repo model has a restriction-merged pair, so P4 is tested synthetically only. MadGraph's own param card records `WH` as moved (`%e` rounding): pinned and documented.
* 2026-10-29: the M3 banked both rows and ran the gate there (PR #82 comment 6499635262): `pp_to_ll_mz95` pull +0.06 and `ee_to_tth_ymt250` pull +0.14, with no FLAGGED or INERT screens. The user cut refdata-12 with them, and the pin was cherry-picked here (9c92bb94).
* 2026-10-30: S-V1 (#83) found that refdata-12's `ee_to_tth_ymt250` is undeclared in the `validate_alphas` and `validate_scales` inventories, so both fail on the new bundle. The user agreed: P-V2 declares it here, and the commit is cherry-picked onto #83.
