"""One coupling per vertex, each with the coupling orders the WEIGHTED search
reads: `GC_FX` carries two orders, `GC_Y4` a WEIGHTED of 7 over a four-point
vertex (3.5 per equivalent three-point vertex)."""

from object_library import all_couplings, Coupling

GC_FV = Coupling(name='GC_FV',
                 value='complex(0,1)*gfv',
                 order={'QED': 1})

GC_FX = Coupling(name='GC_FX',
                 value='complex(0,1)*gfx',
                 order={'QED': 1, 'NP': 1})

GC_X4 = Coupling(name='GC_X4',
                 value='complex(0,1)*gx4',
                 order={'QED': 1})

GC_XY = Coupling(name='GC_XY',
                 value='complex(0,1)*gxy',
                 order={'QED': 1})

GC_Y4 = Coupling(name='GC_Y4',
                 value='complex(0,1)*gy4',
                 order={'QED': 2, 'NP': 3})
