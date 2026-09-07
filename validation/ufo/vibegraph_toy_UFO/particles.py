"""Five fields, named with a `t` so nothing collides with a MadGraph
multiparticle label: a Dirac lepton `lt`, a Dirac colour-triplet quark `qt`, a
singlet scalar `st`, a singlet vector `vt` and a real colour-octet scalar `o8`.

`lt` and `qt` carry hypercharge-like charges that make every vertex below
charge-conserving; nothing else in the model reads them.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

lt = Particle(pdg_code=9000011,
              name='lt',
              antiname='lt~',
              spin=2,
              color=1,
              mass=Param.MLT,
              width=Param.ZERO,
              texname='\\ell_t',
              antitexname='\\bar\\ell_t',
              charge=-1,
              GhostNumber=0,
              LeptonNumber=1)

lt__tilde__ = lt.anti()

qt = Particle(pdg_code=9000004,
              name='qt',
              antiname='qt~',
              spin=2,
              color=3,
              mass=Param.MQT,
              width=Param.ZERO,
              texname='q_t',
              antitexname='\\bar q_t',
              charge=2/3,
              GhostNumber=0,
              LeptonNumber=0)

qt__tilde__ = qt.anti()

st = Particle(pdg_code=9000025,
              name='st',
              antiname='st',
              spin=1,
              color=1,
              mass=Param.MST,
              width=Param.WST,
              texname='S',
              antitexname='S',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)

vt = Particle(pdg_code=9000023,
              name='vt',
              antiname='vt',
              spin=3,
              color=1,
              mass=Param.MVT,
              width=Param.WVT,
              texname='V',
              antitexname='V',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)

o8 = Particle(pdg_code=9000021,
              name='o8',
              antiname='o8',
              spin=1,
              color=8,
              mass=Param.MO8,
              width=Param.WO8,
              texname='O_8',
              antitexname='O_8',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)
