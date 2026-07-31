#!/usr/bin/env python3
"""Dump the MadGraph LO run-card defaults to JSON as a transcription oracle.

Instantiates ``RunCardLO`` (whose ``default_setup`` populates every parameter's
default) and writes a flat ``{name: value}`` map to
``validation/madgraph/runcard_defaults.json``. The Rust ``runcard`` defaults
table is compared against this file per-parameter.

Reads the pinned ``research/refs/mg5amcnlo`` submodule, whose run-card parameter
set this transcription tracks; ``MG5AMCNLO_PATH`` overrides the checkout root.
"""

import json
import os
import sys

_here = os.path.dirname(os.path.abspath(__file__))
_out = os.path.join(_here, "runcard_defaults.json")


def _candidate_roots():
    """Checkout roots to try, after a plain import.

    ``$MG5AMCNLO_PATH`` first, then the pinned ``research/refs/mg5amcnlo``
    submodule. The submodule is the version this transcription is meant to
    track — it is what every other model-level check in the project reads —
    and the two are not interchangeable: the conda ``mg5amcnlo`` package is a
    different release and defines a different set of run-card parameters, so
    generating this file from it would silently retarget the oracle.
    """
    extra = os.environ.get("MG5AMCNLO_PATH")
    if extra:
        yield extra
    yield os.path.join(_here, "..", "..", "research", "refs", "mg5amcnlo")


def _import_run_card_lo():
    try:
        from madgraph.various.banner import RunCardLO  # noqa: E402

        return RunCardLO
    except Exception:
        pass
    for root in _candidate_roots():
        if not os.path.isdir(os.path.join(root, "madgraph")):
            continue
        sys.path.insert(0, os.path.abspath(root))
        from madgraph.various.banner import RunCardLO  # noqa: E402

        return RunCardLO
    raise SystemExit(
        "cannot import madgraph.various.banner.RunCardLO; check out the "
        "research/refs/mg5amcnlo submodule or set MG5AMCNLO_PATH"
    )


def _jsonable(value):
    """Reduce a run-card value to a JSON-friendly scalar or container."""
    if isinstance(value, bool) or value is None:
        return value
    if isinstance(value, (int, float, str)):
        return value
    if isinstance(value, (list, tuple)):
        return [_jsonable(v) for v in value]
    if isinstance(value, dict):
        return {str(k): _jsonable(v) for k, v in value.items()}
    return str(value)


def main():
    RunCardLO = _import_run_card_lo()
    card = RunCardLO()
    defaults = {name: _jsonable(card[name]) for name in card}
    with open(_out, "w") as fh:
        json.dump(defaults, fh, indent=2, sort_keys=True)
    print(f"wrote {len(defaults)} run-card defaults to {_out}")


if __name__ == "__main__":
    main()
