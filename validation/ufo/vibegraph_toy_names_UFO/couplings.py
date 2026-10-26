"""The photon's couplings to the muon and to the up quark."""

from object_library import all_couplings, Coupling

GC_M = Coupling(name='GC_M',
                value='-(complex(0,1)*gm)',
                order={'QED': 1})

GC_U = Coupling(name='GC_U',
                value='complex(0,1)*gu',
                order={'QED': 1})
