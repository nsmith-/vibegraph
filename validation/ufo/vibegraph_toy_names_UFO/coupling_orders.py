"""QED carries every coupling; QCD is declared because MadGraph's WEIGHTED
coupling-order search reads it by name."""

from object_library import all_orders, CouplingOrder

QCD = CouplingOrder(name='QCD',
                    expansion_order=99,
                    hierarchy=1)

QED = CouplingOrder(name='QED',
                    expansion_order=99,
                    hierarchy=2)
