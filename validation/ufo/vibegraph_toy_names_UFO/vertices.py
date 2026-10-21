"""Two photon vertices. Particles are listed all-incoming, the anti-fermion
first."""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_mma = Vertex(name='V_mma',
               particles=[P.m__plus__, P.m__minus__, P.a],
               color=['1'],
               lorentz=[L.FFV1],
               couplings={(0, 0): C.GC_M})

V_uua = Vertex(name='V_uua',
               particles=[P.up__tilde__, P.up, P.a],
               color=['Identity(1,2)'],
               lorentz=[L.FFV1],
               couplings={(0, 0): C.GC_U})
