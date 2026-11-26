"""Eight vertices. Particles are listed as the dim6top model lists them, the
anti-fermion first, so a structure's spinor index 1 is the barred spinor and
index 2 the unbarred one.

`V_ututzt` is the self-conjugate dipole vertex: the gauge coupling beside the
CP-even and the CP-odd dipole terms, under three coupling orders, so MadGraph
splits it into three interactions. `V_utdtzwp` and `V_dtutzwm` are the charged
contact and its Hermitian partner, each with one chirality of
`[gamma^mu, gamma^nu]`, as the dim6top restricted model's `t b a W` contacts.
`V_ututht` is a scalar Yukawa beside a pseudoscalar one with a real coupling, and
`V_ututzht` the dipole's chains beside the scalar.
"""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_ututzt = Vertex(name='V_ututzt',
                  particles=[P.ut__tilde__, P.ut, P.zt],
                  color=['1'],
                  lorentz=[L.FFV1, L.FFV11, L.FFV9],
                  couplings={(0, 0): C.GC_ZU, (0, 1): C.GC_DIP, (0, 2): C.GC_DIPI})

V_dtdtzt = Vertex(name='V_dtdtzt',
                  particles=[P.dt__tilde__, P.dt, P.zt],
                  color=['1'],
                  lorentz=[L.FFV1],
                  couplings={(0, 0): C.GC_ZD})

V_utdtwp = Vertex(name='V_utdtwp',
                  particles=[P.ut__tilde__, P.dt, P.wt__plus__],
                  color=['1'],
                  lorentz=[L.FFV2],
                  couplings={(0, 0): C.GC_W})

V_dtutwm = Vertex(name='V_dtutwm',
                  particles=[P.dt__tilde__, P.ut, P.wt__minus__],
                  color=['1'],
                  lorentz=[L.FFV2],
                  couplings={(0, 0): C.GC_W})

V_utdtzwp = Vertex(name='V_utdtzwp',
                   particles=[P.ut__tilde__, P.dt, P.zt, P.wt__plus__],
                   color=['1'],
                   lorentz=[L.FFVV3],
                   couplings={(0, 0): C.GC_CR})

V_dtutzwm = Vertex(name='V_dtutzwm',
                   particles=[P.dt__tilde__, P.ut, P.zt, P.wt__minus__],
                   color=['1'],
                   lorentz=[L.FFVV1],
                   couplings={(0, 0): C.GC_CL})

V_ututht = Vertex(name='V_ututht',
                  particles=[P.ut__tilde__, P.ut, P.ht],
                  color=['1'],
                  lorentz=[L.FFS1, L.FFS5],
                  couplings={(0, 0): C.GC_YH, (0, 1): C.GC_YA})

V_ututzht = Vertex(name='V_ututzht',
                   particles=[P.ut__tilde__, P.ut, P.zt, P.ht],
                   color=['1'],
                   lorentz=[L.FFVS6, L.FFVS4],
                   couplings={(0, 0): C.GC_HDIP, (0, 1): C.GC_HDIPI})
