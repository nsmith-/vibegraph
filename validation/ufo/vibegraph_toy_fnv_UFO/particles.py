"""Four fields: two Dirac fermions `at` and `bt`, a neutral vector `vt` and a
charged scalar `dq`, which couples to `at` and `bt` the way a diquark couples to
two quarks.

The model is a refusal fixture: `at bt > at bt` reaches the diquark vertex,
`at~ at > at~ at` does not.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

at = Particle(pdg_code=9000001,
              name='at',
              antiname='at~',
              spin=2,
              color=1,
              mass=Param.MAT,
              width=Param.ZERO,
              texname='a_t',
              antitexname='\\bar a_t',
              charge=-1/3,
              GhostNumber=0,
              LeptonNumber=0)

at__tilde__ = at.anti()

bt = Particle(pdg_code=9000003,
              name='bt',
              antiname='bt~',
              spin=2,
              color=1,
              mass=Param.MBT,
              width=Param.ZERO,
              texname='b_t',
              antitexname='\\bar b_t',
              charge=-1/3,
              GhostNumber=0,
              LeptonNumber=0)

bt__tilde__ = bt.anti()

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

dq = Particle(pdg_code=9000006,
              name='dq',
              antiname='dq~',
              spin=1,
              color=1,
              mass=Param.MDQ,
              width=Param.WDQ,
              texname='D',
              antitexname='D^*',
              charge=-2/3,
              GhostNumber=0,
              LeptonNumber=0)

dq__tilde__ = dq.anti()
