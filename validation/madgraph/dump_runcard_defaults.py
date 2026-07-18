#!/usr/bin/env python3
"""Dump the MadGraph LO run-card defaults to JSON as a transcription oracle.

Instantiates ``RunCardLO`` (whose ``default_setup`` populates every parameter's
default) and writes a flat ``{name: value}`` map to
``validation/madgraph/runcard_defaults.json``. The Rust ``runcard`` defaults
table is compared against this file per-parameter.

Run in the ``madgraph`` pixi environment (``mg5amcnlo`` provides ``madgraph``).
If ``madgraph`` is not importable, set ``MG5AMCNLO_PATH`` to a checkout root.
"""

import json
import os
import sys

_here = os.path.dirname(os.path.abspath(__file__))
_out = os.path.join(_here, "runcard_defaults.json")


def _import_run_card_lo():
    try:
        from madgraph.various.banner import RunCardLO  # noqa: E402

        return RunCardLO
    except Exception:
        pass
    extra = os.environ.get("MG5AMCNLO_PATH")
    if extra:
        sys.path.insert(0, extra)
        from madgraph.various.banner import RunCardLO  # noqa: E402

        return RunCardLO
    raise SystemExit(
        "cannot import madgraph.various.banner.RunCardLO; run in the madgraph "
        "pixi env or set MG5AMCNLO_PATH to an mg5amcnlo checkout root"
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
