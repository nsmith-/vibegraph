"""One coupling per (vertex, structure) pair, each carrying the coupling order
that keeps its structure a diagram of its own.

Values are distinct: MadGraph's restriction merges couplings that evaluate to
the same number, and a merge would put two structures back into one interaction.
"""

from object_library import all_couplings, Coupling

GC_LV = Coupling(name='GC_LV',
                 value='complex(0,1)*gl',
                 order={'QED': 1})

GC_QV = Coupling(name='GC_QV',
                 value='complex(0,1)*gq',
                 order={'QED': 1})

GC_QVD = Coupling(name='GC_QVD',
                  value='complex(0,1)*gdip',
                  order={'NP': 1})

GC_4FT = Coupling(name='GC_4FT',
                  value='complex(0,1)*gtens',
                  order={'NP': 1})

GC_4FG = Coupling(name='GC_4FG',
                  value='complex(0,1)*ggam',
                  order={'NPGG': 1})

GC_LS = Coupling(name='GC_LS',
                 value='complex(0,1)*ysl',
                 order={'NP': 1})

GC_QS = Coupling(name='GC_QS',
                 value='complex(0,1)*ysq',
                 order={'NP': 1})

GC_LP = Coupling(name='GC_LP',
                 value='complex(0,1)*ypl',
                 order={'NPCP': 1})

GC_QP = Coupling(name='GC_QP',
                 value='complex(0,1)*ypq',
                 order={'NPCP': 1})

GC_QO = Coupling(name='GC_QO',
                 value='complex(0,1)*gqo',
                 order={'NP': 1})

GC_O3 = Coupling(name='GC_O3',
                 value='complex(0,1)*go3',
                 order={'NP': 1})
