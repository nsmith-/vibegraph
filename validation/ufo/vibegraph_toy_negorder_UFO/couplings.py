"""One coupling per vertex. `GC_X3NP` carries a negative power, `{'NP': 1,
'QED': -1}`, the way SMEFTsim writes a vev insertion (`SMEFT_alpha_UFO`'s
triple-Higgs `GC_5284`): its WEIGHTED is 1 - 2 = -1."""

from object_library import all_couplings, Coupling

GC_FV = Coupling(name='GC_FV',
                 value='complex(0,1)*gfv',
                 order={'QED': 1})

GC_FX = Coupling(name='GC_FX',
                 value='complex(0,1)*gfx',
                 order={'QED': 1})

GC_VX = Coupling(name='GC_VX',
                 value='complex(0,1)*gvx',
                 order={'QED': 1})

GC_X3 = Coupling(name='GC_X3',
                 value='complex(0,1)*gx3',
                 order={'QED': 1})

GC_X3NP = Coupling(name='GC_X3NP',
                   value='complex(0,1)*gx3np',
                   order={'NP': 1, 'QED': -1})

GC_X4 = Coupling(name='GC_X4',
                 value='complex(0,1)*gx4',
                 order={'QED': 2})
