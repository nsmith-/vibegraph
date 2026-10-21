"""Four fields: a Dirac lepton `lt`, a Majorana fermion `nt` (its own
antiparticle, so `name` and `antiname` agree and there is no `anti()` line), a
neutral vector `vt` and a charged scalar `sc`.

The model is a refusal fixture: `nt` reaches a diagram as an external leg
(`lt~ lt > nt nt`) and as an internal propagator only (`lt~ lt > sc sc~`).
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

nt = Particle(pdg_code=9000022,
              name='nt',
              antiname='nt',
              spin=2,
              color=1,
              mass=Param.MNT,
              width=Param.ZERO,
              texname='\\chi_t',
              antitexname='\\chi_t',
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

sc = Particle(pdg_code=9000015,
              name='sc',
              antiname='sc~',
              spin=1,
              color=1,
              mass=Param.MSC,
              width=Param.WSC,
              texname='\\tilde\\ell_t',
              antitexname='\\tilde\\ell_t^*',
              charge=-1,
              GhostNumber=0,
              LeptonNumber=1)

sc__tilde__ = sc.anti()
