"""Seven vertices. Particles are listed all-incoming, the anti-fermion first, so
a structure's spinor index 1 is the barred spinor and index 2 the unbarred one.

The two-structure vertices (`V_qtqtvt`, the two Yukawas, the four-fermion
contact) are deliberate: they are the shape MadGraph splits into one interaction
per coupling-order tuple.
"""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_ltltvt = Vertex(name='V_ltltvt',
                  particles=[P.lt__tilde__, P.lt, P.vt],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_LV})

V_qtqtvt = Vertex(name='V_qtqtvt',
                  particles=[P.qt__tilde__, P.qt, P.vt],
                  color=['Identity(1,2)'],
                  lorentz=[L.FFV1, L.FFVD],
                  couplings={(0, 0): C.GC_QV, (0, 1): C.GC_QVD})

V_ltltst = Vertex(name='V_ltltst',
                  particles=[P.lt__tilde__, P.lt, P.st],
                  color=['1'],
                  lorentz=[L.FFS1, L.FFS5],
                  couplings={(0, 0): C.GC_LS, (0, 1): C.GC_LP})

V_qtqtst = Vertex(name='V_qtqtst',
                  particles=[P.qt__tilde__, P.qt, P.st],
                  color=['Identity(1,2)'],
                  lorentz=[L.FFS1, L.FFS5],
                  couplings={(0, 0): C.GC_QS, (0, 1): C.GC_QP})

V_ltltqtqt = Vertex(name='V_ltltqtqt',
                    particles=[P.lt__tilde__, P.lt, P.qt__tilde__, P.qt],
                    color=['Identity(3,4)'],
                    lorentz=[L.FFFFT, L.FFFFG],
                    couplings={(0, 0): C.GC_4FT, (0, 1): C.GC_4FG})

V_qtqto8 = Vertex(name='V_qtqto8',
                  particles=[P.qt__tilde__, P.qt, P.o8],
                  color=['T(3,2,1)'],
                  lorentz=[L.FFS1],
                  couplings={(0, 0): C.GC_QO})

V_o8o8o8 = Vertex(name='V_o8o8o8',
                  particles=[P.o8, P.o8, P.o8],
                  color=['d(1,2,3)'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_O3})
