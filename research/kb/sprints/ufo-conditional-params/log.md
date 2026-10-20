# ufo-conditional-params log

## 2026-10-20
* **Creation**: Sprint folder created.

## 2026-10-20
* **F1 checked** (09eff16): the manager re-ran the `ufo::` unit tests (113) from the committed tree. F1 refused conditional parameters and undefined references at load but kept `expr::eval`'s panic, and the public `EvaluatedModel::recompute` still panicked on an unknown name, short of the item's "expr::eval no longer panics". The manager resumed the same agent as **F2**.
* **F2 checked** (92152df, e81a28a): the manager re-ran `toy_models` (12) and fmt from the committed tree. `recompute`/`set_alpha_s` return `UnknownParameter`; conditional objects in six UFO files are refused; `eval`'s panic is an internal invariant whose doc lists each public path's guard. `ScaleAwareAmplitude::set_alpha_s` on a model with no `aS` now records the value and leaves the pools alone; HY1 is asked to pin that path.
