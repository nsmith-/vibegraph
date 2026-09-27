#!/usr/bin/env python3
"""Bank MadGraph standalone per-helicity, per-flow JAMPs for processes that sit
outside the MadEvent-backed amplitude rows.

`gen_amplitude_tables.py` banks its tables from a MadEvent work area: a
`launch`ed process directory, its unweighted events and its channel mapping.
Some of what the amplitude gate has to see needs none of that — a process whose
only question is its colour-ordered amplitudes at a few phase-space points —
and a MadEvent integration of it would be the expensive part of the reference
for no gain. This script generates such a process with `output standalone`,
compiles its `MATRIX` against a small Fortran driver, and writes, per point and
per helicity in MadGraph's own NHEL table:

  * the colour-summed |M|^2 of that helicity (MATRIX's own return value), and
  * JAMP(1..NCOLOR), the per-flow amplitudes, copied out of MATRIX by a COMMON
    block patched in after MadGraph builds them.

The per-diagram AMP() are dumped with `--amps` for diagnosis but not banked:
where a contact vertex has several colour structures MadGraph writes one AMP()
per structure, so its graph list is not our diagram list, while its JAMP order
is the colour basis the Rust gate already pairs flow by flow.

The table carries the param card the numbers were computed with, so the Rust
side binds the same parameters rather than re-deriving them.

Usage:
  python validation/madgraph/gen_standalone_jamps.py gg_to_ggg
  python validation/madgraph/gen_standalone_jamps.py gg_to_ggg --work DIR --amps --out FILE
  python validation/madgraph/gen_standalone_jamps.py all    # every table, in key order

The generated process directory is a build product; `--work` (default
`validation/madgraph/output/standalone`) says where it goes. The committed
table lands in `validation/madgraph/standalone/<key>.json`.

Prerequisites: the pinned MadGraph (`mg5_pinned.sh`) and gfortran on PATH, as
`pixi run -e madgraph` provides; for a SMEFTsim row, the models `build.sh`
stages under `output/models` (or `VG_MODELS_DIR`).
"""

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from gen_amplitude import beam_momenta, rambo  # noqa: E402

TABLE_DIR = os.path.join(HERE, "standalone")
DEFAULT_WORK = os.path.join(HERE, "output", "standalone")


# Where `build.sh` stages the vendored UFO models with this repository's restrict
# cards; `VG_MODELS_DIR` points elsewhere (a work area outside the checkout).
MODELS_DIR = os.environ.get("VG_MODELS_DIR", os.path.join(HERE, "output", "models"))
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
    # `None` for MadGraph's own `sm`; otherwise the staged model directory and
    # restrict card, `import model <MODELS_DIR>/<model>-<restrict>`.
    model: str | None = None
    restrict: str | None = None
    # The manifest row whose model and restrict card the Rust side loads for a
    # non-`sm` table.
    model_row: str | None = None
    # Every width set to zero on both sides. MadEvent zeroes the width of a
    # spacelike propagator and this crate follows it; standalone output keeps it,
    # so a process with a massive t-channel line is compared at zero width.
    zero_widths: bool = False

    def import_model(self) -> str:
        if self.model is None:
            return "sm"
        return os.path.join(MODELS_DIR, self.model) + f"-{self.restrict}"


ROWS = {
    r.key: r
    for r in [
        # Five gluons: 25 diagrams, ten of them a four-gluon contact beside a
        # triple-gluon vertex, the smallest multiplicity where a contact is not
        # the whole diagram. 24 flows, 32 helicities.
        Row("gg_to_ggg", "g g > g g g", (21, 21), (21, 21, 21), 500.0, 3, 4041),
        # A quark line as the anchor: triple-gluon vertices and a four-gluon
        # contact appear only as sources, beside quark-exchange diagrams.
        Row("uux_to_ggg", "u u~ > g g g", (2, -2), (21, 21, 21), 500.0, 3, 4043),
        # The quark-gluon channel with the quark first: the gluon exchange puts a
        # triple-gluon source beside the quark-line anchor.
        Row("ug_to_ug", "u g > u g", (2, 21), (2, 21), 500.0, 3, 4054),
        # Colourless contacts (WWZZ, WWAZ) as sources beside a lepton-line anchor.
        Row("ee_to_wpwmz", "e+ e- > w+ w- z", (-11, 11), (24, -24, 23), 800.0, 3, 4052,
            zero_widths=True),
        # A colourless contact as the anchor: the WWWW contact beside the photon,
        # Z and Higgs exchanges, whose gauge cancellation fixes its sign.
        Row("wpwm_to_wpwm", "w+ w- > w+ w-", (24, -24), (24, -24), 800.0, 3, 4053,
            zero_widths=True),
        # O_HG's gluon-pair-scalar vertex producing the s-channel Higgs current
        # beside the top-line anchor, interfering with QCD.
        Row("ttx_to_gg_chg", "t t~ > g g NP<=1", (6, -6), (21, 21), 800.0, 3, 4056,
            model=SMEFT, restrict="vg_cHG", model_row="gg_to_h_cpeven", zero_widths=True),
        # A W pair at the anchor beside a final-state lepton line: disagrees with
        # MadGraph (the neutrino exchange carries the wrong sign relative to the
        # s-channel), banked so the comparison keeps running.
        Row("wpwm_to_epem", "w+ w- > e+ e-", (24, -24), (-11, 11), 500.0, 3, 4049,
            zero_widths=True),
    ]
}


def run_madgraph(row: Row, work: str) -> str:
    """`output standalone` into `work/<key>`; returns the subprocess directory."""
    out = os.path.join(work, row.key)
    if not os.path.isdir(out):
        os.makedirs(work, exist_ok=True)
        script = os.path.join(work, f"{row.key}.mg5")
        with open(script, "w") as f:
            f.write(f"import model {row.import_model()}\n")
            f.write(f"generate {row.process}\n")
            f.write(f"output standalone {out}\n")
        with open(os.path.join(work, f"{row.key}.log"), "w") as log:
            subprocess.run(
                ["bash", os.path.join(HERE, "mg5_pinned.sh"), script],
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
    sub = os.path.join(out, "SubProcesses")
    dirs = [d for d in sorted(os.listdir(sub)) if d.startswith("P")]
    assert len(dirs) == 1, f"{row.key}: expected one P* subprocess dir, got {dirs}"
    return os.path.join(sub, dirs[0])


def read_matrix(pdir: str):
    """(NGRAPHS, NCOLOR, NHEL table) from the standalone matrix.f."""
    with open(os.path.join(pdir, "matrix.f")) as f:
        src = f.read()
    ngraphs = int(re.search(r"NGRAPHS=(\d+)", src).group(1))
    ncolor = int(re.search(r"NCOLOR=(\d+)", src).group(1))
    hels = [
        [int(h) for h in m.group(1).split(",")]
        for m in re.finditer(r"DATA \(NHEL\(I, *\d+\),I=1,\d+\) /([^/]*)/", src)
    ]
    return ngraphs, ncolor, hels


DRIVER = """\
      PROGRAM VGDUMP
      IMPLICIT NONE
      INCLUDE 'nexternal.inc'
      INTEGER NGRAPHS, NCOLOR
      PARAMETER (NGRAPHS={ngraphs}, NCOLOR={ncolor})
      COMPLEX*16 AMPD(NGRAPHS), JAMPD(NCOLOR)
      COMMON/VGDBG/AMPD, JAMPD
      REAL*8 P(0:3,NEXTERNAL), MATRIX, M2
      INTEGER NHEL(NEXTERNAL), IC(NEXTERNAL), I, J, NPT, NCOMB, IP, IH
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
          M2 = MATRIX(P, NHEL, IC)
          WRITE(*,'(A,2I6,ES26.17)') 'M2', IP, IH, M2
          DO I = 1, NCOLOR
            WRITE(*,'(A,I6,2ES26.17)') 'J', I, JAMPD(I)
          ENDDO
          DO I = 1, NGRAPHS
            WRITE(*,'(A,I6,2ES26.17)') 'A', I, AMPD(I)
          ENDDO
        ENDDO
      ENDDO
      END
"""


def build_driver(pdir: str, ngraphs: int, ncolor: int) -> str:
    """Patch MATRIX to publish AMP and JAMP, and link it against the driver."""
    exe = os.path.join(pdir, "vgdump")
    if os.path.exists(exe):
        return exe
    with open(os.path.join(pdir, "matrix.f")) as f:
        src = f.read()
    head, sep, body = src.partition("REAL*8 FUNCTION MATRIX(")
    assert sep, "no MATRIX function"
    decl = re.search(r"\n      COMPLEX\*16 AMP\(NGRAPHS\), JAMP\(NCOLOR\)[^\n]*\n", body)
    assert decl, "no AMP/JAMP declaration in MATRIX"
    body = (
        body[: decl.end()]
        + "      COMPLEX*16 AMPD(NGRAPHS), JAMPD(NCOLOR)\n"
        + "      COMMON/VGDBG/AMPD, JAMPD\n"
        + body[decl.end() :]
    )
    body = body.replace(
        "\n      MATRIX = 0.D0\n",
        "\n      AMPD = AMP\n      JAMPD = JAMP\n      MATRIX = 0.D0\n",
        1,
    )
    with open(os.path.join(pdir, "matrix_vg.f"), "w") as f:
        f.write(head + sep + body)
    with open(os.path.join(pdir, "vgdump.f"), "w") as f:
        f.write(DRIVER.format(ngraphs=ngraphs, ncolor=ncolor))
    subprocess.run(["make", "-C", os.path.join(pdir, "..", "..", "Source")], check=True,
                   stdout=subprocess.DEVNULL)
    lib = os.path.join(pdir, "..", "..", "lib")
    subprocess.run(
        ["gfortran", "-O", "-ffixed-line-length-132", "-o", exe, "vgdump.f", "matrix_vg.f",
         f"-L{lib}", "-ldhelas", "-lmodel"],
        check=True,
        cwd=pdir,
    )
    return exe


def read_masses(card_path: str) -> dict[int, float]:
    masses = {}
    in_block = False
    with open(card_path) as f:
        for line in f:
            s = line.split("#")[0].strip()
            if not s:
                continue
            if s.lower().startswith("block"):
                in_block = s.split()[1].lower() == "mass"
                continue
            if in_block:
                pid, val = s.split()[:2]
                masses[int(pid)] = float(val)
    return masses


def points(row: Row, masses: dict[int, float]) -> list[list[list[float]]]:
    rng = np.random.default_rng(row.seed)
    m_in = [masses.get(abs(p), 0.0) for p in row.pdgs_in]
    m_out = [masses.get(abs(p), 0.0) for p in row.pdgs_out]
    p1, p2 = beam_momenta(row.sqrt_s, *m_in)
    out = []
    for _ in range(row.npoints):
        fs = rambo(len(m_out), row.sqrt_s, m_out, rng)
        out.append([list(p1), list(p2)] + [list(map(float, k)) for k in fs])
    return out


def evaluate(exe: str, card: str, pts, hels):
    lines = [card, f"{len(pts)} {len(hels)}"]
    lines += [" ".join(map(str, h)) for h in hels]
    for pt in pts:
        lines += [" ".join(repr(x) for x in k) for k in pt]
    # SETPARA writes a param.log into its working directory.
    res = subprocess.run([exe], input="\n".join(lines) + "\n", capture_output=True, text=True,
                         check=True, cwd=os.path.dirname(exe))
    per_point = [[None] * len(hels) for _ in pts]
    cur = None
    for line in res.stdout.splitlines():
        tok = line.split()
        if tok[0] == "M2":
            cur = {"m2": float(tok[3]), "jamps": [], "amps": []}
            per_point[int(tok[1]) - 1][int(tok[2]) - 1] = cur
        elif tok[0] == "J":
            cur["jamps"].append([float(tok[2]), float(tok[3])])
        elif tok[0] == "A":
            cur["amps"].append([float(tok[2]), float(tok[3])])
    return per_point


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("key", choices=sorted(ROWS) + ["all"])
    ap.add_argument("--work", default=DEFAULT_WORK)
    ap.add_argument("--amps", action="store_true", help="also write the per-diagram AMP()")
    ap.add_argument("--out", help="write the table here instead of the committed location")
    args = ap.parse_args()
    if args.key == "all":
        if args.out:
            ap.error("--out names one table; it cannot take `all`")
        for key in sorted(ROWS):
            bank(ROWS[key], args)
    else:
        bank(ROWS[args.key], args)


def bank(row: Row, args) -> None:
    """Generate (or reuse) one row's standalone directory and write its table."""
    pdir = run_madgraph(row, os.path.abspath(args.work))
    ngraphs, ncolor, hels = read_matrix(pdir)
    exe = build_driver(pdir, ngraphs, ncolor)
    card = os.path.abspath(os.path.join(pdir, "..", "..", "Cards", "param_card.dat"))
    if row.zero_widths:
        # MadEvent zeroes t-channel widths and this crate does too; standalone
        # output does not, so a comparison through a spacelike massive line
        # sets every width to zero on both sides instead.
        with open(card) as f:
            text = re.sub(r"(?m)^(DECAY\s+\S+\s+)\S+", r"\g<1>0.000000e+00", f.read())
        card = card.replace("param_card.dat", "param_card_zero_widths.dat")
        with open(card, "w") as f:
            f.write(text)
    pts = points(row, read_masses(card))
    per_point = evaluate(exe, card, pts, hels)

    with open(card) as f:
        card_lines = f.read().splitlines()
    table = {
        "key": row.key,
        "process": row.process,
        "model": "sm" if row.model is None else row.model_row,
        "n_ext": len(row.pdgs_in) + len(row.pdgs_out),
        "n_graphs": ngraphs,
        "n_flows": ncolor,
        "helicities": hels,
        "param_card": card_lines,
        "points": [
            {
                "momenta": pt,
                "helicity": [
                    {
                        "m2": h["m2"],
                        "jamps": h["jamps"],
                        **({"amps": h["amps"]} if args.amps else {}),
                    }
                    for h in per_point[i]
                ],
            }
            for i, pt in enumerate(pts)
        ],
    }
    path = args.out or os.path.join(TABLE_DIR, f"{row.key}.json")
    if not args.out:
        os.makedirs(TABLE_DIR, exist_ok=True)
    with open(path, "w") as f:
        json.dump(table, f, indent=None, separators=(",", ":"))
        f.write("\n")
    print(f"wrote {path}: {len(pts)} points x {len(hels)} helicities x {ncolor} flows")


if __name__ == "__main__":
    main()
