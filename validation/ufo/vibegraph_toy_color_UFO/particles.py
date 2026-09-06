"""Five scalars. Every field here is spin 0 on purpose: this model isolates
colour, so its Lorentz side carries nothing at all and a row that disagrees can
only be disagreeing about colour.

Two distinct colour triplets `p3` and `r3` rather than one repeated field,
because `Epsilon(1,2,3)` is antisymmetric in its first two indices and would
annihilate a vertex whose two triplet legs were the same particle.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

p3 = Particle(pdg_code=9000051,
              name='p3',
              antiname='p3~',
              spin=1,
              color=3,
              mass=Param.MP3,
              width=Param.ZERO,
              texname='\\phi_3',
              antitexname='\\bar\\phi_3',
              charge=1/3,
              GhostNumber=0,
              LeptonNumber=0)

p3__tilde__ = p3.anti()

r3 = Particle(pdg_code=9000052,
              name='r3',
              antiname='r3~',
              spin=1,
              color=3,
              mass=Param.MR3,
              width=Param.ZERO,
              texname='\\rho_3',
              antitexname='\\bar\\rho_3',
              charge=1/3,
              GhostNumber=0,
              LeptonNumber=0)

r3__tilde__ = r3.anti()

d3 = Particle(pdg_code=9000042,
              name='d3',
              antiname='d3~',
              spin=1,
              color=3,
              mass=Param.MD3,
              width=Param.WD3,
              texname='D_3',
              antitexname='\\bar D_3',
              charge=-2/3,
              GhostNumber=0,
              LeptonNumber=0)

d3__tilde__ = d3.anti()

d6 = Particle(pdg_code=9000046,
              name='d6',
              antiname='d6~',
              spin=1,
              color=6,
              mass=Param.MD6,
              width=Param.WD6,
              texname='D_6',
              antitexname='\\bar D_6',
              charge=2/3,
              GhostNumber=0,
              LeptonNumber=0)

d6__tilde__ = d6.anti()

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
