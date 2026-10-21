"""Four vertices, particles listed all-incoming with the barred spinor first.

`V_ntntvt` pairs two `nt` legs, a fermion line with no particle/antiparticle
orientation. `V_ltntsc` and its conjugate `V_ntltsc` put `nt` on a line with the
Dirac `lt`; the conjugate pairs two particles, as a Majorana vertex's h.c. term
does.
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

V_ntntvt = Vertex(name='V_ntntvt',
                  particles=[P.nt, P.nt, P.vt],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_NV})

V_ltntsc = Vertex(name='V_ltntsc',
                  particles=[P.lt__tilde__, P.nt, P.sc],
                  color=['1'],
                  lorentz=[L.FFS1],
                  couplings={(0, 0): C.GC_LNS})

V_ntltsc = Vertex(name='V_ntltsc',
                  particles=[P.nt, P.lt, P.sc__tilde__],
                  color=['1'],
                  lorentz=[L.FFS1],
                  couplings={(0, 0): C.GC_LNS})
