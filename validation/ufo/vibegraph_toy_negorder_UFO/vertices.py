"""Six vertices. Particles are listed all-incoming, the anti-fermion first.

The scalar's cubic self-coupling is written twice, as SMEFTsim writes the
Higgs's: `V_xw3` at `QED=1` and `V_xw3np` at `NP=1, QED=-1`, its vev
insertion. Their WEIGHTED are 2 and -1.
"""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_fwfwvw = Vertex(name='V_fwfwvw',
                  particles=[P.fw__tilde__, P.fw, P.vw],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_FV})

V_fwfwxw = Vertex(name='V_fwfwxw',
                  particles=[P.fw__tilde__, P.fw, P.xw],
                  color=['1'],
                  lorentz=[L.FFS1],
                  couplings={(0, 0): C.GC_FX})

V_vwvwxw = Vertex(name='V_vwvwxw',
                  particles=[P.vw, P.vw, P.xw],
                  color=['1'],
                  lorentz=[L.VVS1],
                  couplings={(0, 0): C.GC_VX})

V_xw3 = Vertex(name='V_xw3',
               particles=[P.xw, P.xw, P.xw],
               color=['1'],
               lorentz=[L.SSS1],
               couplings={(0, 0): C.GC_X3})

V_xw3np = Vertex(name='V_xw3np',
                 particles=[P.xw, P.xw, P.xw],
                 color=['1'],
                 lorentz=[L.SSS1],
                 couplings={(0, 0): C.GC_X3NP})

V_xw4 = Vertex(name='V_xw4',
               particles=[P.xw, P.xw, P.xw, P.xw],
               color=['1'],
               lorentz=[L.SSSS1],
               couplings={(0, 0): C.GC_X4})
