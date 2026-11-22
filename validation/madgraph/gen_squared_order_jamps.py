#!/usr/bin/env python3
"""Bank MadGraph standalone per-class, per-helicity, per-flow JAMPs of a process
with squared-order constraints.

A squared-order constraint (`NP^2==1`) makes MadGraph split each JAMP by the
amplitude orders of its diagrams: `JAMP(flow, M)` for every amplitude
split-order class `M` (`AMPSPLITORDERS`), and the per-helicity `MATRIX` returns
one `RES(I)` per squared split order `I` (`SQSPLITORDERS`), the sum of
`JAMP(i,M)·CF·JAMP(j,N)*` over the class pairs whose orders add up to it.
`CHOSEN_SO_CONFIGS` marks the `I` the constraint keeps, and their sum is the
|M|² MadEvent integrates.

This script generates such a process with `output standalone`, patches its
`matrix.f` to copy `JAMP(:,:)` out through a COMMON block, compiles `MATRIX`
against a small driver, and writes per point and per helicity in MadGraph's own
NHEL table:

  * `res`: RES(1..NSQSO), the per-squared-order contributions, and
  * `jamps`: JAMP(1..NCOLOR, 1..NAMPSO), the per-class, per-flow amplitudes,

with the split-order tables (`amp_orders`, `sq_orders`, `chosen`) and the
param card the numbers were computed with.

Usage:
  python validation/madgraph/gen_squared_order_jamps.py KEY --work DIR

`KEY` is one of the rows below. The generated process directory is a build
product under `--work`; the committed table lands in
`validation/madgraph/squared_orders/<KEY>.json`.

Prerequisites: a MadGraph on PATH as `mg5_aMC` (the table records which), and
gfortran.
"""

import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

from gen_amplitude import beam_momenta, rambo  # noqa: E402

TABLE_DIR = os.path.join(HERE, "squared_orders")
SMEFT = "SMEFTsim_topU3l_MwScheme_UFO"


@dataclass
class Row:
    key: str
    # The MadGraph `generate` line, which is also the process the Rust side
    # enumerates from.
    process: str
    pdgs_in: tuple
    pdgs_out: tuple
    sqrt_s: float
    npoints: int
    seed: int
    # `None` for MadGraph's own `sm`; otherwise the vendored model directory and
    # restrict card (`import model <copy>-<restrict>`).
    model: str | None = None
    restrict: str | None = None
    # MadEvent zeroes t-channel widths and this crate does too; standalone output
    # does not, so a row with a spacelike massive line sets every width to zero
    # in the card both sides read.
    zero_widths: bool = False


ROWS = [
    Row("ee_to_ttx_np1", "e+ e- > t t~ NP^2==1", (-11, 11), (6, -6), 500.0, 4, 2601,
        SMEFT, "massless"),
    Row("ee_to_ttx_np2", "e+ e- > t t~ NP^2==2", (-11, 11), (6, -6), 500.0, 4, 2602,
        SMEFT, "massless"),
    Row("uux_to_uux_qcd2", "u u~ > u u~ QCD^2==2", (2, -2), (2, -2), 500.0, 4, 2603,
        zero_widths=True),
]

DRIVER = """      PROGRAM SQSO_DRIVER
      IMPLICIT NONE
      INTEGER NEXTERNAL, NCOLOR, NAMPSO, NSQAMPSO
      PARAMETER (NEXTERNAL=%(nexternal)d, NCOLOR=%(ncolor)d)
      PARAMETER (NAMPSO=%(nampso)d, NSQAMPSO=%(nsqampso)d)
      COMPLEX*16 JAMPD(NCOLOR, NAMPSO)
      COMMON/SQSO_JAMPS/JAMPD
      REAL*8 P(0:3,NEXTERNAL), RES(NSQAMPSO)
      INTEGER NHEL(NEXTERNAL), IC(NEXTERNAL), I, J, M, NPT, NCOMB, IP, IH
      INTEGER HELS(NEXTERNAL, 4096)
      CHARACTER*512 CARD
      READ(*,'(A)') CARD
      CALL SETPARA(CARD)
      READ(*,*) NPT, NCOMB
      DO IH = 1, NCOMB
        READ(*,*) (HELS(I,IH), I=1,NEXTERNAL)
      ENDDO
      DO I = 1, NEXTERNAL
        IC(I) = 1
      ENDDO
      DO IP = 1, NPT
        DO I = 1, NEXTERNAL
          READ(*,*) (P(J,I), J=0,3)
        ENDDO
        DO IH = 1, NCOMB
          DO I = 1, NEXTERNAL
            NHEL(I) = HELS(I,IH)
          ENDDO
          CALL MATRIX(P, NHEL, IC, RES)
          DO I = 1, NSQAMPSO
            WRITE(*,'(A,3I6,ES26.17)') 'R', IP, IH, I, RES(I)
          ENDDO
          DO M = 1, NAMPSO
            DO I = 1, NCOLOR
              WRITE(*,'(A,2I6,2ES26.17)') 'J', I, M, JAMPD(I,M)
            ENDDO
          ENDDO
        ENDDO
      ENDDO
      END
"""


def run(cmd, cwd=None, stdin=None):
    out = subprocess.run(cmd, cwd=cwd, input=stdin, capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"{' '.join(cmd)} failed:\n{out.stdout[-3000:]}\n{out.stderr[-3000:]}")
    return out.stdout


def generate(row: Row, work: str) -> str:
    """The standalone process directory for `row`, generated under `work`."""
    out = os.path.join(work, row.key)
    if os.path.isdir(out):
        return out
    lines = ["set automatic_html_opening False", "set notification_center False"]
    if row.model:
        # MadGraph writes its pickled model beside the UFO, so it imports a copy.
        copy = os.path.join(work, "models", row.model)
        if not os.path.isdir(copy):
            shutil.copytree(os.path.join(REPO, "validation", "ufo", row.model), copy)
        lines.append(f"import model {copy}-{row.restrict}")
    else:
        lines.append("import model sm")
    lines += [f"generate {row.process}", f"output standalone {out} -nojpeg"]
    script = os.path.join(work, f"{row.key}.mg5")
    with open(script, "w") as f:
        f.write("\n".join(lines) + "\n")
    run(["mg5_aMC", script], cwd=work)
    return out


def join_continuations(src: str) -> str:
    """Fortran fixed-form source with its continuation lines joined."""
    return re.sub(r"\n     [^ \n]", "", src)


def data_rows(src: str, name: str) -> list[list[int]]:
    """`DATA (NAME(k,i),i=1,n) /a,b,.../` rows of an integer table, by k."""
    rows = {}
    for m in re.finditer(
        rf"DATA\s*\(\s*{name}\(\s*(\d+)\s*,\s*i\s*\)\s*,\s*i\s*=\s*1\s*,\s*\d+\s*\)\s*/([^/]*)/",
        src,
        re.IGNORECASE,
    ):
        rows[int(m.group(1))] = [int(x) for x in m.group(2).replace(" ", "").split(",")]
    return [rows[k] for k in sorted(rows)]


def parameter(src: str, name: str) -> int:
    m = re.search(rf"\b{name}\s*=\s*(\d+)", src)
    assert m, f"no parameter {name}"
    return int(m.group(1))


def build(pdir: str) -> dict:
    """Patch, compile and describe the subprocess directory's `matrix.f`."""
    matrix = os.path.join(pdir, "matrix.f")
    with open(matrix) as f:
        src = f.read()
    if "SQSO_JAMPS" not in src:
        # Copy the JAMPs out right before the colour contraction.
        anchor = "      RES = 0.D0"
        assert anchor in src, f"{matrix}: no colour contraction"
        src = src.replace(
            anchor,
            "      JAMPD = JAMP\n" + anchor,
            1,
        )
        m = re.search(r"\n\s*COMPLEX\*16 JAMP\(NCOLOR,NAMPSO\)", src)
        assert m, f"{matrix}: no JAMP declaration"
        src = (
            src[: m.end()]
            + "\n      COMPLEX*16 JAMPD(NCOLOR,NAMPSO)\n      COMMON/SQSO_JAMPS/JAMPD"
            + src[m.end():]
        )
        with open(matrix, "w") as f:
            f.write(src)
    joined = join_continuations(src)
    info = {
        "nexternal": parameter(joined, "NEXTERNAL"),
        "ncolor": parameter(joined, "NCOLOR"),
        "nampso": parameter(joined, "NAMPSO"),
        "nsqampso": parameter(joined, "NSQAMPSO"),
        "ngraphs": parameter(joined, "NGRAPHS"),
    }
    m = re.search(r"DATA\s+CHOSEN_SO_CONFIGS\s*/([^/]*)/", joined, re.IGNORECASE)
    assert m, f"{matrix}: no CHOSEN_SO_CONFIGS"
    info["chosen"] = [x.strip().lower() == ".true." for x in m.group(1).split(",")]
    info["amp_orders"] = data_rows(joined, "AMPSPLITORDERS")
    info["sq_orders"] = data_rows(joined, "SQSPLITORDERS")
    m = re.search(r"C\s+\[([^\]]*)\]", joined)
    info["split_orders"] = [s.strip().strip("'\"") for s in m.group(1).split(",")] if m else []
    helicities = []
    for m in re.finditer(
        r"DATA\s*\(NHEL\(I,\s*(\d+)\),I=1,\s*\d+\)\s*/([^/]*)/", joined, re.IGNORECASE
    ):
        helicities.append([int(x) for x in m.group(2).split(",")])
    info["helicities"] = helicities
    with open(os.path.join(pdir, "sqso_driver.f"), "w") as f:
        f.write(DRIVER % info)
    lib = os.path.join(pdir, "..", "..", "lib")
    run(["make", "-C", os.path.join(pdir, "..", "..", "Source")])
    objs = ["sqso_driver.f", "matrix.f"]
    run(
        ["gfortran", "-O", "-w", "-fno-automatic", "-ffixed-line-length-132", "-o", "sqso_driver"]
        + objs
        + [f"-L{lib}", "-ldhelas", "-lmodel"],
        cwd=pdir,
    )
    return info


def mass_block(card_text: str) -> dict:
    masses, inside = {}, False
    for line in card_text.splitlines():
        words = line.split("#")[0].split()
        if not words:
            continue
        if words[0].lower() == "block":
            inside = words[1].lower() == "mass"
            continue
        if inside and len(words) >= 2:
            masses[int(words[0])] = float(words[1])
    return masses


def bank(row: Row, work: str) -> dict:
    out = generate(row, work)
    [pdir] = glob.glob(os.path.join(out, "SubProcesses", "P1_*"))
    info = build(pdir)
    card_path = os.path.join(out, "Cards", "param_card.dat")
    with open(card_path) as f:
        card_text = f.read()
    if row.zero_widths:
        card_text = re.sub(r"(?m)^(DECAY\s+\S+\s+)\S+", r"\g<1>0.000000e+00", card_text)
        card_path = os.path.join(out, "Cards", "param_card_zero_widths.dat")
        with open(card_path, "w") as f:
            f.write(card_text)
    masses = mass_block(card_text)
    m_in = [abs(masses.get(abs(p), 0.0)) for p in row.pdgs_in]
    m_out = [abs(masses.get(abs(p), 0.0)) for p in row.pdgs_out]
    rng = np.random.default_rng(row.seed)
    points = []
    for _ in range(row.npoints):
        b1, b2 = beam_momenta(row.sqrt_s, *m_in)
        fs = rambo(len(m_out), row.sqrt_s, m_out, rng)
        points.append([b1, b2] + [list(map(float, k)) for k in fs])
    lines = [card_path, f"{len(points)} {len(info['helicities'])}"]
    lines += [" ".join(str(h) for h in hel) for hel in info["helicities"]]
    for p in points:
        lines += [" ".join(repr(x) for x in k) for k in p]
    text = run(["./sqso_driver"], cwd=pdir, stdin="\n".join(lines) + "\n")
    nh, nsq, nc, na = len(info["helicities"]), info["nsqampso"], info["ncolor"], info["nampso"]
    entries = [[{"res": [0.0] * nsq, "jamps": [[None] * nc for _ in range(na)]} for _ in range(nh)]
               for _ in points]
    current = None
    for line in text.splitlines():
        w = line.split()
        if not w:
            continue
        if w[0] == "R":
            ip, ih, i = int(w[1]) - 1, int(w[2]) - 1, int(w[3]) - 1
            entries[ip][ih]["res"][i] = float(w[4])
            current = (ip, ih)
        elif w[0] == "J":
            i, m = int(w[1]) - 1, int(w[2]) - 1
            ip, ih = current
            entries[ip][ih]["jamps"][m][i] = [float(w[3]), float(w[4])]
    version = mg_version()
    return {
        "key": row.key,
        "process": row.process,
        "model": "sm" if row.model is None else "ufo",
        "ufo": None if row.model is None else {
            "dir": f"validation/ufo/{row.model}",
            "restrict_card": f"validation/ufo/{row.model}/restrict_{row.restrict}.dat",
        },
        "madgraph": version,
        "sqrt_s": row.sqrt_s,
        "seed": row.seed,
        "split_orders": info["split_orders"],
        "amp_orders": info["amp_orders"],
        "sq_orders": info["sq_orders"],
        "chosen": info["chosen"],
        "n_flows": nc,
        "n_graphs": info["ngraphs"],
        "helicities": info["helicities"],
        "param_card": card_text.splitlines(),
        "points": [{"momenta": p, "entries": e} for p, e in zip(points, entries)],
    }


def mg_version() -> str:
    exe = shutil.which("mg5_aMC")
    root = os.path.dirname(os.path.dirname(os.path.realpath(exe)))
    for cand in [os.path.join(root, "VERSION"), os.path.join(os.path.dirname(exe), "..", "MG5_aMC", "VERSION")]:
        if os.path.exists(cand):
            with open(cand) as f:
                m = re.search(r"version\s*=\s*(\S+)", f.read())
                if m:
                    return m.group(1)
    return "unknown"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("key", choices=[r.key for r in ROWS] + ["all"])
    ap.add_argument("--work", required=True)
    args = ap.parse_args()
    os.makedirs(args.work, exist_ok=True)
    os.makedirs(TABLE_DIR, exist_ok=True)
    for row in ROWS:
        if args.key not in ("all", row.key):
            continue
        table = bank(row, os.path.abspath(args.work))
        path = os.path.join(TABLE_DIR, f"{row.key}.json")
        with open(path, "w") as f:
            json.dump(table, f, indent=1)
            f.write("\n")
        print(f"wrote {path}: {len(table['points'])} points x {len(table['helicities'])} "
              f"helicities, classes {table['amp_orders']}, chosen {table['chosen']}")


if __name__ == "__main__":
    main()
