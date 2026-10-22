"""Four fields, named with a `w` so nothing collides with a MadGraph
multiparticle label: a Dirac fermion `fw`, a neutral vector `vw` and two real
singlet scalars `xw` and `yw`.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

fw = Particle(pdg_code=9000111,
              name='fw',
              antiname='fw~',
              spin=2,
              color=1,
              mass=Param.MFW,
              width=Param.ZERO,
              texname='f_w',
              antitexname='\\bar f_w',
              charge=-1,
              GhostNumber=0,
              LeptonNumber=1)

fw__tilde__ = fw.anti()

vw = Particle(pdg_code=9000122,
              name='vw',
              antiname='vw',
              spin=3,
              color=1,
              mass=Param.MVW,
              width=Param.WVW,
              texname='V_w',
              antitexname='V_w',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)

xw = Particle(pdg_code=9000135,
              name='xw',
              antiname='xw',
              spin=1,
              color=1,
              mass=Param.MXW,
              width=Param.WXW,
              texname='X_w',
              antitexname='X_w',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)

yw = Particle(pdg_code=9000136,
              name='yw',
              antiname='yw',
              spin=1,
              color=1,
              mass=Param.MYW,
              width=Param.WYW,
              texname='Y_w',
              antitexname='Y_w',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)
