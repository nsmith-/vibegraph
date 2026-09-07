"""One coupling order per structure family, all with hierarchy 1 and no
expansion cap.

The split is what makes the per-diagram oracle sharp: MadGraph creates one
interaction per distinct coupling-order tuple, so two structures sharing a
vertex reach the generated matrix element as two diagrams with an `AMP()` each
rather than as one diagram whose amplitude is already their sum. `NPCP` and
`NPGG` exist for exactly that reason -- they carry the pseudoscalar Yukawa and
the gamma-gamma spelling of the tensor contact, whose partners sit on the same
vertices under `NP`.
"""

from object_library import all_orders, CouplingOrder

# No coupling in this model carries QCD. It is declared because MadGraph's
# WEIGHTED coupling-order search reads the order by name and raises KeyError on
# a model that has none, which makes an unbounded `generate` on such a model
# fail before it enumerates anything.
QCD = CouplingOrder(name='QCD',
                    expansion_order=99,
                    hierarchy=1)

QED = CouplingOrder(name='QED',
                    expansion_order=99,
                    hierarchy=1)

NP = CouplingOrder(name='NP',
                   expansion_order=99,
                   hierarchy=1)

NPCP = CouplingOrder(name='NPCP',
                     expansion_order=99,
                     hierarchy=1)

NPGG = CouplingOrder(name='NPGG',
                     expansion_order=99,
                     hierarchy=1)
