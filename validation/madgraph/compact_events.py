"""Project each banked MadGraph run's event file into a columnar Parquet file.

A `.lhe.gz` is a text record of everything MadGraph knew about an event. The
validation gates read a small part of it: the external legs (flavour, status,
mothers, colour lines, four-momentum, mass, lifetime, helicity), the event
header (`IDPRUP`, `XWGTUP`, `SCALUP`, `AQEDUP`, `AQCDUP`) and the `<mgrwt>`
scale-replay payload (`<rscale>`, `<pdfrwt>`). Everything else -- most notably
the `<rwgt>` block, which carries 145 systematic weight variations per event in
the hadronic runs -- no gate consumes.

This writes exactly that subset, one Parquet file per run, as struct-of-arrays
columns: per-event scalars, plus one list column per particle field. Floating
columns are encoded with BYTE_STREAM_SPLIT (which groups the mantissa bytes of
neighbouring values, the only encoding that helps a column of near-random
mantissas) and the whole file is compressed with zstd. The `<init>` block rides
along in the file's key-value metadata as JSON.

Fidelity: Parquet stores IEEE-754 doubles, so every float survives the write
exactly as `float()` produced it, and both Python's and Rust's decimal-to-double
conversions are correctly rounded -- a reader gets the same bits it would have
got from parsing the printed digits itself. `--verify` reads each file back and
compares every retained field against the source, so the claim is measured
rather than assumed.

pylhe parses the events (never a hand-written LHE parser); the `<mgrwt>` block,
which pylhe does not model, is taken from the same ElementTree element pylhe
builds its records from.

Usage:
    python validation/madgraph/compact_events.py            # all banked runs
    python validation/madgraph/compact_events.py uux_to_uux # one run
"""

from __future__ import annotations

import gzip
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import pylhe

VALIDATION = Path(__file__).resolve().parent
WORK_AREA = VALIDATION / "output"
OUT_DIR = VALIDATION / "events"

EVENT_PATH = "Events/run_01/unweighted_events.lhe.gz"

# Compression: zstd at its maximum level. The momenta dominate and are
# near-incompressible, so the level buys little on them and a lot on the
# small-cardinality integer columns.
ZSTD_LEVEL = 22

PARTICLE_INT_FIELDS = ("id", "status", "mother1", "mother2", "color1", "color2")
PARTICLE_FLOAT_FIELDS = ("px", "py", "pz", "e", "m", "lifetime", "spin")
EVENT_FLOAT_FIELDS = ("weight", "scale", "aqed", "aqcd")


def fortran_double(token: str) -> float:
    """A Fortran-printed double; `D` exponents are not decimal-literal syntax."""
    return float(token.replace("D", "E").replace("d", "e"))


def parse_mgrwt(element: ET.Element) -> dict:
    """The `<mgrwt>` scale-replay payload of one `<event>`, or empty defaults.

    `<rscale>` is `n_qcd value`; each `<pdfrwt beam="i">` is `n` followed by `n`
    flavour ids, `n` momentum fractions and `n` scales.
    """
    out = {
        "rscale": None,
        "rscale_nqcd": None,
        "pdfrwt_flavour": [[], []],
        "pdfrwt_x": [[], []],
        "pdfrwt_scale": [[], []],
    }
    block = element.find("mgrwt")
    if block is None:
        return out
    for sub in block:
        if sub.tag == "rscale":
            fields = (sub.text or "").split()
            out["rscale_nqcd"] = int(fields[0])
            out["rscale"] = fortran_double(fields[1])
        elif sub.tag == "pdfrwt":
            beam = int(sub.attrib["beam"]) - 1
            fields = (sub.text or "").split()
            n = int(fields[0])
            out["pdfrwt_flavour"][beam] = [int(f) for f in fields[1 : 1 + n]]
            out["pdfrwt_x"][beam] = [fortran_double(f) for f in fields[1 + n : 1 + 2 * n]]
            out["pdfrwt_scale"][beam] = [
                fortran_double(f) for f in fields[1 + 2 * n : 1 + 3 * n]
            ]
    return out


def read_run(lhe: Path) -> tuple[dict, list[dict]]:
    """`(init, events)` for one banked run.

    One pass with ElementTree over the container; pylhe's own record classes do
    every field conversion, so what lands in the columns is what a pylhe user
    would have got.
    """
    init: dict = {}
    events: list[dict] = []
    with gzip.GzipFile(lhe) as fileobj:
        context = ET.iterparse(fileobj, events=["end"])
        for _, element in context:
            if element.tag == "init":
                fields = (element.text or "").strip().split("\n")
                head = fields[0].split()
                init = {
                    "beam_pdg": [int(head[0]), int(head[1])],
                    "beam_energy": [float(head[2]), float(head[3])],
                    "pdf_group": [int(head[4]), int(head[5])],
                    "pdf_set": [int(head[6]), int(head[7])],
                    "weight_strategy": int(head[8]),
                    "processes": [
                        {
                            "xsec_pb": float(p.split()[0]),
                            "xerr_pb": float(p.split()[1]),
                            "xmax": float(p.split()[2]),
                            "id": int(p.split()[3]),
                        }
                        for p in fields[1 : 1 + int(head[9])]
                    ],
                }
                element.clear()
            elif element.tag == "event":
                lines = (element.text or "").strip().split("\n")
                info = pylhe.LHEEventInfo.fromstring(lines[0])
                particles = [
                    pylhe.LHEParticle.fromstring(line)
                    for line in lines[1:]
                    if not line.strip().startswith("#")
                ]
                row = {
                    "idprup": info.pid,
                    "weight": info.weight,
                    "scale": info.scale,
                    "aqed": info.aqed,
                    "aqcd": info.aqcd,
                }
                for name in PARTICLE_INT_FIELDS + PARTICLE_FLOAT_FIELDS:
                    row[name] = [getattr(p, name) for p in particles]
                row.update(parse_mgrwt(element))
                events.append(row)
                element.clear()
    if not init:
        msg = f"{lhe}: no <init> block"
        raise ValueError(msg)
    return init, events


def to_table(init: dict, events: list[dict]) -> pa.Table:
    schema = pa.schema(
        [
            ("idprup", pa.int32()),
            *[(f, pa.float64()) for f in EVENT_FLOAT_FIELDS],
            ("rscale_nqcd", pa.int32()),
            ("rscale", pa.float64()),
            ("pdfrwt_flavour", pa.list_(pa.list_(pa.int32(), 2))),
            ("pdfrwt_x", pa.list_(pa.list_(pa.float64(), 2))),
            ("pdfrwt_scale", pa.list_(pa.list_(pa.float64(), 2))),
            *[(f, pa.list_(pa.int32())) for f in PARTICLE_INT_FIELDS],
            *[(f, pa.list_(pa.float64())) for f in PARTICLE_FLOAT_FIELDS],
        ],
        metadata={b"init": json.dumps(init, sort_keys=True).encode()},
    )
    columns = {name: [e[name] for e in events] for name in schema.names}
    # `pdfrwt_*` is indexed by beam in the source and by entry in the column, so
    # the two beams' entry lists become the inner fixed-size list. Unequal beam
    # lengths would transpose to a silently truncated column.
    for name in ("pdfrwt_flavour", "pdfrwt_x", "pdfrwt_scale"):
        for event in events:
            beam1, beam2 = event[name]
            if len(beam1) != len(beam2):
                msg = f"{name}: beam entry counts differ ({len(beam1)} vs {len(beam2)})"
                raise ValueError(msg)
        columns[name] = [list(zip(*e[name])) for e in events]
    return pa.table(columns, schema=schema)


def write(table: pa.Table, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    float_columns = [
        *EVENT_FLOAT_FIELDS,
        "rscale",
        "pdfrwt_x",
        "pdfrwt_scale",
        *PARTICLE_FLOAT_FIELDS,
    ]
    pq.write_table(
        table,
        path,
        compression="zstd",
        compression_level=ZSTD_LEVEL,
        use_dictionary=[c for c in table.schema.names if c not in float_columns],
        column_encoding={c: "BYTE_STREAM_SPLIT" for c in float_columns},
        write_statistics=False,
        # One row group per file: the gates read whole runs, and a single group
        # gives the encoders the longest runs to work with.
        row_group_size=len(table),
    )


def verify(table: pa.Table, path: Path, init: dict) -> None:
    """Every retained value comes back bit-identical, and the `<init>` with it."""
    back = pq.read_table(path)
    if json.loads(back.schema.metadata[b"init"]) != init:
        msg = f"{path}: <init> metadata does not round-trip"
        raise ValueError(msg)
    if back.schema.names != table.schema.names:
        msg = f"{path}: column set does not round-trip"
        raise ValueError(msg)
    for name in table.schema.names:
        wrote = table.column(name).to_pylist()
        read = back.column(name).to_pylist()
        if wrote != read:
            index = next(i for i, (a, b) in enumerate(zip(wrote, read)) if a != b)
            msg = f"{path}: column {name} differs at row {index}: {wrote[index]!r} != {read[index]!r}"
            raise ValueError(msg)


def main(argv: list[str]) -> int:
    runs = sorted(
        d.name for d in WORK_AREA.iterdir() if (d / EVENT_PATH).is_file()
    )
    if argv:
        runs = [r for r in runs if r in argv]
    if not runs:
        print(f"no banked runs with {EVENT_PATH} under {WORK_AREA}", file=sys.stderr)
        return 1

    total_lhe = 0
    total_parquet = 0
    print(f"{'run':<26} {'events':>7} {'lhe.gz':>11} {'parquet':>10}  ratio")
    for run in runs:
        lhe = WORK_AREA / run / EVENT_PATH
        init, events = read_run(lhe)
        table = to_table(init, events)
        out = OUT_DIR / f"{run}.parquet"
        write(table, out)
        verify(table, out, init)
        src = lhe.stat().st_size
        dst = out.stat().st_size
        total_lhe += src
        total_parquet += dst
        print(f"{run:<26} {len(events):>7} {src:>11} {dst:>10}  {src / dst:>5.1f}x")
    print(
        f"{'TOTAL':<26} {'':>7} {total_lhe:>11} {total_parquet:>10}  "
        f"{total_lhe / total_parquet:>5.1f}x"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
