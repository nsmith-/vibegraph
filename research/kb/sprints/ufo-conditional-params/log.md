# ufo-conditional-params log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (09eff16): the manager re-ran the `ufo::` unit tests (113) from the committed tree. F1 refused conditional parameters and undefined references at load but kept `expr::eval`'s panic, and the public `EvaluatedModel::recompute` still panicked on an unknown name, short of the item's "expr::eval no longer panics". The manager resumed the same agent as **F2**.
* **F2 checked** (92152df, e81a28a): the manager re-ran `toy_models` (12) and fmt from the committed tree. `recompute`/`set_alpha_s` return `UnknownParameter`; conditional objects in six UFO files are refused; `eval`'s panic is an internal invariant whose doc lists each public path's guard. `ScaleAwareAmplitude::set_alpha_s` on a model with no `aS` now records the value and leaves the pools alone; HY1 is asked to pin that path.
* **HY1 checked** (b810f3d, 215598f): the manager re-ran `toy_models` (13), the `reweight::` unit tests (34) and fmt from the committed tree. HY1 showed F2's no-`aS` branch was unreachable (a scale-dependent plan implies `aS`) and made it an invariant with a test; it also made `ReweightPlan::new` refuse an unknown launch parameter, which F2's `expect` assumed but a hand-built launch could violate.
* **Close-out** (2026-10-20). Filed `reweight-launch-accepts-internal-parameters`, `evaluated-model-param-values-public`, `ufo-conditional-statements-skipped`.
