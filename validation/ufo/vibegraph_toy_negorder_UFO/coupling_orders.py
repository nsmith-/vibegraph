"""The Standard Model's hierarchies with SMEFTsim's NP order: QCD=1, QED=2,
NP=1. QCD is declared because MadGraph's WEIGHTED search reads it by name."""

from object_library import all_orders, CouplingOrder

QCD = CouplingOrder(name='QCD',
                    expansion_order=99,
                    hierarchy=1)

QED = CouplingOrder(name='QED',
                    expansion_order=99,
                    hierarchy=2)

NP = CouplingOrder(name='NP',
                   expansion_order=99,
                   hierarchy=1)
