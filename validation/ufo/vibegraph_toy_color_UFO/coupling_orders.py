"""Coupling orders. Every vertex in the model carries `NP = 1`, so a process
string bounds `NP` alone and MadGraph's WEIGHTED search has nothing to rank.
"""

from object_library import all_orders, CouplingOrder

# No coupling in this model carries QCD. It is declared because MadGraph's
# WEIGHTED coupling-order search reads the order by name and raises KeyError on
# a model that has none, which makes an unbounded `generate` on such a model
# fail before it enumerates anything.
QCD = CouplingOrder(name='QCD',
                    expansion_order=99,
                    hierarchy=1)

NP = CouplingOrder(name='NP',
                   expansion_order=99,
                   hierarchy=1)
