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

`--amps` adds the per-diagram AMP() to the table for diagnosis, and the Rust
gate does not read them: where a contact vertex has several colour structures
MadGraph writes one AMP() per structure, so its graph list is not our diagram
list, while its JAMP order is the colour basis the Rust gate already pairs flow
by flow. A row that sets `bank_amps` always carries the AMP(), with JAMP(1)'s
coefficient on each beside them. It is a single-flow row whose graphs the Rust
gate pairs with groups of its own diagrams.

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
from gen_amplitude_tables import join_continuations, parse_jamp_coefficients  # noqa: E402

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
    # non-`sm` table. A row with a model and no `model_row` names the model
    # directory and restrict card in its table instead.
    model_row: str | None = None
    # Every width set to zero on both sides. MadEvent zeroes the width of a
    # spacelike propagator and this crate follows it; standalone output keeps it,
    # so a process with a massive t-channel line is compared at zero width.
    zero_widths: bool = False
    # Bank AMP(1..NGRAPHS) and their JAMP(1) coefficients. Single-flow rows only.
    bank_amps: bool = False

    def ufo_paths(self) -> dict:
        """The committed model directory and restrict card, repository-relative,
        looked up the way `build.sh` stages them: an authored card under
        `cards/` first, then the model's own."""
        model_dir = f"validation/ufo/{self.model}"
        card_dir = "smeft" if self.model.startswith("SMEFTsim_") else self.model
        card = f"validation/madgraph/cards/{card_dir}/restrict_{self.restrict}.dat"
        if not os.path.isfile(os.path.join(HERE, "..", "..", card)):
            card = f"{model_dir}/restrict_{self.restrict}.dat"
        return {"dir": model_dir, "restrict_card": card}

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
        # A quark line at leg 0: triple-gluon vertices and a four-gluon
        # contact appear only as sources, beside quark-exchange diagrams.
        Row("uux_to_ggg", "u u~ > g g g", (2, -2), (21, 21, 21), 500.0, 3, 4043),
        # The quark-gluon channel with the quark first: the gluon exchange puts a
        # triple-gluon vertex beside the quark line holding leg 0.
        Row("ug_to_ug", "u g > u g", (2, 21), (2, 21), 500.0, 3, 4054),
        # Colourless contacts (WWZZ, WWAZ) beside a lepton line holding leg 0.
        Row("ee_to_wpwmz", "e+ e- > w+ w- z", (-11, 11), (24, -24, 23), 800.0, 3, 4052,
            zero_widths=True),
        # A colourless contact holding leg 0: the WWWW contact beside the photon,
        # Z and Higgs exchanges, whose gauge cancellation fixes its sign.
        Row("wpwm_to_wpwm", "w+ w- > w+ w-", (24, -24), (24, -24), 800.0, 3, 4053,
            zero_widths=True),
        # O_HG's gluon-pair-scalar vertex producing the s-channel Higgs current
        # beside the top line holding leg 0, interfering with QCD.
        Row("ttx_to_gg_chg", "t t~ > g g NP<=1", (6, -6), (21, 21), 800.0, 3, 4056,
            model=SMEFT, restrict="vg_cHG", model_row="gg_to_h_cpeven", zero_widths=True),
        # A W pair at leg 0 beside a final-state lepton line: the neutrino exchange
        # against the photon and Z s-channel, whose diagrams hold leg 0 at a lepton
        # vertex and at the WWV vertex respectively.
        Row("wpwm_to_epem", "w+ w- > e+ e-", (24, -24), (-11, 11), 500.0, 3, 4049,
            zero_widths=True),
        # The tau Yukawa's s-channel Higgs, whose diagrams carry both Yukawas and
        # the only tree-level Higgs self-coupling of a 2 -> 3 SM process here.
        Row("tata_to_ttxh", "ta+ ta- > t t~ h", (-15, 15), (6, -6, 25), 1000.0, 3, 4061),
        # A second Higgs off the top line: final-state top lines of Yukawa vertices
        # alone across zero, one and two propagators, all in one sum.
        Row("tata_to_ttxhh", "ta+ ta- > t t~ h h", (-15, 15), (6, -6, 25, 25), 1500.0, 3, 4062),
        # The Standard Model's own all-Yukawa line: t/u-channel b exchange (one
        # propagator) beside the triple-Higgs annihilation (none).
        Row("bbx_to_hh", "b b~ > h h", (5, -5), (25, 25), 500.0, 3, 4063),
        # The rest of the W-pair-initiated fermion-pair family: the leg order
        # reversed, and a quark line in place of the lepton line.
        Row("wmwp_to_emep", "w- w+ > e- e+", (-24, 24), (11, -11), 500.0, 3, 5001,
            zero_widths=True),
        Row("wpwm_to_uux", "w+ w- > u u~", (24, -24), (2, -2), 500.0, 3, 5002,
            zero_widths=True),
        # A colourless vector at leg 0 on a mixed lepton line: the W-exchange
        # diagram holds leg 0 at the WWZ vertex, the other two at a lepton vertex.
        Row("wpem_to_zve", "w+ e- > z ve", (24, 11), (23, 12), 500.0, 3, 5020,
            zero_widths=True),
        # The photon at leg 0: s-channel W through the AWW vertex beside the
        # electron exchange.
        Row("aem_to_wmve", "a e- > w- ve", (22, 11), (-24, 12), 500.0, 3, 5022,
            zero_widths=True),
        # One graph set under two leg orders. With the W at leg 0 the top and bottom
        # exchanges hold it at the Wtb vertex; with the Higgs at leg 0, at the Yukawa
        # vertex. The s-channel W diagram holds leg 0 at the WWH vertex in both.
        Row("wph_to_tbx", "w+ h > t b~", (24, 25), (6, -5), 800.0, 3, 5023,
            zero_widths=True),
        Row("hwp_to_tbx", "h w+ > t b~", (25, 24), (6, -5), 800.0, 3, 5025,
            zero_widths=True),
        # A Z pair at leg 0: the s-channel Higgs (the ZZH vertex holding leg 0, top
        # Yukawa as the scalar source) beside the top exchange.
        Row("zz_to_ttx", "z z > t t~", (23, 23), (6, -6), 800.0, 3, 5026,
            zero_widths=True),
        # A scalar at leg 0: the triple-Higgs vertex holding it beside the top
        # exchange, which holds it at the Yukawa vertex.
        Row("hh_to_ttx", "h h > t t~", (25, 25), (6, -6), 800.0, 3, 5024,
            zero_widths=True),
        # A gluon at leg 0: the triple-gluon vertex holding it beside the quark
        # exchange, the coloured analogue of `w+ w- > u u~`.
        Row("gg_to_uux", "g g > u u~", (21, 21), (2, -2), 500.0, 3, 5006),
        # 2 -> 3 with a W pair at leg 0: a final-state lepton line beside a Z
        # emitted from it, from the s-channel boson or from a WWZ/WWZZ vertex.
        Row("wpwm_to_epemz", "w+ w- > e+ e- z", (24, -24), (-11, 11, 23), 800.0, 3, 5028,
            zero_widths=True),
        # 2 -> 3 with a W pair at leg 0 and a gluon off the final-state quark line.
        Row("wpwm_to_uuxg", "w+ w- > u u~ g", (24, -24), (2, -2, 21), 800.0, 3, 5045,
            zero_widths=True),
        # Bhabha scattering through the four-lepton contacts: every contact joins
        # one flavour, so the s- and t-channel pairings of one vertex interfere with
        # each other and with photon and Z exchange in both channels. Widths are
        # zero for the spacelike Z.
        Row("ee_to_ee_4f", "e+ e- > e+ e- NP<=1", (-11, 11), (-11, 11), 500.0, 4, 4064,
            model=SMEFT, restrict="vg_c4l", model_row="ee_to_mumu_4f", zero_widths=True,
            bank_amps=True),
        # The top Yukawa spelled as SMEFTsim spells it, a bare `Identity(2,1)`
        # (`FFS2`), with the Higgs emitted off the final-state top pair: rooted as a
        # fermion current whose wrapped leg sits on that crossed line, beside
        # Higgsstrahlung. The only non-zero Wilson coefficient touches the b, so
        # every diagram is the Standard Model's. `SMHLOOP=0` drops the loop-induced
        # h-gamma-gamma and h-Z-gamma vertices, whose Fortran couplings MadGraph
        # writes from seven-digit literals.
        Row("ee_to_ttxh_identity", "e+ e- > t t~ h NP<=1 SMHLOOP=0", (-11, 11), (6, -6, 25),
            1000.0, 3, 4071, model=SMEFT, restrict="vg_cbH", model_row="bbx_to_h_identity",
            bank_amps=True),
        # The same process under MadGraph's own `sm`, whose top Yukawa is `FFS4`
        # (ProjM + ProjP): the control for the `Identity` spelling above.
        Row("ee_to_ttxh", "e+ e- > t t~ h", (-11, 11), (6, -6, 25), 1000.0, 3, 4072,
            bank_amps=True),
        # The toy model's bare `Identity` and bare `Gamma5` Yukawas emitted off a
        # final-state quark pair, beside the vector exchange and emission off the
        # initial lepton line: scalar and pseudoscalar currents rooted on a crossed
        # line in one sum.
        Row("ll_to_qqxs_toy_yukawa", "lt~ lt > qt qt~ st NP<=3 NPCP<=3",
            (-9000011, 9000011), (9000004, -9000004, 9000025), 600.0, 3, 4073,
            model="vibegraph_toy_UFO", restrict="yukawa", model_row="ll_to_qqx_toy_yukawa",
            bank_amps=True),
        # The toy scalar's cubic self-coupling, plain (`SSS1`) and derivative
        # (`SSSD`, under its own coupling order so each is a diagram of its own),
        # producing the s-channel scalar current beside t- and u-channel lepton
        # exchange.
        Row("ll_to_ss_toy_dscalar", "lt~ lt > st st NP<=2 NPD<=1",
            (-9000011, 9000011), (9000025, 9000025), 500.0, 3, 4074,
            model="vibegraph_toy_UFO", restrict="dscalar", bank_amps=True),
        # The same vertices with a scalar as leg 0: the t-channel scalar exchange
        # holds it at the cubic vertex, beside s- and u-channel lepton exchange. Widths are zero for the
        # spacelike scalar.
        Row("sl_to_sl_toy_dscalar", "st lt > st lt NP<=2 NPD<=1",
            (9000025, 9000011), (9000025, 9000011), 500.0, 3, 4075,
            model="vibegraph_toy_UFO", restrict="dscalar", zero_widths=True, bank_amps=True),
        # SMEFTsim's derivative Higgs self-couplings (`SSS3` from O_Hbox, `SSS2` from
        # O_HDD) beside the Standard Model's `SSS1`, each its own diagram: the
        # s-channel Higgs current off an initial top pair, beside t- and u-channel
        # top exchange. Widths are zero for the spacelike top.
        Row("ttx_to_hh_chbox", "t t~ > h h NP<=1", (6, -6), (25, 25), 800.0, 3, 4076,
            model=SMEFT, restrict="vg_cHbox", zero_widths=True, bank_amps=True),
        # The same vertex with a Higgs on leg 0: the t-channel Higgs exchange holds
        # it at the derivative vertex, beside s- and u-channel top exchange. Widths
        # are zero for the spacelike Higgs.
        Row("ht_to_ht_chbox", "h t > h t NP<=1", (25, 6), (25, 6), 800.0, 3, 4077,
            model=SMEFT, restrict="vg_cHbox", zero_widths=True, bank_amps=True),
        # Four Higgs bosons: the derivative quartic contacts (`SSSS3`, `SSSS2`) beside
        # the plain one, and s-, t- and u-channel Higgs exchange with one derivative
        # cubic vertex at either end. Widths are zero for the spacelike Higgs.
        Row("hh_to_hh_chbox", "h h > h h NP<=1", (25, 25), (25, 25), 800.0, 8, 4078,
            model=SMEFT, restrict="vg_cHbox", zero_widths=True, bank_amps=True),
        # The rows below are O_W (`vg_cW`) probes at zero widths: the loop-induced
        # photon-Z-Higgs and photon-photon-Higgs vertices (`VVS4`, `GC_303`/`GC_299`,
        # coupling order SMHLOOP) in each role, and O_W's five-vector contacts.
        # MadGraph's Fortran writes `GC_303` and `GC_299` at seven significant
        # digits, so a row carrying them agrees near 1e-10 rather than at rounding.
        #
        # Controls: O_W's four-vector contacts (WWWW, WWZZ) holding leg 0, beside
        # the photon, Z and Higgs exchanges with O_W's triple-gauge structures.
        Row("wpwm_to_wpwm_cw", "w+ w- > w+ w- NP<=1", (24, -24), (24, -24), 800.0, 3, 4071,
            model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw", zero_widths=True),
        Row("wpwm_to_zz_cw", "w+ w- > z z NP<=1", (24, -24), (23, 23), 800.0, 3, 4072,
            model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw", zero_widths=True),
        # The WWAZ contact, and the photon-Z-Higgs vertex producing the off-shell
        # Higgs from the external photon and Z.
        Row("wpwm_to_az_cw", "w+ w- > a z NP<=1", (24, -24), (22, 23), 800.0, 3, 4073,
            model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw", zero_widths=True),
        # The photon-Z-Higgs vertex holding leg 0, the photon, in the Higgs
        # s-channel diagram.
        Row("az_to_wpwm_cw", "a z > w+ w- NP<=1", (22, 23), (24, -24), 800.0, 3, 4075,
            model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw", zero_widths=True),
        # The same for the photon-photon-Higgs vertex.
        Row("aa_to_wpwm_cw", "a a > w+ w- NP<=1", (22, 22), (24, -24), 800.0, 3, 4078,
            model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw", zero_widths=True),
        # The amplitude row `wpwm_to_wpwmz_cw` itself: O_W's two five-vector
        # contacts each the whole of one diagram, the photon-Z-Higgs vertex
        # producing a Higgs or a photon current, and the four-vector contacts as
        # sources. MadGraph standalone numbers its AMP() as the MadEvent export of
        # the amplitude row does.
        Row("wpwm_to_wpwmz_cw", "w+ w- > w+ w- z NP<=1", (24, -24), (24, -24, 23), 500.0, 4,
            4074, model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw",
            zero_widths=True),
        # Its photon twin: the WWWW-photon five-vector contacts, and the
        # photon-photon-Higgs vertex beside the photon-Z-Higgs one.
        Row("wpwm_to_wpwma_cw", "w+ w- > w+ w- a NP<=1", (24, -24), (24, -24, 22), 500.0, 3,
            4077, model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw",
            zero_widths=True),
        # The five-vector contacts as sources: the s-channel photon or Z off the
        # electron line, which holds leg 0, ends on a WWWW-photon/Z contact.
        Row("ee_to_wpwmwpwm_cw", "e+ e- > w+ w- w+ w- NP<=1", (-11, 11), (24, -24, 24, -24),
            1000.0, 3, 4076, model=SMEFT, restrict="vg_cW", model_row="wpwm_to_wpwmz_cw",
            zero_widths=True),
        # The gluon-initiated half of t t~ + jet: a four-gluon contact beside the
        # top line, where the contact's three colour structures meet its three
        # Lorentz structures. The top exchanged between the two beam gluons is
        # spacelike, so widths are zeroed as for the other massive t-channel rows.
        Row("gg_to_ttxg", "g g > t t~ g", (21, 21), (6, -6, 21), 1000.0, 3, 4064,
            zero_widths=True),
        # The quark-gluon half of t t~ + jet, the gluon either radiated off the
        # light-quark line or attached to the top pair through a triple-gluon
        # vertex. MadEvent's own subprocess puts the gluon first (`g u`, and `g u~`
        # beside it); `u g` is the same amplitude with the beams swapped.
        Row("ug_to_ttxu", "u g > t t~ u", (2, 21), (6, -6, 2), 1000.0, 3, 4065,
            zero_widths=True),
        Row("gu_to_ttxu", "g u > t t~ u", (21, 2), (6, -6, 2), 1000.0, 3, 4066,
            zero_widths=True),
        Row("gux_to_ttxux", "g u~ > t t~ u~", (21, -2), (6, -6, -2), 1000.0, 3, 4067,
            zero_widths=True),
        # Quark annihilation into t t~ + gluon: the gluon off either quark line or
        # off the s-channel gluon through a triple-gluon vertex. Every t-channel
        # line here is a light quark or gluon; the top lines are timelike, so
        # zeroing widths only removes the timelike top width from both sides.
        Row("uux_to_ttxg", "u u~ > t t~ g", (2, -2), (6, -6, 21), 1000.0, 3, 4068,
            zero_widths=True),
        # A gluon off the initial quark line of a four-quark contact: the contact
        # produces the off-shell quark current the emission vertex closes, in both
        # colour contractions, beside the s-channel gluon with the gluon off either
        # quark line or off the triple-gluon vertex.
        Row("uux_to_ttxg_4f", "u u~ > t t~ g NP<=1", (2, -2), (6, -6, 21), 1000.0, 3, 5101,
            model=SMEFT, restrict="vg_c4q", model_row="uux_to_ttx_4f", zero_widths=True),
        # A photon off either line of the tensor four-fermion contact (`O_leQt3`):
        # the contact, read through the tensor path, produces the off-shell lepton
        # or top current the emission closes. `SMHLOOP=0` drops the loop-induced
        # Higgs-photon vertices and their seven-digit Fortran couplings.
        Row("tata_to_ttxa_tensor4f", "ta+ ta- > t t~ a NP<=1 SMHLOOP=0", (-15, 15),
            (6, -6, 22), 1000.0, 3, 5102, model=SMEFT, restrict="vg_cleQt3",
            model_row="tata_to_ttx_tensor4f", bank_amps=True),
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


def read_jamp_coefficients(pdir: str, ngraphs: int) -> list[float]:
    """JAMP(1)'s coefficient on each AMP(), from a single-flow matrix.f.

    The statement is MadEvent's `JAMP(1,1)` with one index, and the parser is
    the amplitude tables'. The coefficients are banked as reals, so a complex
    one is refused.
    """
    with open(os.path.join(pdir, "matrix.f")) as f:
        src = join_continuations(f.read())
    m = re.search(r"\n      JAMP\(1\) =(.*)", src)
    assert m, f"{pdir}: no JAMP(1) assignment"
    coefs = parse_jamp_coefficients(m.group(1), ngraphs, f"{pdir}: JAMP(1)")
    assert all(c.imag == 0.0 for c in coefs), f"{pdir}: a complex JAMP(1) coefficient: {coefs}"
    return [c.real for c in coefs]


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
    if row.bank_amps:
        assert ncolor == 1, f"{row.key}: bank_amps needs a single-flow process"
    amps = args.amps or row.bank_amps
    table = {
        "key": row.key,
        "process": row.process,
        "model": "sm" if row.model is None else (row.model_row or "ufo"),
        **({"ufo": row.ufo_paths()} if row.model is not None and row.model_row is None
           else {}),
        "n_ext": len(row.pdgs_in) + len(row.pdgs_out),
        "n_graphs": ngraphs,
        "n_flows": ncolor,
        "helicities": hels,
        "param_card": card_lines,
        **({"jamp_coefficients": read_jamp_coefficients(pdir, ngraphs)}
           if row.bank_amps else {}),
        "points": [
            {
                "momenta": pt,
                "helicity": [
                    {
                        "m2": h["m2"],
                        "jamps": h["jamps"],
                        **({"amps": h["amps"]} if amps else {}),
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
