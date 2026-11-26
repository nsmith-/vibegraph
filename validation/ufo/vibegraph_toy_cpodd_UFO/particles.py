"""Four fields: two Dirac fermions `ut` and `dt` whose charges differ by one
unit, a charged vector `wt+` coupling them, and a neutral vector `zt` coupling
each to itself.

The charges make every vertex below charge-conserving; nothing else in the model
reads them.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

ut = Particle(pdg_code=9000002,
              name='ut',
              antiname='ut~',
              spin=2,
              color=1,
              mass=Param.MUT,
              width=Param.ZERO,
              texname='u_t',
              antitexname='\\bar u_t',
              charge=2/3,
              GhostNumber=0,
              LeptonNumber=0)

ut__tilde__ = ut.anti()

dt = Particle(pdg_code=9000001,
              name='dt',
              antiname='dt~',
              spin=2,
              color=1,
              mass=Param.MDT,
              width=Param.ZERO,
              texname='d_t',
              antitexname='\\bar d_t',
              charge=-1/3,
              GhostNumber=0,
              LeptonNumber=0)

dt__tilde__ = dt.anti()

wt__plus__ = Particle(pdg_code=9000024,
                      name='wt+',
                      antiname='wt-',
                      spin=3,
                      color=1,
                      mass=Param.MWT,
                      width=Param.ZERO,
                      texname='W_t^+',
                      antitexname='W_t^-',
                      charge=1,
                      GhostNumber=0,
                      LeptonNumber=0)

wt__minus__ = wt__plus__.anti()

zt = Particle(pdg_code=9000023,
              name='zt',
              antiname='zt',
              spin=3,
              color=1,
              mass=Param.MZT,
              width=Param.ZERO,
              texname='Z_t',
              antitexname='Z_t',
              charge=0,
              GhostNumber=0,
              LeptonNumber=0)
