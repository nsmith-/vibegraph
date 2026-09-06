"""Six vertices, all-incoming.

`V_p3r3d3` contracts three colour triplets with the totally antisymmetric
`Epsilon(1,2,3)`; `V_p3r3d6` contracts two triplets into an antisextet with
`K6Bar`, whose index order is the sextet leg first and its two triplet legs
after it (`madgraph/core/color_algebra.py`, and the interaction of
`tests/unit_tests/iolibs/test_export_v4.py` that pairs two triplets with an
antisextet). The two singlet-scalar emissions are what give each of those a
2 -> 2 process with a second, colour-trivial diagram to interfere against.

The two scalar emissions write their colour delta as an explicit `T(2,1)`
rather than as `Identity(1,2)`. MadGraph resolves an `Identity` between a 3 and
a 3bar by looking up which of the two it has labelled the fundamental, and it
learns that labelling only from a three-point vertex carrying an adjoint index:
`import_ufo.find_color_anti_color_rep` records the *first* particle of a vertex
whose colour string is `T(3,2,1)` as the fundamental one, which for the
Standard Model's `u~ u g` is the antiparticle. A model with no such vertex --
this one -- falls through to a default that reads each particle's own colour
sign, the opposite labelling, and its delta comes out index-reversed relative
to the `Epsilon`/`K6Bar` legs. Without an epsilon in the model that reversal is
a uniform transpose of the whole colour basis and nothing can see it; with one,
MadGraph's own colour matrix stops reducing and the generation dies in
`set_Nc`.

Each diquark vertex is listed twice, once conjugated. MadGraph keys its vertex
lookup on the sorted PDG tuple of an interaction's particles and converts a
process's initial-state legs to their antiparticles before it looks anything up,
so a vertex whose particle multiset is not self-conjugate is reachable from only
one side unless the conjugate interaction is in the model too -- the h.c. of the
Lagrangian term, which is what a FeynRules-written model emits. The conjugate
carries the same coupling because the strength is real.
"""

from object_library import all_vertices, Vertex
import particles as P
import couplings as C
import lorentz as L

V_p3p3st = Vertex(name='V_p3p3st',
                  particles=[P.p3__tilde__, P.p3, P.st],
                  color=['T(2,1)'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_PS})

V_r3r3st = Vertex(name='V_r3r3st',
                  particles=[P.r3__tilde__, P.r3, P.st],
                  color=['T(2,1)'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_RS})

V_p3r3d3 = Vertex(name='V_p3r3d3',
                  particles=[P.p3, P.r3, P.d3],
                  color=['Epsilon(1,2,3)'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_EPS})

V_p3r3d3bar = Vertex(name='V_p3r3d3bar',
                     particles=[P.p3__tilde__, P.r3__tilde__, P.d3__tilde__],
                     color=['EpsilonBar(1,2,3)'],
                     lorentz=[L.SSS1],
                     couplings={(0, 0): C.GC_EPS})

V_p3r3d6 = Vertex(name='V_p3r3d6',
                  particles=[P.p3, P.r3, P.d6__tilde__],
                  color=['K6Bar(3,1,2)'],
                  lorentz=[L.SSS1],
                  couplings={(0, 0): C.GC_K6})

V_p3r3d6bar = Vertex(name='V_p3r3d6bar',
                     particles=[P.p3__tilde__, P.r3__tilde__, P.d6],
                     color=['K6(3,1,2)'],
                     lorentz=[L.SSS1],
                     couplings={(0, 0): C.GC_K6})
