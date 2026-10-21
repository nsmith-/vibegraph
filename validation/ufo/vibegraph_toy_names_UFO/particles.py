"""Three fields with Standard-Model PDG codes, two of them under names other
than MadGraph's defaults for those codes.

The muon is called `m-`/`m+`, as `loop_sm` calls it, and the up quark
`up`/`up~`, a spelling of the kind BSM models use. `import model` renames both
to MadGraph's default names by PDG code (`mu-`/`mu+`, `u`/`u~`); `--modelname`
keeps these. The photon already has its default name, `a`.
"""

from __future__ import division
from object_library import all_particles, Particle
import parameters as Param

a = Particle(pdg_code=22,
             name='a',
             antiname='a',
             spin=3,
             color=1,
             mass=Param.ZERO,
             width=Param.ZERO,
             texname='a',
             antitexname='a',
             charge=0,
             GhostNumber=0,
             LeptonNumber=0)

m__minus__ = Particle(pdg_code=13,
                      name='m-',
                      antiname='m+',
                      spin=2,
                      color=1,
                      mass=Param.ZERO,
                      width=Param.ZERO,
                      texname='m-',
                      antitexname='m+',
                      charge=-1,
                      GhostNumber=0,
                      LeptonNumber=1)

m__plus__ = m__minus__.anti()

up = Particle(pdg_code=2,
              name='up',
              antiname='up~',
              spin=2,
              color=3,
              mass=Param.ZERO,
              width=Param.ZERO,
              texname='up',
              antitexname='\\bar{up}',
              charge=2/3,
              GhostNumber=0,
              LeptonNumber=0)

up__tilde__ = up.anti()
