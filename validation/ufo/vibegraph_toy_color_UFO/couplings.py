"""One coupling per vertex, with distinct values so the restriction cannot merge
two of them."""

from object_library import all_couplings, Coupling

GC_PS = Coupling(name='GC_PS',
                 value='complex(0,1)*gps',
                 order={'NP': 1})

GC_RS = Coupling(name='GC_RS',
                 value='complex(0,1)*grs',
                 order={'NP': 1})

GC_EPS = Coupling(name='GC_EPS',
                  value='complex(0,1)*geps',
                  order={'NP': 1})

GC_K6 = Coupling(name='GC_K6',
                 value='complex(0,1)*gk6',
                 order={'NP': 1})
