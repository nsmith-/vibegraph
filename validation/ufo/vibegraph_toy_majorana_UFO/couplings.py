from object_library import all_couplings, Coupling

GC_LV = Coupling(name='GC_LV',
                 value='complex(0,1)*gl',
                 order={'QED': 1})

GC_NV = Coupling(name='GC_NV',
                 value='complex(0,1)*gn',
                 order={'QED': 1})

GC_LNS = Coupling(name='GC_LNS',
                  value='complex(0,1)*ys',
                  order={'QED': 1})
