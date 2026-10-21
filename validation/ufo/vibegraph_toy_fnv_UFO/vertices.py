"""Four vertices, particles listed all-incoming.

`V_atbtdq` pairs two particles on one fermion line and `V_atbtdqc`, its
conjugate, two antiparticles: each changes fermion number by two, and neither
writes the charge-conjugation operator `C`, as a UFO diquark coupling often does
not.
"""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_atatvt = Vertex(name='V_atatvt',
                  particles=[P.at__tilde__, P.at, P.vt],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_AV})

V_btbtvt = Vertex(name='V_btbtvt',
                  particles=[P.bt__tilde__, P.bt, P.vt],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_BV})

V_atbtdq = Vertex(name='V_atbtdq',
                  particles=[P.at, P.bt, P.dq__tilde__],
                  color=['1'],
                  lorentz=[L.FFS1],
                  couplings={(0, 0): C.GC_ABD})

V_atbtdqc = Vertex(name='V_atbtdqc',
                   particles=[P.at__tilde__, P.bt__tilde__, P.dq],
                   color=['1'],
                   lorentz=[L.FFS1],
                   couplings={(0, 0): C.GC_ABD})
