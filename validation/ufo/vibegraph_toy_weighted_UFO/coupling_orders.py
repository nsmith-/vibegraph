"""Two coupling orders with different hierarchies, so a vertex carrying both
has a WEIGHTED above either hierarchy alone.

With QED=2 and NP=1, the vertex `V_fwfwxw` (QED=1, NP=1) weighs 3, more than
the largest hierarchy (2). MadGraph's automatic WEIGHTED search runs up to
(n_ext - 2) times the largest WEIGHTED of any interaction, per equivalent
three-point vertex (`base_objects.py` `get_max_WEIGHTED`), which here is 3.
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
                    hierarchy=2)

NP = CouplingOrder(name='NP',
                   expansion_order=99,
                   hierarchy=1)
