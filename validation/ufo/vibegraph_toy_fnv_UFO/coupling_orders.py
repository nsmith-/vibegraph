from object_library import all_orders, CouplingOrder

# Declared for MadGraph's WEIGHTED search, which reads the order by name.
QCD = CouplingOrder(name='QCD',
                    expansion_order=99,
                    hierarchy=1)

QED = CouplingOrder(name='QED',
                    expansion_order=99,
                    hierarchy=1)
