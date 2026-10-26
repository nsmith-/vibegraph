"""Five vertices. Particles are listed all-incoming, the anti-fermion first.

Per equivalent three-point vertex, with QED=2 and NP=1, their WEIGHTED are:
`V_fwfwvw` 2, `V_fwfwxw` 3, `V_xw4` 1, `V_xwxwyw` 2, `V_yw4` 3.5. The largest,
truncated to an integer as MadGraph's `find_optimal_process_orders` does, is 3.
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

V_xw4 = Vertex(name='V_xw4',
               particles=[P.xw, P.xw, P.xw, P.xw],
               color=['1'],
               lorentz=[L.SSSS1],
               couplings={(0, 0): C.GC_X4})

V_xwxwyw = Vertex(name='V_xwxwyw',
                  particles=[P.xw, P.xw, P.yw],
                  color=['1'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_XY})

V_yw4 = Vertex(name='V_yw4',
               particles=[P.yw, P.yw, P.yw, P.yw],
               color=['1'],
               lorentz=[L.SSSS1],
               couplings={(0, 0): C.GC_Y4})
