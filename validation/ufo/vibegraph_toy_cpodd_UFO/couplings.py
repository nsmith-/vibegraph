"""One coupling per (vertex, structure) pair, written as FeynRules writes the
dim6top model's: a dipole's CP-even term `i c` on `[gamma^mu, pslash]` and its
CP-odd term `c^I` on `[gamma^mu, pslash](P_L - P_R)`; a charged contact's
coupling `c^I - i c` on the right-handed chain of one charge and `c^I + i c` on
the left-handed chain of the other, the pair the Lagrangian's hermiticity ties
together. The `ht` Yukawas are written as the Higgs-characterisation model
writes its `X0 t t~` vertex: `-i y` on the identity and a real `y_a` on `Gamma5`.
"""

from object_library import all_couplings, Coupling

GC_ZU = Coupling(name='GC_ZU',
                 value='complex(0,1)*gzu',
                 order={'QED': 1})

GC_ZD = Coupling(name='GC_ZD',
                 value='complex(0,1)*gzd',
                 order={'QED': 1})

GC_W = Coupling(name='GC_W',
                value='complex(0,1)*gw',
                order={'QED': 1})

GC_DIP = Coupling(name='GC_DIP',
                  value='complex(0,1)*cd',
                  order={'NP': 1})

GC_DIPI = Coupling(name='GC_DIPI',
                   value='cdI',
                   order={'NPCP': 1})

GC_CR = Coupling(name='GC_CR',
                 value='ccI - complex(0,1)*cc',
                 order={'NPC': 1})

GC_CL = Coupling(name='GC_CL',
                 value='ccI + complex(0,1)*cc',
                 order={'NPC': 1})

GC_HDIP = Coupling(name='GC_HDIP',
                   value='complex(0,1)*ch',
                   order={'NP': 1})

GC_HDIPI = Coupling(name='GC_HDIPI',
                    value='chI',
                    order={'NPCP': 1})

GC_YH = Coupling(name='GC_YH',
                 value='-(complex(0,1)*yh)',
                 order={'QED': 1})

GC_YA = Coupling(name='GC_YA',
                 value='ya',
                 order={'NPA': 1})
