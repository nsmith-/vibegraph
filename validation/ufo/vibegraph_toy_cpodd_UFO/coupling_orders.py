"""One coupling order per structure family, all with hierarchy 1.

MadGraph creates one interaction per distinct coupling-order tuple, so the
gauge term, the CP-even dipole term and the CP-odd dipole term of the `ut ut~ zt`
vertex reach the generated matrix element as three diagrams with an `AMP()`
each rather than as one diagram whose amplitude is their sum.
"""

from object_library import all_orders, CouplingOrder

QED = CouplingOrder(name='QED',
                    expansion_order=99,
                    hierarchy=1)

NP = CouplingOrder(name='NP',
                   expansion_order=99,
                   hierarchy=1)

NPCP = CouplingOrder(name='NPCP',
                     expansion_order=99,
                     hierarchy=1)

NPC = CouplingOrder(name='NPC',
                    expansion_order=99,
                    hierarchy=1)

NPA = CouplingOrder(name='NPA',
                    expansion_order=99,
                    hierarchy=1)
